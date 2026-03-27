//! AC-1: Metabolic Interference Factor — Axiom 3 × Axiom 8.
//!
//! "interaction magnitude = base × interference_factor"
//! Applied to metabolic extraction (predation, photosynthesis) — not to spell catalysis.
//!
//! **Semantic difference from catalysis interference:**
//! - Catalysis (spells): factor ∈ [-1, 1] — destructive interference = active damage.
//! - Metabolic extraction: factor ∈ [FLOOR, 1.0] — destructive interference = reduced access.
//!   An extractor never "gives back" energy because it failed to resonate; it just extracts less.
//!
//! SSOT: calls `core_physics::interference` — does not inline the cosine.

use crate::blueprint::constants::METABOLIC_INTERFERENCE_FLOOR;
use crate::blueprint::equations::core_physics;

/// Metabolic access factor from oscillatory alignment between extractor and target.
///
/// Equation: `cos(2π × |Δfreq| × t + Δphase).clamp(FLOOR, 1.0)`
///
/// Range: `[METABOLIC_INTERFERENCE_FLOOR, 1.0]`
/// - `1.0` → perfect resonance — full extraction efficiency
/// - `FLOOR` → destructive or orthogonal — minimal extraction (not zero — basal friction)
///
/// `t` should be `SimulationElapsed.secs` for deterministic results.
pub fn metabolic_interference_factor(
    extractor_freq: f32, extractor_phase: f32,
    target_freq:    f32, target_phase:    f32,
    t: f32,
) -> f32 {
    let raw = core_physics::interference(extractor_freq, extractor_phase, target_freq, target_phase, t);
    raw.clamp(METABOLIC_INTERFERENCE_FLOOR, 1.0)
}

/// Apply the metabolic interference factor to a raw extraction quantity.
///
/// Invariant: `result ≤ raw` (factor ∈ [FLOOR, 1.0] never amplifies).
/// Invariant: `result ≥ 0.0`.
#[inline]
pub fn apply_metabolic_interference(raw: f32, factor: f32) -> f32 {
    (raw * factor).max(0.0)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::METABOLIC_INTERFERENCE_FLOOR;
    use std::f32::consts::PI;

    const EPS: f32 = 1e-5;

    // ── metabolic_interference_factor ─────────────────────────────────────────

    #[test]
    fn factor_same_freq_same_phase_at_t0_is_one() {
        let f = metabolic_interference_factor(75.0, 0.0, 75.0, 0.0, 0.0);
        assert!((f - 1.0).abs() < EPS, "got {f}");
    }

    #[test]
    fn factor_same_freq_opposite_phase_is_floor() {
        // cos(0 + π) = -1 → clamped to FLOOR
        let f = metabolic_interference_factor(75.0, 0.0, 75.0, PI, 0.0);
        assert!((f - METABOLIC_INTERFERENCE_FLOOR).abs() < EPS, "got {f}");
    }

    #[test]
    fn factor_always_in_floor_to_one_range() {
        // Cross-band: Terra(75) vs Lux(1000) — should vary but stay in range
        for t_val in [0.0_f32, 0.001, 0.01, 0.1, 1.0, 10.0] {
            let f = metabolic_interference_factor(75.0, 0.0, 1000.0, 0.0, t_val);
            assert!(
                f >= METABOLIC_INTERFERENCE_FLOOR - EPS && f <= 1.0 + EPS,
                "out of range: f={f} at t={t_val}"
            );
        }
    }

    #[test]
    fn factor_never_below_floor() {
        // Worst-case: max destructive interference
        let f = metabolic_interference_factor(0.0, 0.0, 0.0, PI, 0.0);
        assert!(f >= METABOLIC_INTERFERENCE_FLOOR, "floor violated: {f}");
    }

    #[test]
    fn factor_cross_band_terra_vs_lux_not_full() {
        // Terra(75Hz) extracting from Lux(1000Hz) — at t=0, phase=0:
        // interference = cos(2π × 925 × 0 + 0) = cos(0) = 1.0 at t=0.
        // At t=0.001: cos(2π × 925 × 0.001) ≈ cos(5.81) ≈ 0.84 — varies.
        // The key is that the time-average approaches FLOOR (not guaranteed per-tick).
        // Just verify it stays in range for a variety of t values.
        let values: Vec<f32> = (0..100)
            .map(|i| metabolic_interference_factor(75.0, 0.0, 1000.0, 0.0, i as f32 * 0.001))
            .collect();
        let below_one_count = values.iter().filter(|&&v| v < 1.0 - EPS).count();
        // At many points the factor should be < 1.0 due to rapid oscillation
        assert!(below_one_count > 5, "expected some non-maximal values, got {below_one_count}/100");
    }

    #[test]
    fn factor_same_band_close_freqs_often_near_one() {
        // Terra sub-band: 72 vs 78 Hz — small gap, slow oscillation → stays near 1 for small t
        let f = metabolic_interference_factor(72.0, 0.0, 78.0, 0.0, 0.001);
        assert!(f > 0.9, "close freqs should have high factor at small t: {f}");
    }

    // ── apply_metabolic_interference ──────────────────────────────────────────

    #[test]
    fn apply_factor_one_returns_raw() {
        assert!((apply_metabolic_interference(100.0, 1.0) - 100.0).abs() < EPS);
    }

    #[test]
    fn apply_factor_floor_returns_floor_times_raw() {
        let expected = 100.0 * METABOLIC_INTERFERENCE_FLOOR;
        assert!((apply_metabolic_interference(100.0, METABOLIC_INTERFERENCE_FLOOR) - expected).abs() < EPS);
    }

    #[test]
    fn apply_factor_zero_raw_returns_zero() {
        assert_eq!(apply_metabolic_interference(0.0, 1.0), 0.0);
    }

    #[test]
    fn apply_result_never_exceeds_raw() {
        // factor is always ≤ 1.0 by construction
        for factor in [0.0, METABOLIC_INTERFERENCE_FLOOR, 0.5, 1.0] {
            let result = apply_metabolic_interference(50.0, factor);
            assert!(result <= 50.0 + EPS, "factor={factor} result={result}");
        }
    }

    #[test]
    fn apply_result_always_non_negative() {
        assert!(apply_metabolic_interference(0.0, METABOLIC_INTERFERENCE_FLOOR) >= 0.0);
        assert!(apply_metabolic_interference(100.0, 0.0) >= 0.0);
    }

    // ── round-trip: factor → apply ────────────────────────────────────────────

    #[test]
    fn round_trip_same_freq_preserves_full_extraction() {
        let raw = 80.0_f32;
        let factor = metabolic_interference_factor(75.0, 0.0, 75.0, 0.0, 0.0);
        let result = apply_metabolic_interference(raw, factor);
        assert!((result - raw).abs() < EPS, "expected {raw} got {result}");
    }

    #[test]
    fn round_trip_opposite_phase_limits_to_floor() {
        let raw = 100.0_f32;
        let factor = metabolic_interference_factor(75.0, 0.0, 75.0, PI, 0.0);
        let result = apply_metabolic_interference(raw, factor);
        let expected_max = raw * METABOLIC_INTERFERENCE_FLOOR + EPS;
        assert!(result <= expected_max, "result={result} expected≤{expected_max}");
        assert!(result >= 0.0);
    }
}
