//! ET-10: Multiple Timescales (Baldwin Effect) — ecuaciones puras. Sin deps de Bevy.

/// Fenotipo efectivo integrando los cuatro timescales.
pub fn effective_trait(
    genetic_baseline: f32,
    epigenetic_offset: f32,
    cultural_offset: f32,
    learned_offset: f32,
) -> f32 {
    genetic_baseline + epigenetic_offset + cultural_offset + learned_offset
}

/// Tasa de fijación genética del efecto Baldwin.
/// fitness_delta: mejora de qe/tick. selection_pressure: fuerza de selección.
pub fn baldwin_fixation_rate(
    fitness_delta: f32,
    selection_pressure: f32,
    genetic_timescale: u32,
) -> f32 {
    if genetic_timescale == 0 {
        return 0.0;
    }
    fitness_delta * selection_pressure / genetic_timescale as f32
}

/// Peso relativo de cada timescale según la varianza del entorno.
/// Alta varianza → más peso en aprendizaje (respuesta rápida).
pub fn timescale_weight(env_variance: f32, timescale_tau: f32) -> f32 {
    let responsiveness = 1.0 / (timescale_tau + 1.0);
    (responsiveness * env_variance).clamp(0.0, 1.0)
}

/// Plasticidad fenotípica: capacidad de responder a cambios en la escala τ.
pub fn phenotypic_plasticity(
    max_plastic_range: f32,
    developmental_cost: f32,
    env_predictability: f32,
) -> f32 {
    let need = 1.0 - env_predictability;
    (max_plastic_range * need - developmental_cost).max(0.0)
}

/// Transferencia de offset entre timescales (aprendido → cultural).
pub fn timescale_transfer_rate(
    offset_source: f32,
    transfer_coefficient: f32,
    population_density: f32,
) -> f32 {
    offset_source * transfer_coefficient * population_density.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_trait_sums_all_offsets() {
        assert!((effective_trait(1.0, 0.1, 0.2, 0.3) - 1.6).abs() < 1e-5);
    }

    #[test]
    fn effective_trait_zero_offsets_equals_baseline() {
        assert!((effective_trait(5.0, 0.0, 0.0, 0.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn baldwin_fixation_rate_zero_timescale() {
        assert_eq!(baldwin_fixation_rate(1.0, 1.0, 0), 0.0);
    }

    #[test]
    fn baldwin_fixation_rate_positive() {
        assert!(baldwin_fixation_rate(5.0, 0.5, 1000) > 0.0);
    }

    #[test]
    fn timescale_weight_high_variance_fast_response() {
        let w_fast = timescale_weight(1.0, 10.0);
        let w_slow = timescale_weight(1.0, 100.0);
        assert!(w_fast > w_slow);
    }

    #[test]
    fn phenotypic_plasticity_zero_in_predictable_env() {
        let p = phenotypic_plasticity(1.0, 0.5, 1.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn timescale_transfer_positive() {
        assert!(timescale_transfer_rate(1.0, 0.1, 4.0) > 0.0);
    }
}
