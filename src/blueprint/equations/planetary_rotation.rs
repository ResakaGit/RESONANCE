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
/// Parallel light model: the terminator is a straight line perpendicular
/// to sun_direction passing through grid_center. Cells on the sun side
/// get irradiance proportional to their distance past the terminator.
/// Cells on the dark side get zero.
///
/// This models a distant star (parallel rays), not a point source.
/// No energy created — geometric mask only.
/// `grid_half_extent`: half the grid width in world units (for normalization).
#[inline]
pub fn solar_irradiance_factor(
    cell_pos: Vec2,
    grid_center: Vec2,
    sun_dir: Vec2,
    grid_half_extent: f32,
) -> f32 {
    let to_cell = cell_pos - grid_center;
    let projection = to_cell.dot(sun_dir);
    let half = grid_half_extent.max(1.0);
    // Smooth transition: fully lit at projection > half/2, fully dark at < 0.
    (projection / half * 2.0).clamp(0.0, 1.0)
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
    fn irradiance_far_sun_side_is_one() {
        let sun = Vec2::new(1.0, 0.0);
        let factor = solar_irradiance_factor(Vec2::new(40.0, 0.0), Vec2::ZERO, sun, 30.0);
        assert!((factor - 1.0).abs() < 1e-3);
    }

    #[test]
    fn irradiance_dark_side_is_zero() {
        let sun = Vec2::new(1.0, 0.0);
        let factor = solar_irradiance_factor(Vec2::new(-40.0, 0.0), Vec2::ZERO, sun, 30.0);
        assert_eq!(factor, 0.0);
    }

    #[test]
    fn irradiance_terminator_is_mid() {
        let sun = Vec2::new(1.0, 0.0);
        // At center: projection = 0, factor should be near 0 (just past terminator).
        let factor = solar_irradiance_factor(Vec2::ZERO, Vec2::ZERO, sun, 30.0);
        assert!(factor < 0.1, "center ≈ terminator: {factor}");
    }

    #[test]
    fn irradiance_perpendicular_at_center() {
        let sun = Vec2::new(1.0, 0.0);
        // Cell directly above center: perpendicular to sun direction.
        let factor = solar_irradiance_factor(Vec2::new(0.0, 10.0), Vec2::ZERO, sun, 30.0);
        assert!(factor < 0.1, "perpendicular near terminator: {factor}");
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
