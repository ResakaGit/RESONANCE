//! Absorción espectral agnóstica — color emerge de frecuencia × densidad.
//! Agnostic spectral absorption — color emerges from frequency × density.
//!
//! No lookup tables. No role-based color mapping.
//! Color = frequency the organ cannot absorb = solar_freq - organ_absorption_freq.

/// Organ oscillation frequency derived from entity frequency and density ratio.
/// Frecuencia de oscilación del órgano derivada de frecuencia y ratio de densidad.
///
/// Dense organs oscillate closer to entity frequency (absorb broadly → dark).
/// Low-density organs oscillate at shifted frequency (absorb narrowly → vivid).
#[inline]
pub fn organ_frequency(entity_freq: f32, organ_density: f32, entity_density: f32) -> f32 {
    if entity_density <= 0.0 || entity_freq <= 0.0 {
        return entity_freq.max(0.0);
    }
    let ratio = (organ_density / entity_density).clamp(0.1, 10.0);
    entity_freq * ratio
}

/// Reflected frequency = complement of absorption.
/// Frecuencia reflejada = complemento de la absorción.
///
/// `reflected = solar - absorption`. Clamped to [0, solar].
/// An organ that absorbs at 300 Hz under 1000 Hz sun reflects at 700 Hz.
#[inline]
pub fn reflected_frequency(solar_freq: f32, absorption_freq: f32) -> f32 {
    (solar_freq - absorption_freq).clamp(0.0, solar_freq.max(0.0))
}

/// Convert reflected frequency + albedo to RGB tint.
/// Convertir frecuencia reflejada + albedo a tint RGB.
///
/// Wraps existing frequency_to_tint_rgb but modulates brightness by albedo.
/// High albedo = brighter reflection. Low albedo = dimmer.
#[inline]
pub fn spectral_tint_rgb(reflected_freq: f32, albedo: f32) -> [f32; 3] {
    let base = super::frequency_to_tint_rgb(reflected_freq);
    let brightness = 0.3 + 0.7 * albedo.clamp(0.0, 1.0);
    [base[0] * brightness, base[1] * brightness, base[2] * brightness]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organ_frequency_equal_density_equals_entity() {
        let f = organ_frequency(400.0, 5.0, 5.0);
        assert!((f - 400.0).abs() < 1e-3);
    }

    #[test]
    fn organ_frequency_low_density_shifts_down() {
        let f = organ_frequency(400.0, 2.5, 5.0);
        assert!(f < 400.0);
        assert!((f - 200.0).abs() < 1e-3);
    }

    #[test]
    fn organ_frequency_high_density_shifts_up() {
        let f = organ_frequency(400.0, 10.0, 5.0);
        assert!(f > 400.0);
    }

    #[test]
    fn organ_frequency_zero_entity_density_returns_entity_freq() {
        assert!((organ_frequency(400.0, 5.0, 0.0) - 400.0).abs() < 1e-3);
    }

    #[test]
    fn reflected_frequency_complement() {
        let r = reflected_frequency(1000.0, 300.0);
        assert!((r - 700.0).abs() < 1e-3);
    }

    #[test]
    fn reflected_frequency_clamped_to_zero() {
        let r = reflected_frequency(100.0, 500.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn spectral_tint_high_albedo_brighter() {
        let bright = spectral_tint_rgb(500.0, 0.9);
        let dim = spectral_tint_rgb(500.0, 0.1);
        assert!(bright[0] >= dim[0]);
        assert!(bright[1] >= dim[1]);
        assert!(bright[2] >= dim[2]);
    }
}
