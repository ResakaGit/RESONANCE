use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{DEFAULT_SPHERE_RADIUS, VOLUME_MIN_RADIUS};
use crate::blueprint::equations;

/// Capa 1: Volumen Espacial — El Espacio
/// Layer 1: Spatial Volume — Space
///
/// La energía requiere un contenedor espacial. El radio define volumen y colisión.
/// Energy needs a spatial container. Radius defines volume and collision.
///
/// Derivada / Derived: `densidad = qe / ((4/3) × π × r³)` (computed, not stored)
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct SpatialVolume {
    /// Radio en unidades de mundo. / Radius in world units.
    pub radius: f32,
}

impl Default for SpatialVolume {
    fn default() -> Self {
        Self {
            radius: DEFAULT_SPHERE_RADIUS,
        }
    }
}

impl SpatialVolume {
    pub fn new(radius: f32) -> Self {
        Self {
            radius: radius.max(VOLUME_MIN_RADIUS),
        }
    }

    /// Volumen de la esfera. / Sphere volume. Delegates to `equations::sphere_volume`.
    #[inline]
    pub fn volume(&self) -> f32 {
        equations::sphere_volume(self.radius)
    }

    /// Densidad: ρ = qe / V. / Density. Delegates to `equations::density`.
    #[inline]
    pub fn density(&self, qe: f32) -> f32 {
        equations::density(qe, self.radius)
    }

    /// Actualiza el radio respetando invariantes. / Updates radius, clamped to minimum.
    #[inline]
    pub fn set_radius(&mut self, radius: f32) {
        let r = radius.max(VOLUME_MIN_RADIUS);
        if self.radius != r {
            self.radius = r;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::DEFAULT_SPHERE_RADIUS;
    use crate::blueprint::equations;

    #[test]
    fn volume_and_density_match_equations() {
        let v = SpatialVolume::new(0.75);
        let qe = 42.0;
        assert_eq!(v.volume(), equations::sphere_volume(v.radius));
        assert_eq!(v.density(qe), equations::density(qe, v.radius));
    }

    #[test]
    fn default_radius_matches_ssot() {
        let v = SpatialVolume::default();
        assert!((v.radius - DEFAULT_SPHERE_RADIUS).abs() < 1e-5);
    }

    #[test]
    fn density_positive_for_typical_qe() {
        let v = SpatialVolume::new(0.5);
        let rho = v.density(100.0);
        assert!(rho.is_finite() && rho > 0.0);
    }

    #[test]
    fn density_zero_qe_is_zero() {
        let v = SpatialVolume::new(0.5);
        assert_eq!(v.density(0.0), 0.0);
    }

    #[test]
    fn new_clamps_radius_to_volume_minimum() {
        let v = SpatialVolume::new(-5.0);
        assert!((v.radius - VOLUME_MIN_RADIUS).abs() < 1e-6);
    }

    #[test]
    fn new_nan_clamps_to_minimum() {
        let v = SpatialVolume::new(f32::NAN);
        assert!(v.radius.is_finite());
        assert!((v.radius - VOLUME_MIN_RADIUS).abs() < 1e-6);
    }

    #[test]
    fn set_radius_clamps_negative() {
        let mut v = SpatialVolume::new(1.0);
        v.set_radius(-10.0);
        assert!((v.radius - VOLUME_MIN_RADIUS).abs() < 1e-6);
    }

    #[test]
    fn set_radius_updates_valid_value() {
        let mut v = SpatialVolume::new(1.0);
        v.set_radius(2.5);
        assert!((v.radius - 2.5).abs() < 1e-6);
    }
}
