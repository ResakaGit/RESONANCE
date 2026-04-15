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
    MAX_PRODUCTS_PER_REACTION, MAX_REACTANTS_PER_REACTION,
};
use crate::blueprint::equations::determinism::{next_u64, range_f32};
use crate::layers::reaction::SpeciesId;
use crate::layers::reaction_network::{
    ReactionNetwork, ReactionNetworkSpec, ReactionSpec, StoichSpec,
};

pub mod soup_sim;
pub use soup_sim::SoupSim;

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
        // Modulo sobre u64 completo: evita usar los low-8-bits (peor calidad PCG).
        let n_react = 1 + (next() % max_reactants as u64) as u8;
        let n_prod  = 1 + (next() % max_products  as u64) as u8;

        let reactants = pick_unique_species(&mut next, n_species, n_react);
        let products  = pick_unique_species(&mut next, n_species, n_prod);

        let k    = range_f32(next(), 0.2, 1.5);
        let freq = range_f32(next(), 30.0, 70.0);

        reactions.push(ReactionSpec { reactants, products, k, freq });
    }
    ReactionNetwork::from_spec(ReactionNetworkSpec { reactions })
        .expect("random_reaction_network inputs are pre-clamped — spec must be valid")
}

/// Selección determinística de `k` especies distintas del rango `[0, n_species)`.
/// `k` se clampa a `min(k, n_species)`.
pub fn random_food_set(seed: u64, n_species: u8, k: usize) -> Vec<SpeciesId> {
    let mut state = next_u64(seed.wrapping_mul(0xD9E8_21B4_A5F3_CC01));
    let mut next = || { state = next_u64(state); state };

    let (pool, k) = fisher_yates_prefix(&mut next, n_species, k as u8);
    pool.into_iter().take(k)
        .map(|s| SpeciesId::new(s).expect("s < n_species ≤ 32 ⇒ SpeciesId válido"))
        .collect()
}

// Selección de `count` species distintas ∈ [0, n_species), con stoich 1 (mass-action).
fn pick_unique_species(
    next: &mut impl FnMut() -> u64,
    n_species: u8,
    count: u8,
) -> Vec<StoichSpec> {
    let (pool, k) = fisher_yates_prefix(next, n_species, count);
    pool.into_iter().take(k).map(|s| StoichSpec(s, 1)).collect()
}

/// Fisher-Yates parcial on-stack: devuelve `[u8; 32]` con los primeros `k`
/// slots permutados de `[0, n_species)`.  Elimina la alloc heap que había en
/// `pick_unique_species` + `random_food_set` (se llama 2× por reacción aleatoria).
/// Devuelve `(pool, k_efectivo)` con `k_efectivo = min(k, n_species, 32)`.
fn fisher_yates_prefix(
    next: &mut impl FnMut() -> u64,
    n_species: u8,
    k: u8,
) -> ([u8; 32], usize) {
    let n = n_species.min(32) as usize;
    let k = (k as usize).min(n);
    let mut pool = [0u8; 32];
    for (i, slot) in pool.iter_mut().take(n).enumerate() { *slot = i as u8; }
    for i in 0..k {
        let j = i + (next() % (n - i) as u64) as usize;
        pool.swap(i, j);
    }
    (pool, k)
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

/// Tracking interno por closure inicial: fate + historial de `k_stability`
/// indexado por detección.  Agrupa lo que antes eran parallel arrays
/// (`fates` + `k_history`) acoplados por índice — propensos a desync.
/// No serializable: el reporte público sólo expone `fates`.
pub(crate) struct FateTrack {
    pub(crate) fate: ClosureFate,
    pub(crate) k_history: Vec<f32>,
}

/// Registro de un evento de fisión observado durante la simulación (ADR-041).
/// `parent == 0` ⇒ el blob que fisionó era sopa primordial (ningún linaje lo
/// dominaba).  `children` contiene los dos linajes hijos generados por
/// `apply_fission`, siempre en orden `[side=0, side=1]`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FissionEventRecord {
    pub tick: u64,
    pub parent: u64,
    pub children: [u64; 2],
    pub dissipated_qe: f32,
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
    /// Eventos de fisión registrados por el harness (ADR-041).  `#[serde(default)]`
    /// preserva compatibilidad con reports JSON pre-AP-6c.
    #[serde(default)]
    pub fission_events: Vec<FissionEventRecord>,
}

impl SoupReport {
    /// Serializa a JSON compacto — para CI artifacts / tracking inter-build.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Exporta el snapshot de closures como grafo DOT (Graphviz).
    /// Exports the closure snapshot as a DOT (Graphviz) graph.
    ///
    /// Nodo por `ClosureFate`.  Color verde = sobrevivió, rojo = murió.
    /// Edge por `FissionEventRecord`: `parent → child` etiquetado con el tick.
    /// Label de nodo: `h=<hash8> k=<kstab> p=<pressure>`.
    pub fn to_dot(&self) -> String {
        let mut out = String::with_capacity(
            160 + self.fates.len() * 96 + self.fission_events.len() * 64,
        );
        out.push_str("digraph autopoiesis {\n");
        out.push_str("  rankdir=LR;\n");
        out.push_str("  node [shape=circle, style=filled, fontname=\"monospace\"];\n");
        out.push_str(&format!(
            "  label=\"seed={} ticks={} dissipated={:.3}\\ninitial={} final={} fissions={}\";\n",
            self.seed, self.n_ticks, self.total_dissipated,
            self.n_closures_initial, self.n_closures_final, self.fission_events.len(),
        ));
        for fate in &self.fates {
            let color = if fate.survived { "palegreen" } else { "lightcoral" };
            out.push_str(&format!(
                "  c{:016x} [fillcolor={}, label=\"h={:08x}\\nk={:.2}\\np={}\"];\n",
                fate.hash, color, (fate.hash & 0xFFFF_FFFF) as u32,
                fate.k_stability_mean_last, fate.pressure_events,
            ));
        }
        for ev in &self.fission_events {
            // Nodos child declarados explícitamente — puede no haber un
            // `ClosureFate` para ellos (nacen tras el snapshot inicial).
            for child in ev.children {
                out.push_str(&format!(
                    "  c{:016x} [fillcolor=lightblue, label=\"h={:08x}\\nborn@t{}\"];\n",
                    child, (child & 0xFFFF_FFFF) as u32, ev.tick,
                ));
                out.push_str(&format!(
                    "  c{:016x} -> c{:016x} [label=\"t{}\"];\n",
                    ev.parent, child, ev.tick,
                ));
            }
        }
        out.push_str("}\n");
        out
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
    /// Modo de siembra inicial del food:
    ///   - `None`        ⇒ uniforme (legacy; AP-5 proptest lo asume).
    ///   - `Some(r)`     ⇒ spot centrado `(2r+1)×(2r+1)`.  Rompe la simetría
    ///     traslacional — requisito estructural para que emerjan gradientes,
    ///     blobs y fisiones (AP-6 items 2 + 3).
    ///
    /// Un grid homogéneo produce `local_gradient = 0` en todas las celdas,
    /// por lo que `strength_field ≡ 0` y `find_blobs` retorna vacío.  Sin
    /// blobs no hay `pressure_ratio` que evaluar → `fission_events` queda
    /// vacío por construcción.
    pub food_spot_radius: Option<usize>,
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
            food_spot_radius: None,
        }
    }
}

/// Ejecuta una sopa determinística con red generada aleatoriamente a partir
/// de `config.seed`.  Pure fn: mismas entradas ⇒ mismo output.
pub fn run_soup(config: &SoupConfig) -> SoupReport {
    let net = random_reaction_network(
        config.seed,
        config.n_species,
        config.n_reactions,
        MAX_REACTANTS_PER_REACTION as u8,
        MAX_PRODUCTS_PER_REACTION as u8,
    );
    run_soup_with_network(config, net)
}

/// Variante de `run_soup` que recibe una `ReactionNetwork` pre-construida
/// (p. ej. cargada desde un RON via `ReactionNetwork::from_ron_str`).
/// El food set se sigue derivando determinísticamente de `config.seed`.
///
/// AP-6b: desacopla la generación de red del harness para permitir validar
/// redes canónicas (RAF mínima, formose, GARD) y sopas reproducibles.
/// AP-6c (ADR-040): delega en `SoupSim::run_to_end` — la lógica del stepper
/// vive en `soup_sim.rs`.  Este wrapper preserva la firma pública estable.
pub fn run_soup_with_network(config: &SoupConfig, net: ReactionNetwork) -> SoupReport {
    SoupSim::new(config.clone(), net).run_to_end()
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
            fission_events: vec![FissionEventRecord {
                tick: 250,
                parent: 0xAAAA,
                children: [0xBBBB, 0xCCCC],
                dissipated_qe: 0.75,
            }],
        };
        let json = r.to_json().unwrap();
        let r2: SoupReport = serde_json::from_str(&json).unwrap();
        assert_eq!(r.seed, r2.seed);
        assert_eq!(r.fates, r2.fates);
        assert_eq!(r.fission_events, r2.fission_events);
    }

    #[test]
    fn report_legacy_json_without_fission_events_still_parses() {
        // Regresión ADR-041 §4: `#[serde(default)]` permite leer snapshots
        // generados antes de AP-6c.
        let legacy = r#"{"seed":1,"n_ticks":100,"n_closures_initial":0,
            "n_closures_final":0,"total_dissipated":0.0,"fates":[]}"#;
        let r: SoupReport = serde_json::from_str(legacy).unwrap();
        assert_eq!(r.seed, 1);
        assert!(r.fission_events.is_empty());
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
            food_spot_radius: None,
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

    // ── to_dot (AP-6a) ─────────────────────────────────────────────────────

    #[test]
    fn dot_export_empty_report_is_well_formed() {
        let r = SoupReport::default();
        let dot = r.to_dot();
        assert!(dot.starts_with("digraph autopoiesis {"));
        assert!(dot.trim_end().ends_with('}'));
        assert!(dot.contains("seed=0"));
        // Sin nodos de closures (fates vacía).
        assert!(!dot.contains("fillcolor="));
    }

    #[test]
    fn dot_export_colors_nodes_by_survival() {
        let r = SoupReport {
            seed: 7, n_ticks: 100, n_closures_initial: 2, n_closures_final: 1,
            total_dissipated: 1.5,
            fates: vec![
                ClosureFate { hash: 0xABCD, survived: true,  pressure_events: 2, k_stability_mean_last: 1.3 },
                ClosureFate { hash: 0x1234, survived: false, pressure_events: 0, k_stability_mean_last: 0.4 },
            ],
            fission_events: Vec::new(),
        };
        let dot = r.to_dot();
        assert!(dot.contains("c000000000000abcd"));
        assert!(dot.contains("c0000000000001234"));
        assert!(dot.contains("palegreen"));
        assert!(dot.contains("lightcoral"));
        assert!(dot.contains("k=1.30"));
        assert!(dot.contains("p=2"));
    }

    // ── AP-6b2: canonical asset RONs ──────────────────────────────────────

    fn load_asset(path: &str) -> ReactionNetwork {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("asset {path}: {e}"));
        ReactionNetwork::from_ron_str(&text)
            .unwrap_or_else(|e| panic!("parse {path}: {e:?}"))
    }

    #[test]
    fn asset_formose_loads_with_four_reactions() {
        let net = load_asset("assets/reactions/formose.ron");
        assert_eq!(net.len(), 4, "Breslow 1959 cycle has 4 reactions");
        assert!(net.reactions().iter().all(|r| r.is_well_formed()));
        // r3 es el paso autocatalítico: 1 reactivo (tetrose), 1 producto
        // (glycolaldehyde) con coef 2.  Verificamos la duplicación.
        let r3 = &net.reactions()[3];
        let prods: Vec<u8> = r3.products.iter()
            .filter(|e| e.is_active()).map(|e| e.count).collect();
        assert!(prods.contains(&2), "r3 must double glycolaldehyde (coef=2)");
    }

    #[test]
    fn asset_hypercycle_loads_with_four_closing_reactions() {
        let net = load_asset("assets/reactions/hypercycle.ron");
        assert_eq!(net.len(), 4, "4-member Eigen-Schuster hypercycle");
        assert!(net.reactions().iter().all(|r| r.is_well_formed()));
        // Todas las k iguales (no rate-bias).
        let k0 = net.reactions()[0].k;
        assert!(net.reactions().iter().all(|r| (r.k - k0).abs() < 1e-6));
    }

    #[test]
    fn asset_raf_minimal_still_loads() {
        // Regresión: el asset legacy de AP-0 sigue parseando.
        let net = load_asset("assets/reactions/raf_minimal.ron");
        assert_eq!(net.len(), 3);
    }

    #[test]
    fn formose_runs_without_panic_via_run_soup_with_network() {
        let net = load_asset("assets/reactions/formose.ron");
        let cfg = SoupConfig {
            seed: 42, n_species: 4, food_size: 1,
            ticks: 200, grid: (6, 6), ..SoupConfig::default()
        };
        let r = run_soup_with_network(&cfg, net);
        assert_eq!(r.n_ticks, 200);
        assert!(r.total_dissipated >= 0.0);
    }

    #[test]
    fn hypercycle_runs_without_panic_via_run_soup_with_network() {
        let net = load_asset("assets/reactions/hypercycle.ron");
        let cfg = SoupConfig {
            seed: 7, n_species: 5, food_size: 1,
            ticks: 200, grid: (6, 6), ..SoupConfig::default()
        };
        let r = run_soup_with_network(&cfg, net);
        assert_eq!(r.n_ticks, 200);
        assert!(r.total_dissipated >= 0.0);
    }

    #[test]
    fn dot_export_from_run_soup_is_nonempty_and_balanced() {
        let c = fast_config(123);
        let r = run_soup(&c);
        let dot = r.to_dot();
        // Graph es válido independientemente de si hubo closures.
        let opens = dot.matches('{').count();
        let closes = dot.matches('}').count();
        assert_eq!(opens, closes);
        assert!(dot.contains(&format!("seed={}", c.seed)));
    }
}
