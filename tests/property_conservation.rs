//! Property-based tests for conservation invariants and pool equations.
//! Uses proptest to fuzz extraction functions with arbitrary valid inputs.

use proptest::prelude::*;
use resonance::blueprint::equations::conservation::{
    conservation_error, global_conservation_error, has_invalid_values, is_valid_qe,
};
use resonance::blueprint::equations::{
    dissipation_loss, extract_aggressive, extract_competitive,
    extract_greedy, extract_proportional, extract_regulated, pool_next_tick, relative_fitness,
    scale_extractions_to_available, ticks_to_collapse,
};

// ─── Generators ──────────────────────────────────────────────────────────────

fn qe_value() -> impl Strategy<Value = f32> {
    prop_oneof![
        Just(0.0_f32),
        0.001_f32..1e6,
    ]
}

#[allow(dead_code)]
fn rate_value() -> impl Strategy<Value = f32> {
    0.0_f32..1.0
}

fn unit_value() -> impl Strategy<Value = f32> {
    0.0_f32..=1.0
}

fn sibling_count() -> impl Strategy<Value = u32> {
    1_u32..32
}

// ─── Conservation: is_valid_qe ───────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_valid_qe_is_finite_non_negative(qe in qe_value()) {
        prop_assert!(is_valid_qe(qe));
    }

    #[test]
    fn prop_nan_inf_always_invalid(qe in prop::num::f32::ANY) {
        if !qe.is_finite() || qe < 0.0 {
            prop_assert!(!is_valid_qe(qe));
        }
    }

    #[test]
    fn prop_has_invalid_values_consistent_with_is_valid(
        vals in prop::collection::vec(prop::num::f32::ANY, 1..16)
    ) {
        let any_bad = vals.iter().any(|v| !v.is_finite());
        prop_assert_eq!(has_invalid_values(&vals), any_bad);
    }
}

// ─── Conservation: global_conservation_error ─────────────────────────────────

proptest! {
    #[test]
    fn prop_global_conservation_error_non_negative(
        available in qe_value(),
        extracted in prop::collection::vec(qe_value(), 1..8),
    ) {
        let err = global_conservation_error(available, &extracted);
        prop_assert!(err >= 0.0, "error must be >= 0, got {err}");
        prop_assert!(err.is_finite(), "error must be finite, got {err}");
    }

    #[test]
    fn prop_global_conservation_no_overshoot_means_zero(
        available in 500.0_f32..1e6,
        n in 1_usize..8,
    ) {
        let per = available / (n as f32 * 2.0);
        let extracted: Vec<f32> = vec![per; n];
        let err = global_conservation_error(available, &extracted);
        prop_assert!((err - 0.0).abs() < 1e-3, "err={err}");
    }
}

// ─── Conservation: per-pool conservation_error ───────────────────────────────

proptest! {
    #[test]
    fn prop_per_pool_conservation_error_non_negative(
        pool_before in qe_value(),
        intake in qe_value(),
        extracted in qe_value(),
        dissipated in qe_value(),
    ) {
        let pool_after = (pool_before + intake - extracted - dissipated).max(0.0);
        let err = conservation_error(pool_before, pool_after, intake, extracted, dissipated);
        prop_assert!(err >= 0.0);
        prop_assert!(err.is_finite());
    }
}

// ─── Pool: pool_next_tick ────────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_pool_next_tick_never_negative(
        pool in qe_value(),
        intake in qe_value(),
        extracted in qe_value(),
        rate in 0.001_f32..0.5,
    ) {
        let next = pool_next_tick(pool, intake, extracted, rate);
        prop_assert!(next >= 0.0, "pool_next_tick returned {next}");
        prop_assert!(next.is_finite(), "pool_next_tick returned {next}");
    }

    #[test]
    fn prop_pool_next_tick_monotone_in_intake(
        pool in qe_value(),
        intake_a in qe_value(),
        intake_b in qe_value(),
        extracted in qe_value(),
        rate in 0.001_f32..0.5,
    ) {
        let a = pool_next_tick(pool, intake_a, extracted, rate);
        let b = pool_next_tick(pool, intake_b, extracted, rate);
        if intake_a <= intake_b {
            prop_assert!(a <= b + 1e-5, "more intake => more pool: a={a} b={b}");
        }
    }
}

// ─── Pool: dissipation_loss ──────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_dissipation_loss_bounded_by_pool(
        pool in qe_value(),
        rate in 0.0_f32..1.0,
    ) {
        let loss = dissipation_loss(pool, rate);
        prop_assert!(loss >= 0.0);
        prop_assert!(loss <= pool + 1e-5, "loss={loss} pool={pool}");
        prop_assert!(loss.is_finite());
    }
}

// ─── Extraction: proportional ────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_extract_proportional_bounded(
        available in qe_value(),
        n in sibling_count(),
    ) {
        let result = extract_proportional(available, n);
        prop_assert!(result >= 0.0);
        prop_assert!(result <= available + 1e-5);
        prop_assert!(result.is_finite());
    }

    #[test]
    fn prop_extract_proportional_sum_le_available(
        available in qe_value(),
        n in sibling_count(),
    ) {
        let per = extract_proportional(available, n);
        let total = per * n as f32;
        // f32 rounding: n × (available/n) can exceed available by ~n ULPs.
        let tolerance = (n as f32) * available * f32::EPSILON * 4.0 + 1e-5;
        prop_assert!(total <= available + tolerance, "total={total} available={available} n={n}");
    }
}

// ─── Extraction: greedy ──────────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_extract_greedy_bounded(
        available in qe_value(),
        capacity in qe_value(),
    ) {
        let result = extract_greedy(available, capacity);
        prop_assert!(result >= 0.0);
        prop_assert!(result <= available + 1e-5);
        prop_assert!(result <= capacity + 1e-5);
    }
}

// ─── Extraction: competitive ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_extract_competitive_non_negative_finite(
        available in qe_value(),
        fitness in qe_value(),
        total_fitness in qe_value(),
    ) {
        let result = extract_competitive(available, fitness, total_fitness);
        prop_assert!(result >= 0.0);
        prop_assert!(result.is_finite());
    }

    /// When individual fitness ≤ total, extraction ≤ available.
    #[test]
    fn prop_extract_competitive_bounded_when_fitness_le_total(
        available in qe_value(),
        fitness in 0.0_f32..100.0,
        extra in 0.0_f32..100.0,
    ) {
        let total_fitness = fitness + extra;
        let result = extract_competitive(available, fitness, total_fitness);
        prop_assert!(result <= available + 1e-3, "result={result} available={available}");
    }
}

// ─── Extraction: aggressive ──────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_extract_aggressive_bounded(
        available in qe_value(),
        aggression in unit_value(),
        damage_rate in unit_value(),
    ) {
        let (taken, damage) = extract_aggressive(available, aggression, damage_rate);
        prop_assert!(taken >= 0.0);
        prop_assert!(taken <= available + 1e-5);
        prop_assert!(damage >= 0.0);
        prop_assert!(damage <= taken + 1e-5);
        prop_assert!(taken.is_finite());
        prop_assert!(damage.is_finite());
    }
}

// ─── Extraction: regulated ───────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_extract_regulated_non_negative(
        available in qe_value(),
        pool_ratio in unit_value(),
        base_rate in 0.0_f32..1000.0,
        lo in unit_value(),
        hi in unit_value(),
    ) {
        let result = extract_regulated(available, pool_ratio, base_rate, lo, hi);
        prop_assert!(result >= 0.0);
        prop_assert!(result.is_finite());
    }
}

// ─── Fitness: relative_fitness ───────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_relative_fitness_in_unit_range(
        fitness in qe_value(),
        siblings in prop::collection::vec(qe_value(), 1..8),
    ) {
        let result = relative_fitness(fitness, &siblings);
        prop_assert!(result >= 0.0);
        prop_assert!(result <= 1.0 + 1e-5);
        prop_assert!(result.is_finite());
    }
}

// ─── Scaling: scale_extractions_to_available ─────────────────────────────────

proptest! {
    #[test]
    fn prop_scale_extractions_invariant(
        mut extractions in prop::collection::vec(qe_value(), 1..8),
        available in qe_value(),
    ) {
        scale_extractions_to_available(&mut extractions, available);
        let sum: f32 = extractions.iter().sum();
        // POOL_CONSERVATION_EPSILON = 1e-3; f32 accumulation adds error proportional to n.
        let tolerance = 1e-3 + extractions.len() as f32 * 1e-1;
        prop_assert!(sum <= available + tolerance, "sum={sum} available={available}");
        for v in &extractions {
            prop_assert!(*v >= 0.0);
            prop_assert!(v.is_finite());
        }
    }
}

// ─── State: ticks_to_collapse ────────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_ticks_to_collapse_consistent(
        pool in qe_value(),
        drain in qe_value(),
    ) {
        let ticks = ticks_to_collapse(pool, drain);
        if drain <= 0.0 {
            prop_assert_eq!(ticks, u32::MAX);
        } else {
            let expected = (pool / drain).ceil() as u32;
            prop_assert_eq!(ticks, expected);
        }
    }
}
