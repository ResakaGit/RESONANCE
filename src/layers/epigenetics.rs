//! ET-6: Epigenetic Expression — EpigeneticState component. Capa T2-2.

use bevy::prelude::*;

/// Capa T2-2: EpigeneticState — máscara de expresión sobre InferenceProfile.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct EpigeneticState {
    pub expression_mask: [f32; 4], // [0,1] por dimensión del InferenceProfile
    pub adaptation_speed: f32,
    pub silencing_cost: f32,
    pub env_sample_rate: u8, // cada cuántos ticks re-samplea el entorno
}

impl Default for EpigeneticState {
    fn default() -> Self {
        Self {
            expression_mask: [1.0; 4],
            adaptation_speed: 0.05,
            silencing_cost: 0.5,
            env_sample_rate: 16,
        }
    }
}

impl EpigeneticState {
    pub fn expression(&self, dim: usize) -> f32 {
        *self.expression_mask.get(dim).unwrap_or(&0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_expression_mask_is_fully_expressed() {
        let e = EpigeneticState::default();
        for dim in 0..4 {
            assert!((e.expression(dim) - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn expression_out_of_range_returns_zero() {
        let e = EpigeneticState::default();
        assert_eq!(e.expression(4), 0.0);
        assert_eq!(e.expression(100), 0.0);
    }

    #[test]
    fn custom_mask_values() {
        let e = EpigeneticState {
            expression_mask: [0.5, 0.0, 1.0, 0.3],
            ..Default::default()
        };
        assert!((e.expression(0) - 0.5).abs() < 1e-5);
        assert_eq!(e.expression(1), 0.0);
        assert!((e.expression(2) - 1.0).abs() < 1e-5);
        assert!((e.expression(3) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn default_adaptation_speed() {
        let e = EpigeneticState::default();
        assert!((e.adaptation_speed - 0.05).abs() < 1e-5);
    }
}
