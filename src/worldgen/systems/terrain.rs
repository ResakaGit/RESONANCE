//! Generación del resource [`TerrainField`](crate::topology::TerrainField) alineado al `EnergyFieldGrid`.
//! Pipeline: noise → normalizar → erosión → fill_pits → slope/aspect → drenaje → clasificación (T2–T5).

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::topology::config::{TerrainConfig, TerrainConfigRuntime, sanitize_terrain_config};
use crate::topology::generators::classifier::classify_all;
use crate::topology::generators::drainage::{
    compute_flow_accumulation, compute_flow_direction, fill_pits,
};
use crate::topology::generators::hydraulics::erode_hydraulic;
use crate::topology::generators::noise::{generate_heightmap, normalize_heightmap};
use crate::topology::generators::slope::derive_slope_aspect;
use crate::topology::{
    DirtyRegion, TerrainField, TerrainMutationEvent, apply_mutation, rederive_region,
};
use crate::worldgen::EnergyFieldGrid;

#[derive(Resource, Default)]
pub struct TerrainMutationQueue {
    pending: VecDeque<TerrainMutationEvent>,
}

/// Construye un `TerrainField` con las mismas dimensiones que el grid de energía.
///
/// `raw_cfg` se sanea con [`sanitize_terrain_config`]. Si `enabled == false`, relieve plano y todo `Plain`.
pub fn build_terrain_field(grid: &EnergyFieldGrid, raw_cfg: &TerrainConfig) -> TerrainField {
    let cfg = sanitize_terrain_config(raw_cfg);
    let w = grid.width;
    let h = grid.height;
    let cs = grid.cell_size;
    let origin = grid.origin;
    let seed = cfg.seed;

    let mut field = TerrainField::new(w, h, cs, origin, seed);

    if !cfg.enabled {
        return field;
    }

    let mut altitude = generate_heightmap(w, h, cs, origin, seed, &cfg.noise);
    normalize_heightmap(&mut altitude, cfg.noise.min_height, cfg.noise.max_height);
    erode_hydraulic(&mut altitude, w, h, cs, &cfg.erosion, seed);
    fill_pits(&mut altitude, w, h);

    let (slope, aspect) = derive_slope_aspect(&altitude, w, h, cs);
    let drainage = compute_flow_direction(&altitude, w, h);
    let drainage_accumulation = compute_flow_accumulation(&altitude, &drainage, w, h);
    let terrain_type = classify_all(
        &altitude,
        &slope,
        &drainage_accumulation,
        &cfg.classification,
    );

    field.altitude = altitude;
    field.slope = slope;
    field.aspect = aspect;
    field.drainage = drainage;
    field.drainage_accumulation = drainage_accumulation;
    field.terrain_type = terrain_type;
    field
}

/// Startup: tras fijar `EnergyFieldGrid` desde el mapa, materializa el terreno procedural.
pub fn insert_terrain_field_startup_system(
    mut commands: Commands,
    grid: Res<EnergyFieldGrid>,
    terrain_runtime: Res<TerrainConfigRuntime>,
) {
    let cfg = terrain_runtime
        .effective
        .as_ref()
        .cloned()
        .unwrap_or_else(|| sanitize_terrain_config(&TerrainConfig::default()));
    let field = build_terrain_field(grid.as_ref(), &cfg);
    commands.insert_resource(field);
}

/// Runtime: procesa mutaciones de terreno en batch y re-deriva una sola región unificada.
pub fn terrain_mutation_system(
    mut terrain: ResMut<TerrainField>,
    mut events: EventReader<TerrainMutationEvent>,
    mut queue: ResMut<TerrainMutationQueue>,
    terrain_runtime: Res<TerrainConfigRuntime>,
) {
    const MAX_MUTATIONS_PER_TICK: usize = 64;
    queue.pending.extend(events.read().copied());
    let mut batched_region: Option<DirtyRegion> = None;
    let to_process = queue.pending.len().min(MAX_MUTATIONS_PER_TICK);
    for ev in queue.pending.drain(..to_process) {
        let Some(region) = apply_mutation(&mut terrain, &ev.0) else {
            continue;
        };
        batched_region = Some(match batched_region {
            Some(current) => current.union(region),
            None => region,
        });
    }
    if let Some(region) = batched_region {
        let cfg = terrain_runtime
            .effective
            .as_ref()
            .cloned()
            .unwrap_or_else(|| sanitize_terrain_config(&TerrainConfig::default()));
        rederive_region(&mut terrain, &region, &cfg.classification);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::contracts::TerrainType;

    #[test]
    fn disabled_yields_flat_plain_grid() {
        let grid = EnergyFieldGrid::new(4, 3, 1.0, Vec2::ZERO);
        let cfg = TerrainConfig {
            enabled: false,
            ..Default::default()
        };
        let f = build_terrain_field(&grid, &cfg);
        assert_eq!(f.total_cells(), 12);
        assert!(f.altitude.iter().all(|&z| z == 0.0));
        assert!(f.terrain_type.iter().all(|&t| t == TerrainType::Plain));
    }

    #[test]
    fn enabled_small_grid_deterministic_twice() {
        let grid = EnergyFieldGrid::new(8, 8, 1.0, Vec2::new(-2.0, 1.0));
        let cfg = TerrainConfig {
            seed: 4242,
            enabled: true,
            erosion: crate::topology::ErosionParams {
                cycles: 2,
                ..Default::default()
            },
            ..Default::default()
        };
        let a = build_terrain_field(&grid, &cfg);
        let b = build_terrain_field(&grid, &cfg);
        assert_eq!(a.altitude, b.altitude);
        assert_eq!(a.terrain_type, b.terrain_type);
        assert_eq!(a.drainage.len(), a.total_cells());
    }
}
