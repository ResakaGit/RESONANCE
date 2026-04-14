//! Fototropismo — dirección de crecimiento sigue gradiente de irradiancia.
//! Phototropism — growth direction follows irradiance gradient.
//!
//! Auxin redistribution model: shadow side accumulates more energy (not spent
//! on photosynthesis) → grows faster → organ tilts toward light.
//! No WillActuator needed — this is differential growth, not movement.

use crate::blueprint::constants::PHOTOTROPISM_SENSITIVITY;

/// Compute irradiance gradient direction from entity position and light sources.
/// Calcular dirección del gradiente de irradiancia.
///
/// Returns normalized direction toward strongest light source, weighted by 1/d².
/// Returns zero vector if no sources provided.
pub fn irradiance_gradient_direction(
    entity_x: f32,
    entity_y: f32,
    sources: &[(f32, f32, f32)], // (x, y, intensity)
) -> (f32, f32) {
    let mut gx = 0.0_f32;
    let mut gy = 0.0_f32;
    for &(sx, sy, intensity) in sources {
        let dx = sx - entity_x;
        let dy = sy - entity_y;
        let d2 = dx * dx + dy * dy + 1.0; // +1 softening
        let weight = intensity / d2;
        gx += dx * weight;
        gy += dy * weight;
    }
    let mag = (gx * gx + gy * gy).sqrt();
    if mag < 1e-6 {
        return (0.0, 0.0);
    }
    (gx / mag, gy / mag)
}

/// Compute phototropic spine bias from gradient direction and strength.
/// Calcular bias del spine GF1 desde dirección y fuerza del gradiente.
///
/// Returns a (bx, by, bz) bias vector that tilts the GF1 spine toward light.
/// Strength is modulated by PHOTOTROPISM_SENSITIVITY.
pub fn phototropic_spine_bias(
    gradient_x: f32,
    gradient_y: f32,
    gradient_strength: f32,
) -> (f32, f32, f32) {
    let scale = gradient_strength.min(1.0) * PHOTOTROPISM_SENSITIVITY;
    (gradient_x * scale, gradient_y * scale, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_no_sources_returns_zero() {
        let (gx, gy) = irradiance_gradient_direction(0.0, 0.0, &[]);
        assert_eq!(gx, 0.0);
        assert_eq!(gy, 0.0);
    }

    #[test]
    fn gradient_single_source_points_toward() {
        let (gx, _gy) = irradiance_gradient_direction(0.0, 0.0, &[(10.0, 0.0, 100.0)]);
        assert!(gx > 0.0, "should point toward source");
    }

    #[test]
    fn gradient_normalized_magnitude() {
        let (gx, gy) = irradiance_gradient_direction(0.0, 0.0, &[(10.0, 0.0, 100.0)]);
        let mag = (gx * gx + gy * gy).sqrt();
        assert!((mag - 1.0).abs() < 1e-3, "should be normalized, got {mag}");
    }

    #[test]
    fn spine_bias_zero_gradient_zero_bias() {
        let (bx, by, bz) = phototropic_spine_bias(0.0, 0.0, 0.0);
        assert_eq!(bx, 0.0);
        assert_eq!(by, 0.0);
        assert_eq!(bz, 0.0);
    }

    #[test]
    fn spine_bias_scales_with_sensitivity() {
        let (bx, _, _) = phototropic_spine_bias(1.0, 0.0, 1.0);
        assert!((bx - PHOTOTROPISM_SENSITIVITY).abs() < 1e-5);
    }
}
