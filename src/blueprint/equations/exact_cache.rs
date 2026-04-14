//! Funciones puras para optimización de cache sin pérdida de precisión.
//! Pure functions for zero-precision-loss cache optimization.
//!
//! Cada función precomputa un valor exacto que los sistemas almacenan como componente
//! para evitar recómputo per-tick de operaciones caras (`powf`, `exp`, `sqrt`).

use crate::blueprint::equations::derived_thresholds as dt;

// ─── Kleiber volume factor ──────────────────────────────────────────────────

/// Factor de volumen Kleiber: `radius^0.75`. Precomputable on growth event.
/// Kleiber volume factor: `radius^0.75`. Precomputable on growth event.
///
/// Sublineal y compresivo: 1% de error en radius → 0.75% en output.
/// Non-finite inputs (NaN, Inf) se tratan como 0.
#[inline]
pub fn kleiber_volume_factor(radius: f32) -> f32 {
    if !radius.is_finite() {
        return 0.0;
    }
    radius.max(0.0).powf(dt::KLEIBER_EXPONENT)
}

// ─── Gompertz exact death tick ──────────────────────────────────────────────

/// Tick exacto de muerte por Gompertz: resuelve `S(t) = exp(-2)` algebraicamente.
/// Exact Gompertz death tick: solves `S(t) = exp(-2)` algebraically.
///
/// `S(t) = exp(-base×t - 0.5×coeff×t²) = exp(-2)`
/// → `base×t + 0.5×coeff×t² = 2`
/// → `t = (-base + √(base² + 4×coeff)) / coeff`   (fórmula cuadrática)
///
/// Retorna `birth_tick + min(t_gompertz, max_viable_age)`.
#[inline]
pub fn exact_death_tick(
    birth_tick: u64,
    base_dissipation: f32,
    senescence_coeff: f32,
    max_viable_age: u64,
) -> u64 {
    if senescence_coeff <= 0.0 || !senescence_coeff.is_finite() {
        return birth_tick.saturating_add(max_viable_age);
    }
    if !base_dissipation.is_finite() || base_dissipation < 0.0 {
        return birth_tick.saturating_add(max_viable_age);
    }

    // base×t + 0.5×coeff×t² = 2  →  0.5×coeff×t² + base×t - 2 = 0
    // a = 0.5×coeff,  b = base,  c = -2
    // discriminant = base² + 4×0.5×coeff×2 = base² + 4×coeff
    let disc = base_dissipation * base_dissipation + 4.0 * senescence_coeff;
    if disc < 0.0 {
        return birth_tick.saturating_add(max_viable_age);
    }

    let t = (-base_dissipation + disc.sqrt()) / senescence_coeff;
    if !t.is_finite() || t < 0.0 {
        return birth_tick.saturating_add(max_viable_age);
    }

    let age_ticks = (t as u64).min(max_viable_age);
    birth_tick.saturating_add(age_ticks)
}

// ─── Frequency alignment (stateless, for lookup table population) ───────────

/// Alignment gaussiano entre dos frecuencias. `exp(-Δf²/(2×bw²))`.
/// Gaussian alignment between two frequencies. `exp(-Δf²/(2×bw²))`.
///
/// Para lookup tables: computar una vez por par, reusar indefinidamente.
#[inline]
pub fn frequency_alignment_exact(freq_a: f32, freq_b: f32, bandwidth: f32) -> f32 {
    if !freq_a.is_finite() || !freq_b.is_finite() || !bandwidth.is_finite() || bandwidth <= 0.0 {
        return 0.0;
    }
    let delta = freq_a - freq_b;
    let sigma_sq = bandwidth * bandwidth * 2.0;
    (-delta * delta / sigma_sq).exp()
}

// ─── Shape optimization input hash (for Converged<MorphogenesisShapeParams>) ──

/// Hash determinista de los inputs de shape optimization para convergence detection.
/// Deterministic hash of shape optimization inputs for convergence detection.
///
/// Knuth multiplicative hash sobre bits exactos de f32. Zero allocation.
/// Colisión cuando dos conjuntos de inputs distintos producen el mismo u64 — rate < 1/2^32.
#[inline]
pub fn hash_shape_inputs(density: f32, velocity: f32, radius: f32, vasc_cost: f32) -> u64 {
    use crate::layers::converged::hash_f32;
    const KNUTH_PHI: u64 = 2_654_435_761;
    let mut h = hash_f32(density);
    h = h.wrapping_mul(KNUTH_PHI).wrapping_add(hash_f32(velocity));
    h = h.wrapping_mul(KNUTH_PHI).wrapping_add(hash_f32(radius));
    h.wrapping_mul(KNUTH_PHI).wrapping_add(hash_f32(vasc_cost))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── kleiber_volume_factor ──

    #[test]
    fn kleiber_volume_factor_unit_radius_returns_one() {
        assert!((kleiber_volume_factor(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn kleiber_volume_factor_zero_radius_returns_zero() {
        assert_eq!(kleiber_volume_factor(0.0), 0.0);
    }

    #[test]
    fn kleiber_volume_factor_negative_radius_clamps_to_zero() {
        assert_eq!(kleiber_volume_factor(-5.0), 0.0);
    }

    #[test]
    fn kleiber_volume_factor_large_radius_sublinear() {
        let f2 = kleiber_volume_factor(2.0);
        // 2^0.75 ≈ 1.6818
        assert!((f2 - 2.0_f32.powf(0.75)).abs() < 1e-5);
        assert!(f2 < 2.0, "sublinear: f(2) < 2");
    }

    #[test]
    fn kleiber_volume_factor_nan_returns_zero() {
        assert_eq!(kleiber_volume_factor(f32::NAN), 0.0);
    }

    #[test]
    fn kleiber_volume_factor_infinity_returns_zero() {
        assert_eq!(kleiber_volume_factor(f32::INFINITY), 0.0);
    }

    #[test]
    fn kleiber_volume_factor_neg_infinity_returns_zero() {
        assert_eq!(kleiber_volume_factor(f32::NEG_INFINITY), 0.0);
    }

    // ── exact_death_tick ──

    #[test]
    fn exact_death_tick_fauna_within_max_age() {
        let coeff = dt::senescence_coeff_fauna();
        let max_age = dt::max_age_fauna();
        let tick = exact_death_tick(0, coeff, coeff, max_age);
        assert!(
            tick <= max_age,
            "death_tick {tick} should not exceed max_age {max_age}"
        );
        assert!(tick > 0, "fauna should survive at least 1 tick");
    }

    #[test]
    fn exact_death_tick_materialized_within_max_age() {
        let coeff = dt::senescence_coeff_materialized();
        let max_age = dt::max_age_materialized();
        let tick = exact_death_tick(0, coeff, coeff, max_age);
        assert!(tick <= max_age);
        assert!(tick > 0);
    }

    #[test]
    fn exact_death_tick_flora_within_max_age() {
        let coeff = dt::senescence_coeff_flora();
        let max_age = dt::max_age_flora();
        let tick = exact_death_tick(0, coeff, coeff, max_age);
        assert!(tick <= max_age);
        assert!(tick > 0);
    }

    #[test]
    fn exact_death_tick_zero_coeff_returns_birth_plus_max() {
        assert_eq!(exact_death_tick(100, 0.01, 0.0, 500), 600);
    }

    #[test]
    fn exact_death_tick_negative_coeff_returns_birth_plus_max() {
        assert_eq!(exact_death_tick(100, 0.01, -1.0, 500), 600);
    }

    #[test]
    fn exact_death_tick_nan_coeff_returns_birth_plus_max() {
        assert_eq!(exact_death_tick(100, 0.01, f32::NAN, 500), 600);
    }

    #[test]
    fn exact_death_tick_nan_base_returns_birth_plus_max() {
        assert_eq!(exact_death_tick(100, f32::NAN, 0.02, 500), 600);
    }

    #[test]
    fn exact_death_tick_birth_offset_applied() {
        let tick_a = exact_death_tick(0, 0.02, 0.02, 200);
        let tick_b = exact_death_tick(1000, 0.02, 0.02, 200);
        assert_eq!(
            tick_b - tick_a,
            1000,
            "birth offset should shift death_tick by same amount"
        );
    }

    #[test]
    fn exact_death_tick_saturating_on_overflow() {
        let tick = exact_death_tick(u64::MAX - 10, 0.02, 0.02, 200);
        // Should not panic; saturating_add prevents overflow
        assert!(tick >= u64::MAX - 10);
    }

    // ── frequency_alignment_exact ──

    #[test]
    fn frequency_alignment_same_freq_returns_one() {
        let a = frequency_alignment_exact(100.0, 100.0, 50.0);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn frequency_alignment_symmetric() {
        let ab = frequency_alignment_exact(100.0, 200.0, 50.0);
        let ba = frequency_alignment_exact(200.0, 100.0, 50.0);
        assert!((ab - ba).abs() < 1e-6);
    }

    #[test]
    fn frequency_alignment_one_bandwidth_apart() {
        let a = frequency_alignment_exact(100.0, 150.0, 50.0);
        // exp(-50²/5000) = exp(-0.5) ≈ 0.6065
        assert!((a - 0.6065).abs() < 0.01);
    }

    #[test]
    fn frequency_alignment_far_apart_near_zero() {
        let a = frequency_alignment_exact(100.0, 400.0, 50.0);
        assert!(
            a < 0.01,
            "300 Hz apart with 50 Hz bandwidth should be near zero, got {a}"
        );
    }

    #[test]
    fn frequency_alignment_zero_bandwidth_returns_zero() {
        assert_eq!(frequency_alignment_exact(100.0, 100.0, 0.0), 0.0);
    }

    #[test]
    fn frequency_alignment_negative_bandwidth_returns_zero() {
        assert_eq!(frequency_alignment_exact(100.0, 100.0, -50.0), 0.0);
    }

    #[test]
    fn frequency_alignment_nan_bandwidth_returns_zero() {
        assert_eq!(frequency_alignment_exact(100.0, 100.0, f32::NAN), 0.0);
    }

    #[test]
    fn frequency_alignment_result_in_unit_interval() {
        let a = frequency_alignment_exact(100.0, 100.0, 50.0);
        assert!(a >= 0.0 && a <= 1.0);
    }

    #[test]
    fn frequency_alignment_nan_freq_returns_zero() {
        assert_eq!(frequency_alignment_exact(f32::NAN, 100.0, 50.0), 0.0);
        assert_eq!(frequency_alignment_exact(100.0, f32::NAN, 50.0), 0.0);
    }

    #[test]
    fn frequency_alignment_inf_freq_returns_zero() {
        assert_eq!(frequency_alignment_exact(f32::INFINITY, 100.0, 50.0), 0.0);
    }

    // ── hash_shape_inputs ──

    #[test]
    fn hash_shape_inputs_deterministic() {
        let a = hash_shape_inputs(1.0, 2.0, 3.0, 4.0);
        let b = hash_shape_inputs(1.0, 2.0, 3.0, 4.0);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_shape_inputs_different_inputs_differ() {
        let a = hash_shape_inputs(1.0, 2.0, 3.0, 4.0);
        let b = hash_shape_inputs(1.0, 2.0, 3.0, 4.1);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_shape_inputs_order_matters() {
        let a = hash_shape_inputs(1.0, 2.0, 3.0, 4.0);
        let b = hash_shape_inputs(4.0, 3.0, 2.0, 1.0);
        assert_ne!(a, b);
    }
}
