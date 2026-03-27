//! Morph Robustness (R7) — pure validation functions for morphological inference.
//! Guarantees that inferred organs are energy-viable and that transitions are hysteresis-guarded.

use crate::layers::{OrganRole, OrganSpec};

// Epsilon below which an organ's scale_factor is considered "free" (no structural investment).
const SCALE_COST_EPSILON: f32 = 1e-6;

/// Returns true if an organ has a non-trivial scale factor (proxy for energy cost > 0).
/// `OrganSpec` has no explicit `energy_cost` field; `scale_factor` is the structural investment
/// weight computed during inference. A zero scale_factor means the organ has no presence —
/// equivalent to a "free" organ that should not be manifested.
#[inline]
pub fn has_valid_energy_cost(organ: &OrganSpec) -> bool {
    organ.scale_factor() > SCALE_COST_EPSILON
}

/// Returns true if every organ in a slice has a valid (non-zero) scale_factor.
/// A manifest containing any free organ is considered malformed.
#[inline]
pub fn all_organs_have_valid_cost(organs: &[OrganSpec]) -> bool {
    organs.iter().all(has_valid_energy_cost)
}

/// Hysteresis guard: returns true only if `new_value` differs from `old_value` by more than
/// `threshold`. Prevents organ flickering when a signal hovers near a transition boundary.
/// Returns false when `threshold` is negative (degenerate input treated as no-cross).
#[inline]
pub fn hysteresis_threshold_crossed(old_value: f32, new_value: f32, threshold: f32) -> bool {
    if threshold < 0.0 {
        return false;
    }
    (new_value - old_value).abs() > threshold
}

/// Returns true if `base_energy` can sustain `organ_count` organs at `per_organ_cost` each
/// for at least `min_ticks_viable` ticks.
/// Returns false for any degenerate input (zero energy, negative cost, zero min_ticks).
#[inline]
pub fn is_energy_viable(
    base_energy: f32,
    organ_count: usize,
    per_organ_cost: f32,
    min_ticks_viable: u32,
) -> bool {
    if base_energy <= 0.0 || per_organ_cost <= 0.0 || min_ticks_viable == 0 {
        return false;
    }
    let total_cost = organ_count as f32 * per_organ_cost * min_ticks_viable as f32;
    base_energy >= total_cost
}

/// Environment scenario type for robustness testing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnvScenario {
    /// High energy, low stress.
    Abundant,
    /// Moderate energy, moderate stress.
    Hostile,
    /// Minimal energy, high stress.
    Extreme,
}

/// Returns the base energy level for each scenario (test fixture).
/// Abundant: 2000.0, Hostile: 500.0, Extreme: 50.0.
#[inline]
pub fn scenario_energy(scenario: EnvScenario) -> f32 {
    match scenario {
        EnvScenario::Abundant => 2000.0,
        EnvScenario::Hostile => 500.0,
        EnvScenario::Extreme => 50.0,
    }
}

/// Returns the expected maximum viable organ count for each scenario given a per-organ cost.
/// Computed as `floor(scenario_energy / per_organ_cost)`, clamped to 0.
/// Returns 0 if `per_organ_cost <= 0.0`.
#[inline]
pub fn scenario_max_organs(scenario: EnvScenario, per_organ_cost: f32) -> usize {
    if per_organ_cost <= 0.0 {
        return 0;
    }
    let energy = scenario_energy(scenario);
    (energy / per_organ_cost).floor() as usize
}

// ── Suppress unused import warning: OrganRole is required by the public API surface ──
const _: () = {
    let _ = OrganRole::Stem;
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::OrganSpec;

    fn organ_with_scale(role: OrganRole, scale: f32) -> OrganSpec {
        OrganSpec::new(role, 1, scale)
    }

    #[test]
    fn has_valid_energy_cost_zero_scale_returns_false() {
        let organ = organ_with_scale(OrganRole::Stem, 0.0);
        assert!(!has_valid_energy_cost(&organ));
    }

    #[test]
    fn has_valid_energy_cost_positive_scale_returns_true() {
        let organ = organ_with_scale(OrganRole::Leaf, 0.5);
        assert!(has_valid_energy_cost(&organ));
    }

    #[test]
    fn all_organs_valid_cost_empty_slice_returns_true() {
        // vacuous truth: no organs to fail validation
        assert!(all_organs_have_valid_cost(&[]));
    }

    #[test]
    fn all_organs_valid_cost_mixed_returns_false() {
        let valid = organ_with_scale(OrganRole::Root, 0.4);
        let free = organ_with_scale(OrganRole::Bud, 0.0);
        assert!(!all_organs_have_valid_cost(&[valid, free]));
    }

    #[test]
    fn hysteresis_threshold_crossed_zero_threshold_always_true_for_nonzero_delta() {
        assert!(hysteresis_threshold_crossed(0.5, 0.5 + 1e-5, 0.0));
    }

    #[test]
    fn hysteresis_threshold_crossed_negative_threshold_always_false() {
        assert!(!hysteresis_threshold_crossed(0.0, 100.0, -1.0));
    }

    #[test]
    fn is_energy_viable_sufficient_energy_returns_true() {
        // 100.0 >= 2 organs * 5.0 cost * 5 ticks = 50.0
        assert!(is_energy_viable(100.0, 2, 5.0, 5));
    }

    #[test]
    fn is_energy_viable_insufficient_energy_returns_false() {
        // 10.0 < 2 organs * 5.0 cost * 5 ticks = 50.0
        assert!(!is_energy_viable(10.0, 2, 5.0, 5));
    }

    #[test]
    fn scenario_max_organs_abundant_gt_extreme() {
        let abundant = scenario_max_organs(EnvScenario::Abundant, 10.0);
        let extreme = scenario_max_organs(EnvScenario::Extreme, 10.0);
        assert!(abundant > extreme, "abundant={abundant} extreme={extreme}");
    }

    #[test]
    fn scenario_max_organs_zero_cost_returns_zero() {
        assert_eq!(scenario_max_organs(EnvScenario::Abundant, 0.0), 0);
    }
}
