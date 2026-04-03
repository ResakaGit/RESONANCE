//! Rugosidad de superficie inferida por balance termodinámico (MG-7A).
//! Solo entidades con EntropyLedger lo reciben.

use bevy::prelude::*;

use crate::blueprint::constants::morphogenesis as mg;

/// Rugosidad de superficie inferida por balance Q/V.
/// Controla complejidad geométrica superficial en GF1.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct MorphogenesisSurface {
    /// Ratio de superficie real vs esfera equivalente [1.0, 4.0].
    rugosity: f32,
    /// Q/V ratio usado para el cálculo (diagnóstico).
    heat_volume_ratio: f32,
}

impl MorphogenesisSurface {
    /// Construye con clamp de rugosity a [RUGOSITY_MIN, RUGOSITY_MAX].
    pub fn new(rugosity: f32, heat_volume_ratio: f32) -> Self {
        Self {
            rugosity: rugosity.clamp(mg::RUGOSITY_MIN, mg::RUGOSITY_MAX),
            heat_volume_ratio: if heat_volume_ratio.is_finite() {
                heat_volume_ratio.max(0.0)
            } else {
                0.0
            },
        }
    }

    #[inline]
    pub fn rugosity(&self) -> f32 {
        self.rugosity
    }

    #[inline]
    pub fn heat_volume_ratio(&self) -> f32 {
        self.heat_volume_ratio
    }

    /// Setter con clamp — para uso desde sistemas con guard externo.
    pub fn set_rugosity(&mut self, val: f32) {
        self.rugosity = val.clamp(mg::RUGOSITY_MIN, mg::RUGOSITY_MAX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::morphogenesis as mg;
    use bevy::ecs::component::StorageType;

    #[test]
    fn new_standard_value_preserved() {
        let s = MorphogenesisSurface::new(2.5, 10.0);
        assert!((s.rugosity() - 2.5).abs() < 1e-6);
        assert!((s.heat_volume_ratio() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn new_below_min_clamps_to_rugosity_min() {
        let s = MorphogenesisSurface::new(0.5, 10.0);
        assert!((s.rugosity() - mg::RUGOSITY_MIN).abs() < 1e-6);
    }

    #[test]
    fn new_above_max_clamps_to_rugosity_max() {
        let s = MorphogenesisSurface::new(6.0, 10.0);
        assert!((s.rugosity() - mg::RUGOSITY_MAX).abs() < 1e-6);
    }

    #[test]
    fn is_copy() {
        let a = MorphogenesisSurface::new(2.0, 5.0);
        let b = a;
        assert_eq!(a.rugosity(), b.rugosity());
    }

    #[test]
    fn is_sparse_set() {
        assert_eq!(MorphogenesisSurface::STORAGE_TYPE, StorageType::SparseSet);
    }

    #[test]
    fn set_rugosity_clamps() {
        let mut s = MorphogenesisSurface::new(2.0, 5.0);
        s.set_rugosity(5.0);
        assert!((s.rugosity() - mg::RUGOSITY_MAX).abs() < 1e-6);
        s.set_rugosity(-1.0);
        assert!((s.rugosity() - mg::RUGOSITY_MIN).abs() < 1e-6);
    }

    #[test]
    fn nan_heat_volume_ratio_sanitized() {
        let s = MorphogenesisSurface::new(2.0, f32::NAN);
        assert_eq!(s.heat_volume_ratio(), 0.0);
    }

    #[test]
    fn negative_heat_volume_ratio_clamped() {
        let s = MorphogenesisSurface::new(2.0, -5.0);
        assert_eq!(s.heat_volume_ratio(), 0.0);
    }
}
