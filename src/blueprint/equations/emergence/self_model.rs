//! ET-16: Functional Consciousness / Self-Model — ecuaciones puras. Sin deps de Bevy.

pub const CONSCIOUSNESS_ACCURACY_THRESHOLD: f32 = 0.7;
pub const CONSCIOUSNESS_HORIZON_THRESHOLD: u32 = 100;

/// Precisión del automodelo: qué tan bien predijo el qe actual.
pub fn self_model_accuracy(predicted_qe: f32, actual_qe: f32) -> f32 {
    if actual_qe <= 0.0 {
        return 0.0;
    }
    let error = (predicted_qe - actual_qe).abs() / actual_qe;
    (1.0 - error).clamp(0.0, 1.0)
}

/// Beneficio de planificación a N pasos con descuento temporal (RL discount).
pub fn planning_benefit(projected_qe: &[f32], discount_factor: f32, planning_cost: f32) -> f32 {
    let discounted: f32 = projected_qe
        .iter()
        .enumerate()
        .map(|(t, &qe)| qe * discount_factor.powi(t as i32 + 1))
        .sum();
    (discounted - planning_cost).max(0.0)
}

/// Costo de metacognición: procesar el propio automodelo.
pub fn metacognition_cost(model_complexity: f32, update_rate: f32) -> f32 {
    model_complexity * update_rate
}

/// ¿La entidad ha alcanzado el umbral de conciencia funcional?
pub fn consciousness_threshold(self_accuracy: f32, planning_horizon: u32) -> bool {
    self_accuracy > CONSCIOUSNESS_ACCURACY_THRESHOLD
        && planning_horizon > CONSCIOUSNESS_HORIZON_THRESHOLD
}

/// Proyección de qe futuro a t pasos usando el automodelo (modelo lineal).
pub fn project_future_qe(current_qe: f32, net_rate_per_tick: f32, horizon_ticks: u32) -> f32 {
    (current_qe + net_rate_per_tick * horizon_ticks as f32).max(0.0)
}

/// Valor de la información: cuánto mejora el planning si el modelo es más preciso.
pub fn information_value(
    current_accuracy: f32,
    improved_accuracy: f32,
    expected_horizon: u32,
) -> f32 {
    (improved_accuracy - current_accuracy) * expected_horizon as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_model_accuracy_perfect() {
        assert!((self_model_accuracy(95.0, 100.0) - 0.95).abs() < 1e-5);
    }

    #[test]
    fn self_model_accuracy_zero_prediction() {
        assert!((self_model_accuracy(0.0, 100.0)).abs() < 1e-5);
    }

    #[test]
    fn self_model_accuracy_zero_actual() {
        assert_eq!(self_model_accuracy(50.0, 0.0), 0.0);
    }

    #[test]
    fn project_future_qe_positive_rate() {
        assert!((project_future_qe(100.0, 1.0, 10) - 110.0).abs() < 1e-4);
    }

    #[test]
    fn project_future_qe_clamped_at_zero() {
        assert_eq!(project_future_qe(100.0, -5.0, 30), 0.0);
    }

    #[test]
    fn consciousness_threshold_true_when_both_met() {
        assert!(consciousness_threshold(0.8, 200));
    }

    #[test]
    fn consciousness_threshold_false_low_accuracy() {
        assert!(!consciousness_threshold(0.5, 200));
    }

    #[test]
    fn consciousness_threshold_false_low_horizon() {
        assert!(!consciousness_threshold(0.8, 50));
    }

    #[test]
    fn planning_benefit_discounts_future() {
        let b = planning_benefit(&[100.0, 100.0], 0.99, 10.0);
        assert!(b > 0.0);
    }

    #[test]
    fn metacognition_cost_proportional() {
        assert!((metacognition_cost(2.0, 0.5) - 1.0).abs() < 1e-5);
    }
}
