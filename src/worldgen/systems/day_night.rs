//! Day/night cycle: angular modulation of solar nucleus emission.
//!
//! The Sun doesn't move. The surface rotates under it (frame of reference).
//! Each cell's accumulated solar energy is scaled by cos(angle_to_sun_direction).
//! Dark side gets near-zero. Lit side gets full emission.
//!
//! Stateless: reads tick_id + grid + nucleus positions, writes grid qe.
//! Phase: ThermodynamicLayer, AFTER propagation (modulates what propagation deposited).

use bevy::prelude::*;

use crate::blueprint::equations::planetary_rotation::{
    angular_velocity_from_period, effective_irradiance, solar_irradiance_factor, sun_direction,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

/// Resource: day/night cycle configuration. Inserted from MapConfig.
#[derive(Resource, Debug, Clone)]
pub struct DayNightConfig {
    /// Ticks per full rotation (day period).
    pub period_ticks: f32,
    /// Grid center for angular calculation.
    pub grid_center: crate::math_types::Vec2,
    /// Precomputed angular velocity.
    pub omega: f32,
}

impl DayNightConfig {
    pub fn new(period_ticks: f32, grid_center: crate::math_types::Vec2) -> Self {
        Self {
            period_ticks,
            grid_center,
            omega: angular_velocity_from_period(period_ticks),
        }
    }
}

/// Modulates field energy by solar angle. Cells on the dark side lose energy.
/// Stateless: reads clock + config + grid, writes grid cells.
pub fn day_night_modulation_system(
    config: Option<Res<DayNightConfig>>,
    clock: Res<SimulationClock>,
    mut grid: Option<ResMut<EnergyFieldGrid>>,
) {
    let Some(config) = config else { return };
    let Some(ref mut grid) = grid else { return };
    if config.omega == 0.0 { return; }

    let sun_dir = sun_direction(clock.tick_id, config.omega);
    let center = config.grid_center;

    for y in 0..grid.height {
        for x in 0..grid.width {
            let Some(world_pos) = grid.world_pos(x, y) else { continue };
            let solar_factor = solar_irradiance_factor(world_pos, center, sun_dir);
            let effective = effective_irradiance(solar_factor);

            if let Some(cell) = grid.cell_xy_mut(x, y) {
                // Scale accumulated energy by irradiance factor.
                // Day side keeps energy. Night side loses most (ambient floor).
                let before = cell.accumulated_qe;
                let modulated = before * effective;
                if (before - modulated).abs() > 0.01 {
                    cell.accumulated_qe = modulated;
                    grid.mark_cell_dirty(x, y);
                }
            }
        }
    }
}
