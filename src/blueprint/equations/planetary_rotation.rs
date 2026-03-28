/// Planetary rotation: day/night cycle from angular modulation of solar irradiance.
///
/// The Sun doesn't move. The surface rotates under it.
/// Each cell receives solar energy proportional to cos(angle_to_sun).
///
/// Axiom 5: no energy created — only redistributed by geometry.
/// Axiom 7: attenuation with distance (already in propagation).
/// This adds angular attenuation: cells facing the sun get full energy,
/// cells on the dark side get zero.

use crate::math_types::Vec2;

/// Sun direction at a given tick (unit vector rotating at ω rad/tick).
///
/// `direction(tick) = (cos(ω × tick), sin(ω × tick))`
/// Pure. Deterministic. No state.
#[inline]
pub fn sun_direction(tick: u64, angular_velocity: f32) -> Vec2 {
    let angle = angular_velocity * tick as f32;
    Vec2::new(angle.cos(), angle.sin())
}

/// Angular velocity from day period in ticks.
///
/// `ω = 2π / period_ticks`
/// Derived from initial conditions, not hardcoded.
#[inline]
pub fn angular_velocity_from_period(period_ticks: f32) -> f32 {
    if period_ticks <= 0.0 { return 0.0; }
    std::f32::consts::TAU / period_ticks
}

/// Solar irradiance factor at a cell position.
///
/// `factor = max(0, cos(angle_between(cell_to_center, sun_direction)))`
/// Cells facing the sun: factor ≈ 1.0 (full irradiance).
/// Cells perpendicular: factor = 0.0 (terminator line).
/// Cells on dark side: factor = 0.0 (night).
///
/// No energy created — this is a geometric mask on existing emission.
#[inline]
pub fn solar_irradiance_factor(
    cell_pos: Vec2,
    grid_center: Vec2,
    sun_dir: Vec2,
) -> f32 {
    let to_cell = cell_pos - grid_center;
    let len = to_cell.length();
    if len < 1e-6 { return 1.0; } // center gets full irradiance
    let normalized = to_cell / len;
    let cos_angle = normalized.dot(sun_dir);
    cos_angle.max(0.0)
}

/// Minimum irradiance factor (ambient: scattered light, twilight).
/// Even the dark side gets some light from atmospheric scattering.
/// Derived: DISSIPATION_SOLID (the minimum energy floor).
pub const AMBIENT_IRRADIANCE: f32 = 0.05;

/// Apply rotation factor with ambient floor.
///
/// `effective = ambient + (1 - ambient) × solar_factor`
/// Even at night, ambient > 0 prevents total darkness.
#[inline]
pub fn effective_irradiance(solar_factor: f32) -> f32 {
    AMBIENT_IRRADIANCE + (1.0 - AMBIENT_IRRADIANCE) * solar_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_direction_at_zero_is_east() {
        let dir = sun_direction(0, 1.0);
        assert!((dir.x - 1.0).abs() < 1e-5);
        assert!(dir.y.abs() < 1e-5);
    }

    #[test]
    fn sun_direction_rotates() {
        let d0 = sun_direction(0, 0.1);
        let d100 = sun_direction(100, 0.1);
        assert!((d0 - d100).length() > 0.1, "should rotate over time");
    }

    #[test]
    fn angular_velocity_from_period_correct() {
        let omega = angular_velocity_from_period(1000.0);
        assert!((omega - std::f32::consts::TAU / 1000.0).abs() < 1e-6);
    }

    #[test]
    fn angular_velocity_zero_period_safe() {
        assert_eq!(angular_velocity_from_period(0.0), 0.0);
    }

    #[test]
    fn irradiance_facing_sun_is_one() {
        let sun = Vec2::new(1.0, 0.0);
        let factor = solar_irradiance_factor(Vec2::new(10.0, 0.0), Vec2::ZERO, sun);
        assert!((factor - 1.0).abs() < 1e-5);
    }

    #[test]
    fn irradiance_opposite_sun_is_zero() {
        let sun = Vec2::new(1.0, 0.0);
        let factor = solar_irradiance_factor(Vec2::new(-10.0, 0.0), Vec2::ZERO, sun);
        assert_eq!(factor, 0.0);
    }

    #[test]
    fn irradiance_perpendicular_is_zero() {
        let sun = Vec2::new(1.0, 0.0);
        let factor = solar_irradiance_factor(Vec2::new(0.0, 10.0), Vec2::ZERO, sun);
        assert!(factor.abs() < 1e-5);
    }

    #[test]
    fn irradiance_center_is_full() {
        let factor = solar_irradiance_factor(Vec2::ZERO, Vec2::ZERO, Vec2::X);
        assert!((factor - 1.0).abs() < 1e-5);
    }

    #[test]
    fn effective_irradiance_night_has_ambient() {
        let eff = effective_irradiance(0.0);
        assert!((eff - AMBIENT_IRRADIANCE).abs() < 1e-5);
    }

    #[test]
    fn effective_irradiance_day_near_one() {
        let eff = effective_irradiance(1.0);
        assert!((eff - 1.0).abs() < 1e-5);
    }
}
