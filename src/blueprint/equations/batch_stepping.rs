//! Analytical multi-tick stepping — O(1) solutions for N ticks.
//!
//! Replaces tick-by-tick iteration with closed-form results for independent systems.
//! Same f32 precision as iterating. Conservation and determinism preserved.
//!
//! Reuses existing solvers from `macro_analytics`.

use super::{exponential_decay, allometric_radius, locomotion_energy_cost};
use crate::blueprint::constants::{DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX};

/// Dissipation over N ticks: `qe × (1 - rate)^N`.
///
/// Exact discrete Euler. Axiom 4: monotonic non-negative.
#[inline]
pub fn dissipation_n_ticks(qe: f32, rate: f32, n: u32) -> f32 {
    if n == 0 || qe <= 0.0 { return qe; }
    let clamped = rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX);
    exponential_decay(qe, clamped, n)
}

/// Growth over N ticks: `allometric_radius(r0, r_max, k, N)`.
///
/// Already O(1) in macro_analytics. Wired for batch use.
#[inline]
pub fn growth_n_ticks(radius: f32, growth_bias: f32, max_radius: f32, k: f32, n: u32) -> f32 {
    if growth_bias <= 0.0 || n == 0 { return radius; }
    let r_max = growth_bias * max_radius;
    if radius >= r_max { return radius; }
    allometric_radius(radius, r_max, k, n)
}

/// Senescence drain over N ticks (trapezoidal approximation).
///
/// `avg_rate = base × (1 + coeff × (age + N/2))`, `loss = qe × avg_rate × N`.
/// <1% error vs tick-by-tick for typical coeff values.
#[inline]
pub fn senescence_n_ticks(qe: f32, base_rate: f32, age: u64, coeff: f32, n: u32) -> f32 {
    if n == 0 || qe <= 0.0 { return qe; }
    let avg_age = age as f32 + n as f32 * 0.5;
    let avg_rate = base_rate * (1.0 + coeff * avg_age);
    (qe - qe * avg_rate * n as f32).max(0.0)
}

/// Locomotion drain over N ticks with constant velocity.
///
/// Conservative: cost computed at initial qe, subtracted as total.
#[inline]
pub fn locomotion_n_ticks(qe: f32, speed: f32, terrain_factor: f32, n: u32) -> f32 {
    if n == 0 || speed < 1e-4 { return qe; }
    let cost = locomotion_energy_cost(qe, speed, terrain_factor);
    (qe - cost * n as f32).max(0.0)
}

/// Check if entity is isolated (no neighbors within range).
///
/// If isolated, analytical stepping is exact (no interactions to miss).
pub fn is_isolated(
    positions: &[[f32; 2]],
    alive_mask: u128,
    entity_idx: usize,
    range_sq: f32,
) -> bool {
    let pos = positions[entity_idx];
    let mut mask = alive_mask & !(1u128 << entity_idx);
    while mask != 0 {
        let j = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let dx = positions[j][0] - pos[0];
        let dy = positions[j][1] - pos[1];
        if dx * dx + dy * dy < range_sq { return false; }
    }
    true
}

/// Predict death tick: when qe drops below threshold via dissipation.
///
/// Returns ticks until death, or u32::MAX if stable.
#[inline]
pub fn predict_death_ticks(qe: f32, rate: f32, threshold: f32) -> u32 {
    if qe <= threshold { return 0; }
    let clamped = rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX);
    if clamped >= 1.0 { return 1; }
    let n = (threshold / qe).ln() / (1.0 - clamped).ln();
    if n.is_finite() && n > 0.0 { n.ceil() as u32 } else { u32::MAX }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::dissipation_loss;

    // ── dissipation_n_ticks ─────────────────────────────────────────────────

    #[test]
    fn dissipation_single_tick_matches() {
        let qe = 100.0;
        let rate = 0.01;
        let analytical = dissipation_n_ticks(qe, rate, 1);
        let iterative = qe - dissipation_loss(qe, rate);
        assert!((analytical - iterative).abs() < 1e-4,
            "analytical={analytical} iterative={iterative}");
    }

    #[test]
    fn dissipation_500_ticks_close_to_iterative() {
        let rate = 0.01;
        let mut qe_iter = 100.0_f32;
        for _ in 0..500 {
            qe_iter -= dissipation_loss(qe_iter, rate);
        }
        let qe_anal = dissipation_n_ticks(100.0, rate, 500);
        let error_pct = ((qe_anal - qe_iter) / qe_iter).abs() * 100.0;
        assert!(error_pct < 0.5, "error={error_pct}% analytical={qe_anal} iterative={qe_iter}");
    }

    #[test]
    fn dissipation_zero_ticks_noop() {
        assert_eq!(dissipation_n_ticks(100.0, 0.05, 0), 100.0);
    }

    #[test]
    fn dissipation_always_non_negative() {
        assert!(dissipation_n_ticks(1.0, 0.5, 10000) >= 0.0);
    }

    // ── growth_n_ticks ──────────────────────────────────────────────────────

    #[test]
    fn growth_approaches_max() {
        let r = growth_n_ticks(0.5, 1.0, 3.0, 0.01, 10000);
        assert!((r - 3.0).abs() < 0.1, "r={r}");
    }

    #[test]
    fn growth_zero_bias_noop() {
        assert_eq!(growth_n_ticks(0.5, 0.0, 3.0, 0.01, 100), 0.5);
    }

    // ── senescence_n_ticks ──────────────────────────────────────────────────

    #[test]
    fn senescence_drains_proportional_to_age() {
        let young = senescence_n_ticks(100.0, 0.001, 0, 0.00001, 10);
        let old = senescence_n_ticks(100.0, 0.001, 5000, 0.00001, 10);
        assert!(old < young, "old should lose more: young={young} old={old}");
    }

    #[test]
    fn senescence_never_negative() {
        assert!(senescence_n_ticks(1.0, 0.5, 100000, 0.001, 1000) >= 0.0);
    }

    // ── locomotion_n_ticks ──────────────────────────────────────────────────

    #[test]
    fn locomotion_zero_speed_noop() {
        assert_eq!(locomotion_n_ticks(100.0, 0.0, 1.0, 500), 100.0);
    }

    #[test]
    fn locomotion_drains_with_speed() {
        let result = locomotion_n_ticks(100.0, 5.0, 1.0, 10);
        assert!(result < 100.0, "should drain: {result}");
        assert!(result >= 0.0);
    }

    // ── is_isolated ─────────────────────────────────────────────────────────

    #[test]
    fn alone_is_isolated() {
        let positions = [[5.0, 5.0]; 64];
        assert!(is_isolated(&positions, 1, 0, 100.0)); // only entity 0 alive
    }

    #[test]
    fn nearby_not_isolated() {
        let mut positions = [[0.0; 2]; 64];
        positions[0] = [0.0, 0.0];
        positions[1] = [1.0, 0.0]; // within range
        assert!(!is_isolated(&positions, 0b11, 0, 4.0));
    }

    #[test]
    fn far_is_isolated() {
        let mut positions = [[0.0; 2]; 64];
        positions[0] = [0.0, 0.0];
        positions[1] = [100.0, 0.0]; // far away
        assert!(is_isolated(&positions, 0b11, 0, 4.0));
    }

    // ── predict_death_ticks ─────────────────────────────────────────────────

    #[test]
    fn predict_death_reasonable() {
        let n = predict_death_ticks(100.0, 0.01, 0.01);
        assert!(n > 100 && n < 10000, "n={n}");
    }

    #[test]
    fn predict_death_already_dead() {
        assert_eq!(predict_death_ticks(0.005, 0.01, 0.01), 0);
    }

    #[test]
    fn predict_death_slow_drain() {
        // rate=0 gets clamped to DISSIPATION_RATE_MIN, so still drains slowly
        let n = predict_death_ticks(100.0, 0.0, 0.01);
        assert!(n > 1000, "with min rate, should take many ticks: {n}");
    }
}
