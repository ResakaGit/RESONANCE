//! M1/M3/M6: Macro-step analytical solvers for distant-entity LOD simulation.
//! M6: `normalize_score` — BridgeCache added as `CompetitionNormBridge` (key: `competition_norm`).
//!     Use `bridge_compute::<CompetitionNormBridge>` when called >100/tick.
//!     `exponential_decay` — O(1), inputs rarely repeat across ticks — no cache needed.

/// Energy after `n_ticks` ticks of exponential decay.
/// E(n) = E0 * (1 - rate)^n
pub fn exponential_decay(e0: f32, rate: f32, n_ticks: u32) -> f32 {
    if n_ticks == 0 {
        return e0;
    }
    let r = rate.clamp(0.0, 1.0 - f32::EPSILON);
    e0 * (1.0 - r).powf(n_ticks as f32)
}

/// Radius after `n_ticks` ticks of allometric growth towards `r_max`.
/// r(n) = r_max - (r_max - r0) * exp(-k * n)
pub fn allometric_radius(r0: f32, r_max: f32, k: f32, n_ticks: u32) -> f32 {
    let r0 = r0.clamp(0.0, r_max);
    let k = k.max(f32::EPSILON);
    r_max - (r_max - r0) * (-k * n_ticks as f32).exp()
}

/// Ticks until energy reaches `threshold` under exponential decay.
/// n = ceil(log(threshold / E0) / log(1 - rate))
/// Returns `u32::MAX` if already at or below threshold, or rate == 0.
pub fn ticks_until_threshold(e0: f32, threshold: f32, rate: f32) -> u32 {
    if e0 <= threshold || rate == 0.0 {
        return u32::MAX;
    }
    let r = rate.clamp(0.0, 1.0 - f32::EPSILON);
    let n = (threshold / e0).ln() / (1.0 - r).ln();
    n.ceil() as u32
}

/// Relative error |euler - exact| / E0 between n-step discrete Euler and
/// continuous exact solution E0 * exp(-rate * n).
/// Discrete Euler: E0 * (1 - rate)^n.  Exact continuous: E0 * exp(-rate * n).
pub fn euler_vs_exponential_error(e0: f32, rate: f32, n_ticks: u32) -> f32 {
    if e0 == 0.0 {
        return 0.0;
    }
    let r = rate.clamp(0.0, 1.0 - f32::EPSILON);
    let n = n_ticks as f32;
    let euler = e0 * (1.0 - r).powf(n);
    let exact = e0 * (-r * n).exp();
    (euler - exact).abs() / e0
}

/// M3-A: Inverse of exponential_decay — finds n_ticks to reach `target_qe` from `e0`.
/// Returns 0 if already below or at target. Uses ln() — no NaN guard needed for positive inputs.
/// `n = ceil(ln(e0 / target_qe) / rate)` — returned as u32.
pub fn ticks_to_reach(e0: f32, target_qe: f32, rate: f32) -> u32 {
    if e0 <= target_qe || rate <= 0.0 {
        return 0;
    }
    ((e0 / target_qe.max(1e-9)).ln() / rate.max(1e-9)).ceil() as u32
}

/// M3-B: Normalization barrier — maps raw_score in [0, ∞) → [0, 1).
/// Uses logistic: `1 / (1 + exp(-k * (x - midpoint)))` shifted to [0,1).
/// Prevents runaway scores in competitive pool metrics.
pub fn normalize_score(raw_score: f32, midpoint: f32, k: f32) -> f32 {
    let k_safe = k.max(1e-6);
    (1.0 / (1.0 + (-(k_safe * (raw_score - midpoint))).exp())).min(1.0 - f32::EPSILON)
}

/// M3-C: Inverse barrier — given a normalized score in (0,1), recover raw_score.
/// `raw = midpoint - ln((1/n) - 1) / k`
/// Returns `midpoint` if n is out of (0,1) to avoid ln domain error.
pub fn inverse_normalize_score(normalized: f32, midpoint: f32, k: f32) -> f32 {
    if normalized <= 0.0 || normalized >= 1.0 {
        return midpoint;
    }
    let k_safe = k.max(1e-6);
    midpoint - ((1.0 / normalized - 1.0).ln()) / k_safe
}

/// M3-D: Decay rate needed to go from `e0` to `target` in exactly `n_ticks`.
/// `rate = ln(e0 / target) / n_ticks` — returns 0 if n_ticks == 0 or inputs invalid.
pub fn required_decay_rate(e0: f32, target: f32, n_ticks: u32) -> f32 {
    if n_ticks == 0 || e0 <= 0.0 || target <= 0.0 || target >= e0 {
        return 0.0;
    }
    (e0 / target).ln() / n_ticks as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_zero_ticks_returns_e0() {
        assert_eq!(exponential_decay(100.0, 0.1, 0), 100.0);
    }

    #[test]
    fn decay_full_rate_goes_to_zero() {
        // rate clamped to 1-epsilon, after many ticks result is near zero
        let result = exponential_decay(100.0, 1.0, 100);
        assert!(result < 1e-10, "expected near-zero, got {result}");
    }

    #[test]
    fn allometric_converges_to_r_max() {
        let r = allometric_radius(0.0, 10.0, 0.5, 50);
        assert!((r - 10.0).abs() < 1e-5, "expected ~10.0, got {r}");
    }

    #[test]
    fn ticks_until_threshold_finite_rate() {
        let n = ticks_until_threshold(100.0, 50.0, 0.1);
        // Verify E(n) <= threshold and E(n-1) > threshold
        assert!(exponential_decay(100.0, 0.1, n) <= 50.0);
        assert!(n > 0 && exponential_decay(100.0, 0.1, n - 1) > 50.0);
    }

    #[test]
    fn euler_error_small_for_small_rate() {
        let err = euler_vs_exponential_error(1.0, 0.01, 10);
        assert!(err < 0.005, "expected small error, got {err}");
    }

    // M3 inverse solvers
    #[test]
    fn ticks_to_reach_nominal() {
        // e0=100, target=50, rate=0.01 → ~70 ticks
        let t = ticks_to_reach(100.0, 50.0, 0.01);
        assert!(t > 0 && t < 200, "got {t}");
    }

    #[test]
    fn ticks_to_reach_already_below() {
        assert_eq!(ticks_to_reach(10.0, 50.0, 0.01), 0);
    }

    #[test]
    fn normalize_score_midpoint_is_half() {
        let v = normalize_score(5.0, 5.0, 1.0);
        assert!((v - 0.5).abs() < 1e-5, "got {v}");
    }

    #[test]
    fn normalize_score_in_range() {
        for x in [0.0_f32, 1.0, 5.0, 10.0, 100.0] {
            let v = normalize_score(x, 5.0, 1.0);
            assert!(v >= 0.0 && v <= 1.0, "out of range: {v}");
        }
    }

    #[test]
    fn inverse_normalize_roundtrip() {
        let raw = 7.3_f32;
        let n = normalize_score(raw, 5.0, 2.0);
        let back = inverse_normalize_score(n, 5.0, 2.0);
        assert!(
            (back - raw).abs() < 1e-3,
            "roundtrip failed: {back} vs {raw}"
        );
    }

    #[test]
    fn inverse_normalize_out_of_range_returns_midpoint() {
        assert!((inverse_normalize_score(0.0, 5.0, 1.0) - 5.0).abs() < 1e-5);
        assert!((inverse_normalize_score(1.0, 5.0, 1.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn required_decay_rate_nominal() {
        let rate = required_decay_rate(100.0, 50.0, 100);
        // exponential_decay(100, rate, 100) should ≈ 50
        let result = 100.0_f32 * (-rate * 100.0).exp();
        assert!((result - 50.0).abs() < 0.1, "got {result}");
    }

    #[test]
    fn required_decay_rate_zero_ticks_returns_zero() {
        assert_eq!(required_decay_rate(100.0, 50.0, 0), 0.0);
    }
}
