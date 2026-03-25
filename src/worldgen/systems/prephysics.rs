//! Orden canónico PrePhysics: índice espacial + worldgen mutation + campo + materialización delta.
//! Fuente única consumida por `SimulationPlugin` y tests de regresión de schedule.

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use super::materialization::{
    clear_stale_materialized_cell_refs_system, materialization_delta_system,
    season_change_begin_system, season_transition_tick_system,
    worldgen_nucleus_freq_changed_notify_system, worldgen_nucleus_freq_seed_system,
    worldgen_runtime_nucleus_created_system,
};
use super::performance::{
    sync_materialization_cache_len_system, worldgen_lod_refresh_system,
    worldgen_mat_budget_reset_system, worldgen_perf_reset_dirty_system,
    worldgen_propagation_budget_reset_system, worldgen_sim_tick_advance_system,
};
use super::propagation::{
    derive_cell_state_system, dissipate_field_system, propagate_nuclei_system,
};
use super::terrain::TerrainMutationQueue;
use super::terrain::terrain_mutation_system;
use super::visual::flush_pending_energy_visual_rebuild_system;
use crate::eco::boundary_field::EcoBoundaryField;
use crate::eco::eco_boundaries_system;
use crate::simulation::ability_targeting::channeling_grimoire_emit_system;
use crate::simulation::input::grimoire_cast_resolve_system;
use crate::topology::config::TerrainConfigRuntime;
use crate::topology::{TerrainField, TerrainMutationEvent};
use crate::world::update_spatial_index_system;
use crate::worldgen::cell_field_snapshot::{CellFieldSnapshotCache, cell_field_snapshot_sync_system};
use crate::worldgen::sync_nutrient_field_len_system;

use super::materialization::worldgen_nucleus_death_notify_system;
use crate::simulation::Phase;
use crate::simulation::fog_of_war::{fog_of_war_provider_system, fog_visibility_mask_system};
use crate::simulation::metabolic_stress::metabolic_stress_death_system;
use crate::simulation::post::faction_identity_system;
use crate::simulation::states::{GameState, PlayState};

/// Cadena worldgen/campo/materialización delta (dirty, presupuestos, LOD, cache, tick sim) en el orden de `SimulationPlugin`.
pub fn register_worldgen_core_prephysics_chain<S: ScheduleLabel + Clone>(
    app: &mut App,
    schedule: S,
) {
    app.init_resource::<EcoBoundaryField>();
    app.add_systems(
        schedule,
        (
            (
                worldgen_perf_reset_dirty_system,
                sync_nutrient_field_len_system,
                worldgen_propagation_budget_reset_system,
                worldgen_mat_budget_reset_system,
                worldgen_lod_refresh_system,
                sync_materialization_cache_len_system,
                season_change_begin_system,
                season_transition_tick_system,
                worldgen_nucleus_freq_seed_system,
                worldgen_runtime_nucleus_created_system,
                worldgen_nucleus_freq_changed_notify_system,
                terrain_mutation_system
                    .run_if(resource_exists::<Events<TerrainMutationEvent>>)
                    .run_if(resource_exists::<TerrainMutationQueue>)
                    .run_if(resource_exists::<TerrainField>)
                    .run_if(resource_exists::<TerrainConfigRuntime>),
                propagate_nuclei_system,
                dissipate_field_system,
                derive_cell_state_system,
            )
                .chain(),
            (
                eco_boundaries_system,
                clear_stale_materialized_cell_refs_system,
                materialization_delta_system,
                flush_pending_energy_visual_rebuild_system,
            )
                .chain(),
            // Tras mutaciones de celdas por materialización / stale refs (EPI1 + verificación schedule).
            cell_field_snapshot_sync_system.run_if(resource_exists::<CellFieldSnapshotCache>),
            worldgen_sim_tick_advance_system,
        )
            .chain()
            .in_set(Phase::ThermodynamicLayer)
            .run_if(in_state(GameState::Playing)),
    );
}

/// Grimoire emit + cast resolve + spatial index — non-worldgen ThermodynamicLayer head.
/// Separada para que `WorldgenPlugin` (v7) pueda registrar la cadena worldgen independientemente.
pub fn register_grimoire_and_spatial_index<S: ScheduleLabel + Clone>(
    app: &mut App,
    schedule: S,
) {
    app.add_systems(
        schedule.clone(),
        (
            channeling_grimoire_emit_system,
            grimoire_cast_resolve_system,
        )
            .chain()
            .in_set(Phase::ThermodynamicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
    );
    app.add_systems(
        schedule,
        update_spatial_index_system
            .in_set(Phase::ThermodynamicLayer)
            .run_if(in_state(GameState::Playing))
            .after(grimoire_cast_resolve_system),
    );
}

/// Cabeza PrePhysics (`grimoire` + índice) + cadena worldgen, con enlace `after` explícito.
pub fn register_prephysics_worldgen_through_delta<S: ScheduleLabel + Clone>(
    app: &mut App,
    schedule: S,
) {
    app.init_resource::<EcoBoundaryField>();
    register_grimoire_and_spatial_index(app, schedule.clone());
    app.add_systems(
        schedule,
        (
            (
                worldgen_perf_reset_dirty_system,
                sync_nutrient_field_len_system,
                worldgen_propagation_budget_reset_system,
                worldgen_mat_budget_reset_system,
                worldgen_lod_refresh_system,
                sync_materialization_cache_len_system,
                season_change_begin_system,
                season_transition_tick_system,
                worldgen_nucleus_freq_seed_system,
                worldgen_runtime_nucleus_created_system,
                worldgen_nucleus_freq_changed_notify_system,
                terrain_mutation_system
                    .run_if(resource_exists::<Events<TerrainMutationEvent>>)
                    .run_if(resource_exists::<TerrainMutationQueue>)
                    .run_if(resource_exists::<TerrainField>)
                    .run_if(resource_exists::<TerrainConfigRuntime>),
                propagate_nuclei_system,
                dissipate_field_system,
                derive_cell_state_system,
            )
                .chain(),
            (
                eco_boundaries_system,
                clear_stale_materialized_cell_refs_system,
                materialization_delta_system,
                flush_pending_energy_visual_rebuild_system,
            )
                .chain(),
            cell_field_snapshot_sync_system.run_if(resource_exists::<CellFieldSnapshotCache>),
            worldgen_sim_tick_advance_system,
        )
            .chain()
            .in_set(Phase::ThermodynamicLayer)
            .after(update_spatial_index_system)
            .run_if(in_state(GameState::Playing)),
    );
}

/// PostPhysics: notificación worldgen **antes** de `faction_identity` (mismo orden que el plugin).
pub fn register_postphysics_nucleus_death_before_faction<S: ScheduleLabel + Clone>(
    app: &mut App,
    schedule: S,
) {
    app.add_systems(
        schedule,
        (
            fog_of_war_provider_system
                .after(metabolic_stress_death_system),
            fog_visibility_mask_system,
            worldgen_nucleus_death_notify_system,
            faction_identity_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
    );
}
