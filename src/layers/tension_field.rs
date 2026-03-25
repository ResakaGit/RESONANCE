use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Modo de decaimiento para fuerzas del campo de tensión.
#[derive(Clone, Copy, Debug, Reflect, Serialize, Deserialize, PartialEq, Eq)]
pub enum FieldFalloffMode {
    InverseSquare,
    InverseLinear,
}

/// Capa 11: Campo de tensión a distancia.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct TensionField {
    pub radius: f32,
    pub gravity_gain: f32,
    pub magnetic_gain: f32,
    pub falloff_mode: FieldFalloffMode,
}

impl TensionField {
    pub fn new(
        radius: f32,
        gravity_gain: f32,
        magnetic_gain: f32,
        falloff_mode: FieldFalloffMode,
    ) -> Self {
        Self {
            radius: radius.max(0.0),
            gravity_gain,
            magnetic_gain,
            falloff_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_negative_radius() {
        let f = TensionField::new(-10.0, 1.0, 1.0, FieldFalloffMode::InverseLinear);
        assert_eq!(f.radius, 0.0);
    }

    #[test]
    fn falloff_modes_are_distinct() {
        assert_ne!(
            FieldFalloffMode::InverseSquare,
            FieldFalloffMode::InverseLinear
        );
    }

    #[test]
    fn tension_field_stores_gains_and_mode() {
        let f = TensionField::new(5.0, 2.0, -1.0, FieldFalloffMode::InverseSquare);
        assert!((f.radius - 5.0).abs() < 1e-5);
        assert!((f.gravity_gain - 2.0).abs() < 1e-5);
        assert!((f.magnetic_gain - (-1.0)).abs() < 1e-5);
        assert_eq!(f.falloff_mode, FieldFalloffMode::InverseSquare);
    }
}
