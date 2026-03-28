//! Day/night cycle: solar meridian sweeps across the grid X axis.
//!
//! The planet rotates on its axis. The illuminated band advances from
//! left to right, wrapping cyclically. Cosine falloff from meridian.
//!
//! Night cooling follows Newton's law: proportional drain (hot cells cool
//! faster, cold cells approach zero asymptotically — never empty).
//!
//! Stateless: reads tick_id + config, writes grid qe.
//! Phase: ThermodynamicLayer, after propagation.

use bevy::prelude::*;

use crate::blueprint::equations::planetary_rotation::{
    angular_velocity_from_period, night_cooling_fraction, solar_irradiance_factor, solar_meridian_x,
    AMBIENT_IRRADIANCE,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

/// Resource: day/night cycle configuration.
#[derive(Resource, Debug, Clone)]
pub struct DayNightConfig {
    pub period_ticks: f32,
    pub grid_width_world: f32,
    pub omega: f32,
}

impl DayNightConfig {
    pub fn new(period_ticks: f32, grid_width_world: f32) -> Self {
        Self {
            period_ticks,
            grid_width_world,
            omega: angular_velocity_from_period(period_ticks),
        }
    }
}

/// Drains energy on the dark side of the rotating planet (Newton's law).
/// Day side: no change (propagation handles solar emission).
/// Night side: proportional drain `cell_qe × cooling_fraction × shadow`.
pub fn day_night_modulation_system(
    config: Option<Res<DayNightConfig>>,
    clock: Res<SimulationClock>,
    mut grid: Option<ResMut<EnergyFieldGrid>>,
) {
    let Some(config) = config else { return };
    let Some(ref mut grid) = grid else { return };
    if config.omega == 0.0 { return; }

    let grid_w = config.grid_width_world;
    let meridian = solar_meridian_x(clock.tick_id, config.period_ticks, grid_w);
    let cooling = night_cooling_fraction();

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell_x_world = x as f32 * grid.cell_size;
            let solar_factor = solar_irradiance_factor(cell_x_world, meridian, grid_w);

            // Fully lit: solar factor above ambient threshold → no cooling.
            if solar_factor >= 1.0 - AMBIENT_IRRADIANCE { continue; }

            if let Some(cell) = grid.cell_xy_mut(x, y) {
                if cell.accumulated_qe <= 0.0 { continue; }
                let shadow = 1.0 - solar_factor;
                // Newton's law: drain ∝ current energy × shadow depth.
                let drain = cell.accumulated_qe * cooling * shadow;
                let new_qe = cell.accumulated_qe - drain;
                if drain > 0.01 {
                    cell.accumulated_qe = new_qe;
                    grid.mark_cell_dirty(x, y);
                }
            }
        }
    }
}
