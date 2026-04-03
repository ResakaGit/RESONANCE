//! IWG-6 — Atmosphere inference equations: sun, fog, bloom, ambient from world state.

use std::f32::consts::FRAC_PI_2;

use crate::math_types::Vec3;

use crate::blueprint::constants::inferred_world_geometry::{
    AMBIENT_BASE_INTENSITY, AMBIENT_CANOPY_REDUCTION, BLOOM_MAX, BLOOM_QE_SCALE, FOG_END_RATIO,
    FOG_MAX_END, FOG_MIN_START, FOG_START_RATIO, SUN_BASE_INTENSITY, SUN_MIN_INTENSITY,
};

/// Sun direction from latitude and time angle.
///
/// `latitude` in [0, 1]: 0 = equator, 1 = pole.
/// `time_angle` in radians: azimuthal rotation of the sun.
pub fn inferred_sun_direction(latitude: f32, time_angle: f32) -> Vec3 {
    let elevation = (1.0 - latitude.clamp(0.0, 1.0)) * FRAC_PI_2;
    Vec3::new(
        time_angle.cos() * elevation.cos(),
        elevation.sin(),
        time_angle.sin() * elevation.cos(),
    )
    .normalize_or_zero()
}

/// Sun intensity from direction (dot with Y+). Below horizon returns `SUN_MIN_INTENSITY`.
pub fn inferred_sun_intensity(sun_direction: Vec3) -> f32 {
    let dot = sun_direction.dot(Vec3::Y).max(0.0);
    SUN_MIN_INTENSITY + (SUN_BASE_INTENSITY - SUN_MIN_INTENSITY) * dot
}

/// (fog_start, fog_end) from world radius, average density, and canopy factor.
pub fn inferred_fog_params(world_radius: f32, avg_density: f32, canopy_factor: f32) -> (f32, f32) {
    let base_start = world_radius * FOG_START_RATIO;
    let base_end = world_radius * FOG_END_RATIO;
    let density_mod = 1.0 - avg_density.clamp(0.0, 1.0) * 0.5;
    let canopy_mod = 1.0 - canopy_factor.clamp(0.0, 1.0) * 0.3;
    let fog_start = (base_start * density_mod * canopy_mod).clamp(FOG_MIN_START, FOG_MAX_END);
    let fog_end = (base_end * density_mod * canopy_mod).clamp(fog_start + 1.0, FOG_MAX_END);
    (fog_start, fog_end)
}

/// Fog color from sun direction and density.
pub fn inferred_fog_color(sun_direction: Vec3, avg_density: f32) -> [f32; 3] {
    let base = [0.7_f32, 0.75, 0.85];
    let warm = [0.9_f32, 0.8, 0.6];
    let sun_factor = sun_direction.dot(Vec3::Y).max(0.0);
    let t = sun_factor * 0.3;
    let darken = 1.0 - avg_density.clamp(0.0, 1.0) * 0.2;
    [
        ((base[0] + (warm[0] - base[0]) * t) * darken).clamp(0.0, 1.0),
        ((base[1] + (warm[1] - base[1]) * t) * darken).clamp(0.0, 1.0),
        ((base[2] + (warm[2] - base[2]) * t) * darken).clamp(0.0, 1.0),
    ]
}

/// Bloom intensity from average qe_norm.
pub fn inferred_bloom_intensity(avg_qe_norm: f32) -> f32 {
    (avg_qe_norm.clamp(0.0, 1.0) * BLOOM_QE_SCALE).min(BLOOM_MAX)
}

/// Ambient light intensity and color from canopy density and sun intensity.
pub fn inferred_ambient_light(canopy_density: f32, sun_intensity: f32) -> (f32, [f32; 3]) {
    let intensity = (AMBIENT_BASE_INTENSITY
        * (1.0 - canopy_density.clamp(0.0, 1.0) * AMBIENT_CANOPY_REDUCTION)
        * (sun_intensity / SUN_BASE_INTENSITY).max(0.1))
    .clamp(0.0, 1.0);
    let color = [0.6_f32, 0.65, 0.8];
    (intensity, color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn inferred_sun_direction_equator_overhead() {
        let dir = inferred_sun_direction(0.0, 0.0);
        // At equator (latitude=0), elevation = PI/2, so Y component should be ~1
        assert!(
            dir.y > 0.99,
            "equator sun should be nearly overhead, got y={}",
            dir.y
        );
        assert!(dir.length() > 0.99 && dir.length() < 1.01);
    }

    #[test]
    fn inferred_sun_direction_pole_low() {
        let dir = inferred_sun_direction(1.0, 0.0);
        // At pole (latitude=1), elevation = 0, so Y component should be ~0
        assert!(
            dir.y.abs() < 0.01,
            "pole sun should be at horizon, got y={}",
            dir.y
        );
        assert!(dir.length() > 0.99 && dir.length() < 1.01);
    }

    #[test]
    fn inferred_sun_intensity_above_horizon() {
        let dir = Vec3::new(0.0, 1.0, 0.0);
        let intensity = inferred_sun_intensity(dir);
        assert!(
            (intensity - SUN_BASE_INTENSITY).abs() < 1.0,
            "overhead sun should give max intensity, got {intensity}"
        );
    }

    #[test]
    fn inferred_sun_intensity_below_horizon_zero() {
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let intensity = inferred_sun_intensity(dir);
        assert!(
            (intensity - SUN_MIN_INTENSITY).abs() < 1.0,
            "below horizon should give min intensity, got {intensity}"
        );
    }

    #[test]
    fn inferred_fog_params_small_world() {
        let (start, end) = inferred_fog_params(20.0, 0.0, 0.0);
        assert!(start >= FOG_MIN_START);
        assert!(end > start);
        assert!(end <= FOG_MAX_END);
    }

    #[test]
    fn inferred_fog_params_large_world() {
        let (start, end) = inferred_fog_params(500.0, 0.5, 0.5);
        assert!(start >= FOG_MIN_START);
        assert!(end > start);
        assert!(end <= FOG_MAX_END);
    }

    #[test]
    fn inferred_fog_color_channels_clamped() {
        // Extreme inputs
        let color = inferred_fog_color(Vec3::new(0.0, 10.0, 0.0), 5.0);
        for ch in color {
            assert!(ch >= 0.0 && ch <= 1.0, "channel {ch} out of [0,1]");
        }
        let color2 = inferred_fog_color(Vec3::new(0.0, -10.0, 0.0), -5.0);
        for ch in color2 {
            assert!(ch >= 0.0 && ch <= 1.0, "channel {ch} out of [0,1]");
        }
    }

    #[test]
    fn inferred_bloom_intensity_zero_qe() {
        assert_eq!(inferred_bloom_intensity(0.0), 0.0);
    }

    #[test]
    fn inferred_bloom_intensity_full_qe_clamped() {
        let bloom = inferred_bloom_intensity(1.0);
        assert!(bloom <= BLOOM_MAX, "bloom {bloom} exceeds max {BLOOM_MAX}");
        assert!((bloom - BLOOM_QE_SCALE).abs() < 1e-5);
    }

    #[test]
    fn inferred_ambient_light_no_canopy() {
        let (intensity, color) = inferred_ambient_light(0.0, SUN_BASE_INTENSITY);
        assert!(
            (intensity - AMBIENT_BASE_INTENSITY).abs() < 1e-4,
            "no canopy + full sun should give base intensity, got {intensity}"
        );
        assert_eq!(color, [0.6, 0.65, 0.8]);
    }

    #[test]
    fn all_atmosphere_determinism() {
        let dir_a = inferred_sun_direction(0.3, PI * 0.7);
        let dir_b = inferred_sun_direction(0.3, PI * 0.7);
        assert_eq!(dir_a, dir_b);

        let int_a = inferred_sun_intensity(dir_a);
        let int_b = inferred_sun_intensity(dir_b);
        assert_eq!(int_a, int_b);

        let fog_a = inferred_fog_params(50.0, 0.4, 0.2);
        let fog_b = inferred_fog_params(50.0, 0.4, 0.2);
        assert_eq!(fog_a, fog_b);

        let fc_a = inferred_fog_color(dir_a, 0.3);
        let fc_b = inferred_fog_color(dir_b, 0.3);
        assert_eq!(fc_a, fc_b);

        let bloom_a = inferred_bloom_intensity(0.6);
        let bloom_b = inferred_bloom_intensity(0.6);
        assert_eq!(bloom_a, bloom_b);

        let amb_a = inferred_ambient_light(0.5, 10000.0);
        let amb_b = inferred_ambient_light(0.5, 10000.0);
        assert_eq!(amb_a, amb_b);
    }
}
