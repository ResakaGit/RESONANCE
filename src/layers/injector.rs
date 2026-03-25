use bevy::prelude::*;

use crate::blueprint::constants::{
    INJECTOR_DEFAULT_FORCED_FREQUENCY, INJECTOR_DEFAULT_INFLUENCE_RADIUS,
    INJECTOR_DEFAULT_PROJECTED_QE, INJECTOR_MIN_INFLUENCE_RADIUS,
};

/// Capa 8: Catálisis — El Gestor de Reacciones Emergentes
///
/// Arquetipo dinámico que representa el evento de alteración de la realidad.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct AlchemicalInjector {
    /// Cuánta energía intenta forzar en el objetivo.
    pub projected_qe: f32,

    /// Qué frecuencia/elemento intenta imponer.
    pub forced_frequency: f32,

    /// Radio del área de efecto del hechizo.
    pub influence_radius: f32,
}

impl Default for AlchemicalInjector {
    fn default() -> Self {
        Self {
            projected_qe: INJECTOR_DEFAULT_PROJECTED_QE,
            forced_frequency: INJECTOR_DEFAULT_FORCED_FREQUENCY,
            influence_radius: INJECTOR_DEFAULT_INFLUENCE_RADIUS,
        }
    }
}

impl AlchemicalInjector {
    pub fn new(qe: f32, frequency: f32, radius: f32) -> Self {
        Self {
            projected_qe: qe.max(0.0),
            forced_frequency: frequency.max(0.0),
            influence_radius: radius.max(INJECTOR_MIN_INFLUENCE_RADIUS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        INJECTOR_DEFAULT_FORCED_FREQUENCY, INJECTOR_DEFAULT_INFLUENCE_RADIUS,
        INJECTOR_DEFAULT_PROJECTED_QE, INJECTOR_MIN_INFLUENCE_RADIUS,
    };

    #[test]
    fn default_matches_ssot_constants() {
        let i = AlchemicalInjector::default();
        assert!((i.projected_qe - INJECTOR_DEFAULT_PROJECTED_QE).abs() < 1e-5);
        assert!((i.forced_frequency - INJECTOR_DEFAULT_FORCED_FREQUENCY).abs() < 1e-5);
        assert!((i.influence_radius - INJECTOR_DEFAULT_INFLUENCE_RADIUS).abs() < 1e-5);
    }

    #[test]
    fn new_clamps_influence_radius_to_minimum() {
        let i = AlchemicalInjector::new(10.0, 100.0, 0.001);
        assert!((i.influence_radius - INJECTOR_MIN_INFLUENCE_RADIUS).abs() < 1e-6);
    }

    #[test]
    fn new_clamps_negative_qe_and_frequency() {
        let i = AlchemicalInjector::new(-5.0, -10.0, 1.0);
        assert_eq!(i.projected_qe, 0.0);
        assert_eq!(i.forced_frequency, 0.0);
    }
}
