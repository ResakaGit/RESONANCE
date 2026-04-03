//! ET-9: Multidimensional Niche (Hutchinson) — NicheProfile component. Capa T2-5.

use bevy::prelude::*;

/// Capa T2-5: NicheProfile — hipervolumen de Hutchinson en 4D.
/// Dim 0: frecuencia preferida. Dim 1: x espacial. Dim 2: z espacial. Dim 3: fase temporal.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct NicheProfile {
    pub center: [f32; 4],
    pub width: [f32; 4],
    pub displacement_rate: f32,
    pub specialization: f32, // [0,1] — 0=generalista, 1=especialista
}

impl Default for NicheProfile {
    fn default() -> Self {
        Self {
            center: [0.0; 4],
            width: [1.0; 4],
            displacement_rate: 0.01,
            specialization: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_center_is_origin() {
        let n = NicheProfile::default();
        assert_eq!(n.center, [0.0; 4]);
    }

    #[test]
    fn default_width_is_unit() {
        let n = NicheProfile::default();
        assert_eq!(n.width, [1.0; 4]);
    }

    #[test]
    fn default_specialization_is_midpoint() {
        let n = NicheProfile::default();
        assert!((n.specialization - 0.5).abs() < 1e-5);
    }

    #[test]
    fn default_displacement_rate() {
        let n = NicheProfile::default();
        assert!((n.displacement_rate - 0.01).abs() < 1e-5);
    }
}
