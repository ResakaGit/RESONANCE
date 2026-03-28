/// Planetary rotation: day/night as a solar illumination band sweeping across the grid.
///
/// The planet rotates on its axis. From the surface's frame of reference,
/// the Sun appears to move from east to west. In a 2D grid, this means
/// the illuminated column advances one position per tick-fraction.
///
/// Implementation: a cosine-shaped illumination profile centered on the
/// "solar meridian" which sweeps across the X axis of the grid.
///
/// Axiom 5: no energy created — modulation of existing propagation.
/// Axiom 2: angular momentum conservation justifies rotation.


/// Angular velocity from day period in ticks.
/// `ω = 2π / period_ticks`
#[inline]
pub fn angular_velocity_from_period(period_ticks: f32) -> f32 {
    if period_ticks <= 0.0 { return 0.0; }
    std::f32::consts::TAU / period_ticks
}

/// Solar meridian X position at a given tick.
///
/// The illuminated band sweeps across the grid width cyclically.
/// Returns a value in [0, grid_width) representing the X coordinate
/// currently facing the Sun.
///
/// `meridian_x = (tick × grid_width / period) mod grid_width`
#[inline]
pub fn solar_meridian_x(tick: u64, period_ticks: f32, grid_width: f32) -> f32 {
    if period_ticks <= 0.0 { return 0.0; }
    let progress = (tick as f32 / period_ticks).fract(); // [0, 1)
    progress * grid_width
}

/// Solar irradiance factor for a cell based on its X distance from the solar meridian.
///
/// Cosine falloff: cells at the meridian get full light (1.0).
/// Cells half a grid away (opposite side) get zero (night).
/// Smooth transition simulates dawn/dusk.
///
/// `factor = max(0, cos(π × distance_from_meridian / half_width))`
///
/// This models a cylindrical planet rotating under parallel light:
/// the illuminated hemisphere is always facing the sun, the dark hemisphere
/// is always away. The meridian sweeps from left to right.
#[inline]
pub fn solar_irradiance_factor(
    cell_x: f32,
    meridian_x: f32,
    grid_width: f32,
) -> f32 {
    if grid_width <= 0.0 { return 1.0; }
    let half = grid_width * 0.5;

    // Shortest distance on a wrapping grid (cylindrical topology).
    let raw_dist = (cell_x - meridian_x).abs();
    let wrapped_dist = raw_dist.min(grid_width - raw_dist);

    // Cosine falloff: 0 at meridian → π at antipode.
    let angle = std::f32::consts::PI * wrapped_dist / half;
    angle.cos().max(0.0)
}

/// Minimum irradiance (ambient light from atmospheric scattering).
/// `AMBIENT_IRRADIANCE = DISSIPATION_SOLID / DISSIPATION_GAS` — ratio of solid to gas loss.
/// Solid retains heat; gas radiates. Their ratio gives the floor of residual light.
pub const AMBIENT_IRRADIANCE: f32 = {
    use super::derived_thresholds::{DISSIPATION_SOLID, DISSIPATION_GAS};
    DISSIPATION_SOLID / DISSIPATION_GAS
};

/// Apply ambient floor to solar factor.
#[inline]
pub fn effective_irradiance(solar_factor: f32) -> f32 {
    AMBIENT_IRRADIANCE + (1.0 - AMBIENT_IRRADIANCE) * solar_factor
}

/// Seasonal irradiance modifier based on axial tilt.
///
/// The sub-solar latitude oscillates over the year: `center ± tilt × half_height × sin(year_angle)`.
/// Returns a multiplier [0.5, 1.0] — cells far from sub-solar latitude get less light.
/// Latitude is linear (not wrapped) — north/south poles have opposite seasons.
/// `tilt = 0` → no seasons (1.0 everywhere).
#[inline]
pub fn seasonal_irradiance_modifier(
    cell_y: f32,
    grid_height: f32,
    tick: u64,
    year_period_ticks: f32,
    axial_tilt: f32,
) -> f32 {
    if year_period_ticks <= 0.0 || axial_tilt.abs() < 1e-6 { return 1.0; }
    let half_h = grid_height * 0.5;
    let year_progress = (tick as f32 / year_period_ticks).fract();
    // Sub-solar latitude oscillates with the year.
    let sub_solar_y = half_h + axial_tilt * half_h * (std::f32::consts::TAU * year_progress).sin();
    // Linear distance (no wrap — latitude is not cyclic on a sphere).
    let dy = (cell_y - sub_solar_y).abs();
    // Cosine falloff: near sub-solar → 1.0, far → 0.5.
    let angle = (std::f32::consts::PI * 0.5 * dy / half_h).min(std::f32::consts::FRAC_PI_2);
    0.5 + 0.5 * angle.cos()
}

/// Fractional radiative cooling per tick on the dark side (Newton's law).
/// `cooling_fraction = DISSIPATION_SOLID` — solid ground dissipation rate.
/// Applied proportionally: `drain = cell_qe × fraction × shadow`.
/// Hot cells cool faster, cold cells asymptotically approach zero — never empty.
#[inline]
pub fn night_cooling_fraction() -> f32 {
    use super::derived_thresholds::DISSIPATION_SOLID;
    DISSIPATION_SOLID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angular_velocity_correct() {
        let omega = angular_velocity_from_period(1000.0);
        assert!((omega - std::f32::consts::TAU / 1000.0).abs() < 1e-6);
    }

    #[test]
    fn angular_velocity_zero_period_safe() {
        assert_eq!(angular_velocity_from_period(0.0), 0.0);
    }

    #[test]
    fn meridian_sweeps_across_grid() {
        let m0 = solar_meridian_x(0, 100.0, 48.0);
        let m50 = solar_meridian_x(50, 100.0, 48.0);
        let m100 = solar_meridian_x(100, 100.0, 48.0);
        assert!(m0 < 1.0, "starts at left: {m0}");
        assert!(m50 > 20.0, "midway: {m50}");
        assert!(m100 < 1.0, "wraps back: {m100}");
    }

    #[test]
    fn irradiance_at_meridian_is_one() {
        let factor = solar_irradiance_factor(24.0, 24.0, 48.0);
        assert!((factor - 1.0).abs() < 1e-5);
    }

    #[test]
    fn irradiance_at_antipode_is_zero() {
        // Antipode: half a grid away.
        let factor = solar_irradiance_factor(0.0, 24.0, 48.0);
        assert!(factor < 0.01, "antipode should be dark: {factor}");
    }

    #[test]
    fn irradiance_wraps_cyclically() {
        // Cell at x=47, meridian at x=1 → distance = 2 (wrapping), not 46.
        let factor = solar_irradiance_factor(47.0, 1.0, 48.0);
        assert!(factor > 0.9, "should wrap: {factor}");
    }

    #[test]
    fn irradiance_smooth_transition() {
        let near = solar_irradiance_factor(20.0, 24.0, 48.0);
        let far = solar_irradiance_factor(10.0, 24.0, 48.0);
        assert!(near > far, "closer to meridian = more light: {near} > {far}");
    }

    #[test]
    fn effective_night_has_ambient() {
        let eff = effective_irradiance(0.0);
        assert!((eff - AMBIENT_IRRADIANCE).abs() < 1e-5);
    }

    #[test]
    fn effective_day_near_one() {
        let eff = effective_irradiance(1.0);
        assert!((eff - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ambient_irradiance_derived_from_dissipation() {
        use crate::blueprint::equations::derived_thresholds::{DISSIPATION_SOLID, DISSIPATION_GAS};
        assert!((AMBIENT_IRRADIANCE - DISSIPATION_SOLID / DISSIPATION_GAS).abs() < 1e-6);
        assert!(AMBIENT_IRRADIANCE > 0.0 && AMBIENT_IRRADIANCE < 0.2);
    }

    #[test]
    fn night_cooling_fraction_equals_dissipation_solid() {
        use crate::blueprint::equations::derived_thresholds::DISSIPATION_SOLID;
        let frac = night_cooling_fraction();
        assert!((frac - DISSIPATION_SOLID).abs() < 1e-8);
    }

    #[test]
    fn night_cooling_preserves_energy_over_full_night() {
        // 300 ticks of full shadow: cell should retain >20% (Newton's cooling).
        let frac = night_cooling_fraction();
        let remaining = (1.0 - frac).powi(300);
        assert!(remaining > 0.15, "cell must survive one night: {remaining:.3}");
    }
}
