//! Day/night cycle + seasonal modulation + directional solar input.
//!
//! Solar energy is DIRECTIONAL — only the illuminated hemisphere receives it.
//! The dark side cools via Newton's law (proportional drain).
//! This models planetary occlusion without 3D ray casting.
//!
//! Phase: ThermodynamicLayer, after propagation.

use bevy::prelude::*;

use crate::blueprint::equations::planetary_rotation::{
    AMBIENT_IRRADIANCE, angular_velocity_from_period, night_cooling_fraction,
    seasonal_irradiance_modifier, solar_irradiance_factor, solar_meridian_x,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

/// Reference tick rate for dt normalization.
const REFERENCE_HZ: f32 = 60.0;

/// Configuración del ciclo día/noche + estaciones + sol direccional.
/// Day/night cycle + seasons + directional solar input config.
#[derive(Resource, Debug, Clone)]
pub struct DayNightConfig {
    pub period_ticks: f32,
    pub grid_width_world: f32,
    pub omega: f32,
    pub year_period_ticks: f32,
    pub axial_tilt: f32,
    /// Directional solar emission (qe/s). Only day-side cells receive this.
    /// Replaces omnidirectional sol nucleus for correct day/night contrast.
    pub solar_emission_qe_s: f32,
}

impl DayNightConfig {
    pub fn new(period_ticks: f32, grid_width_world: f32) -> Self {
        Self {
            period_ticks,
            grid_width_world,
            omega: angular_velocity_from_period(period_ticks),
            year_period_ticks: 0.0,
            axial_tilt: 0.0,
            solar_emission_qe_s: 0.0,
        }
    }

    pub fn with_seasons(mut self, year_period_ticks: f32, axial_tilt: f32) -> Self {
        self.year_period_ticks = year_period_ticks;
        self.axial_tilt = axial_tilt;
        self
    }

    pub fn with_solar_emission(mut self, qe_s: f32) -> Self {
        self.solar_emission_qe_s = qe_s.max(0.0);
        self
    }
}

/// Directional solar input + night cooling.
/// Day side: inject energy proportional to solar_irradiance_factor.
/// Night side: drain proportional to shadow depth (Newton's law).
/// Tick-rate-independent via dt normalization.
pub fn day_night_modulation_system(
    config: Option<Res<DayNightConfig>>,
    clock: Res<SimulationClock>,
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut grid: Option<ResMut<EnergyFieldGrid>>,
) {
    let Some(config) = config else { return };
    let Some(ref mut grid) = grid else { return };
    if config.omega == 0.0 {
        return;
    }

    let dt = fixed
        .as_ref()
        .map(|f| f.delta_secs())
        .unwrap_or_else(|| time.delta_secs());
    let dt_ratio = dt * REFERENCE_HZ;

    let grid_w = config.grid_width_world;
    let grid_h = grid.height as f32 * grid.cell_size;
    let meridian = solar_meridian_x(clock.tick_id, config.period_ticks, grid_w);
    let cooling = night_cooling_fraction();

    // Solar injection per cell per tick (distributed across illuminated cells).
    let total_cells = (grid.width * grid.height) as f32;
    let solar_per_cell = if config.solar_emission_qe_s > 0.0 && total_cells > 0.0 {
        config.solar_emission_qe_s * dt / (total_cells * 0.5) // half the planet is lit
    } else {
        0.0
    };

    for y in 0..grid.height {
        let cell_y_world = y as f32 * grid.cell_size;
        let seasonal = seasonal_irradiance_modifier(
            cell_y_world,
            grid_h,
            clock.tick_id,
            config.year_period_ticks,
            config.axial_tilt,
        );

        for x in 0..grid.width {
            let cell_x_world = x as f32 * grid.cell_size;
            let solar_factor = solar_irradiance_factor(cell_x_world, meridian, grid_w) * seasonal;

            if let Some(cell) = grid.cell_xy_mut(x, y) {
                if solar_factor > AMBIENT_IRRADIANCE {
                    // Day side: inject directional solar energy.
                    let injection = solar_per_cell * solar_factor;
                    if injection > 0.001 {
                        cell.accumulated_qe += injection;
                        grid.mark_cell_dirty(x, y);
                    }
                } else {
                    // Night side: proportional cooling (Newton's law).
                    if cell.accumulated_qe <= 0.0 {
                        continue;
                    }
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
}
