//! AP-0: Mass-action kinetics + difusión Laplaciana sobre `SpeciesGrid`.
//! AP-0: Mass-action kinetics + Laplacian diffusion over `SpeciesGrid`.
//!
//! Funciones 100% stateless: reciben datos por referencia, retornan datos.
//! La única mutación autorizada es in-place sobre argumentos `&mut` explícitos.
//! No hay caches ocultos, no hay globals, no hay RNG.
//!
//! Axiom 1: reacciones transforman qe entre canales (especies).
//! Axiom 2: `apply_reaction` garantiza `Σ qe_out ≤ Σ qe_in` (pool invariant).
//! Axiom 4: `REACTION_EFFICIENCY < 1` ⇒ cada paso disipa.
//! Axiom 5: `ReactionOutcome::dissipated` reportado para trazabilidad.
//! Axiom 7: difusión atenúa con distancia (Laplaciano 4-vecino).
//! Axiom 8: `mass_action_rate` modula por alineación Gaussiana de frecuencia.

use crate::blueprint::constants::chemistry::{
    DIFFUSION_CFL_MAX, MAX_SPECIES, REACTION_EFFICIENCY,
};
use crate::layers::reaction::Reaction;
use crate::layers::reaction_network::ReactionNetwork;
use crate::layers::species_grid::{SpeciesCell, SpeciesGrid};

// ── Outcome type ────────────────────────────────────────────────────────────

/// Resultado de aplicar UNA reacción en UNA celda durante UN tick.
/// Named struct en vez de tupla — el contrato queda en la firma, no en el
/// cuerpo de la función.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ReactionOutcome {
    /// qe disipado al ambiente (≥ 0). Axiom 4.
    pub dissipated: f32,
    /// Extensión aplicada: moles de reacción consumidos (≥ 0).
    /// `extent == 0` ⇔ ninguna reacción ocurrió (reactivos insuficientes o rate=0).
    pub extent: f32,
}

impl ReactionOutcome {
    pub const NONE: Self = Self { dissipated: 0.0, extent: 0.0 };
    #[inline] pub fn occurred(self) -> bool { self.extent > 0.0 }
}

// ── Frequency alignment (Axiom 8) ───────────────────────────────────────────

/// Alineación Gaussiana `exp(-½ ((f1 − f2) / bw)²)`.  Pura, determinista, ∈ [0, 1].
/// `bw` clampeado a `f32::EPSILON` para evitar división por cero.
/// Inputs no-finitos ⇒ retorna `0.0`.
#[inline]
pub fn frequency_alignment(f1: f32, f2: f32, bandwidth: f32) -> f32 {
    if !f1.is_finite() || !f2.is_finite() { return 0.0; }
    let bw = bandwidth.max(f32::EPSILON);
    let df = (f1 - f2) / bw;
    (-0.5 * df * df).exp()
}

// ── Mass-action rate ────────────────────────────────────────────────────────

/// Tasa de reacción en una celda.  Axiom 8: factor de alineación.
/// Retorna `0.0` si algún reactivo está a concentración cero/negativa/NaN.
pub fn mass_action_rate(
    species: &[f32; MAX_SPECIES],
    reaction: &Reaction,
    cell_freq: f32,
    bandwidth: f32,
) -> f32 {
    let mut rate = reaction.k;
    for e in reaction.reactants_active() {
        let c = species[e.species.index()];
        if !c.is_finite() || c <= 0.0 { return 0.0; }
        rate *= match e.count {
            1 => c,
            2 => c * c,
            n => c.powi(n as i32),
        };
    }
    rate * frequency_alignment(reaction.freq, cell_freq, bandwidth)
}

// ── Reaction extent (mass-conserving step) ──────────────────────────────────

/// Mayor `extent` que puede aplicarse sin dejar reactivos negativos.
#[inline]
fn limit_extent(species: &[f32; MAX_SPECIES], reaction: &Reaction, desired: f32) -> f32 {
    let mut max_extent = desired.max(0.0);
    for e in reaction.reactants_active() {
        let avail = species[e.species.index()];
        if e.count > 0 {
            let cap = avail / e.count as f32;
            if cap < max_extent { max_extent = cap; }
        }
    }
    max_extent
}

/// Aplica UNA reacción sobre una celda durante `dt` segundos.
///
/// # Conservación enforced (Axiom 2 + 4 + 5)
///
/// ```text
/// consumed    = extent × Σ stoich_reactants
/// producible  = consumed × REACTION_EFFICIENCY            (cota superior)
/// product[s] += producible × (stoich_product[s] / Σ stoich_products)
/// dissipated  = consumed − producible  ≥ 0
/// ```
///
/// Los coeficientes del lado producto expresan **reparto relativo**, no
/// creación neta.  Así una reacción del tipo `C → A + D` reparte el `qe`
/// consumido al 50/50 entre A y D, sin violar conservación.  Si se quisiera
/// "duplicar masa", habría que duplicar también el lado reactivo —
/// fundamentalmente imposible bajo Axiom 2.
pub fn apply_reaction(
    species: &mut [f32; MAX_SPECIES],
    reaction: &Reaction,
    cell_freq: f32,
    bandwidth: f32,
    dt: f32,
) -> ReactionOutcome {
    if dt <= 0.0 { return ReactionOutcome::NONE; }
    let rate = mass_action_rate(species, reaction, cell_freq, bandwidth);
    if rate <= 0.0 { return ReactionOutcome::NONE; }
    let extent = limit_extent(species, reaction, rate * dt);
    if extent <= 0.0 { return ReactionOutcome::NONE; }

    let total_product_stoich: f32 =
        reaction.products_active().map(|e| e.count as f32).sum();
    if total_product_stoich <= 0.0 { return ReactionOutcome::NONE; }

    let mut consumed = 0.0_f32;
    for e in reaction.reactants_active() {
        let amount = extent * e.count as f32;
        species[e.species.index()] -= amount;
        consumed += amount;
    }

    let producible = consumed * REACTION_EFFICIENCY;
    let mut produced = 0.0_f32;
    for e in reaction.products_active() {
        let share = e.count as f32 / total_product_stoich;
        let amount = producible * share;
        species[e.species.index()] += amount;
        produced += amount;
    }

    let dissipated = (consumed - produced).max(0.0);
    debug_assert!(
        produced <= consumed + 1e-5,
        "Axiom 2 violated: consumed={consumed} produced={produced}",
    );
    ReactionOutcome { dissipated, extent }
}

/// Aplica todas las reacciones de la red a una celda durante `dt`.
/// Retorna qe total disipado en la celda.
pub fn step_cell_reactions(
    species: &mut [f32; MAX_SPECIES],
    network: &ReactionNetwork,
    cell_freq: f32,
    bandwidth: f32,
    dt: f32,
) -> f32 {
    let mut total_dissipated = 0.0_f32;
    for r in network.reactions() {
        total_dissipated += apply_reaction(species, r, cell_freq, bandwidth, dt).dissipated;
    }
    total_dissipated
}

/// Itera todas las celdas del grid aplicando la red.  Retorna qe total disipado.
/// `bandwidth` es explícito — no hay default oculto.  Caller decide.
pub fn step_grid_reactions(
    grid: &mut SpeciesGrid,
    network: &ReactionNetwork,
    bandwidth: f32,
    dt: f32,
) -> f32 {
    let mut total = 0.0_f32;
    for cell in grid.cells_mut() {
        total += step_cell_reactions(&mut cell.species, network, cell.freq, bandwidth, dt);
    }
    total
}

// ── Diffusion (Axiom 7) ─────────────────────────────────────────────────────

/// Difusión Laplaciana 4-vecino, condición de borde reflectiva (celdas fantasma
/// igualan al centro → flux cero en el borde → preserva masa).
///
/// `scratch` es un buffer reutilizable (patrón `ScratchPad`); se rellena desde
/// el grid y se descarta — sin heap churn por tick si el caller lo mantiene.
/// `rate × dt` se clampa a `DIFFUSION_CFL_MAX = 0.25` (estabilidad CFL).
pub fn diffuse_species(
    grid: &mut SpeciesGrid,
    scratch: &mut Vec<SpeciesCell>,
    rate: f32,
    dt: f32,
) {
    let r = (rate * dt).clamp(0.0, DIFFUSION_CFL_MAX);
    if r <= 0.0 || grid.is_empty() { return; }

    scratch.clear();
    scratch.extend_from_slice(grid.cells());

    let w = grid.width();
    let h = grid.height();
    for (i, out) in grid.cells_mut().iter_mut().enumerate() {
        let y = i / w;
        let x = i % w;
        let c = &scratch[i];
        let xm = if x > 0     { &scratch[i - 1] } else { c };
        let xp = if x + 1 < w { &scratch[i + 1] } else { c };
        let ym = if y > 0     { &scratch[i - w] } else { c };
        let yp = if y + 1 < h { &scratch[i + w] } else { c };
        for s in 0..MAX_SPECIES {
            let lap = xm.species[s] + xp.species[s] + ym.species[s] + yp.species[s]
                    - 4.0 * c.species[s];
            out.species[s] = c.species[s] + r * lap;
        }
        // freq no difunde — propiedad ambiental fija.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::chemistry::REACTION_FREQ_BANDWIDTH_DEFAULT as BW;
    use crate::layers::reaction::{SpeciesId, StoichEntry};
    use crate::layers::reaction_network::ReactionNetwork;

    fn zero_species() -> [f32; MAX_SPECIES] { [0.0; MAX_SPECIES] }

    fn rx(reactants: &[(u8, u8)], products: &[(u8, u8)], k: f32, freq: f32) -> Reaction {
        let mut r = Reaction::default();
        for (slot, &(s, c)) in r.reactants.iter_mut().zip(reactants.iter()) {
            *slot = StoichEntry::new(s, c).expect("valid stoich");
        }
        for (slot, &(s, c)) in r.products.iter_mut().zip(products.iter()) {
            *slot = StoichEntry::new(s, c).expect("valid stoich");
        }
        r.k = k;
        r.freq = freq;
        r
    }

    // ── frequency_alignment ────────────────────────────────────────────────

    #[test]
    fn alignment_peaks_at_matching_frequencies() {
        assert!((frequency_alignment(50.0, 50.0, 10.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn alignment_decays_with_distance() {
        let near = frequency_alignment(50.0, 55.0, 10.0);
        let far  = frequency_alignment(50.0, 80.0, 10.0);
        assert!(near > far);
    }

    #[test]
    fn alignment_is_bounded_zero_one() {
        for f1 in [0.0_f32, 10.0, 100.0] {
            for f2 in [0.0_f32, 25.0, 100.0, 1000.0] {
                let a = frequency_alignment(f1, f2, 50.0);
                assert!((0.0..=1.0).contains(&a), "a={a}");
            }
        }
    }

    #[test]
    fn alignment_handles_non_finite_inputs() {
        assert_eq!(frequency_alignment(f32::NAN, 50.0, 10.0), 0.0);
        assert_eq!(frequency_alignment(50.0, f32::INFINITY, 10.0), 0.0);
    }

    #[test]
    fn alignment_handles_zero_bandwidth() {
        // Clamp a f32::EPSILON — no panic, resultado finito.
        let a = frequency_alignment(50.0, 50.0, 0.0);
        assert!(a.is_finite() && a >= 0.0);
    }

    // ── mass_action_rate ───────────────────────────────────────────────────

    #[test]
    fn rate_zero_without_reactants() {
        let r = rx(&[(0, 1), (1, 1)], &[(2, 1)], 1.0, 50.0);
        assert_eq!(mass_action_rate(&zero_species(), &r, 50.0, BW), 0.0);
    }

    #[test]
    fn rate_proportional_to_concentrations() {
        let r = rx(&[(0, 1), (1, 1)], &[(2, 1)], 2.0, 50.0);
        let mut s = zero_species();
        s[0] = 3.0; s[1] = 4.0;
        // k × [A] × [B] × alignment(50,50) = 2 × 3 × 4 × 1 = 24.
        assert!((mass_action_rate(&s, &r, 50.0, BW) - 24.0).abs() < 1e-5);
    }

    #[test]
    fn rate_modulates_by_frequency_alignment() {
        let r = rx(&[(0, 1)], &[(1, 1)], 1.0, 50.0);
        let mut s = zero_species();
        s[0] = 1.0;
        let aligned    = mass_action_rate(&s, &r, 50.0,  BW);
        let misaligned = mass_action_rate(&s, &r, 200.0, BW);
        assert!(aligned > misaligned);
    }

    #[test]
    fn rate_uses_stoich_power() {
        let r = rx(&[(0, 2)], &[(1, 1)], 1.0, 50.0);
        let mut s = zero_species();
        s[0] = 3.0;
        assert!((mass_action_rate(&s, &r, 50.0, BW) - 9.0).abs() < 1e-5);
    }

    // ── apply_reaction ─────────────────────────────────────────────────────

    #[test]
    fn outcome_none_sentinel_is_inactive() {
        assert!(!ReactionOutcome::NONE.occurred());
        assert_eq!(ReactionOutcome::default(), ReactionOutcome::NONE);
    }

    #[test]
    fn apply_conserves_qe_modulo_dissipation() {
        let r = rx(&[(0, 1), (1, 1)], &[(2, 1)], 1.0, 50.0);
        let mut s = zero_species();
        s[0] = 10.0; s[1] = 10.0;
        let before: f32 = s.iter().sum();
        let out = apply_reaction(&mut s, &r, 50.0, BW, 0.1);
        let after: f32 = s.iter().sum();
        assert!(out.occurred());
        assert!((before - after - out.dissipated).abs() < 1e-4);
        assert!(out.dissipated > 0.0, "Axiom 4: always dissipates");
    }

    #[test]
    fn apply_respects_reactant_availability() {
        let r = rx(&[(0, 1), (1, 1)], &[(2, 1)], 1000.0, 50.0);
        let mut s = zero_species();
        s[0] = 0.5; s[1] = 10.0;
        let _ = apply_reaction(&mut s, &r, 50.0, BW, 1.0);
        assert!(s[0] >= -1e-6);
        assert!(s[1] >= -1e-6);
    }

    #[test]
    fn apply_noop_on_zero_dt() {
        let r = rx(&[(0, 1)], &[(1, 1)], 1.0, 50.0);
        let mut s = zero_species();
        s[0] = 5.0;
        assert_eq!(apply_reaction(&mut s, &r, 50.0, BW, 0.0), ReactionOutcome::NONE);
        assert_eq!(s[0], 5.0);
    }

    #[test]
    fn apply_noop_when_rate_zero() {
        let r = rx(&[(0, 1)], &[(1, 1)], 1.0, 50.0);
        let mut s = zero_species();
        assert_eq!(apply_reaction(&mut s, &r, 50.0, BW, 1.0), ReactionOutcome::NONE);
    }

    #[test]
    fn step_grid_aggregates_dissipation() {
        let spec = r#"(reactions: [
            (reactants: [(0,1)], products: [(1,1)], k: 1.0, freq: 50.0),
        ])"#;
        let net = ReactionNetwork::from_ron_str(spec).unwrap();
        let mut g = SpeciesGrid::new(2, 2, 50.0);
        for y in 0..2 { for x in 0..2 {
            g.seed(x, y, SpeciesId::new(0).unwrap(), 1.0);
        }}
        let pre = g.total_qe();
        let d = step_grid_reactions(&mut g, &net, BW, 0.1);
        let post = g.total_qe();
        assert!((pre - post - d).abs() < 1e-4);
        assert!(d > 0.0);
    }

    // ── diffusion ──────────────────────────────────────────────────────────

    #[test]
    fn diffuse_conserves_mass() {
        let mut g = SpeciesGrid::new(4, 4, 0.0);
        g.seed(1, 1, SpeciesId::new(0).unwrap(), 4.0);
        let pre = g.total_qe();
        let mut scratch = Vec::new();
        for _ in 0..20 {
            diffuse_species(&mut g, &mut scratch, 0.1, 1.0);
        }
        assert!((pre - g.total_qe()).abs() < 1e-4);
    }

    #[test]
    fn diffuse_reduces_variance() {
        let mut g = SpeciesGrid::new(5, 5, 0.0);
        g.seed(2, 2, SpeciesId::new(0).unwrap(), 10.0);
        let variance = |g: &SpeciesGrid| -> f32 {
            let vals: Vec<f32> = g.cells().iter().map(|c| c.species[0]).collect();
            let m = vals.iter().sum::<f32>() / vals.len() as f32;
            vals.iter().map(|v| (v - m).powi(2)).sum::<f32>() / vals.len() as f32
        };
        let v_before = variance(&g);
        let mut scratch = Vec::new();
        for _ in 0..30 { diffuse_species(&mut g, &mut scratch, 0.1, 1.0); }
        let v_after = variance(&g);
        assert!(v_after < v_before);
    }

    #[test]
    fn diffuse_noop_on_zero_rate() {
        let mut g = SpeciesGrid::new(3, 3, 0.0);
        g.seed(1, 1, SpeciesId::new(0).unwrap(), 5.0);
        let snapshot: Vec<f32> = g.cells().iter().map(|c| c.species[0]).collect();
        let mut scratch = Vec::new();
        diffuse_species(&mut g, &mut scratch, 0.0, 1.0);
        let after: Vec<f32> = g.cells().iter().map(|c| c.species[0]).collect();
        assert_eq!(snapshot, after);
    }

    #[test]
    fn diffuse_clamps_to_cfl() {
        let mut g = SpeciesGrid::new(3, 3, 0.0);
        g.seed(1, 1, SpeciesId::new(0).unwrap(), 1.0);
        let mut scratch = Vec::new();
        diffuse_species(&mut g, &mut scratch, 1000.0, 1000.0);
        for c in g.cells() { assert!(c.species[0].is_finite()); }
    }

    #[test]
    fn diffuse_scratch_is_reusable() {
        let mut g = SpeciesGrid::new(3, 3, 0.0);
        g.seed(1, 1, SpeciesId::new(0).unwrap(), 3.0);
        let mut scratch = Vec::new();
        for _ in 0..5 {
            diffuse_species(&mut g, &mut scratch, 0.1, 1.0);
            // El buffer se rellena cada vez — capacidad ≥ n_cells tras primer uso.
            assert!(scratch.len() == g.len() || scratch.is_empty() == false);
        }
    }
}
