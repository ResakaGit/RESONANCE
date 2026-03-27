//! Cell materialization delta: spawn, sync, and despawn of materialized ECS entities.

use bevy::math::Vec2;
use bevy::prelude::*;

use super::super::performance::{
    MatBudgetCounters, MatCacheStats, MaterializationCellCache, WorldgenLodContext,
    WorldgenPerfSettings,
};
use crate::blueprint::AlchemicalAlmanac;
use crate::eco::boundary_field::EcoBoundaryField;
use crate::eco::context_lookup::eco_field_aligned_with_grid;
use crate::layers::{BaseEnergy, MatterCoherence, OscillatorySignature, SenescenceProfile, SpatialVolume};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::topology::TerrainField;
use crate::worldgen::constants::{
    ABIOGENESIS_FIELD_OCCUPANT_NAME, MATERIALIZED_COLLIDER_RADIUS_FACTOR,
    MATERIALIZED_MIN_COLLIDER_RADIUS, MATERIALIZED_SPAWN_BOND_ENERGY,
    MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
};
use crate::worldgen::lod::{
    materialization_allowed, materialize_input_signature, terrain_type_cache_tag,
};
use crate::worldgen::materialization_rules::{
    boundary_marker_cache_tag, boundary_visual_from_marker, materialize_cell_at_time_with_boundary,
};
use crate::worldgen::{
    BoundaryVisual, EnergyFieldGrid, EnergyVisual, Materialized, PendingEnergyVisualRebuild,
    PhenologyPhaseCache, PhenologyVisualParams,
};

fn phenology_params_for_frequency(
    almanac: &AlchemicalAlmanac,
    frequency_hz: f32,
) -> Option<PhenologyVisualParams> {
    let element_id = almanac.find_stable_band_id(frequency_hz)?;
    let def = almanac.get(element_id)?;
    if def.phenology.is_none() {
        return None;
    }
    Some(PhenologyVisualParams {
        element_id,
        growth_norm_ceiling: crate::blueprint::constants::PHENOLOGY_DEFAULT_GROWTH_NORM_CEILING,
        qe_reference: crate::blueprint::constants::VISUAL_QE_REFERENCE,
        epsilon: crate::blueprint::constants::PHENOLOGY_DEFAULT_EPSILON,
    })
}

fn sync_phenology_for_materialized(
    commands: &mut Commands,
    entity: Entity,
    almanac: &AlchemicalAlmanac,
    dominant_frequency_hz: f32,
) {
    if let Some(p) = phenology_params_for_frequency(almanac, dominant_frequency_hz) {
        commands
            .entity(entity)
            .insert(p)
            .remove::<PhenologyPhaseCache>();
    } else {
        commands
            .entity(entity)
            .remove::<PhenologyVisualParams>()
            .remove::<PhenologyPhaseCache>();
    }
}

#[inline]
fn is_abiogenesis_field_occupant(e: Entity, names: &Query<&Name>) -> bool {
    names
        .get(e)
        .is_ok_and(|n| n.as_str() == ABIOGENESIS_FIELD_OCCUPANT_NAME)
}

#[inline]
fn terrain_field_aligned_with_grid(grid: &EnergyFieldGrid, terrain: &TerrainField) -> bool {
    terrain.width == grid.width
        && terrain.height == grid.height
        && (terrain.cell_size - grid.cell_size).abs() <= f32::EPSILON
        && terrain.origin == grid.origin
}

/// Quita `EnergyVisual` y encola rebuild inmediato en PrePhysics (ver `flush_pending_energy_visual_rebuild_system`).
fn invalidate_materialized_energy_visual(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<EnergyVisual>()
        .insert(PendingEnergyVisualRebuild);
}

/// Limpia `materialized_entity` si la entidad ya no existe o está zombi.
pub fn clear_stale_materialized_cell_refs_system(
    mut grid: ResMut<EnergyFieldGrid>,
    materialized_query: Query<(), With<Materialized>>,
    names: Query<&Name>,
) {
    for cell in grid.iter_cells_mut() {
        let Some(e) = cell.materialized_entity else {
            continue;
        };
        if materialized_query.get(e).is_ok() {
            continue;
        }
        if is_abiogenesis_field_occupant(e, &names) {
            continue;
        }
        cell.materialized_entity = None;
    }
}

pub fn materialization_delta_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    mut grid: ResMut<EnergyFieldGrid>,
    terrain: Option<Res<TerrainField>>,
    boundaries: Option<Res<EcoBoundaryField>>,
    names: Query<&Name>,
    sync_query: Query<(
        Entity,
        &Materialized,
        &OscillatorySignature,
        Option<&BoundaryVisual>,
    )>,
    lod: Res<WorldgenLodContext>,
    settings: Res<WorldgenPerfSettings>,
    mut cache: ResMut<MaterializationCellCache>,
    mut mat_budget: ResMut<MatBudgetCounters>,
    mut cache_stats: ResMut<MatCacheStats>,
    clock: Option<Res<SimulationClock>>,
) {
    let tick_birth = clock.map(|c| c.tick_id).unwrap_or(0);
    let t = time.elapsed_secs();
    let cell_size = grid.cell_size;
    let width = grid.width;
    let height = grid.height;
    let terrain_ref = terrain
        .as_ref()
        .filter(|tf| terrain_field_aligned_with_grid(&grid, tf));

    for y in 0..height {
        for x in 0..width {
            let world_pos = grid.world_pos(x, y).unwrap_or(Vec2::ZERO);
            let idx = y as usize * width as usize + x as usize;
            let marker_at_cell = boundaries
                .as_ref()
                .filter(|bf| eco_field_aligned_with_grid(&grid, bf))
                .and_then(|bf| bf.markers.get(idx).copied());
            let allowed = materialization_allowed(
                world_pos,
                lod.focus_world,
                lod.sim_tick,
                settings.lod_materialization_cull_distance,
                settings.lod_mid_period,
                settings.lod_far_period,
            );

            if !allowed {
                let Some(cell) = grid.cell_xy_mut(x, y) else {
                    continue;
                };
                if let Some(e) = cell.materialized_entity {
                    if mat_budget.despawns_this_tick < settings.max_material_despawn_per_tick {
                        if sync_query.get(e).is_ok() {
                            commands.entity(e).despawn();
                        }
                        cell.materialized_entity = None;
                        mat_budget.despawns_this_tick += 1;
                    }
                }
                if let Some(slot) = cache.0.get_mut(idx) {
                    *slot = None;
                }
                continue;
            }

            let result_opt = {
                let Some(cell_ref) = grid.cell_xy(x, y) else {
                    continue;
                };
                let terrain_type = terrain_ref.and_then(|tf| {
                    let idx = y as usize * tf.width as usize + x as usize;
                    tf.terrain_type.get(idx).copied()
                });
                let sig = materialize_input_signature(cell_ref, t, &almanac)
                    ^ boundary_marker_cache_tag(marker_at_cell).rotate_left(19)
                    ^ terrain_type_cache_tag(terrain_type).rotate_left(37);
                let Some(slot) = cache.0.get_mut(idx) else {
                    continue;
                };
                match slot.as_ref() {
                    Some((cs, res)) if *cs == sig && *cs != 0 => {
                        cache_stats.hits += 1;
                        Some(res.clone())
                    }
                    _ => {
                        cache_stats.misses += 1;
                        let r = materialize_cell_at_time_with_boundary(
                            cell_ref,
                            &almanac,
                            t,
                            cell_size,
                            terrain_type,
                            marker_at_cell,
                        );
                        *slot = r.as_ref().map(|mr| (sig, mr.clone()));
                        r
                    }
                }
            };

            let Some(cell) = grid.cell_xy_mut(x, y) else {
                continue;
            };

            let want_boundary = marker_at_cell.and_then(boundary_visual_from_marker);

            let Some(result) = result_opt else {
                if let Some(e) = cell.materialized_entity {
                    if is_abiogenesis_field_occupant(e, &names) {
                        if let Some(slot) = cache.0.get_mut(idx) {
                            *slot = None;
                        }
                        continue;
                    }
                    if mat_budget.despawns_this_tick < settings.max_material_despawn_per_tick {
                        if sync_query.get(e).is_ok() {
                            commands.entity(e).despawn();
                        }
                        cell.materialized_entity = None;
                        mat_budget.despawns_this_tick += 1;
                    }
                }
                if let Some(slot) = cache.0.get_mut(idx) {
                    *slot = None;
                }
                continue;
            };

            match cell.materialized_entity {
                None => {
                    if mat_budget.spawns_this_tick >= settings.max_material_spawn_per_tick {
                        continue;
                    }
                    mat_budget.spawns_this_tick += 1;
                    let id = if let Some(bv) = want_boundary {
                        commands
                            .spawn((
                                Materialized {
                                    cell_x: x as i32,
                                    cell_y: y as i32,
                                    archetype: result.archetype,
                                },
                                bv,
                                BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                                OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                                SpatialVolume::new(
                                    (cell_size * MATERIALIZED_COLLIDER_RADIUS_FACTOR)
                                        .max(MATERIALIZED_MIN_COLLIDER_RADIUS),
                                ),
                                MatterCoherence::new(
                                    cell.matter_state,
                                    MATERIALIZED_SPAWN_BOND_ENERGY,
                                    MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                                ),
                                layout.materialized_tile_transform(world_pos),
                                GlobalTransform::default(),
                                Sprite::default(),
                                SenescenceProfile {
                                    tick_birth,
                                    senescence_coeff: crate::blueprint::constants::senescence_coeff_materialized(),
                                    max_viable_age: crate::blueprint::constants::senescence_max_age_materialized(),
                                    strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
                                },
                            ))
                            .id()
                    } else {
                        commands
                            .spawn((
                                Materialized {
                                    cell_x: x as i32,
                                    cell_y: y as i32,
                                    archetype: result.archetype,
                                },
                                BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                                OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                                SpatialVolume::new(
                                    (cell_size * MATERIALIZED_COLLIDER_RADIUS_FACTOR)
                                        .max(MATERIALIZED_MIN_COLLIDER_RADIUS),
                                ),
                                MatterCoherence::new(
                                    cell.matter_state,
                                    MATERIALIZED_SPAWN_BOND_ENERGY,
                                    MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                                ),
                                layout.materialized_tile_transform(world_pos),
                                GlobalTransform::default(),
                                Sprite::default(),
                                SenescenceProfile {
                                    tick_birth,
                                    senescence_coeff: crate::blueprint::constants::senescence_coeff_materialized(),
                                    max_viable_age: crate::blueprint::constants::senescence_max_age_materialized(),
                                    strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
                                },
                            ))
                            .id()
                    };
                    sync_phenology_for_materialized(
                        &mut commands,
                        id,
                        &almanac,
                        cell.dominant_frequency_hz,
                    );
                    cell.materialized_entity = Some(id);
                }
                Some(e) if sync_query.get(e).is_err() => {
                    if is_abiogenesis_field_occupant(e, &names) {
                        continue;
                    }
                    if mat_budget.despawns_this_tick >= settings.max_material_despawn_per_tick
                        || mat_budget.spawns_this_tick >= settings.max_material_spawn_per_tick
                    {
                        continue;
                    }
                    mat_budget.despawns_this_tick += 1;
                    if let Some(mut ec) = commands.get_entity(e) {
                        ec.despawn();
                    }
                    cell.materialized_entity = None;
                    mat_budget.spawns_this_tick += 1;
                    let id = if let Some(bv) = want_boundary {
                        commands
                            .spawn((
                                Materialized {
                                    cell_x: x as i32,
                                    cell_y: y as i32,
                                    archetype: result.archetype,
                                },
                                bv,
                                BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                                OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                                SpatialVolume::new(
                                    (cell_size * MATERIALIZED_COLLIDER_RADIUS_FACTOR)
                                        .max(MATERIALIZED_MIN_COLLIDER_RADIUS),
                                ),
                                MatterCoherence::new(
                                    cell.matter_state,
                                    MATERIALIZED_SPAWN_BOND_ENERGY,
                                    MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                                ),
                                layout.materialized_tile_transform(world_pos),
                                GlobalTransform::default(),
                                Sprite::default(),
                                SenescenceProfile {
                                    tick_birth,
                                    senescence_coeff: crate::blueprint::constants::senescence_coeff_materialized(),
                                    max_viable_age: crate::blueprint::constants::senescence_max_age_materialized(),
                                    strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
                                },
                            ))
                            .id()
                    } else {
                        commands
                            .spawn((
                                Materialized {
                                    cell_x: x as i32,
                                    cell_y: y as i32,
                                    archetype: result.archetype,
                                },
                                BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                                OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                                SpatialVolume::new(
                                    (cell_size * MATERIALIZED_COLLIDER_RADIUS_FACTOR)
                                        .max(MATERIALIZED_MIN_COLLIDER_RADIUS),
                                ),
                                MatterCoherence::new(
                                    cell.matter_state,
                                    MATERIALIZED_SPAWN_BOND_ENERGY,
                                    MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                                ),
                                layout.materialized_tile_transform(world_pos),
                                GlobalTransform::default(),
                                Sprite::default(),
                                SenescenceProfile {
                                    tick_birth,
                                    senescence_coeff: crate::blueprint::constants::senescence_coeff_materialized(),
                                    max_viable_age: crate::blueprint::constants::senescence_max_age_materialized(),
                                    strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
                                },
                            ))
                            .id()
                    };
                    sync_phenology_for_materialized(
                        &mut commands,
                        id,
                        &almanac,
                        cell.dominant_frequency_hz,
                    );
                    cell.materialized_entity = Some(id);
                }
                Some(e) => {
                    let Ok((_, mat, sig, prev_bv)) = sync_query.get(e) else {
                        continue;
                    };
                    let archetype_changed = mat.archetype != result.archetype;
                    let freq_changed =
                        (sig.frequency_hz() - cell.dominant_frequency_hz).abs() > 0.5;
                    let bv_changed = prev_bv.copied() != want_boundary;
                    if freq_changed {
                        commands.entity(e).insert((
                            Materialized {
                                cell_x: x as i32,
                                cell_y: y as i32,
                                archetype: result.archetype,
                            },
                            BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                            OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                            MatterCoherence::new(
                                cell.matter_state,
                                MATERIALIZED_SPAWN_BOND_ENERGY,
                                MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                            ),
                            Sprite::default(),
                        ));
                    } else if archetype_changed {
                        commands.entity(e).insert(Materialized {
                            cell_x: x as i32,
                            cell_y: y as i32,
                            archetype: result.archetype,
                        });
                    }
                    if bv_changed {
                        match want_boundary {
                            Some(bv) => {
                                commands.entity(e).insert(bv);
                            }
                            None => {
                                commands.entity(e).remove::<BoundaryVisual>();
                            }
                        }
                    }
                    if archetype_changed || freq_changed || bv_changed {
                        invalidate_materialized_energy_visual(&mut commands, e);
                        sync_phenology_for_materialized(
                            &mut commands,
                            e,
                            &almanac,
                            cell.dominant_frequency_hz,
                        );
                    }
                }
            }
        }
    }
}
