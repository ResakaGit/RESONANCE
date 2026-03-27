//! ET-2: Theory of Mind — ecuaciones puras. Sin deps de Bevy.

/// Precisión del modelo: 1 = predicción perfecta, 0 = completamente errado.
pub fn model_accuracy(predicted_freq: f32, actual_freq: f32, max_freq_deviation: f32) -> f32 {
    let error = (predicted_freq - actual_freq).abs();
    (1.0 - error / max_freq_deviation.max(f32::EPSILON)).clamp(0.0, 1.0)
}

/// Actualiza la predicción del modelo con un error observado (gradiente).
pub fn update_prediction(current_prediction: f32, actual_value: f32, learning_rate: f32) -> f32 {
    current_prediction + learning_rate * (actual_value - current_prediction)
}

/// Costo de mantener un modelo de otro agente. Modelos precisos son más caros.
pub fn model_maintenance_cost(accuracy: f32, base_cost: f32) -> f32 {
    base_cost * (1.0 + accuracy)
}

/// Valor de la deception: qe ganado si el rival tiene modelo incorrecto de ti.
pub fn deception_value(misprediction_magnitude: f32, energy_at_stake: f32, false_signal_cost: f32) -> f32 {
    misprediction_magnitude * energy_at_stake - false_signal_cost
}

/// ¿Vale mantener el modelo? Rentable si la intercepción esperada supera el costo.
pub fn is_model_worth_maintaining(expected_interception: f32, maintenance_cost: f32) -> bool {
    expected_interception > maintenance_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_accuracy_perfect_prediction() {
        assert!((model_accuracy(440.0, 440.0, 500.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn model_accuracy_max_error_returns_zero() {
        assert!((model_accuracy(440.0, 940.0, 500.0)).abs() < 1e-5);
    }

    #[test]
    fn model_accuracy_partial_error() {
        assert!((model_accuracy(440.0, 540.0, 500.0) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn update_prediction_converges() {
        assert!((update_prediction(440.0, 500.0, 0.1) - 446.0).abs() < 1e-4);
    }

    #[test]
    fn update_prediction_zero_rate_unchanged() {
        assert!((update_prediction(440.0, 500.0, 0.0) - 440.0).abs() < 1e-5);
    }

    #[test]
    fn is_model_worth_maintaining_profitable() {
        assert!(is_model_worth_maintaining(5.0, 3.0));
    }

    #[test]
    fn is_model_worth_maintaining_not_profitable() {
        assert!(!is_model_worth_maintaining(1.0, 3.0));
    }

    #[test]
    fn model_maintenance_cost_scales_with_accuracy() {
        let cheap = model_maintenance_cost(0.0, 1.0);
        let expensive = model_maintenance_cost(1.0, 1.0);
        assert!(expensive > cheap);
    }

    #[test]
    fn deception_value_positive_when_profitable() {
        assert!(deception_value(1.0, 10.0, 2.0) > 0.0);
    }

    #[test]
    fn deception_value_negative_when_costly() {
        assert!(deception_value(0.1, 1.0, 5.0) < 0.0);
    }
}
