//! ET-2: Theory of Mind — OtherModelSet component. Capa T1-2.

use bevy::prelude::*;

/// Modelo interno de otro agente (dato inmutable inline — no Component directo).
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct OtherModel {
    pub target_id: u32,      // WorldEntityId del agente modelado
    pub predicted_freq: f32, // frecuencia predicha del target
    pub accuracy: f32,       // [0,1] precisión histórica
    pub update_cost: f32,    // qe gastado en actualizar este tick
}

pub const MAX_MODELS: usize = 4;

/// Capa T1-2: OtherModelSet — conjunto de modelos internos de otros agentes.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct OtherModelSet {
    pub models: [OtherModel; MAX_MODELS],
    pub model_count: u8,
    pub update_interval: u8,
    pub base_model_cost: f32,
}

impl Default for OtherModelSet {
    fn default() -> Self {
        Self {
            models: [OtherModel::default(); MAX_MODELS],
            model_count: 0,
            update_interval: 5,
            base_model_cost: 0.2,
        }
    }
}

impl OtherModelSet {
    pub fn model_count(&self) -> usize {
        self.model_count as usize
    }
    pub fn models_active(&self) -> &[OtherModel] {
        &self.models[..self.model_count as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_count_is_zero() {
        let set = OtherModelSet::default();
        assert_eq!(set.model_count(), 0);
    }

    #[test]
    fn default_models_active_is_empty() {
        let set = OtherModelSet::default();
        assert!(set.models_active().is_empty());
    }

    #[test]
    fn model_count_matches_active_slice_len() {
        let mut set = OtherModelSet::default();
        set.models[0] = OtherModel {
            target_id: 42,
            predicted_freq: 75.0,
            accuracy: 0.8,
            update_cost: 0.1,
        };
        set.models[1] = OtherModel {
            target_id: 99,
            predicted_freq: 120.0,
            accuracy: 0.5,
            update_cost: 0.2,
        };
        set.model_count = 2;
        assert_eq!(set.model_count(), 2);
        assert_eq!(set.models_active().len(), 2);
        assert_eq!(set.models_active()[0].target_id, 42);
        assert_eq!(set.models_active()[1].target_id, 99);
    }

    #[test]
    fn max_models_is_four() {
        assert_eq!(MAX_MODELS, 4);
        let mut set = OtherModelSet::default();
        set.model_count = MAX_MODELS as u8;
        assert_eq!(set.models_active().len(), MAX_MODELS);
    }

    #[test]
    fn default_update_interval_and_cost() {
        let set = OtherModelSet::default();
        assert_eq!(set.update_interval, 5);
        assert!((set.base_model_cost - 0.2).abs() < 1e-5);
    }
}
