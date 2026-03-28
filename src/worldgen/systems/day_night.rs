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
    angular_velocity_from_period, solar_irradiance_factor, sun_direction,
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

    // Compute the maximum solar contribution this tick (from propagation).
    // We subtract a fraction of energy on the dark side, not multiply the total.
    // Night doesn't destroy accumulated energy — it only reduces the solar input rate.
    let max_solar_drain_per_tick = 2.0; // small drain on dark side per tick (not exponential)

    for y in 0..grid.height {
        for x in 0..grid.width {
            let Some(world_pos) = grid.world_pos(x, y) else { continue };
            let solar_factor = solar_irradiance_factor(world_pos, center, sun_dir);

            if solar_factor >= 0.95 { continue; } // fully lit — no change needed

            if let Some(cell) = grid.cell_xy_mut(x, y) {
                if cell.accumulated_qe <= 0.0 { continue; }
                // Dark side: drain a small fixed amount (not multiplicative).
                // Represents cooling without solar input. Axiom 4: dissipation.
                let shadow_strength = 1.0 - solar_factor; // 0=lit, 1=dark
                let drain = max_solar_drain_per_tick * shadow_strength;
                let new_qe = (cell.accumulated_qe - drain).max(0.0);
                if (cell.accumulated_qe - new_qe).abs() > 0.01 {
                    cell.accumulated_qe = new_qe;
                    grid.mark_cell_dirty(x, y);
                }
            }
        }
    }
}
