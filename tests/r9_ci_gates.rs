//! R9 — CI Reliability Gates
//! Validates that all critical reliability thresholds are correctly configured
//! and that the gate system is internally consistent.
//! Run with: cargo test --test r9_ci_gates

use resonance::blueprint::constants::{
    CONSERVATION_ERROR_TOLERANCE, POOL_CONSERVATION_EPSILON,
    SURROGATE_FITNESS_EPSILON, SURROGATE_MIN_HIT_RATE,
};
use resonance::blueprint::equations::{
    is_within_epsilon,
    surrogate_error::cache_hit_rate,
};
use resonance::blueprint::equations::observability::is_conservation_violation;

// ─── Threshold configuration gates ───────────────────────────────────────────

/// Conservation tolerance is within valid range [1e-6, 0.1].
#[test]
fn conservation_tolerance_in_valid_range() {
    assert!(
        CONSERVATION_ERROR_TOLERANCE >= 1e-6,
        "CONSERVATION_ERROR_TOLERANCE={CONSERVATION_ERROR_TOLERANCE} below minimum 1e-6"
    );
    assert!(
        CONSERVATION_ERROR_TOLERANCE <= 0.1,
        "CONSERVATION_ERROR_TOLERANCE={CONSERVATION_ERROR_TOLERANCE} above maximum 0.1"
    );
}

/// Pool conservation epsilon matches conservation tolerance.
#[test]
fn pool_epsilon_matches_conservation_tolerance() {
    assert_eq!(
        POOL_CONSERVATION_EPSILON,
        CONSERVATION_ERROR_TOLERANCE,
        "POOL_CONSERVATION_EPSILON={POOL_CONSERVATION_EPSILON} \
         != CONSERVATION_ERROR_TOLERANCE={CONSERVATION_ERROR_TOLERANCE}"
    );
}

/// Surrogate fitness epsilon is within valid range [0.001, 0.5].
#[test]
fn surrogate_fitness_epsilon_in_valid_range() {
    assert!(
        SURROGATE_FITNESS_EPSILON >= 0.001,
        "SURROGATE_FITNESS_EPSILON={SURROGATE_FITNESS_EPSILON} below minimum 0.001"
    );
    assert!(
        SURROGATE_FITNESS_EPSILON <= 0.5,
        "SURROGATE_FITNESS_EPSILON={SURROGATE_FITNESS_EPSILON} above maximum 0.5"
    );
}

/// Minimum hit rate is within valid range [0.5, 1.0].
#[test]
fn surrogate_min_hit_rate_in_valid_range() {
    assert!(
        SURROGATE_MIN_HIT_RATE >= 0.5,
        "SURROGATE_MIN_HIT_RATE={SURROGATE_MIN_HIT_RATE} below minimum 0.5"
    );
    assert!(
        SURROGATE_MIN_HIT_RATE <= 1.0,
        "SURROGATE_MIN_HIT_RATE={SURROGATE_MIN_HIT_RATE} above maximum 1.0"
    );
}

// ─── Conservation gate ────────────────────────────────────────────────────────

/// Conservation gate: no violation at tolerance boundary (error just below limit).
#[test]
fn conservation_gate_passes_at_boundary() {
    let error = CONSERVATION_ERROR_TOLERANCE - 1e-7;
    assert!(
        !is_conservation_violation(error),
        "error={error} should not trigger a conservation violation \
         (below CONSERVATION_ERROR_TOLERANCE={CONSERVATION_ERROR_TOLERANCE})"
    );
}

/// Conservation gate: triggers at overshoot (error above tolerance).
#[test]
fn conservation_gate_fails_above_tolerance() {
    let error = CONSERVATION_ERROR_TOLERANCE + 1e-3;
    assert!(
        is_conservation_violation(error),
        "error={error} should trigger a conservation violation \
         (above CONSERVATION_ERROR_TOLERANCE={CONSERVATION_ERROR_TOLERANCE})"
    );
}

// ─── Surrogate accuracy gate ──────────────────────────────────────────────────

/// Surrogate gate: passes when within epsilon (2% error < 5% epsilon).
#[test]
fn surrogate_gate_passes_within_epsilon() {
    // surrogate=0.98, exact=1.0 → relative error = 2% < SURROGATE_FITNESS_EPSILON (5%)
    assert!(
        is_within_epsilon(0.98, 1.0, SURROGATE_FITNESS_EPSILON),
        "2% error should be within SURROGATE_FITNESS_EPSILON={SURROGATE_FITNESS_EPSILON}"
    );
}

/// Surrogate gate: fails when outside epsilon (10% error > 5% epsilon).
#[test]
fn surrogate_gate_fails_outside_epsilon() {
    // surrogate=0.90, exact=1.0 → relative error = 10% > SURROGATE_FITNESS_EPSILON (5%)
    assert!(
        !is_within_epsilon(0.90, 1.0, SURROGATE_FITNESS_EPSILON),
        "10% error should exceed SURROGATE_FITNESS_EPSILON={SURROGATE_FITNESS_EPSILON}"
    );
}

// ─── Cache hit rate gate ──────────────────────────────────────────────────────

/// Cache hit rate gate: passes when hit rate is above minimum threshold.
#[test]
fn cache_hit_rate_gate_passes_above_minimum() {
    // 8 hits, 2 misses → hit_rate = 0.8 >= SURROGATE_MIN_HIT_RATE (0.7)
    let (hit_rate, _) = cache_hit_rate(8, 2);
    assert!(
        hit_rate >= SURROGATE_MIN_HIT_RATE,
        "hit_rate={hit_rate} should be >= SURROGATE_MIN_HIT_RATE={SURROGATE_MIN_HIT_RATE}"
    );
}

/// Cache hit rate gate: fails when hit rate is below minimum threshold.
#[test]
fn cache_hit_rate_gate_fails_below_minimum() {
    // 5 hits, 5 misses → hit_rate = 0.5 < SURROGATE_MIN_HIT_RATE (0.7)
    let (hit_rate, _) = cache_hit_rate(5, 5);
    assert!(
        hit_rate < SURROGATE_MIN_HIT_RATE,
        "hit_rate={hit_rate} should be < SURROGATE_MIN_HIT_RATE={SURROGATE_MIN_HIT_RATE}"
    );
}
