//! Pure error-metric functions for surrogate reliability validation (R8).

/// Absolute error between surrogate and exact computation.
pub fn surrogate_absolute_error(surrogate_value: f32, exact_value: f32) -> f32 {
    (surrogate_value - exact_value).abs()
}

/// Relative error between surrogate and exact computation.
/// Returns `f32::INFINITY` if `exact_value` is near zero (< 1e-6 magnitude).
pub fn surrogate_relative_error(surrogate_value: f32, exact_value: f32) -> f32 {
    if exact_value.abs() < 1e-6 {
        return f32::INFINITY;
    }
    (surrogate_value - exact_value).abs() / exact_value.abs()
}

/// Returns `true` if the surrogate error is within the acceptable epsilon.
/// Uses relative error; falls back to absolute when exact is near zero.
pub fn is_within_epsilon(surrogate: f32, exact: f32, epsilon: f32) -> bool {
    surrogate_relative_error(surrogate, exact) <= epsilon
}

/// Maximum absolute error across a set of (surrogate, exact) pairs.
/// Returns `0.0` for an empty slice.
pub fn max_surrogate_error(pairs: &[(f32, f32)]) -> f32 {
    pairs
        .iter()
        .map(|&(s, e)| surrogate_absolute_error(s, e))
        .fold(0.0_f32, f32::max)
}

/// Mean absolute error across a set of (surrogate, exact) pairs.
/// Returns `0.0` for an empty slice.
pub fn mean_absolute_error(pairs: &[(f32, f32)]) -> f32 {
    if pairs.is_empty() {
        return 0.0;
    }
    let sum: f32 = pairs
        .iter()
        .map(|&(s, e)| surrogate_absolute_error(s, e))
        .sum();
    sum / pairs.len() as f32
}

/// Cache hit statistics: returns `(hit_rate, miss_rate)` where `hit_rate + miss_rate == 1.0`.
/// Both are `0.0` when `hits + misses == 0`.
pub fn cache_hit_rate(hits: u32, misses: u32) -> (f32, f32) {
    let total = hits + misses;
    if total == 0 {
        return (0.0, 0.0);
    }
    let total_f = total as f32;
    let hit_rate = hits as f32 / total_f;
    (hit_rate, 1.0 - hit_rate)
}

/// Top-K convergence check: returns `true` if the rank-ordered top-K values match
/// between `surrogate_values` and `exact_values` within `epsilon` (relative).
///
/// Algorithm:
/// 1. Sort indices of `exact_values` descending → top-K exact indices.
/// 2. Sort indices of `surrogate_values` descending → top-K surrogate indices.
/// 3. For each rank `r` in `0..K`, check
///    `|surrogate_values[surrogate_rank[r]] - exact_values[exact_rank[r]]| <= epsilon`
///    using relative error (absolute fallback when exact is near zero).
///
/// `K` is capped at `min(k, len, 16)` for stack allocation.
/// Returns `false` if either slice is empty or `k == 0`.
pub fn top_k_converged(
    surrogate_values: &[f32],
    exact_values: &[f32],
    k: usize,
    epsilon: f32,
) -> bool {
    let len = surrogate_values.len().min(exact_values.len());
    let k = k.min(len).min(16);
    if k == 0 {
        return false;
    }

    // Stack-allocated index arrays (max 16).
    let mut exact_idx: [usize; 16] = [0; 16];
    let mut surr_idx: [usize; 16] = [0; 16];
    for i in 0..len {
        exact_idx[i] = i;
        surr_idx[i] = i;
    }

    // Sort top-K portion descending by value (partial selection — O(len * k) but len <= 16 cap).
    for r in 0..k {
        for j in (r + 1)..len {
            if exact_values[exact_idx[j]] > exact_values[exact_idx[r]] {
                exact_idx.swap(r, j);
            }
            if surrogate_values[surr_idx[j]] > surrogate_values[surr_idx[r]] {
                surr_idx.swap(r, j);
            }
        }
    }

    for r in 0..k {
        let s_val = surrogate_values[surr_idx[r]];
        let e_val = exact_values[exact_idx[r]];
        if !is_within_epsilon(s_val, e_val, epsilon) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_error_exact_match_is_zero() {
        assert_eq!(surrogate_absolute_error(100.0, 100.0), 0.0);
    }

    #[test]
    fn relative_error_near_zero_exact_returns_infinity() {
        assert!(surrogate_relative_error(1.0, 0.0).is_infinite());
    }

    #[test]
    fn cache_hit_rate_zero_total_returns_zero_zero() {
        assert_eq!(cache_hit_rate(0, 0), (0.0, 0.0));
    }

    #[test]
    fn max_surrogate_error_empty_returns_zero() {
        assert_eq!(max_surrogate_error(&[]), 0.0);
    }

    #[test]
    fn mean_absolute_error_empty_returns_zero() {
        assert_eq!(mean_absolute_error(&[]), 0.0);
    }
}
