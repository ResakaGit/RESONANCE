//! ET-3 + AC-3: Cultural Transmission — ecuaciones puras.
//!
//! AC-3 adds oscillatory affinity weighting (Axiom 6×8):
//! entities share culture more readily when their frequencies are compatible.

/// Fitness de un comportamiento (meme): mejora esperada de extracción menos costos.
pub fn meme_fitness(extraction_improvement: f32, adoption_cost: f32, maintenance_cost: f32) -> f32 {
    extraction_improvement - adoption_cost - maintenance_cost
}

/// Tasa de propagación de un meme. Cero si fitness negativa.
pub fn spread_rate(fitness: f32, contact_rate: f32, imitation_prob: f32) -> f32 {
    if fitness <= 0.0 { return 0.0; }
    fitness * contact_rate * imitation_prob
}

/// ¿Vale imitar? Compara qe/tick observado con propio ajustando incertidumbre.
pub fn should_imitate(
    observer_current_rate: f32,
    target_observed_rate: f32,
    adoption_cost: f32,
    uncertainty: f32,
) -> bool {
    let expected_gain = (target_observed_rate - observer_current_rate) * (1.0 - uncertainty);
    expected_gain > adoption_cost
}

// ── AC-3: Frequency × Culture (Axiom 6×8) ───────────────────────────────────

/// Imitation affinity from oscillatory alignment between observer and model.
///
/// Equation: `cos(2π × |Δfreq| × t + Δphase).clamp(0.0, 1.0)`
///
/// Range: `[0.0, 1.0]`
/// - `1.0` → same-band, in-phase — maximum imitation affinity
/// - `0.0` → opposite phase or cross-band — imitation suppressed
///
/// Note: uses `clamp(0, 1)` not `[-1, 1]` — destructive interference blocks
/// imitation rather than reversing it. This differs from catalysis interference.
///
/// `t` should be `SimulationElapsed.secs` for deterministic results.
pub fn frequency_imitation_affinity(
    observer_freq: f32, observer_phase: f32,
    model_freq:    f32, model_phase:    f32,
    t: f32,
) -> f32 {
    use crate::blueprint::equations::core_physics;
    // SSOT: reuse core_physics::interference; clamp [0,1] vs [-1,1] used by catalysis.
    core_physics::interference(observer_freq, observer_phase, model_freq, model_phase, t)
        .clamp(0.0, 1.0)
}

/// Group coherence bonus for imitation: high-coherence groups are more worth imitating.
///
/// Range: `[1.0, 1.0 + bonus_cap]`
/// - `1.0` → incoherent group (no bonus)
/// - `1.0 + bonus_cap` → perfectly coherent group (maximum bonus)
pub fn group_coherence_imitation_bonus(group_coherence: f32, bonus_cap: f32) -> f32 {
    1.0 + (group_coherence.clamp(0.0, 1.0) * bonus_cap.max(0.0))
}

/// ¿Vale imitar considerando afinidad oscilatoria? (AC-3 extended version).
///
/// Multiplies the expected gain by `affinity × coherence_bonus` before
/// comparing against `adoption_cost`. Cross-band or incoherent models are
/// harder to imitate — the gain signal is attenuated.
pub fn should_imitate_with_affinity(
    observer_current_rate: f32,
    target_observed_rate:  f32,
    adoption_cost:         f32,
    uncertainty:           f32,
    affinity:              f32,
    coherence_bonus:       f32,
) -> bool {
    let base_gain = (target_observed_rate - observer_current_rate) * (1.0 - uncertainty);
    let effective_gain = base_gain * affinity.clamp(0.0, 1.0) * coherence_bonus.max(1.0);
    effective_gain > adoption_cost
}

/// Distancia cultural entre dos poblaciones (L2 en espacio de comportamiento 4D).
pub fn cultural_distance(behavior_a: [f32; 4], behavior_b: [f32; 4]) -> f32 {
    behavior_a.iter()
        .zip(behavior_b.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meme_fitness_positive() {
        assert!((meme_fitness(5.0, 1.0, 0.5) - 3.5).abs() < 1e-5);
    }

    #[test]
    fn meme_fitness_negative_behavior() {
        assert!(meme_fitness(-1.0, 1.0, 0.5) < 0.0);
    }

    #[test]
    fn spread_rate_zero_for_negative_fitness() {
        assert_eq!(spread_rate(-1.0, 5.0, 0.5), 0.0);
    }

    #[test]
    fn spread_rate_positive_for_good_meme() {
        assert!(spread_rate(2.0, 5.0, 0.5) > 0.0);
    }

    #[test]
    fn should_imitate_better_model() {
        assert!(should_imitate(100.0, 200.0, 1.0, 0.0));
    }

    #[test]
    fn should_not_imitate_worse_model() {
        assert!(!should_imitate(200.0, 100.0, 1.0, 0.0));
    }

    #[test]
    fn cultural_distance_orthogonal_vectors() {
        let d = cultural_distance([1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]);
        assert!((d - 2.0f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn cultural_distance_identical_is_zero() {
        assert!((cultural_distance([1.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0])).abs() < 1e-5);
    }

    // ── AC-3: frequency_imitation_affinity ──────────────────────────────────

    #[test]
    fn affinity_same_freq_same_phase_at_t0_is_one() {
        let a = frequency_imitation_affinity(75.0, 0.0, 75.0, 0.0, 0.0);
        assert!((a - 1.0).abs() < 1e-5, "got {a}");
    }

    #[test]
    fn affinity_same_freq_opposite_phase_is_zero() {
        use std::f32::consts::PI;
        let a = frequency_imitation_affinity(75.0, 0.0, 75.0, PI, 0.0);
        assert!(a < 1e-5, "opposite phase should suppress: {a}");
    }

    #[test]
    fn affinity_always_in_unit_range() {
        use std::f32::consts::PI;
        for t in [0.0_f32, 0.1, 1.0, 10.0] {
            let a = frequency_imitation_affinity(75.0, 0.0, 75.0, PI / 2.0, t);
            assert!((0.0..=1.0).contains(&a), "out of range: {a} at t={t}");
        }
    }

    #[test]
    fn affinity_never_negative() {
        use std::f32::consts::PI;
        // Destructive interference → clamp to 0, not negative
        let a = frequency_imitation_affinity(75.0, 0.0, 75.0, PI, 0.0);
        assert!(a >= 0.0, "must not be negative: {a}");
    }

    // ── AC-3: group_coherence_imitation_bonus ───────────────────────────────

    #[test]
    fn coherence_bonus_zero_coherence_returns_one() {
        let b = group_coherence_imitation_bonus(0.0, 0.5);
        assert!((b - 1.0).abs() < 1e-5, "got {b}");
    }

    #[test]
    fn coherence_bonus_full_coherence_returns_one_plus_cap() {
        let b = group_coherence_imitation_bonus(1.0, 0.5);
        assert!((b - 1.5).abs() < 1e-5, "got {b}");
    }

    #[test]
    fn coherence_bonus_always_at_least_one() {
        for c in [0.0_f32, 0.3, 0.7, 1.0] {
            let b = group_coherence_imitation_bonus(c, 0.5);
            assert!(b >= 1.0, "bonus < 1: {b}");
        }
    }

    // ── AC-3: should_imitate_with_affinity ──────────────────────────────────

    #[test]
    fn imitate_with_affinity_one_same_as_base_should_imitate() {
        // affinity=1.0, bonus=1.0 → equivalent to base should_imitate
        let base = should_imitate(100.0, 200.0, 1.0, 0.0);
        let affinity = should_imitate_with_affinity(100.0, 200.0, 1.0, 0.0, 1.0, 1.0);
        assert_eq!(base, affinity);
    }

    #[test]
    fn cross_band_affinity_suppresses_imitation() {
        // Without affinity weighting: would imitate (gain=100 > cost=1)
        assert!(should_imitate(100.0, 200.0, 1.0, 0.0));
        // With affinity=0.0 (opposite phase): effective_gain = 100 * 0.0 = 0 < cost=1 → no imitate
        assert!(!should_imitate_with_affinity(100.0, 200.0, 1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn coherence_bonus_enables_marginal_imitation() {
        // Marginal case: affinity=0.5, base_gain=2, cost=1
        // Without bonus: effective = 2 * 0.5 = 1.0, not > 1.0 → no imitate
        assert!(!should_imitate_with_affinity(100.0, 102.0, 1.0, 0.0, 0.5, 1.0));
        // With coherence bonus=1.2: effective = 2 * 0.5 * 1.2 = 1.2 > 1.0 → imitate
        assert!(should_imitate_with_affinity(100.0, 102.0, 1.0, 0.0, 0.5, 1.2));
    }
}
