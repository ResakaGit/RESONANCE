//! Espectro reflejado — frecuencia que el órgano no puede absorber.
//! Reflected spectrum — frequency the organ cannot absorb.
//!
//! Computed from solar_freq - organ_absorption_freq.
//! Used by entity_shape_inference to color organ sub-meshes.

use bevy::prelude::*;

/// Per-entity reflected frequency for spectral pigmentation.
/// Frecuencia reflejada per-entidad para pigmentación espectral.
///
/// Inserted by spectral absorption system. Read by shape inference.
/// SparseSet: only entities with irradiance have reflected spectrum.
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct ReflectedSpectrum {
    /// Dominant reflected frequency in Hz.
    pub reflected_freq_hz: f32,
}

impl Default for ReflectedSpectrum {
    fn default() -> Self {
        Self {
            reflected_freq_hz: 0.0,
        }
    }
}
