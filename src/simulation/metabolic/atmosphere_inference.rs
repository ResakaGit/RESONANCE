//! IWG-6: Atmosphere inference — sun/fog/bloom inferred from world state.
//!
//! Phase: [`Phase::MorphologicalLayer`] (FixedUpdate), atmosphere_sync in Update.

use bevy::prelude::*;

use crate::blueprint::constants::inferred_world_geometry::{
    ATMOSPHERE_DENSITY_THRESHOLD_RATIO, ATMOSPHERE_UPDATE_INTERVAL, DEFAULT_LATITUDE,
    SUN_BASE_INTENSITY, SUN_PLACEMENT_DISTANCE, SUN_ROTATION_SPEED,
};
use crate::blueprint::equations;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

/// Global atmosphere state inferred from energy field (written by FixedUpdate, read by Update).
#[derive(Resource, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource)]
pub struct AtmosphereState {
    pub sun_direction: Vec3,
    pub sun_intensity: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 3],
    pub bloom_intensity: f32,
    pub ambient_intensity: f32,
    pub ambient_color: [f32; 3],
}

impl Default for AtmosphereState {
    fn default() -> Self {
        let sun_direction = Vec3::new(0.5, 0.7, 0.5).normalize();
        Self {
            sun_direction,
            sun_intensity: SUN_BASE_INTENSITY,
            fog_start: 30.0,
            fog_end: 80.0,
            fog_color: [0.7, 0.75, 0.85],
            bloom_intensity: 0.0,
            ambient_intensity: 0.15,
            ambient_color: [0.6, 0.65, 0.8],
        }
    }
}

/// Marker for the directional light that represents the sun. SparseSet (transient marker).
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct SunMarker;

/// Infer atmosphere parameters from energy field state (FixedUpdate, MorphologicalLayer).
pub fn atmosphere_inference_system(
    field: Option<Res<EnergyFieldGrid>>,
    mut atmosphere: ResMut<AtmosphereState>,
    tick: Option<Res<SimulationClock>>,
) {
    // Throttle: only run every ATMOSPHERE_UPDATE_INTERVAL ticks.
    if let Some(ref clock) = tick {
        if clock.tick_id % ATMOSPHERE_UPDATE_INTERVAL as u64 != 0 {
            return;
        }
    }

    let Some(field) = field else { return };

    // Aggregate field statistics.
    let cell_count = (field.width as usize) * (field.height as usize);
    if cell_count == 0 {
        return;
    }
    let inv_count = 1.0 / cell_count as f32;

    let mut sum_qe: f32 = 0.0;
    let mut max_qe: f32 = 0.0;
    let mut materialized_count: u32 = 0;
    for cell in field.iter_cells() {
        let qe = cell.accumulated_qe.max(0.0);
        sum_qe += qe;
        if qe > max_qe {
            max_qe = qe;
        }
        if cell.materialized_entity.is_some() {
            materialized_count += 1;
        }
    }

    let max_qe_safe = max_qe.max(1.0);
    let avg_qe_norm = (sum_qe * inv_count / max_qe_safe).clamp(0.0, 1.0);

    // Density: fraction of cells with meaningful energy.
    let density_threshold = max_qe_safe * ATMOSPHERE_DENSITY_THRESHOLD_RATIO;
    let dense_count = field
        .iter_cells()
        .filter(|c| c.accumulated_qe > density_threshold)
        .count() as f32;
    let avg_density = (dense_count * inv_count).clamp(0.0, 1.0);

    // Canopy: fraction of cells with materialized entities.
    let canopy_factor = (materialized_count as f32 * inv_count).clamp(0.0, 1.0);

    // World radius: half-diagonal of the field extent.
    let extent_x = field.width as f32 * field.cell_size;
    let extent_y = field.height as f32 * field.cell_size;
    let world_radius = (extent_x * extent_x + extent_y * extent_y).sqrt() * 0.5;

    // Time angle: slow rotation using tick counter.
    let time_angle = if let Some(ref clock) = tick {
        (clock.tick_id as f32) * SUN_ROTATION_SPEED
    } else {
        0.0
    };

    let latitude = DEFAULT_LATITUDE;

    let sun_direction = equations::inferred_sun_direction(latitude, time_angle);
    let sun_intensity = equations::inferred_sun_intensity(sun_direction);
    let (fog_start, fog_end) =
        equations::inferred_fog_params(world_radius, avg_density, canopy_factor);
    let fog_color = equations::inferred_fog_color(sun_direction, avg_density);
    let bloom_intensity = equations::inferred_bloom_intensity(avg_qe_norm);
    let (ambient_intensity, ambient_color) =
        equations::inferred_ambient_light(canopy_factor, sun_intensity);

    let new_state = AtmosphereState {
        sun_direction,
        sun_intensity,
        fog_start,
        fog_end,
        fog_color,
        bloom_intensity,
        ambient_intensity,
        ambient_color,
    };

    // Guard change detection.
    if *atmosphere == new_state {
        return;
    }
    *atmosphere = new_state;
}

/// Sync atmosphere state to the directional light (Update schedule).
pub fn atmosphere_sync_system(
    atmosphere: Res<AtmosphereState>,
    mut lights: Query<(&mut DirectionalLight, &mut Transform), With<SunMarker>>,
) {
    if !atmosphere.is_changed() {
        return;
    }
    for (mut light, mut transform) in &mut lights {
        if light.illuminance != atmosphere.sun_intensity {
            light.illuminance = atmosphere.sun_intensity;
        }
        // Point light toward scene origin from sun direction.
        let sun_pos = atmosphere.sun_direction * SUN_PLACEMENT_DISTANCE;
        let new_transform = Transform::from_translation(sun_pos).looking_at(Vec3::ZERO, Vec3::Y);
        if *transform != new_transform {
            *transform = new_transform;
        }
    }
}
