//! ET-16: Functional Consciousness — SelfModel + FunctionallyConscious. Capa T4-3.

use crate::blueprint::equations::emergence::self_model as self_model_eq;
use bevy::prelude::*;

/// Capa T4-3: SelfModel — automodelo para planificación a largo plazo.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SelfModel {
    pub predicted_qe: f32,
    pub planning_horizon: u32,
    pub self_accuracy: f32, // [0,1] precisión del automodelo
    pub metacog_cost: f32,  // qe/tick para mantener el automodelo
}

impl Default for SelfModel {
    fn default() -> Self {
        Self {
            predicted_qe: 0.0,
            planning_horizon: 1,
            self_accuracy: 0.0,
            metacog_cost: 0.05,
        }
    }
}

impl SelfModel {
    /// True si la entidad tiene conciencia funcional.
    pub fn is_functionally_conscious(&self) -> bool {
        self_model_eq::consciousness_threshold(self.self_accuracy, self.planning_horizon)
    }
}

/// Marker: entidad con conciencia funcional activa (SparseSet — subset pequeño).
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct FunctionallyConscious;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_model_not_conscious_by_default() {
        let m = SelfModel::default();
        assert!(!m.is_functionally_conscious());
    }

    #[test]
    fn self_model_conscious_when_thresholds_met() {
        let m = SelfModel {
            predicted_qe: 0.0,
            planning_horizon: 200,
            self_accuracy: 0.9,
            metacog_cost: 0.05,
        };
        assert!(m.is_functionally_conscious());
    }
}
