//! Multi-scale state inference — axiom-constrained generation of child states
//! from parent observables (ADR-036, §D2).
//!
//! Pure functions. Zero side effects. Deterministic by seed.
//!
//! Axiom compliance:
//! - **Axiom 2 (Pool Invariant):** `sum(children.qe) <= parent.qe`
//! - **Axiom 4 (Dissipation):** zoom costs energy; `factor = 1 - dissipation_rate`
//! - **Axiom 5 (Conservation):** total qe monotone decreases
//! - **Axiom 8 (Oscillatory):** child frequencies sampled from `N(parent.freq, bandwidth)`

use crate::blueprint::equations::determinism;
use crate::blueprint::equations::derived_thresholds::{
    DISSIPATION_GAS, DISSIPATION_LIQUID, DISSIPATION_PLASMA, DISSIPATION_SOLID, KLEIBER_EXPONENT,
};

// ─── Matter state (local enum to avoid cross-module dependency) ──────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InferenceMatterState { Solid, Liquid, Gas, Plasma }

impl InferenceMatterState {
    /// Dissipation rate asociada al estado.
    pub fn dissipation(self) -> f64 {
        (match self {
            Self::Solid => DISSIPATION_SOLID,
            Self::Liquid => DISSIPATION_LIQUID,
            Self::Gas => DISSIPATION_GAS,
            Self::Plasma => DISSIPATION_PLASMA,
        }) as f64
    }
}

// ─── Child count (Kleiber scaling) ────────────────────────────────────────

/// Cuántas entidades hijas caben en un padre de energía `parent_qe`.
///
/// Kleiber: `N ∝ qe^0.75`. La constante de proporcionalidad depende de la escala
/// (más entidades pequeñas en escalas finas, menos entidades grandes en cosmológica).
#[inline]
pub fn kleiber_child_count(parent_qe: f64, scale_factor: f64, min: usize, max: usize) -> usize {
    if parent_qe <= 0.0 { return 0; }
    let raw = scale_factor * parent_qe.powf(KLEIBER_EXPONENT as f64);
    (raw.round() as usize).clamp(min, max)
}

// ─── Energy distribution (Pool Invariant) ─────────────────────────────────

/// Distribuir qe del padre entre N hijos respetando Pool Invariant + Dissipation.
///
/// Algoritmo: generar N pesos uniformes u_i ∈ (0,1), normalizar a suma 1,
/// multiplicar por `parent_qe × (1 - dissipation)`.
///
/// **Garantía:** `sum(result) <= parent_qe × (1 - dissipation) < parent_qe`.
pub fn distribute_energy(
    parent_qe: f64,
    n_children: usize,
    state: InferenceMatterState,
    seed: u64,
) -> Vec<f64> {
    if n_children == 0 || parent_qe <= 0.0 { return Vec::new(); }

    let available = parent_qe * (1.0 - state.dissipation());
    let mut weights = vec![0.0_f64; n_children];
    let mut rng = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut sum = 0.0;
    for w in weights.iter_mut() {
        rng = determinism::next_u64(rng);
        // Uniform (0,1] — evitar 0 exacto
        let u = ((rng as f64) / (u64::MAX as f64)).max(1e-12);
        *w = u;
        sum += u;
    }
    let scale = available / sum;
    for w in weights.iter_mut() {
        *w *= scale;
    }
    weights
}

// ─── Frequency distribution (Axiom 8) ─────────────────────────────────────

/// Frecuencias de hijos: `freq_i ~ N(parent_freq, bandwidth)`.
///
/// Usa Box-Muller via `gaussian_f32` (suficiente precisión; la dispersión
/// es observacional, no numérica).
pub fn distribute_frequencies(
    parent_freq: f64,
    n_children: usize,
    bandwidth: f64,
    seed: u64,
) -> Vec<f64> {
    let mut out = Vec::with_capacity(n_children);
    let mut rng = seed.wrapping_add(0xBF58_476D_1CE4_E5B9);
    for _ in 0..n_children {
        rng = determinism::next_u64(rng);
        let z = determinism::gaussian_f32(rng, bandwidth as f32) as f64;
        out.push((parent_freq + z).max(0.0));
    }
    out
}

// ─── Position distribution ────────────────────────────────────────────────

/// Posiciones iniciales uniformes dentro de una esfera de radio `parent_radius`
/// centrada en `parent_pos`.
///
/// Muestreo por rejection sampling (uniforme en volumen).
pub fn distribute_positions_3d(
    parent_pos: [f64; 3],
    parent_radius: f64,
    n_children: usize,
    seed: u64,
) -> Vec<[f64; 3]> {
    let mut out = Vec::with_capacity(n_children);
    let mut rng = seed.wrapping_add(0x94D0_49BB_1331_11EB);

    for _ in 0..n_children {
        // Rejection sampling: muestrear en cubo [-r,r]^3, aceptar si dentro de esfera.
        // Max ~50 intentos (probabilidad de fallar 50 veces ≈ 0.48^50 ≈ 0).
        let mut point = [0.0; 3];
        for _ in 0..64 {
            let mut candidate = [0.0_f64; 3];
            let mut r_sq = 0.0;
            for d in 0..3 {
                rng = determinism::next_u64(rng);
                let u = (rng as f64) / (u64::MAX as f64);
                candidate[d] = (u * 2.0 - 1.0) * parent_radius;
                r_sq += candidate[d] * candidate[d];
            }
            if r_sq <= parent_radius * parent_radius {
                point = candidate;
                break;
            }
        }
        out.push([
            parent_pos[0] + point[0],
            parent_pos[1] + point[1],
            parent_pos[2] + point[2],
        ]);
    }
    out
}

// ─── Aggregation (zoom-out) ───────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct AggregateState {
    pub qe: f64,
    pub frequency_hz: f64,
    pub position: [f64; 3],
    pub radius: f64,
}

/// Agregar hijos en estado del padre.
///
/// - `qe` = suma
/// - `frequency_hz` = media ponderada por qe (dominante)
/// - `position` = centroide ponderado por qe
/// - `radius` = max distancia del centroide a un hijo
pub fn aggregate_to_parent(qes: &[f64], freqs: &[f64], positions: &[[f64; 3]]) -> AggregateState {
    let n = qes.len();
    debug_assert_eq!(freqs.len(), n);
    debug_assert_eq!(positions.len(), n);

    if n == 0 {
        return AggregateState {
            qe: 0.0,
            frequency_hz: 0.0,
            position: [0.0; 3],
            radius: 0.0,
        };
    }

    let total_qe: f64 = qes.iter().sum();
    if total_qe <= 0.0 {
        return AggregateState {
            qe: 0.0,
            frequency_hz: 0.0,
            position: [0.0; 3],
            radius: 0.0,
        };
    }

    let freq: f64 = qes.iter().zip(freqs).map(|(q, f)| q * f).sum::<f64>() / total_qe;

    let mut centroid = [0.0_f64; 3];
    for (q, p) in qes.iter().zip(positions) {
        for d in 0..3 {
            centroid[d] += q * p[d];
        }
    }
    for d in 0..3 {
        centroid[d] /= total_qe;
    }

    let radius = positions.iter().map(|p| {
        let dx = p[0] - centroid[0];
        let dy = p[1] - centroid[1];
        let dz = p[2] - centroid[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }).fold(0.0_f64, f64::max);

    AggregateState { qe: total_qe, frequency_hz: freq, position: centroid, radius }
}

// ─── Verification helpers ─────────────────────────────────────────────────

/// Pool Invariant: `sum(children) <= parent × (1 - dissipation)`.
pub fn verify_pool_invariant(parent_qe: f64, children_qe: &[f64], state: InferenceMatterState) -> bool {
    let max_allowed = parent_qe * (1.0 - state.dissipation()) + 1e-9;
    children_qe.iter().sum::<f64>() <= max_allowed
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kleiber_count_scales_with_qe() {
        let small = kleiber_child_count(100.0, 1.0, 1, 100);
        let large = kleiber_child_count(10000.0, 1.0, 1, 1000);
        assert!(large > small);
    }

    #[test]
    fn kleiber_count_respects_min_max() {
        assert_eq!(kleiber_child_count(0.1, 1.0, 3, 100), 3);
        assert_eq!(kleiber_child_count(1e9, 1.0, 1, 20), 20);
    }

    #[test]
    fn kleiber_count_zero_for_zero_qe() {
        assert_eq!(kleiber_child_count(0.0, 1.0, 3, 100), 0);
    }

    #[test]
    fn distribute_energy_respects_pool_invariant() {
        let parent_qe = 1000.0;
        let children = distribute_energy(parent_qe, 10, InferenceMatterState::Liquid, 42);
        assert_eq!(children.len(), 10);
        let sum: f64 = children.iter().sum();
        let max_allowed = parent_qe * (1.0 - DISSIPATION_LIQUID as f64);
        assert!(sum <= max_allowed + 1e-6, "sum={sum} max_allowed={max_allowed}");
        // All non-negative
        for c in &children {
            assert!(*c >= 0.0);
        }
    }

    #[test]
    fn distribute_energy_deterministic_with_seed() {
        let a = distribute_energy(500.0, 5, InferenceMatterState::Solid, 123);
        let b = distribute_energy(500.0, 5, InferenceMatterState::Solid, 123);
        assert_eq!(a, b);
    }

    #[test]
    fn distribute_energy_different_seed_different_result() {
        let a = distribute_energy(500.0, 5, InferenceMatterState::Solid, 1);
        let b = distribute_energy(500.0, 5, InferenceMatterState::Solid, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn distribute_energy_applies_dissipation() {
        let parent = 1000.0;
        let solid = distribute_energy(parent, 20, InferenceMatterState::Solid, 42);
        let plasma = distribute_energy(parent, 20, InferenceMatterState::Plasma, 42);
        let sum_solid: f64 = solid.iter().sum();
        let sum_plasma: f64 = plasma.iter().sum();
        assert!(sum_plasma < sum_solid, "Plasma should dissipate more than Solid");
    }

    #[test]
    fn distribute_frequencies_centered_on_parent() {
        let freqs = distribute_frequencies(100.0, 1000, 10.0, 42);
        let mean: f64 = freqs.iter().sum::<f64>() / freqs.len() as f64;
        assert!((mean - 100.0).abs() < 2.0, "mean freq {mean} far from parent 100");
    }

    #[test]
    fn distribute_frequencies_within_bandwidth_sigma() {
        let freqs = distribute_frequencies(200.0, 10000, 10.0, 42);
        let mean: f64 = freqs.iter().sum::<f64>() / freqs.len() as f64;
        let var: f64 = freqs.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / freqs.len() as f64;
        let stddev = var.sqrt();
        // Box-Muller gaussian_f32 con sigma=bandwidth debe dar stddev ~ bandwidth
        assert!((stddev - 10.0).abs() < 2.0, "stddev {stddev} not near bandwidth 10");
    }

    #[test]
    fn distribute_frequencies_no_negative() {
        let freqs = distribute_frequencies(5.0, 100, 50.0, 42);
        for f in &freqs {
            assert!(*f >= 0.0, "negative frequency {f}");
        }
    }

    #[test]
    fn distribute_positions_within_sphere() {
        let parent = [10.0, 20.0, 30.0];
        let r = 5.0;
        let positions = distribute_positions_3d(parent, r, 100, 42);
        for p in &positions {
            let dx = p[0] - parent[0];
            let dy = p[1] - parent[1];
            let dz = p[2] - parent[2];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            assert!(d <= r + 1e-6, "position at distance {d} > radius {r}");
        }
    }

    #[test]
    fn aggregate_qe_is_sum() {
        let qes = vec![10.0, 20.0, 30.0];
        let freqs = vec![100.0; 3];
        let positions = vec![[0.0; 3]; 3];
        let agg = aggregate_to_parent(&qes, &freqs, &positions);
        assert!((agg.qe - 60.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_weighted_frequency_mean() {
        let qes = vec![10.0, 90.0];
        let freqs = vec![0.0, 100.0];
        let positions = vec![[0.0; 3]; 2];
        let agg = aggregate_to_parent(&qes, &freqs, &positions);
        // Mean weighted: (10*0 + 90*100) / 100 = 90
        assert!((agg.frequency_hz - 90.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_empty_input_safe() {
        let agg = aggregate_to_parent(&[], &[], &[]);
        assert_eq!(agg.qe, 0.0);
        assert_eq!(agg.radius, 0.0);
    }

    #[test]
    fn pool_invariant_verification() {
        assert!(verify_pool_invariant(1000.0, &[100.0, 200.0, 300.0], InferenceMatterState::Solid));
        assert!(!verify_pool_invariant(100.0, &[50.0, 60.0], InferenceMatterState::Solid));
    }

    #[test]
    fn round_trip_zoom_preserves_conservation() {
        // Parent → children via distribute → aggregate → parent'
        let parent_qe = 500.0;
        let parent_freq = 80.0;
        let parent_pos = [0.0, 0.0, 0.0];
        let parent_radius = 10.0;

        let children_qe = distribute_energy(parent_qe, 8, InferenceMatterState::Liquid, 42);
        let children_freq = distribute_frequencies(parent_freq, 8, 5.0, 42);
        let children_pos = distribute_positions_3d(parent_pos, parent_radius, 8, 42);

        let agg = aggregate_to_parent(&children_qe, &children_freq, &children_pos);

        // qe decrece (dissipation)
        assert!(agg.qe < parent_qe, "aggregated qe {} should be < parent {}", agg.qe, parent_qe);
        assert!(agg.qe > 0.0);
        // freq cercana al padre
        assert!((agg.frequency_hz - parent_freq).abs() < 20.0);
        // radius: max distance from centroid to any child, bounded by diameter.
        assert!(agg.radius <= 2.0 * parent_radius + 1e-6);
    }
}
