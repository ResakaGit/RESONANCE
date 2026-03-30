//! EC-7: Scale-Invariant Composition — fitness inferido + propagación cross-scale.
//! EC-7A: `infer_pool_fitness`, `infer_intake_rate`.
//! EC-7B: `propagate_fitness_to_link`, `classify_competitive_regime`, `CompetitiveRegime`.


use crate::blueprint::constants::{
    COMPLEXITY_CAP, COMPLEXITY_FITNESS_WEIGHT, EXTRACTION_EPSILON, FITNESS_MAX,
    REGIME_ABUNDANCE_INTENSITY_THRESHOLD, REGIME_DOMINANCE_INTENSITY_THRESHOLD,
};
use super::dynamics::PoolHealthStatus;

// ─── EC-7B: Régimen Competitivo ───────────────────────────────────────────────

/// Régimen competitivo de un pool energético inferido desde sus métricas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompetitiveRegime {
    /// Recursos abundantes, poca competencia.
    Abundance,
    /// Competencia moderada, equilibrio posible.
    Contested,
    /// Competencia intensa, dominancia emergiendo.
    Dominance,
    /// Recursos insuficientes, colapso inminente.
    Scarcity,
}

// ─── EC-7A: Fitness Inferido ──────────────────────────────────────────────────

/// Infiere el fitness de un pool-padre desde el desempeño de sus hijos.
/// `efficiency = total_retained / max(total_extracted, ε)`.
/// `complexity_bonus = (structural_complexity * COMPLEXITY_FITNESS_WEIGHT).min(COMPLEXITY_CAP)`.
/// `fitness = efficiency * (1 + complexity_bonus)`, clamped `[0, FITNESS_MAX]`.
pub fn infer_pool_fitness(
    total_retained: f32,
    _total_dissipated: f32,
    total_extracted: f32,
    structural_complexity: f32,
) -> f32 {
    let efficiency       = total_retained / total_extracted.max(EXTRACTION_EPSILON);
    let complexity_bonus = (structural_complexity * COMPLEXITY_FITNESS_WEIGHT).min(COMPLEXITY_CAP);
    (efficiency * (1.0 + complexity_bonus)).clamp(0.0, FITNESS_MAX)
}

/// Infiere la tasa de intake de un pool a partir de la eficiencia de su pipeline interno.
/// `effective_intake = base_intake * internal_efficiency.clamp(0, 1)`.
pub fn infer_intake_rate(base_intake: f32, internal_efficiency: f32) -> f32 {
    base_intake * internal_efficiency.clamp(0.0, 1.0)
}

// ─── EC-7B: Propagación Cross-Scale ──────────────────────────────────────────

/// Propaga fitness inferido al `primary_param` de un `PoolParentLink` vía lerp.
/// `blend_rate = 0.0` → sin cambio; `blend_rate = 1.0` → reemplaza por completo.
pub fn propagate_fitness_to_link(
    inferred_fitness: f32,
    current_primary_param: f32,
    blend_rate: f32,
) -> f32 {
    let rate = blend_rate.clamp(0.0, 1.0);
    current_primary_param + rate * (inferred_fitness - current_primary_param)
}

/// Clasifica el régimen competitivo de un pool desde sus métricas.
/// `_active_children` reservado para v2 (proxy de diversidad).
pub fn classify_competitive_regime(
    competition_intensity: f32,
    health_status: PoolHealthStatus,
    _active_children: u16,
) -> CompetitiveRegime {
    match health_status {
        PoolHealthStatus::Collapsed | PoolHealthStatus::Collapsing => CompetitiveRegime::Scarcity,
        PoolHealthStatus::Healthy if competition_intensity < REGIME_ABUNDANCE_INTENSITY_THRESHOLD  => CompetitiveRegime::Abundance,
        _ if competition_intensity >= REGIME_DOMINANCE_INTENSITY_THRESHOLD                        => CompetitiveRegime::Dominance,
        _                                                                                          => CompetitiveRegime::Contested,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::dynamics::PoolHealthStatus;
    use crate::blueprint::constants::FITNESS_MAX;

    // ── EC-7A: infer_pool_fitness ────────────────────────────────────────────

    #[test]
    fn infer_pool_fitness_nominal_case() {
        // efficiency=0.8, complexity_bonus=(3*COMPLEXITY_FITNESS_WEIGHT).min(COMPLEXITY_CAP)
        // raw=efficiency*(1+complexity_bonus), then clamped to [0, FITNESS_MAX]
        let f = infer_pool_fitness(800.0, 100.0, 1000.0, 3.0);
        assert!(f >= 0.0 && f <= FITNESS_MAX, "out of range: got {f}");
        assert!(f.is_finite(), "got {f}");
    }

    #[test]
    fn infer_pool_fitness_all_dissipated() {
        let f = infer_pool_fitness(0.0, 500.0, 500.0, 2.0);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn infer_pool_fitness_no_complexity_bonus() {
        let f = infer_pool_fitness(900.0, 50.0, 1000.0, 0.0);
        assert!((f - 0.9).abs() < 1e-5, "got {f}");
    }

    #[test]
    fn infer_pool_fitness_always_in_range() {
        let cases = [
            (1200.0_f32, 0.0, 1000.0, 100.0), // retained > extracted → clamped to FITNESS_MAX
            (0.0,        0.0,    0.0,   0.0),  // zero inputs
            (500.0,    500.0, 1000.0,  20.0),  // 50% efficiency, high complexity
        ];
        for (retained, dissipated, extracted, complexity) in cases {
            let f = infer_pool_fitness(retained, dissipated, extracted, complexity);
            assert!(
                f >= 0.0 && f <= FITNESS_MAX,
                "out of [0, FITNESS_MAX]: {f} for ({retained},{dissipated},{extracted},{complexity})",
            );
        }
    }

    #[test]
    fn infer_pool_fitness_zero_extracted_no_nan() {
        let f = infer_pool_fitness(0.0, 0.0, 0.0, 1.0);
        assert!(f.is_finite(), "must be finite: {f}");
    }

    // ── EC-7A: infer_intake_rate ─────────────────────────────────────────────

    #[test]
    fn infer_intake_rate_nominal() {
        assert!((infer_intake_rate(100.0, 0.8) - 80.0).abs() < 1e-5);
    }

    #[test]
    fn infer_intake_rate_zero_efficiency() {
        assert_eq!(infer_intake_rate(100.0, 0.0), 0.0);
    }

    #[test]
    fn infer_intake_rate_clamps_over_one() {
        assert!((infer_intake_rate(100.0, 2.0) - 100.0).abs() < 1e-5);
    }

    // ── EC-7B: propagate_fitness_to_link ────────────────────────────────────

    #[test]
    fn propagate_fitness_to_link_partial_blend() {
        // 0.5 + 0.1*(0.9 - 0.5) = 0.54
        let v = propagate_fitness_to_link(0.9, 0.5, 0.1);
        assert!((v - 0.54).abs() < 1e-5, "got {v}");
    }

    #[test]
    fn propagate_fitness_to_link_zero_blend() {
        assert!((propagate_fitness_to_link(0.9, 0.5, 0.0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn propagate_fitness_to_link_full_blend() {
        assert!((propagate_fitness_to_link(0.9, 0.5, 1.0) - 0.9).abs() < 1e-5);
    }

    // ── EC-7B: classify_competitive_regime ──────────────────────────────────

    #[test]
    fn classify_regime_abundance() {
        assert_eq!(
            classify_competitive_regime(0.1, PoolHealthStatus::Healthy, 5),
            CompetitiveRegime::Abundance,
        );
    }

    #[test]
    fn classify_regime_dominance() {
        assert_eq!(
            classify_competitive_regime(0.8, PoolHealthStatus::Stressed, 10),
            CompetitiveRegime::Dominance,
        );
    }

    #[test]
    fn classify_regime_scarcity_collapsing() {
        assert_eq!(
            classify_competitive_regime(0.5, PoolHealthStatus::Collapsing, 8),
            CompetitiveRegime::Scarcity,
        );
    }

    #[test]
    fn classify_regime_scarcity_collapsed() {
        assert_eq!(
            classify_competitive_regime(0.0, PoolHealthStatus::Collapsed, 0),
            CompetitiveRegime::Scarcity,
        );
    }

    #[test]
    fn classify_regime_contested_stressed_moderate_intensity() {
        // Stressed + intensity in (0.3, 0.6) → Contested
        assert_eq!(
            classify_competitive_regime(0.4, PoolHealthStatus::Stressed, 5),
            CompetitiveRegime::Contested,
        );
    }

    #[test]
    fn classify_regime_healthy_high_intensity_is_dominance() {
        // Healthy but high Gini → Dominance (one child takes most)
        assert_eq!(
            classify_competitive_regime(0.7, PoolHealthStatus::Healthy, 4),
            CompetitiveRegime::Dominance,
        );
    }
}
