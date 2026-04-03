//! R5 — Sensitivity and Uncertainty: integration tests.
//! Validates pure math functions for parameter sensitivity analysis.
//! No Bevy, no RNG — deterministic by construction.

use resonance::blueprint::equations::sensitivity::{
    coefficient_of_variation, confidence_band, is_unstable, normalized_sensitivity,
    parameter_sweep_16, partial_sensitivity, rank_by_sensitivity_16,
};

#[test]
fn partial_sensitivity_linear_fn_returns_slope() {
    // f(x) = 2x → central diff at x=5, delta=0.1:
    // perturbed = f(5.1) - f(4.9) = 10.2 - 9.8 = 0.4 → 0.4 / (2*0.1) = 2.0
    let delta = 0.1f32;
    let base_out = 2.0f32 * 4.9; // f(x - delta)
    let pert_out = 2.0f32 * 5.1; // f(x + delta)
    let s = partial_sensitivity(base_out, pert_out, delta);
    assert!((s - 2.0).abs() < 1e-4, "expected ≈2.0, got {s}");
}

#[test]
fn normalized_sensitivity_clamps_zero_nominal() {
    // nominal=0.0 → must not panic, returns 0.0 (guarded)
    let s = normalized_sensitivity(5.0, 0.0);
    assert!(
        s == 0.0 || s.is_infinite(),
        "got {s}; must be 0.0 or INFINITY, must not panic"
    );
}

#[test]
fn parameter_sweep_returns_ordered_pairs() {
    // f(x) = x*x from 1..3 in 4 steps → outputs must be monotone increasing
    let pairs = parameter_sweep_16(1.0, 3.0, 4, |x| x * x);
    for i in 1..4 {
        assert!(
            pairs[i].1 >= pairs[i - 1].1,
            "pairs[{i}].output={} must be >= pairs[{}].output={}",
            pairs[i].1,
            i - 1,
            pairs[i - 1].1,
        );
    }
}

#[test]
fn confidence_band_symmetric_around_mean() {
    // All values equal → std=0 → band collapses to (mean, mean)
    let (low, high): (f32, f32) = confidence_band(&[10.0, 10.0, 10.0], 2.0);
    assert!((low - 10.0).abs() < 1e-5, "low={low} expected≈10.0");
    assert!((high - 10.0).abs() < 1e-5, "high={high} expected≈10.0");
}

#[test]
fn coefficient_of_variation_constant_series_zero() {
    let cv = coefficient_of_variation(&[5.0, 5.0, 5.0, 5.0]);
    assert!(
        cv.abs() < 1e-5,
        "cv of constant series must be ≈0.0, got {cv}"
    );
}

#[test]
fn is_unstable_high_variance_returns_true() {
    // Values span 3 orders of magnitude → cv is far above threshold=1.0
    assert!(
        is_unstable(&[0.1, 100.0, 0.1], 1.0),
        "high-variance series must be flagged as unstable"
    );
}

#[test]
fn rank_by_sensitivity_orders_descending() {
    let mut params = [("a", 0.5f32), ("b", 2.0f32), ("c", 1.0f32)];
    rank_by_sensitivity_16(&mut params);
    assert_eq!(params[0].0, "b", "highest |sensitivity| must be first");
    assert_eq!(params[1].0, "c");
    assert_eq!(params[2].0, "a");
}
