//! ET-11: Multi-Scale Information — ecuaciones puras. Sin deps de Bevy.

/// Señal agregada ponderada: combina local, regional y global según pesos.
pub fn aggregate_signal(local: f32, regional: f32, global: f32, weights: [f32; 3]) -> f32 {
    local * weights[0] + regional * weights[1] + global * weights[2]
}

/// Relevancia de una escala para un horizonte de planificación dado.
/// Gaussiana: más relevante cuando horizon ≈ tau.
pub fn scale_relevance(horizon_ticks: u32, scale_tau: u32) -> f32 {
    if scale_tau == 0 {
        return 1.0;
    }
    let ratio = horizon_ticks as f32 / scale_tau as f32;
    (-(ratio - 1.0).powi(2)).exp() // peaked at horizon == tau
}

/// Gradiente de señal entre escalas: dirección de movimiento óptimo.
pub fn information_gradient(local: f32, regional: f32, scale_distance: f32) -> f32 {
    if scale_distance <= 0.0 {
        return 0.0;
    }
    (regional - local) / scale_distance
}

/// Atenuación de señal con la distancia (ley de potencias).
pub fn signal_attenuation(base_signal: f32, distance: f32, attenuation_exp: f32) -> f32 {
    if distance <= 0.0 {
        return base_signal;
    }
    base_signal / (1.0 + distance.powf(attenuation_exp))
}

/// Ruido de información: incertidumbre al agregar señales heterogéneas.
pub fn aggregation_noise(n_sources: u32, source_variance: f32) -> f32 {
    if n_sources == 0 {
        return source_variance;
    }
    source_variance / (n_sources as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_signal_uniform_weights() {
        let s = aggregate_signal(3.0, 6.0, 9.0, [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
        assert!((s - 6.0).abs() < 1e-4);
    }

    #[test]
    fn scale_relevance_peak_at_matching_tau() {
        let r_match = scale_relevance(100, 100);
        let r_mismatch = scale_relevance(1, 100);
        assert!(r_match > r_mismatch);
    }

    #[test]
    fn scale_relevance_zero_tau_returns_one() {
        assert!((scale_relevance(100, 0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn information_gradient_zero_scale_distance() {
        assert_eq!(information_gradient(5.0, 10.0, 0.0), 0.0);
    }

    #[test]
    fn information_gradient_positive_when_regional_higher() {
        assert!(information_gradient(5.0, 10.0, 32.0) > 0.0);
    }

    #[test]
    fn signal_attenuation_no_distance() {
        assert!((signal_attenuation(100.0, 0.0, 2.0) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn aggregation_noise_decreases_with_sources() {
        let n1 = aggregation_noise(1, 1.0);
        let n4 = aggregation_noise(4, 1.0);
        assert!(n1 > n4);
    }
}
