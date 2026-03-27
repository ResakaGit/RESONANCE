//! ET-7: Programmed Senescence — ecuaciones puras. Sin deps de Bevy.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReproductionStrategy { Iteroparous, Semelparous }

/// Tasa de disipación dependiente de la edad.
/// `dissipation_rate(t) = base × (1 + coeff × t)`
pub fn age_dependent_dissipation(base_dissipation: f32, tick_age: u64, senescence_coeff: f32) -> f32 {
    base_dissipation * (1.0 + senescence_coeff * tick_age as f32)
}

/// Probabilidad de sobrevivir hasta la edad t (aproximación discreta de la integral).
pub fn survival_probability(tick_age: u64, base_dissipation: f32, senescence_coeff: f32) -> f32 {
    let integrated = base_dissipation * tick_age as f32
        + 0.5 * base_dissipation * senescence_coeff * (tick_age as f32).powi(2);
    (-integrated).exp().clamp(0.0, 1.0)
}

/// Estrategia óptima de reproducción dada la varianza ambiental.
pub fn optimal_reproduction_strategy(env_variance: f32, offspring_survival_rate: f32) -> ReproductionStrategy {
    if env_variance > 0.5 || offspring_survival_rate < 0.3 {
        ReproductionStrategy::Iteroparous
    } else {
        ReproductionStrategy::Semelparous
    }
}

/// Presión kin-selection: valor de un acto de ayuda a un pariente.
/// relatedness: [0,1] parentesco genético.
pub fn kin_selection_value(relatedness: f32, benefit_to_kin: f32, cost_to_self: f32) -> f32 {
    relatedness * benefit_to_kin - cost_to_self
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_dependent_dissipation_young() {
        assert!((age_dependent_dissipation(1.0, 0, 0.0001) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn age_dependent_dissipation_old_doubles() {
        assert!((age_dependent_dissipation(1.0, 10000, 0.0001) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn survival_probability_at_birth_is_one() {
        assert!((survival_probability(0, 0.01, 0.0001) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn survival_probability_decreases_with_age() {
        let s0 = survival_probability(0, 0.01, 0.0001);
        let s10k = survival_probability(10000, 0.01, 0.0001);
        assert!(s10k < s0);
        assert!(s10k < 0.5);
    }

    #[test]
    fn kin_selection_value_positive_when_beneficial() {
        assert!((kin_selection_value(0.5, 10.0, 3.0) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn kin_selection_value_negative_when_too_costly() {
        assert!(kin_selection_value(0.1, 5.0, 10.0) < 0.0);
    }

    #[test]
    fn optimal_strategy_high_variance_is_iteroparous() {
        assert_eq!(optimal_reproduction_strategy(0.8, 0.5), ReproductionStrategy::Iteroparous);
    }

    #[test]
    fn optimal_strategy_stable_env_is_semelparous() {
        assert_eq!(optimal_reproduction_strategy(0.1, 0.8), ReproductionStrategy::Semelparous);
    }
}
