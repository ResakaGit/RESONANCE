//! AC-2: Kuramoto Entrainment — Axiom 8 consequence.
//!
//! Oscillators with compatible signatures gradually align their frequencies.
//! Coupling strength falls with distance (AC-4: `entrainment_coupling_at_distance`).
//!
//! Equation: `Δω_i = (K/N) × Σ_j [ coupling(d_ij) × (ω_j − ω_i) ]`
//!
//! This is the linear-regime limit of Kuramoto (`sin(x) ≈ x` for `|Δω| ≪ 2π`),
//! suitable for same-band entrainment where the frequency gap is small relative to
//! the total band width. Cross-band coupling is suppressed by distance decay (AC-4).
//!
//! SSOT: calls `signal_propagation::entrainment_coupling_at_distance` — does not
//! duplicate the distance-decay logic.

use crate::blueprint::equations::signal_propagation::entrainment_coupling_at_distance;

// ── Constants (algorithmic limits — not tuning values) ───────────────────────

/// Maximum number of neighbours considered in one entrainment step (stack-allocated).
pub const ENTRAINMENT_MAX_NEIGHBOURS: usize = 8;

// ── Pure functions ────────────────────────────────────────────────────────────

/// Kuramoto frequency delta from one neighbour (linear regime).
///
/// `Δω = K_eff × (ω_j − ω_i)`
///
/// The coupling is pre-modulated by distance purity via AC-4;
/// pass `entrainment_coupling_at_distance(base, d, λ)` as `effective_coupling`.
///
/// Invariant: returns 0 when `freq_i == freq_j` or `effective_coupling ≤ 0`.
#[inline]
pub fn kuramoto_pair_delta(freq_i: f32, freq_j: f32, effective_coupling: f32) -> f32 {
    effective_coupling.max(0.0) * (freq_j - freq_i)
}

/// Aggregate Kuramoto step for entity `i` over up to `ENTRAINMENT_MAX_NEIGHBOURS`.
///
/// `neighbours`: slice of `(freq_j, distance_to_j)` — max `ENTRAINMENT_MAX_NEIGHBOURS` entries.
/// Returns the new frequency after the step.
///
/// Invariants:
/// - Returns `freq_i` unchanged when `neighbours` is empty or `dt ≤ 0`.
/// - Normalised by N (number of neighbours) so coupling is density-independent.
pub fn kuramoto_entrainment_step(
    freq_i: f32,
    neighbours: &[(f32, f32)], // (freq_j, distance)
    base_coupling: f32,
    lambda_coherence: f32,
    dt: f32,
) -> f32 {
    if neighbours.is_empty() || dt <= 0.0 {
        return freq_i;
    }
    let n = neighbours.len().min(ENTRAINMENT_MAX_NEIGHBOURS) as f32;
    let sum: f32 = neighbours
        .iter()
        .take(ENTRAINMENT_MAX_NEIGHBOURS)
        .map(|&(fj, d)| {
            let k = entrainment_coupling_at_distance(base_coupling, d, lambda_coherence);
            kuramoto_pair_delta(freq_i, fj, k)
        })
        .sum();
    freq_i + (sum / n) * dt
}

/// Whether two oscillators are within frequency-lock distance.
///
/// When `|ω_i − ω_j| ≤ threshold_hz` they are considered phase-locked and
/// the entrainment system can stop updating them until one is perturbed.
#[inline]
pub fn entrainment_lock_achieved(freq_i: f32, freq_j: f32, threshold_hz: f32) -> bool {
    (freq_i - freq_j).abs() <= threshold_hz.max(0.0)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    // ── kuramoto_pair_delta ──────────────────────────────────────────────────

    #[test]
    fn pair_delta_same_freq_is_zero() {
        let d = kuramoto_pair_delta(75.0, 75.0, 0.5);
        assert!(d.abs() < EPS, "got {d}");
    }

    #[test]
    fn pair_delta_pulls_toward_higher_neighbour() {
        // ω_j > ω_i → positive delta (i gets pulled up)
        let d = kuramoto_pair_delta(70.0, 80.0, 1.0);
        assert!(d > 0.0, "should pull up: {d}");
    }

    #[test]
    fn pair_delta_pulls_toward_lower_neighbour() {
        // ω_j < ω_i → negative delta (i gets pulled down)
        let d = kuramoto_pair_delta(80.0, 70.0, 1.0);
        assert!(d < 0.0, "should pull down: {d}");
    }

    #[test]
    fn pair_delta_negative_coupling_clamped_to_zero() {
        // Negative coupling → zero delta (no anti-entrainment)
        let d = kuramoto_pair_delta(70.0, 80.0, -1.0);
        assert!(d.abs() < EPS, "negative coupling → zero delta: {d}");
    }

    #[test]
    fn pair_delta_proportional_to_gap() {
        let d_small = kuramoto_pair_delta(75.0, 76.0, 1.0);
        let d_large = kuramoto_pair_delta(75.0, 85.0, 1.0);
        assert!(
            d_large > d_small,
            "larger gap → larger delta: small={d_small} large={d_large}"
        );
    }

    // ── kuramoto_entrainment_step ────────────────────────────────────────────

    #[test]
    fn step_no_neighbours_returns_unchanged() {
        let freq = kuramoto_entrainment_step(75.0, &[], 0.5, 12.0, 1.0);
        assert!((freq - 75.0).abs() < EPS, "got {freq}");
    }

    #[test]
    fn step_zero_dt_returns_unchanged() {
        let freq = kuramoto_entrainment_step(75.0, &[(80.0, 2.0)], 0.5, 12.0, 0.0);
        assert!((freq - 75.0).abs() < EPS, "got {freq}");
    }

    #[test]
    fn step_moves_toward_neighbour_at_contact() {
        // distance=0 → coupling = base_coupling (full). Should increase freq.
        let new_freq = kuramoto_entrainment_step(70.0, &[(80.0, 0.0)], 1.0, 12.0, 1.0);
        assert!(new_freq > 70.0, "should increase toward 80: {new_freq}");
    }

    #[test]
    fn step_far_neighbour_has_less_effect() {
        let near = kuramoto_entrainment_step(70.0, &[(80.0, 1.0)], 1.0, 12.0, 1.0);
        let far = kuramoto_entrainment_step(70.0, &[(80.0, 50.0)], 1.0, 12.0, 1.0);
        assert!(near > far, "near={near} should move more than far={far}");
    }

    #[test]
    fn step_capped_at_max_neighbours() {
        // 10 neighbours → only ENTRAINMENT_MAX_NEIGHBOURS (8) used, but result still valid
        let neighbours: Vec<(f32, f32)> = (0..10).map(|i| (80.0 + i as f32, 2.0)).collect();
        let freq = kuramoto_entrainment_step(70.0, &neighbours, 0.5, 12.0, 1.0);
        assert!(freq.is_finite(), "result must be finite: {freq}");
    }

    #[test]
    fn step_normalised_by_n_density_independent() {
        // 1 neighbour at d=0 vs 4 identical neighbours at d=0 → same update per entity
        let one = kuramoto_entrainment_step(70.0, &[(80.0, 0.0)], 1.0, 12.0, 1.0);
        let four = kuramoto_entrainment_step(70.0, &[(80.0, 0.0); 4], 1.0, 12.0, 1.0);
        assert!(
            (one - four).abs() < EPS,
            "density-independent: one={one} four={four}"
        );
    }

    // ── entrainment_lock_achieved ────────────────────────────────────────────

    #[test]
    fn lock_achieved_same_freq() {
        assert!(entrainment_lock_achieved(75.0, 75.0, 1.0));
    }

    #[test]
    fn lock_not_achieved_when_gap_exceeds_threshold() {
        assert!(!entrainment_lock_achieved(70.0, 80.0, 1.0));
    }

    #[test]
    fn lock_achieved_within_threshold() {
        assert!(entrainment_lock_achieved(75.0, 75.5, 1.0));
    }

    #[test]
    fn lock_threshold_zero_exact_match_returns_true() {
        assert!(entrainment_lock_achieved(75.0, 75.0, 0.0));
    }

    #[test]
    fn lock_threshold_zero_any_gap_returns_false() {
        assert!(!entrainment_lock_achieved(75.0, 75.001, 0.0));
    }
}
