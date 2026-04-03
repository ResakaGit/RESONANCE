//! R7 — Morph Inference Robustness: unit tests for pure-math validation functions.
//! No Bevy runtime — purely deterministic math tests.

use resonance::blueprint::equations::{
    EnvScenario, all_organs_have_valid_cost, has_valid_energy_cost, hysteresis_threshold_crossed,
    is_energy_viable, scenario_energy, scenario_max_organs,
};
use resonance::layers::{OrganRole, OrganSpec};

// Helper: creates an OrganSpec using scale_factor as the energy-cost proxy.
// OrganSpec has no explicit energy_cost field; scale_factor is the structural investment weight.
fn organ_with_cost(role: OrganRole, cost: f32) -> OrganSpec {
    OrganSpec::new(role, 1, cost)
}

// ─── scenario_energy ─────────────────────────────────────────────────────────

#[test]
fn scenario_energy_values_are_ordered() {
    let abundant = scenario_energy(EnvScenario::Abundant);
    let hostile = scenario_energy(EnvScenario::Hostile);
    let extreme = scenario_energy(EnvScenario::Extreme);
    assert!(abundant > hostile, "abundant={abundant} hostile={hostile}");
    assert!(hostile > extreme, "hostile={hostile} extreme={extreme}");
}

#[test]
fn scenario_energy_abundant_is_2000() {
    assert!((scenario_energy(EnvScenario::Abundant) - 2000.0).abs() < 1e-6);
}

#[test]
fn scenario_energy_hostile_is_500() {
    assert!((scenario_energy(EnvScenario::Hostile) - 500.0).abs() < 1e-6);
}

#[test]
fn scenario_energy_extreme_is_50() {
    assert!((scenario_energy(EnvScenario::Extreme) - 50.0).abs() < 1e-6);
}

// ─── abundant_scenario_all_organs_viable ─────────────────────────────────────

#[test]
fn abundant_scenario_all_organs_viable() {
    // 2000.0 energy / 10.0 per_organ_cost = 200 viable organs
    let count = scenario_max_organs(EnvScenario::Abundant, 10.0);
    assert!(
        count > 0,
        "abundant scenario must support at least one organ, got {count}"
    );
    assert!(is_energy_viable(
        scenario_energy(EnvScenario::Abundant),
        count,
        10.0,
        1
    ));
}

// ─── extreme_scenario_limits_organ_count ─────────────────────────────────────

#[test]
fn extreme_scenario_limits_organ_count() {
    let abundant = scenario_max_organs(EnvScenario::Abundant, 10.0);
    let extreme = scenario_max_organs(EnvScenario::Extreme, 10.0);
    assert!(
        extreme < abundant,
        "extreme ({extreme}) should support fewer organs than abundant ({abundant})"
    );
}

// ─── hostile_scenario_between_abundant_and_extreme ───────────────────────────

#[test]
fn hostile_scenario_between_abundant_and_extreme() {
    let abundant = scenario_max_organs(EnvScenario::Abundant, 10.0);
    let hostile = scenario_max_organs(EnvScenario::Hostile, 10.0);
    let extreme = scenario_max_organs(EnvScenario::Extreme, 10.0);
    assert!(
        hostile > extreme,
        "hostile ({hostile}) should exceed extreme ({extreme})"
    );
    assert!(
        hostile < abundant,
        "hostile ({hostile}) should be below abundant ({abundant})"
    );
}

// ─── hysteresis_blocks_small_change ──────────────────────────────────────────

#[test]
fn hysteresis_blocks_small_change() {
    // old=0.5, new=0.52, threshold=0.1 → |0.02| < 0.1 → false
    assert!(!hysteresis_threshold_crossed(0.5, 0.52, 0.1));
}

// ─── hysteresis_allows_large_change ──────────────────────────────────────────

#[test]
fn hysteresis_allows_large_change() {
    // old=0.5, new=0.7, threshold=0.1 → |0.2| > 0.1 → true
    assert!(hysteresis_threshold_crossed(0.5, 0.7, 0.1));
}

// ─── hysteresis edge cases ────────────────────────────────────────────────────

#[test]
fn hysteresis_exact_threshold_is_not_crossed() {
    // |new - old| == threshold → not strictly greater → false
    // Use integer-safe f32 values to avoid floating-point rounding (0.25 is exact in f32).
    assert!(!hysteresis_threshold_crossed(0.0, 0.25, 0.25));
}

#[test]
fn hysteresis_negative_delta_respects_absolute_value() {
    // old=0.7, new=0.5 → |−0.2| > 0.1 → true
    assert!(hysteresis_threshold_crossed(0.7, 0.5, 0.1));
}

// ─── all_organs_valid_cost_rejects_free_organ ────────────────────────────────

#[test]
fn all_organs_valid_cost_rejects_free_organ() {
    let valid = organ_with_cost(OrganRole::Stem, 0.5);
    let free = organ_with_cost(OrganRole::Leaf, 0.0);
    assert!(
        !all_organs_have_valid_cost(&[valid, free]),
        "manifest with a zero-cost organ must fail validation"
    );
}

#[test]
fn all_organs_valid_cost_accepts_all_positive_costs() {
    let organs = [
        organ_with_cost(OrganRole::Stem, 0.5),
        organ_with_cost(OrganRole::Root, 0.4),
        organ_with_cost(OrganRole::Leaf, 0.3),
    ];
    assert!(all_organs_have_valid_cost(&organs));
}

#[test]
fn has_valid_energy_cost_zero_cost_returns_false() {
    let organ = organ_with_cost(OrganRole::Bud, 0.0);
    assert!(!has_valid_energy_cost(&organ));
}

// ─── energy_viability_zero_base_energy_fails ─────────────────────────────────

#[test]
fn energy_viability_zero_base_energy_fails() {
    assert!(!is_energy_viable(0.0, 1, 10.0, 1));
}

#[test]
fn energy_viability_exact_budget_succeeds() {
    // 100.0 == 2 organs * 10.0 cost * 5 ticks = 100.0
    assert!(is_energy_viable(100.0, 2, 10.0, 5));
}

#[test]
fn energy_viability_zero_organs_fails_due_to_zero_min_ticks_guard() {
    // min_ticks_viable=0 is degenerate — no ticks means no viability window
    assert!(!is_energy_viable(1000.0, 0, 10.0, 0));
}

#[test]
fn energy_viability_negative_cost_fails() {
    assert!(!is_energy_viable(1000.0, 5, -1.0, 10));
}
