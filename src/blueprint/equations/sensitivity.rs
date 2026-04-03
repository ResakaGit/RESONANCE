//! R5: Sensitivity and Uncertainty — pure functions for parameter sensitivity analysis.
//! Quantifies how much output changes per unit input change to avoid tuning overfitting.
//! Tests in `tests/r5_sensitivity.rs`.

/// Partial derivative approximation via central difference.
/// sensitivity = (f(x + delta) - f(x - delta)) / (2 * delta)
#[inline]
pub fn partial_sensitivity(base_output: f32, perturbed_output: f32, delta: f32) -> f32 {
    if delta.abs() < f32::EPSILON {
        return 0.0;
    }
    (perturbed_output - base_output) / (2.0 * delta)
}

/// Normalized sensitivity: sensitivity relative to nominal output magnitude.
/// Returns 0.0 when nominal_output ≈ 0 to avoid division-by-zero panics.
#[inline]
pub fn normalized_sensitivity(sensitivity: f32, nominal_output: f32) -> f32 {
    if nominal_output.abs() < f32::EPSILON {
        return 0.0;
    }
    sensitivity / nominal_output
}

/// Sweep a parameter over N steps from `param_min` to `param_max`, returning `(param, output)` pairs.
/// `n_steps` is clamped to `1..=16`. Unused slots (n_steps < 16) are filled with `(0.0, 0.0)`.
pub fn parameter_sweep_16(
    param_min: f32,
    param_max: f32,
    n_steps: usize,
    f: fn(f32) -> f32,
) -> [(f32, f32); 16] {
    let n = n_steps.clamp(1, 16);
    let mut out = [(0.0f32, 0.0f32); 16];
    let range = param_max - param_min;
    for i in 0..n {
        let t = if n == 1 {
            0.0
        } else {
            i as f32 / (n - 1) as f32
        };
        let x = param_min + t * range;
        out[i] = (x, f(x));
    }
    out
}

/// Welford online mean and variance over a slice.
/// Returns `(mean, variance)`. Variance is the population variance (not sample).
#[inline]
fn welford_mean_variance(values: &[f32]) -> (f32, f32) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mut mean = 0.0f32;
    let mut m2 = 0.0f32;
    for (n, &x) in values.iter().enumerate() {
        let delta = x - mean;
        mean += delta / (n + 1) as f32;
        let delta2 = x - mean;
        m2 += delta * delta2;
    }
    let variance = m2 / values.len() as f32;
    (mean, variance)
}

/// Confidence band: returns `(low, high)` = `(mean - k*std, mean + k*std)`.
/// Uses Welford online algorithm for mean + variance over the slice.
pub fn confidence_band(values: &[f32], k: f32) -> (f32, f32) {
    let (mean, variance) = welford_mean_variance(values);
    let std = variance.sqrt();
    (mean - k * std, mean + k * std)
}

/// Coefficient of variation: `std / mean`.
/// Returns `f32::INFINITY` if `mean ≈ 0`.
pub fn coefficient_of_variation(values: &[f32]) -> f32 {
    let (mean, variance) = welford_mean_variance(values);
    if mean.abs() < f32::EPSILON {
        return f32::INFINITY;
    }
    variance.sqrt() / mean
}

/// Returns `true` if the coefficient of variation exceeds `threshold`.
#[inline]
pub fn is_unstable(values: &[f32], threshold: f32) -> bool {
    coefficient_of_variation(values) > threshold
}

/// Rank parameters by absolute normalized sensitivity, descending in place.
/// Input/output: slice of `(&'static str, normalized_sensitivity)`.
/// N ≤ 16 for stack allocation; caller is responsible for slice size.
pub fn rank_by_sensitivity_16(params: &mut [(&'static str, f32)]) {
    params.sort_unstable_by(|a, b| {
        b.1.abs()
            .partial_cmp(&a.1.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_sensitivity_linear_fn_returns_slope() {
        // f(x) = 2x → f(5.1) - f(4.9) = 10.2 - 9.8 = 0.4 → 0.4 / 0.2 = 2.0
        let delta = 0.1f32;
        let base = 2.0 * 4.9; // f(x - delta)
        let perturbed = 2.0 * 5.1; // f(x + delta)
        let s = partial_sensitivity(base, perturbed, delta);
        assert!((s - 2.0).abs() < 1e-5, "expected ≈2.0, got {s}");
    }

    #[test]
    fn partial_sensitivity_zero_delta_returns_zero() {
        let s = partial_sensitivity(10.0, 12.0, 0.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn normalized_sensitivity_nonzero_nominal() {
        let s = normalized_sensitivity(4.0, 2.0);
        assert!((s - 2.0).abs() < 1e-6, "expected 2.0, got {s}");
    }

    #[test]
    fn normalized_sensitivity_zero_nominal_returns_zero() {
        let s = normalized_sensitivity(5.0, 0.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn parameter_sweep_returns_ordered_pairs() {
        // f(x) = x*x from 1..3 in 4 steps → (1, 1), (1.667, 2.778), (2.333, 5.444), (3, 9)
        let pairs = parameter_sweep_16(1.0, 3.0, 4, |x| x * x);
        // Outputs must be increasing
        for i in 1..4 {
            assert!(
                pairs[i].1 >= pairs[i - 1].1,
                "pair[{i}]={:?} not >= pair[{}]={:?}",
                pairs[i],
                i - 1,
                pairs[i - 1]
            );
        }
        // Slots 4..16 must be zeroed
        for i in 4..16 {
            assert_eq!(pairs[i], (0.0, 0.0), "slot {i} should be (0,0)");
        }
    }

    #[test]
    fn parameter_sweep_single_step_uses_param_min() {
        let pairs = parameter_sweep_16(5.0, 10.0, 1, |x| x + 1.0);
        assert!(
            (pairs[0].0 - 5.0).abs() < 1e-6,
            "first param should be min=5.0"
        );
        assert!(
            (pairs[0].1 - 6.0).abs() < 1e-6,
            "first output should be 6.0"
        );
        assert_eq!(pairs[1], (0.0, 0.0));
    }

    #[test]
    fn parameter_sweep_clamps_n_steps_to_16() {
        // n_steps=20 → clamped to 16, all slots used
        let pairs = parameter_sweep_16(0.0, 1.0, 20, |x| x);
        assert!(pairs[15].0 > 0.0, "last slot should be filled with n=16");
    }

    #[test]
    fn confidence_band_symmetric_around_mean() {
        let band = confidence_band(&[10.0, 10.0, 10.0], 2.0);
        assert!((band.0 - 10.0).abs() < 1e-5, "low={}", band.0);
        assert!((band.1 - 10.0).abs() < 1e-5, "high={}", band.1);
    }

    #[test]
    fn confidence_band_k_zero_returns_mean_mean() {
        let band = confidence_band(&[1.0, 2.0, 3.0, 4.0, 5.0], 0.0);
        assert!((band.0 - band.1).abs() < 1e-5, "k=0 → low==high");
    }

    #[test]
    fn confidence_band_empty_slice_returns_zero() {
        let band = confidence_band(&[], 2.0);
        assert_eq!(band, (0.0, 0.0));
    }

    #[test]
    fn coefficient_of_variation_constant_series_zero() {
        let cv = coefficient_of_variation(&[5.0, 5.0, 5.0, 5.0]);
        assert!(cv.abs() < 1e-5, "cv of constant series must be 0, got {cv}");
    }

    #[test]
    fn coefficient_of_variation_zero_mean_returns_infinity() {
        let cv = coefficient_of_variation(&[0.0, 0.0, 0.0]);
        assert!(cv.is_infinite(), "cv with zero mean must be INFINITY");
    }

    #[test]
    fn is_unstable_high_variance_returns_true() {
        // values spread over 3 orders of magnitude → cv >> 1.0
        assert!(is_unstable(&[0.1, 100.0, 0.1], 1.0));
    }

    #[test]
    fn is_unstable_constant_returns_false() {
        assert!(!is_unstable(&[42.0, 42.0, 42.0], 0.1));
    }

    #[test]
    fn rank_by_sensitivity_orders_descending() {
        let mut params = [("a", 0.5f32), ("b", 2.0f32), ("c", 1.0f32)];
        rank_by_sensitivity_16(&mut params);
        assert_eq!(params[0].0, "b", "highest |sensitivity| must be first");
        assert_eq!(params[1].0, "c");
        assert_eq!(params[2].0, "a");
    }

    #[test]
    fn rank_by_sensitivity_handles_negative_values() {
        let mut params = [("x", -3.0f32), ("y", 1.0f32), ("z", 2.0f32)];
        rank_by_sensitivity_16(&mut params);
        assert_eq!(params[0].0, "x", "|−3| is largest");
    }

    #[test]
    fn rank_by_sensitivity_empty_slice_no_panic() {
        let mut params: [(&'static str, f32); 0] = [];
        rank_by_sensitivity_16(&mut params); // must not panic
    }
}
