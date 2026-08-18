//! R8 Surrogate Reliability — pure-function integration tests.
//! No Bevy App required.

use resonance::blueprint::{
    constants::{
        SURROGATE_ENERGY_EPSILON, SURROGATE_FITNESS_EPSILON, SURROGATE_MIN_HIT_RATE,
        SURROGATE_TOP_K_EPSILON,
    },
    equations::surrogate_error::*,
};

#[test]
fn absolute_error_exact_match_is_zero() {
    assert_eq!(surrogate_absolute_error(100.0, 100.0), 0.0);
}

#[test]
fn relative_error_small_deviation_within_epsilon() {
    // surrogate=95.0, exact=100.0 → relative error = 0.05 = SURROGATE_FITNESS_EPSILON
    let err = surrogate_relative_error(95.0, 100.0);
    assert!(
        (err - SURROGATE_FITNESS_EPSILON).abs() < 1e-6,
        "expected {SURROGATE_FITNESS_EPSILON}, got {err}"
    );
}

#[test]
fn is_within_epsilon_passes_when_close() {
    // surrogate=99.0, exact=100.0 → 1% error < 5% epsilon → true
    assert!(
        is_within_epsilon(99.0, 100.0, SURROGATE_FITNESS_EPSILON),
        "1% error should be within 5% epsilon"
    );
}

#[test]
fn max_surrogate_error_finds_worst_case() {
    // pairs=[(99,100),(85,100),(98,100)] → absolute errors: 1, 15, 2 → max=15.0
    let pairs = [(99.0_f32, 100.0_f32), (85.0, 100.0), (98.0, 100.0)];
    assert_eq!(max_surrogate_error(&pairs), 15.0);
}

#[test]
fn mean_absolute_error_computes_correctly() {
    // pairs=[(90,100),(80,100)] → mae=(10+20)/2=15.0
    let pairs = [(90.0_f32, 100.0_f32), (80.0, 100.0)];
    assert_eq!(mean_absolute_error(&pairs), 15.0);
}

#[test]
fn cache_hit_rate_100_percent() {
    let (hit_rate, miss_rate) = cache_hit_rate(10, 0);
    assert_eq!(hit_rate, 1.0);
    assert_eq!(miss_rate, 0.0);
}

#[test]
fn cache_hit_rate_meets_minimum_threshold() {
    // 7 hits, 3 misses → 70% hit rate = SURROGATE_MIN_HIT_RATE
    let (hit_rate, _) = cache_hit_rate(7, 3);
    assert!(
        hit_rate >= SURROGATE_MIN_HIT_RATE,
        "hit_rate={hit_rate} should be >= SURROGATE_MIN_HIT_RATE={SURROGATE_MIN_HIT_RATE}"
    );
}

#[test]
fn surrogate_energy_epsilon_constant_is_positive() {
    assert!(SURROGATE_ENERGY_EPSILON > 0.0);
}

#[test]
fn top_k_converged_matching_top_3() {
    // top-3 by exact:    [100,80,60] (idx 0,1,2)
    // top-3 by surrogate:[99,81,59]  (idx 0,1,2)
    // rank 0: |99-100|/100=0.01 <= 0.05 ✓
    // rank 1: |81-80|/80 =0.0125 <= 0.05 ✓
    // rank 2: |59-60|/60 =0.0167 <= 0.05 ✓
    let surrogate = [99.0_f32, 81.0, 59.0, 40.0, 21.0];
    let exact = [100.0_f32, 80.0, 60.0, 38.0, 20.0];
    assert!(
        top_k_converged(&surrogate, &exact, 3, 0.05),
        "top-3 should converge with 5% epsilon"
    );
}

#[test]
fn top_k_converged_diverging_top_3() {
    // surrogate top-3 idx: 0(100),1(80),2(60)
    // exact top-3 idx:     3(80),4(100)... wait — exact=[20,40,60,80,100] → top-3: idx 4(100),3(80),2(60)
    // rank 0: surrogate=100 vs exact=100 → 0% ✓
    // rank 1: surrogate=80  vs exact=80  → 0% ✓
    // rank 2: surrogate=60  vs exact=60  → 0% ✓
    // That would pass. Use a truly inverted set instead:
    // surrogate=[100,80,60,40,20], exact=[20,40,60,80,100]
    // surrogate top-3: idx 0(100),1(80),2(60) → values 100,80,60
    // exact top-3:     idx 4(100),3(80),2(60) → values 100,80,60
    // Those match! Use a completely disjoint ranking instead.
    // surrogate=[100,90,80,1,2], exact=[1,2,3,100,90]
    // surrogate top-3: idx 0(100),1(90),2(80)   → values 100,90,80
    // exact top-3:     idx 3(100),4(90),2(3)    → values 100,90,3
    // rank 2: |80 - 3| / 3 >> epsilon → false ✓
    let surrogate = [100.0_f32, 90.0, 80.0, 1.0, 2.0];
    let exact = [1.0_f32, 2.0, 3.0, 100.0, 90.0];
    assert!(
        !top_k_converged(&surrogate, &exact, 3, SURROGATE_TOP_K_EPSILON),
        "top-3 surrogate and exact differ at rank 2 and should NOT converge"
    );
}

#[test]
fn top_k_converged_k_zero_returns_false() {
    let s = [1.0_f32, 2.0];
    let e = [1.0_f32, 2.0];
    assert!(!top_k_converged(&s, &e, 0, 0.05));
}

#[test]
fn top_k_converged_k_larger_than_slice_uses_len() {
    // k=10 but only 2 elements — should compare 2 ranks and succeed if values match
    let s = [10.0_f32, 5.0];
    let e = [10.0_f32, 5.0];
    assert!(top_k_converged(&s, &e, 10, 0.01));
}
