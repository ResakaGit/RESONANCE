//! R10 — Emergence (Axiom 6) Gates
//! Validates that the emergent-synchronization thresholds are correctly configured
//! and internally consistent, and that the verdict function behaves at the boundary.
//! Run with: cargo test --test r10_emergence_gates

use resonance::blueprint::constants::{
    KURAMOTO_LOCK_THRESHOLD_HZ, SYNC_ABLATED_R_MAX, SYNC_ECS_ABLATED_S_MAX, SYNC_ECS_CENTER_HZ,
    SYNC_ECS_S_PASS_MIN, SYNC_ECS_SPREAD_HZ, SYNC_ECS_SPREAD_RATIO, SYNC_ECS_TICKS, SYNC_GAP_MIN,
    SYNC_R_PASS_MIN,
};
use resonance::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;
use resonance::blueprint::equations::emergence::synchronization::{
    emergence_sync_verdict, sync_ecs_verdict,
};

// ─── Threshold configuration gates ───────────────────────────────────────────

/// Sync PASS threshold is within valid range [0.3, 0.95].
#[test]
fn sync_r_pass_min_in_valid_range() {
    assert!(
        SYNC_R_PASS_MIN >= 0.3,
        "SYNC_R_PASS_MIN={SYNC_R_PASS_MIN} below minimum 0.3"
    );
    assert!(
        SYNC_R_PASS_MIN <= 0.95,
        "SYNC_R_PASS_MIN={SYNC_R_PASS_MIN} above maximum 0.95"
    );
}

/// Ablated-R ceiling is within valid range [0.0, 0.3].
#[test]
fn sync_ablated_r_max_in_valid_range() {
    assert!(
        SYNC_ABLATED_R_MAX >= 0.0,
        "SYNC_ABLATED_R_MAX={SYNC_ABLATED_R_MAX} below minimum 0.0"
    );
    assert!(
        SYNC_ABLATED_R_MAX <= 0.3,
        "SYNC_ABLATED_R_MAX={SYNC_ABLATED_R_MAX} above maximum 0.3"
    );
}

/// Consistency: the guaranteed gap between coupled PASS and ablated ceiling
/// must be at least the declared minimum emergence gap.
#[test]
fn thresholds_guarantee_the_emergence_gap() {
    let guaranteed = SYNC_R_PASS_MIN - SYNC_ABLATED_R_MAX;
    assert!(
        guaranteed >= SYNC_GAP_MIN,
        "SYNC_R_PASS_MIN - SYNC_ABLATED_R_MAX = {guaranteed} < SYNC_GAP_MIN={SYNC_GAP_MIN}"
    );
}

// ─── Verdict boundary gates ───────────────────────────────────────────────────

/// Verdict passes just inside both thresholds.
#[test]
fn verdict_passes_inside_boundary() {
    let r_final = SYNC_R_PASS_MIN + 1e-4;
    let r_ablated = SYNC_ABLATED_R_MAX - 1e-4;
    assert!(
        emergence_sync_verdict(r_final, r_ablated),
        "verdict should PASS at R_final={r_final}, R_ablated={r_ablated}"
    );
}

/// Verdict fails when coupled R is below the PASS threshold.
#[test]
fn verdict_fails_when_not_synchronized() {
    let r_final = SYNC_R_PASS_MIN - 1e-4;
    let r_ablated = SYNC_ABLATED_R_MAX - 1e-4;
    assert!(
        !emergence_sync_verdict(r_final, r_ablated),
        "verdict should FAIL below SYNC_R_PASS_MIN (R_final={r_final})"
    );
}

/// Verdict fails when ablated R is above the ceiling (no causal collapse).
#[test]
fn verdict_fails_when_ablated_too_high() {
    let r_final = SYNC_R_PASS_MIN + 1e-4;
    let r_ablated = SYNC_ABLATED_R_MAX + 1e-4;
    assert!(
        !emergence_sync_verdict(r_final, r_ablated),
        "verdict should FAIL when ablation does not collapse order (R_ablated={r_ablated})"
    );
}

// ─── ECS probe (SYNC_ECS_*) configuration gates — ADR-047 ────────────────────

/// The initial spread ratio sits strictly inside its derived bounds:
/// `1 < ratio < COHERENCE_BANDWIDTH / KURAMOTO_LOCK_THRESHOLD_HZ` (= 50).
/// Below 1 the population is born locked; above the upper bound it leaves the
/// Axiom-8 coherence window.
#[test]
fn sync_ecs_spread_ratio_inside_derived_bounds() {
    let upper = COHERENCE_BANDWIDTH / KURAMOTO_LOCK_THRESHOLD_HZ;
    assert!(
        SYNC_ECS_SPREAD_RATIO > 1.0,
        "SYNC_ECS_SPREAD_RATIO={SYNC_ECS_SPREAD_RATIO} must exceed 1 (disordered start)"
    );
    assert!(
        SYNC_ECS_SPREAD_RATIO < upper,
        "SYNC_ECS_SPREAD_RATIO={SYNC_ECS_SPREAD_RATIO} must stay below {upper} (coherence window)"
    );
}

/// Derivation consistency: the spread in Hz is the ratio times the engine's own
/// lock threshold — not an independent magic number.
#[test]
fn sync_ecs_spread_hz_derives_from_lock_threshold() {
    let expected = SYNC_ECS_SPREAD_RATIO * KURAMOTO_LOCK_THRESHOLD_HZ;
    assert_eq!(
        SYNC_ECS_SPREAD_HZ.to_bits(),
        expected.to_bits(),
        "SYNC_ECS_SPREAD_HZ={SYNC_ECS_SPREAD_HZ} must equal ratio×lock={expected}"
    );
}

/// The L2 `max(0.0)` clamp must be inert for the whole population:
/// `center - 3·spread > 0` covers ~99.7 % of the Gaussian draws.
#[test]
fn sync_ecs_center_keeps_gaussian_off_the_clamp() {
    let floor = SYNC_ECS_CENTER_HZ - 3.0 * SYNC_ECS_SPREAD_HZ;
    assert!(
        floor > 0.0,
        "SYNC_ECS_CENTER_HZ - 3·SYNC_ECS_SPREAD_HZ = {floor} ≤ 0 → rectified population"
    );
}

/// ECS PASS threshold is within valid range [0.3, 0.95].
#[test]
fn sync_ecs_s_pass_min_in_valid_range() {
    assert!(
        SYNC_ECS_S_PASS_MIN >= 0.3,
        "SYNC_ECS_S_PASS_MIN={SYNC_ECS_S_PASS_MIN} below minimum 0.3"
    );
    assert!(
        SYNC_ECS_S_PASS_MIN <= 0.95,
        "SYNC_ECS_S_PASS_MIN={SYNC_ECS_S_PASS_MIN} above maximum 0.95"
    );
}

/// Ablated-S ceiling is within valid range [0.0, 0.3].
#[test]
fn sync_ecs_ablated_s_max_in_valid_range() {
    assert!(
        SYNC_ECS_ABLATED_S_MAX >= 0.0,
        "SYNC_ECS_ABLATED_S_MAX={SYNC_ECS_ABLATED_S_MAX} below minimum 0.0"
    );
    assert!(
        SYNC_ECS_ABLATED_S_MAX <= 0.3,
        "SYNC_ECS_ABLATED_S_MAX={SYNC_ECS_ABLATED_S_MAX} above maximum 0.3"
    );
}

/// Consistency: the guaranteed gap between coupled PASS and ablated ceiling meets
/// the same minimum emergence gap the analytic experiment declares.
#[test]
fn sync_ecs_thresholds_guarantee_the_emergence_gap() {
    let guaranteed = SYNC_ECS_S_PASS_MIN - SYNC_ECS_ABLATED_S_MAX;
    assert!(
        guaranteed >= SYNC_GAP_MIN,
        "SYNC_ECS_S_PASS_MIN - SYNC_ECS_ABLATED_S_MAX = {guaranteed} < SYNC_GAP_MIN={SYNC_GAP_MIN}"
    );
}

/// Tick budget is a positive, bounded integration horizon (runtime contract < 60 s).
#[test]
fn sync_ecs_ticks_in_valid_range() {
    assert!(SYNC_ECS_TICKS >= 100, "SYNC_ECS_TICKS={SYNC_ECS_TICKS} too short to reach plateau");
    assert!(SYNC_ECS_TICKS <= 10_000, "SYNC_ECS_TICKS={SYNC_ECS_TICKS} breaks the runtime budget");
}

// ─── ECS verdict boundary gates ──────────────────────────────────────────────

/// ECS verdict passes just inside both thresholds.
#[test]
fn ecs_verdict_passes_inside_boundary() {
    let s = SYNC_ECS_S_PASS_MIN + 1e-4;
    let s_ablated = SYNC_ECS_ABLATED_S_MAX - 1e-4;
    assert!(
        sync_ecs_verdict(s, s_ablated),
        "verdict should PASS at S={s}, S_ablated={s_ablated}"
    );
}

/// ECS verdict fails when the coupled collapse is below the PASS threshold.
#[test]
fn ecs_verdict_fails_when_spread_does_not_collapse() {
    let s = SYNC_ECS_S_PASS_MIN - 1e-4;
    assert!(
        !sync_ecs_verdict(s, 0.0),
        "verdict should FAIL below SYNC_ECS_S_PASS_MIN (S={s})"
    );
}

/// ECS verdict fails when an ablated condition still collapses the spread.
#[test]
fn ecs_verdict_fails_when_ablated_too_high() {
    let s_ablated = SYNC_ECS_ABLATED_S_MAX + 1e-4;
    assert!(
        !sync_ecs_verdict(SYNC_ECS_S_PASS_MIN + 1e-4, s_ablated),
        "verdict should FAIL when ablation does not suppress order (S_ablated={s_ablated})"
    );
}
