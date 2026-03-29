//! Day/night cycle + seasonal modulation.
//!
//! The planet rotates on its axis (day/night: X sweep) and orbits its star
//! (seasons: Y latitude oscillation from axial tilt).
//!
//! Night cooling follows Newton's law: proportional drain (hot cells cool
//! faster, cold cells approach zero asymptotically — never empty).
//! All rates are tick-rate-independent via dt normalization.
//!
//! Stateless: reads tick_id + config, writes grid qe.
//! Phase: ThermodynamicLayer, after propagation.

use bevy::prelude::*;

use crate::blueprint::equations::planetary_rotation::{
    angular_velocity_from_period, night_cooling_fraction, seasonal_irradiance_modifier,
    solar_irradiance_factor, solar_meridian_x, AMBIENT_IRRADIANCE,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

/// Reference tick rate at which dissipation constants were calibrated.
const REFERENCE_HZ: f32 = 60.0;

/// Resource: day/night cycle + seasonal configuration.
#[derive(Resource, Debug, Clone)]
pub struct DayNightConfig {
    pub period_ticks: f32,
    pub grid_width_world: f32,
    pub omega: f32,
    pub year_period_ticks: f32,
    pub axial_tilt: f32,
}

impl DayNightConfig {
    pub fn new(period_ticks: f32, grid_width_world: f32) -> Self {
        Self {
            period_ticks,
            grid_width_world,
            omega: angular_velocity_from_period(period_ticks),
            year_period_ticks: 0.0,
            axial_tilt: 0.0,
        }
    }

    pub fn with_seasons(mut self, year_period_ticks: f32, axial_tilt: f32) -> Self {
        self.year_period_ticks = year_period_ticks;
        self.axial_tilt = axial_tilt;
        self
    }
}

/// Drains energy on the dark side of the rotating planet (Newton's law).
/// Tick-rate-independent: drain scaled by `dt × REFERENCE_HZ`.
pub fn day_night_modulation_system(
    config: Option<Res<DayNightConfig>>,
    clock: Res<SimulationClock>,
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut grid: Option<ResMut<EnergyFieldGrid>>,
) {
    let Some(config) = config else { return };
    let Some(ref mut grid) = grid else { return };
    if config.omega == 0.0 { return; }

    // dt normalization: at 60 Hz → dt_ratio = 1.0. At 250 Hz → 0.24.
    let dt = fixed.as_ref().map(|f| f.delta_secs()).unwrap_or_else(|| time.delta_secs());
    let dt_ratio = dt * REFERENCE_HZ;

    let grid_w = config.grid_width_world;
    let grid_h = grid.height as f32 * grid.cell_size;
    let meridian = solar_meridian_x(clock.tick_id, config.period_ticks, grid_w);
    let cooling = night_cooling_fraction();

    for y in 0..grid.height {
        let cell_y_world = y as f32 * grid.cell_size;
        let seasonal = seasonal_irradiance_modifier(
            cell_y_world, grid_h, clock.tick_id,
            config.year_period_ticks, config.axial_tilt,
        );

        for x in 0..grid.width {
            let cell_x_world = x as f32 * grid.cell_size;
            let solar_factor = solar_irradiance_factor(cell_x_world, meridian, grid_w) * seasonal;

            if solar_factor >= 1.0 - AMBIENT_IRRADIANCE { continue; }

            if let Some(cell) = grid.cell_xy_mut(x, y) {
                if cell.accumulated_qe <= 0.0 { continue; }
                let shadow = 1.0 - solar_factor;
                let drain = cell.accumulated_qe * cooling * shadow * dt_ratio;
                if drain > 0.01 {
                    cell.accumulated_qe -= drain;
                    grid.mark_cell_dirty(x, y);
                }
            }
        }
    }
}
