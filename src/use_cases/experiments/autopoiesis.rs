//! AP-5: Persistence Property Test — operacionalización del cap. 10 del paper.
//! AP-5: Persistence property test — operationalization of paper §10.
//!
//! > "Lo que persiste es aquello que encontró una forma de copiarse antes de disiparse."
//!
//! Este módulo expone tres capas desacopladas:
//!
//! 1. **Generators** (`random_reaction_network`, `random_food_set`) — redes
//!    arbitrarias determinísticas a partir de una `seed`.
//! 2. **Harness** (`run_soup`) — orquestador puro sin Bevy: aplica AP-0/1/2/3
//!    y AP-4 sobre una sopa hasta `ticks`, devuelve un `SoupReport`.
//! 3. **Report** (`SoupReport`, `ClosureFate`) — data estructurada con serde
//!    para `tests/property_autopoiesis.rs` y tracking inter-build (JSON).
//!
//! Sin side-effects.  Sin `rand` externo — usamos el PCG interno
//! (`determinism::next_u64`) para mantener la cadena de dependencias cerrada.
//!
//! Axioms: este harness no introduce axiomas nuevos.  Sólo cablea primitivas
//! existentes — cualquier violación detectada aquí es un bug en AP-0..AP-4.

use serde::{Deserialize, Serialize};

use crate::blueprint::constants::chemistry::{
    FISSION_PRESSURE_RATIO, MAX_PRODUCTS_PER_REACTION, MAX_REACTANTS_PER_REACTION,
    REACTION_FREQ_BANDWIDTH_DEFAULT, SPECIES_DIFFUSION_RATE,
};
use crate::blueprint::equations::determinism::{next_u64, range_f32};
use crate::blueprint::equations::{
    compute_membrane_field, compute_strength_field, diffuse_species, find_blobs,
    kinetic_stability, pressure_ratio, raf_closures, step_grid_reactions,
};
use crate::layers::closure_membrane_mask::ClosureMembraneMask;
use crate::layers::reaction::SpeciesId;
use crate::layers::reaction_network::{
    ReactionNetwork, ReactionNetworkSpec, ReactionSpec, StoichSpec,
};
use crate::layers::species_grid::{SpeciesCell, SpeciesGrid};

// ── Generators (Tier B — determinísticos, puros) ────────────────────────────

/// Genera una red de reacciones determinística a partir de `seed`.
/// `n_species ≤ 32` (clampeado).  Cada reacción recibe 1..=max_reactants
/// reactivos y 1..=max_products productos, todos distintos entre sí para
/// evitar slots redundantes.  `k ∈ [0.2, 1.5]`, `freq ∈ [30, 70] Hz`.
///
/// El resultado respeta `Reaction::is_well_formed()` — ReactionNetwork rechaza
/// el resto vía `ReactionNetworkError`, por lo que nunca retorna `Err` si los
/// parámetros están dentro de rango.  Ante error inesperado ⇒ red vacía.
pub fn random_reaction_network(
    seed: u64,
    n_species: u8,
    n_reactions: usize,
    max_reactants: u8,
    max_products: u8,
) -> ReactionNetwork {
    let n_species = n_species.clamp(2, 32);
    let max_reactants = max_reactants.clamp(1, MAX_REACTANTS_PER_REACTION as u8);
    let max_products = max_products.clamp(1, MAX_PRODUCTS_PER_REACTION as u8);

    let mut state = next_u64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut next = || { state = next_u64(state); state };

    let mut reactions = Vec::with_capacity(n_reactions);
    for _ in 0..n_reactions {
        let n_react = 1 + (next() as u8) % max_reactants;
        let n_prod  = 1 + (next() as u8) % max_products;

        let reactants = pick_unique_species(&mut next, n_species, n_react);
        let products  = pick_unique_species(&mut next, n_species, n_prod);

        let k    = range_f32(next(), 0.2, 1.5);
        let freq = range_f32(next(), 30.0, 70.0);

        reactions.push(ReactionSpec { reactants, products, k, freq });
    }
    ReactionNetwork::from_spec(ReactionNetworkSpec { reactions })
        .unwrap_or_else(|_| ReactionNetwork::empty())
}

/// Selección determinística de `k` especies distintas del rango `[0, n_species)`.
/// `k` se clampa a `min(k, n_species)`.
pub fn random_food_set(seed: u64, n_species: u8, k: usize) -> Vec<SpeciesId> {
    let n_species = n_species.min(32);
    if n_species == 0 { return Vec::new(); }
    let k = k.min(n_species as usize);
    if k == 0 { return Vec::new(); }

    let mut state = next_u64(seed.wrapping_mul(0xD9E8_21B4_A5F3_CC01));
    let mut next = || { state = next_u64(state); state };

    let mut pool: Vec<u8> = (0..n_species).collect();
    // Fisher-Yates parcial (primeros k).
    for i in 0..k {
        let j = i + (next() as usize) % (pool.len() - i);
        pool.swap(i, j);
    }
    pool.into_iter()
        .take(k)
        .filter_map(SpeciesId::new)
        .collect()
}

// Selección de `count` species distintas ∈ [0, n_species), con stoich 1 (mass-action).
fn pick_unique_species(
    next: &mut impl FnMut() -> u64,
    n_species: u8,
    count: u8,
) -> Vec<StoichSpec> {
    let count = count.min(n_species);
    let mut pool: Vec<u8> = (0..n_species).collect();
    for i in 0..count as usize {
        let j = i + (next() as usize) % (pool.len() - i);
        pool.swap(i, j);
    }
    pool.into_iter()
        .take(count as usize)
        .map(|s| StoichSpec(s, 1))
        .collect()
}

// ── Report (Tier C — serializable, JSON-friendly) ──────────────────────────

/// Destino de una closure observada en el snapshot inicial (post-equilibración).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClosureFate {
    /// Identidad estable de la closure (FNV-1a de reactions+species ordenadas).
    pub hash: u64,
    /// `true` si `hash` sigue apareciendo en el snapshot final.
    pub survived: bool,
    /// Detecciones donde algún blob asociado cruzó `FISSION_PRESSURE_RATIO`.
    pub pressure_events: u32,
    /// Media de `kinetic_stability` sobre la última ventana configurada.
    pub k_stability_mean_last: f32,
}

/// Reporte agregado de una simulación completa.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SoupReport {
    pub seed: u64,
    pub n_ticks: u64,
    pub n_closures_initial: u32,
    pub n_closures_final: u32,
    pub total_dissipated: f32,
    pub fates: Vec<ClosureFate>,
}

impl SoupReport {
    /// Serializa a JSON compacto — para CI artifacts / tracking inter-build.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

// ── Harness (Tier C — orquestación pura, cero Bevy) ────────────────────────

/// Configuración de una sopa aleatoria.  Max 4-5 campos críticos + agrupación.
#[derive(Clone, Debug)]
pub struct SoupConfig {
    pub seed: u64,
    pub n_species: u8,
    pub n_reactions: usize,
    pub food_size: usize,
    /// Dimensiones del grid `(w, h)`.
    pub grid: (usize, usize),
    pub ticks: u64,
    /// Ticks de equilibración antes del snapshot inicial de closures.
    pub equilibration_ticks: u64,
    /// Cada cuántos ticks re-detectar closures + evaluar pressure.
    pub detection_every: u64,
    /// Ventana final (ticks) sobre la que se promedia `k_stability`.
    pub last_window_ticks: u64,
    /// Qe inicial sembrado homogéneamente por cada species food.
    pub initial_food_qe: f32,
    pub dt: f32,
}

impl Default for SoupConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            n_species: 8,
            n_reactions: 16,
            food_size: 3,
            grid: (12, 12),
            ticks: 2_000,
            equilibration_ticks: 100,
            detection_every: 50,
            last_window_ticks: 500,
            initial_food_qe: 2.0,
            dt: 0.1,
        }
    }
}

/// Ejecuta una sopa determinística y reporta el destino de sus closures
/// iniciales (post-equilibración).  Pure fn: mismas entradas ⇒ mismo output.
pub fn run_soup(config: &SoupConfig) -> SoupReport {
    let net = random_reaction_network(
        config.seed,
        config.n_species,
        config.n_reactions,
        MAX_REACTANTS_PER_REACTION as u8,
        MAX_PRODUCTS_PER_REACTION as u8,
    );
    let food = random_food_set(config.seed, config.n_species, config.food_size);

    let (w, h) = config.grid;
    let mut grid = SpeciesGrid::new(w, h, 50.0);
    for &s in &food {
        for y in 0..h {
            for x in 0..w {
                grid.seed(x, y, s, config.initial_food_qe);
            }
        }
    }

    let mut scratch_cells: Vec<SpeciesCell> = Vec::with_capacity(grid.len());
    let mut damp_field: Vec<f32> = Vec::with_capacity(grid.len());
    let mut strength_field: Vec<f32> = Vec::with_capacity(grid.len());
    let mut mask = ClosureMembraneMask::new();

    let mut total_dissipated = 0.0_f32;
    let bw = REACTION_FREQ_BANDWIDTH_DEFAULT;

    // Tracking estado de las closures iniciales.
    let mut initial_hashes: Vec<u64> = Vec::new();
    let mut fates: Vec<ClosureFate> = Vec::new();
    let mut k_history: Vec<Vec<f32>> = Vec::new();
    let mut initial_snapshot_taken = false;
    let mut last_final_hashes: Vec<u64> = Vec::new();

    for tick in 0..config.ticks {
        total_dissipated += step_grid_reactions(&mut grid, &net, bw, config.dt);

        compute_membrane_field(&grid, mask.as_array(), 1.0, &mut damp_field);
        let damping = if mask.is_empty() { None } else { Some(damp_field.as_slice()) };
        diffuse_species(&mut grid, &mut scratch_cells, SPECIES_DIFFUSION_RATE, config.dt, damping);

        if tick % config.detection_every != 0 { continue; }

        let closures = raf_closures(&net, &food);
        let alive_hashes: Vec<u64> = closures.iter().map(|c| c.hash).collect();
        mask.clear();
        for c in &closures { mask.mark_closure_products(c, &net); }

        // Snapshot inicial tras equilibración.
        if !initial_snapshot_taken && tick >= config.equilibration_ticks {
            initial_hashes = alive_hashes.clone();
            fates = closures.iter().map(|c| ClosureFate {
                hash: c.hash,
                survived: false,
                pressure_events: 0,
                k_stability_mean_last: 0.0,
            }).collect();
            k_history = (0..fates.len()).map(|_| Vec::new()).collect();
            initial_snapshot_taken = true;
        }

        // Evaluar pressure sobre blobs actuales (proxy de fisión).
        let pressure_crossed = if !mask.is_empty() {
            compute_strength_field(&grid, mask.as_array(), 1.0, &mut strength_field);
            // Threshold: 5% del máximo observable ⇒ blob = agregación notable.
            let max_s = strength_field.iter().cloned().fold(0.0_f32, f32::max);
            let thr = max_s * 0.05;
            if thr > 0.0 {
                let blobs = find_blobs(&strength_field, w, h, thr);
                blobs.iter().any(|b| {
                    pressure_ratio(b, &grid, &net, &strength_field, bw) > FISSION_PRESSURE_RATIO
                })
            } else { false }
        } else { false };

        // Actualizar fates con closures vivas que pertenezcan al snapshot inicial.
        // k_stability se computa sobre el agregado bulk (totales por especie):
        // mide la reconstrucción vs decay a nivel de sopa, no de celda.
        if initial_snapshot_taken {
            let totals = grid.totals_per_species();
            let freq_ref = grid.cell(0, 0).freq;
            for (fate, hist) in fates.iter_mut().zip(k_history.iter_mut()) {
                if let Some(pos) = alive_hashes.iter().position(|&h| h == fate.hash) {
                    let k = kinetic_stability(&closures[pos], &totals, &net, freq_ref, bw);
                    hist.push(k);
                    if pressure_crossed { fate.pressure_events += 1; }
                }
            }
        }

        last_final_hashes = alive_hashes;
    }

    // Finalizar: supervivencia + media k_stability sobre última ventana.
    let window_samples =
        (config.last_window_ticks / config.detection_every.max(1)).max(1) as usize;
    for (i, fate) in fates.iter_mut().enumerate() {
        fate.survived = last_final_hashes.contains(&fate.hash);
        let h = &k_history[i];
        if !h.is_empty() {
            let n = window_samples.min(h.len());
            let slice = &h[h.len() - n..];
            fate.k_stability_mean_last = slice.iter().sum::<f32>() / slice.len() as f32;
        }
    }

    SoupReport {
        seed: config.seed,
        n_ticks: config.ticks,
        n_closures_initial: initial_hashes.len() as u32,
        n_closures_final: last_final_hashes.len() as u32,
        total_dissipated,
        fates,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── random_reaction_network ────────────────────────────────────────────

    #[test]
    fn generator_is_deterministic() {
        let a = random_reaction_network(42, 8, 10, 2, 2);
        let b = random_reaction_network(42, 8, 10, 2, 2);
        assert_eq!(a.len(), b.len());
        for (ra, rb) in a.reactions().iter().zip(b.reactions()) {
            assert!((ra.k - rb.k).abs() < 1e-6);
            assert!((ra.freq - rb.freq).abs() < 1e-6);
        }
    }

    #[test]
    fn generator_produces_well_formed_reactions() {
        let net = random_reaction_network(7, 6, 20, 2, 2);
        assert_eq!(net.len(), 20);
        for r in net.reactions() {
            assert!(r.is_well_formed());
            assert!(r.k > 0.0);
            assert!((30.0..=70.0).contains(&r.freq));
        }
    }

    #[test]
    fn generator_different_seeds_differ() {
        let a = random_reaction_network(1, 8, 8, 2, 2);
        let b = random_reaction_network(2, 8, 8, 2, 2);
        let k_a: Vec<f32> = a.reactions().iter().map(|r| r.k).collect();
        let k_b: Vec<f32> = b.reactions().iter().map(|r| r.k).collect();
        assert_ne!(k_a, k_b, "distinct seeds should produce distinct networks");
    }

    #[test]
    fn generator_clamps_params_defensively() {
        let net = random_reaction_network(0, 255, 5, 99, 99);
        // n_species clampeado a 32; max_* clampeado a MAX_* .
        assert_eq!(net.len(), 5);
        for r in net.reactions() {
            assert!(r.reactants_active().all(|e| (e.species.raw() as u8) < 32));
            assert!(r.products_active().all(|e| (e.species.raw() as u8) < 32));
        }
    }

    // ── random_food_set ────────────────────────────────────────────────────

    #[test]
    fn food_set_deterministic_and_distinct() {
        let a = random_food_set(42, 8, 3);
        let b = random_food_set(42, 8, 3);
        assert_eq!(a, b);
        assert_eq!(a.len(), 3);
        let mut raws: Vec<u8> = a.iter().map(|s| s.raw()).collect();
        raws.sort();
        raws.dedup();
        assert_eq!(raws.len(), 3, "food species should be distinct");
    }

    #[test]
    fn food_set_respects_bounds() {
        assert!(random_food_set(0, 0, 3).is_empty());
        assert_eq!(random_food_set(1, 4, 99).len(), 4, "k clamp to n_species");
    }

    // ── SoupReport serialization ───────────────────────────────────────────

    #[test]
    fn report_roundtrips_through_json() {
        let r = SoupReport {
            seed: 42,
            n_ticks: 1000,
            n_closures_initial: 2,
            n_closures_final: 1,
            total_dissipated: 12.5,
            fates: vec![ClosureFate {
                hash: 0xdeadbeef,
                survived: true,
                pressure_events: 3,
                k_stability_mean_last: 1.25,
            }],
        };
        let json = r.to_json().unwrap();
        let r2: SoupReport = serde_json::from_str(&json).unwrap();
        assert_eq!(r.seed, r2.seed);
        assert_eq!(r.fates, r2.fates);
    }

    #[test]
    fn default_report_is_empty_but_valid_json() {
        let r = SoupReport::default();
        let json = r.to_json().unwrap();
        assert!(json.contains("\"seed\""));
    }

    // ── run_soup harness ───────────────────────────────────────────────────

    fn fast_config(seed: u64) -> SoupConfig {
        SoupConfig {
            seed,
            n_species: 6,
            n_reactions: 12,
            food_size: 2,
            grid: (6, 6),
            ticks: 400,
            equilibration_ticks: 50,
            detection_every: 50,
            last_window_ticks: 200,
            initial_food_qe: 2.0,
            dt: 0.1,
        }
    }

    #[test]
    fn harness_is_deterministic() {
        let c = fast_config(11);
        let a = run_soup(&c);
        let b = run_soup(&c);
        assert_eq!(a.fates, b.fates);
        assert_eq!(a.n_closures_initial, b.n_closures_initial);
        assert!((a.total_dissipated - b.total_dissipated).abs() < 1e-3);
    }

    #[test]
    fn harness_conservation_is_monotone() {
        // Axiom 4: total_dissipated ≥ 0, monotónico cuando corremos más ticks.
        let mut short = fast_config(3);
        short.ticks = 100;
        let mut long = fast_config(3);
        long.ticks = 400;
        let r_short = run_soup(&short);
        let r_long = run_soup(&long);
        assert!(r_short.total_dissipated >= 0.0);
        assert!(r_long.total_dissipated >= r_short.total_dissipated);
    }

    #[test]
    fn harness_finalizes_fates_consistently() {
        let c = fast_config(5);
        let r = run_soup(&c);
        for f in &r.fates {
            // Invariant: si no sobrevivió, su k_stability_mean_last queda en 0
            // o positivo (nunca NaN/negativo).
            assert!(f.k_stability_mean_last.is_finite());
            assert!(f.k_stability_mean_last >= 0.0);
            // pressure_events ≤ total ticks / detection_every.
            let max_events = (c.ticks / c.detection_every) as u32;
            assert!(f.pressure_events <= max_events);
        }
    }

    #[test]
    fn harness_empty_food_produces_no_initial_closures() {
        // food_size=0 ⇒ ningún reactivo disponible ⇒ raf_closures vacía.
        let c = SoupConfig { food_size: 0, ..fast_config(9) };
        let r = run_soup(&c);
        assert_eq!(r.n_closures_initial, 0);
        assert!(r.fates.is_empty());
    }
}
