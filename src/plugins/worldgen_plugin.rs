//! V7 WorldgenPlugin — orquestación de campo energético, materialización y visual.
//!
//! Registra todos los sistemas V7 en el pipeline de Bevy con el orden correcto.
//! Gated behind `v7_worldgen` feature flag.
//!
//! **Sprint 07:** Plugin puro de wiring — no define lógica de sistemas.

use bevy::prelude::*;

use crate::eco::eco_boundaries_system;
use crate::simulation::Phase;
use crate::simulation::states::{GameState, PlayState};
use crate::topology::config::TerrainConfigRuntime;
use crate::topology::{TerrainField, TerrainMutationEvent};
use crate::world::update_spatial_index_system;
use crate::worldgen::cell_field_snapshot::{cell_field_snapshot_sync_system, CellFieldSnapshotCache};
use crate::worldgen::sync_nutrient_field_len_system;
use crate::worldgen::systems::materialization::{
    clear_stale_materialized_cell_refs_system, materialization_delta_system,
    season_change_begin_system, season_transition_tick_system,
    worldgen_nucleus_death_notify_system, worldgen_nucleus_freq_changed_notify_system,
    worldgen_nucleus_freq_seed_system, worldgen_runtime_nucleus_created_system,
};
use crate::worldgen::systems::performance::{
    reset_visual_derivation_frame_system, sync_materialization_cache_len_system,
    worldgen_lod_refresh_system, worldgen_mat_budget_reset_system,
    worldgen_perf_reset_dirty_system, worldgen_propagation_budget_reset_system,
    worldgen_sim_tick_advance_system,
};
use crate::worldgen::systems::propagation::{
    derive_cell_state_system, dissipate_field_system, propagate_nuclei_system,
};
use crate::worldgen::systems::terrain::{terrain_mutation_system, TerrainMutationQueue};
use crate::worldgen::systems::visual::flush_pending_energy_visual_rebuild_system;
use crate::worldgen::EnergyFieldGrid;

/// SystemSet para todos los sistemas V7 worldgen en `FixedUpdate`.
/// Añadir `run_if(...)` a este set deshabilita V7 entero.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldgenPhase;

/// V7 WorldgenPlugin — registra campo, materialización, visual y tipos reflectivos.
pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        init_worldgen_resources(app);

        app.configure_sets(
            FixedUpdate,
            WorldgenPhase
                .in_set(Phase::ThermodynamicLayer)
                .after(update_spatial_index_system)
                .run_if(in_state(GameState::Playing)),
        );

        register_field_pipeline(app);
        register_postphysics_chain(app);
        register_visual_pipeline(app);
        register_worldgen_reflect_types(app);

        app.add_systems(Startup, init_worldgen_system);
    }
}

/// Recursos y eventos worldgen (idempotente — seguro si bootstrap ya los creó).
fn init_worldgen_resources(app: &mut App) {
    app.init_resource::<crate::worldgen::systems::materialization::SeasonTransition>()
        .init_resource::<crate::worldgen::systems::materialization::NucleusFreqTrack>()
        .init_resource::<crate::worldgen::systems::performance::WorldgenPerfSettings>()
        .init_resource::<crate::worldgen::systems::performance::WorldgenLodContext>()
        .init_resource::<crate::worldgen::systems::performance::MaterializationCellCache>()
        .init_resource::<crate::worldgen::CellFieldSnapshotCache>()
        .init_resource::<crate::worldgen::systems::performance::MatBudgetCounters>()
        .init_resource::<crate::worldgen::systems::performance::MatCacheStats>()
        .init_resource::<crate::worldgen::systems::performance::PropagationWriteBudget>()
        .init_resource::<crate::worldgen::systems::performance::VisualDerivationFrameState>()
        .init_resource::<crate::worldgen::systems::terrain::TerrainMutationQueue>()
        .init_resource::<crate::eco::boundary_field::EcoBoundaryField>()
        .init_resource::<crate::eco::context_lookup::EcoPlayfieldMargin>();

    // Eventos worldgen (idempotente).
    app.add_event::<crate::events::SeasonChangeEvent>()
        .add_event::<crate::events::WorldgenMutationEvent>()
        .add_event::<crate::events::DeathEvent>()
        .add_event::<crate::topology::TerrainMutationEvent>();
}

/// Startup: valida que el grid existe (creado por bootstrap).
fn init_worldgen_system(grid: Res<EnergyFieldGrid>) {
    info!(
        "WorldgenPlugin: grid {}x{}, cell_size={}",
        grid.width, grid.height, grid.cell_size
    );
}

/// FixedUpdate: perf reset → season → nuclei → propagate → dissipate → derive → eco → materialize.
fn register_field_pipeline(app: &mut App) {
    app.add_systems(
        FixedUpdate,
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
            .in_set(WorldgenPhase),
    );
}

/// FixedUpdate post-sim: fog → nucleus death → faction identity.
fn register_postphysics_chain(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            crate::simulation::fog_of_war::fog_of_war_provider_system
                .after(crate::simulation::metabolic_stress::metabolic_stress_death_system),
            crate::simulation::fog_of_war::fog_visibility_mask_system,
            worldgen_nucleus_death_notify_system,
            crate::simulation::post::faction_identity_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
    );
}

/// Update: derivación visual (color, emisión, escala, opacidad, shape inference).
fn register_visual_pipeline(app: &mut App) {
    app.init_resource::<crate::worldgen::systems::visual::VisualDerivationStats>();
    app.init_resource::<crate::worldgen::ShapeInferenceFrameState>();
    app.add_systems(
        Update,
        (
            reset_visual_derivation_frame_system,
            crate::worldgen::systems::visual::visual_derivation_update_changed_system,
            crate::worldgen::systems::visual::visual_derivation_insert_missing_system,
            crate::worldgen::systems::phenology_visual::phenology_visual_apply_system,
            crate::worldgen::systems::visual::visual_sync_to_render_system,
            crate::worldgen::reset_shape_inference_frame_system,
            crate::worldgen::shape_color_inference_system,
            crate::worldgen::growth_morphology_system,
        )
            .chain()
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
    );
}

/// Registra tipos worldgen para reflection/inspector.
fn register_worldgen_reflect_types(app: &mut App) {
    app.register_type::<crate::worldgen::EnergyNucleus>()
        .register_type::<crate::worldgen::Materialized>()
        .register_type::<crate::worldgen::EnergyVisual>()
        .register_type::<crate::worldgen::WorldArchetype>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use bevy::state::prelude::State;
    use std::time::Duration;

    use crate::blueprint::AlchemicalAlmanac;
    use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
    use crate::simulation::states::{GameState, PlayState};
    use crate::simulation::Phase;
    use crate::world::Scoreboard;
    use crate::worldgen::{EnergyFieldGrid, MapConfig, Materialized};
    use crate::worldgen::systems::materialization::{NucleusFreqTrack, SeasonTransition};
    use crate::worldgen::systems::performance::{
        MatBudgetCounters, MatCacheStats, MaterializationCellCache, PropagationWriteBudget,
        WorldgenLodContext, WorldgenPerfSettings,
    };

    /// Minimal app con resources worldgen (patrón de materialization tests).
    fn minimal_worldgen_app() -> App {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>().add_sub_state::<PlayState>();
        app.init_resource::<Time>()
            .init_resource::<Scoreboard>()
            .insert_resource(MapConfig::default())
            .insert_resource(AlchemicalAlmanac::default())
            .insert_resource(EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO))
            .init_resource::<NucleusFreqTrack>()
            .init_resource::<SeasonTransition>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<WorldgenLodContext>()
            .init_resource::<MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>()
            .init_resource::<PropagationWriteBudget>()
            .insert_resource(SimWorldTransformParams::default())
            .configure_sets(
                FixedUpdate,
                (Phase::ThermodynamicLayer, Phase::MetabolicLayer).chain(),
            );
        // Estado Playing+Active para que los run_if pasen.
        let world = app.world_mut();
        world.insert_resource(State::new(GameState::Playing));
        world.insert_resource(State::new(PlayState::Active));
        app
    }

    fn step_sim(app: &mut App, steps: u32) {
        let step = Duration::from_secs_f32(1.0 / 60.0);
        for _ in 0..steps {
            app.world_mut().resource_mut::<Time>().advance_by(step);
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    #[test]
    fn worldgen_phase_is_valid_system_set() {
        let a = WorldgenPhase;
        let b = WorldgenPhase;
        assert_eq!(a, b);
    }

    #[test]
    fn worldgen_plugin_builds_without_panic() {
        let mut app = minimal_worldgen_app();
        app.add_plugins(WorldgenPlugin);
        // FixedUpdate only — visual pipeline (Update) requiere Assets<Mesh> del render full.
        step_sim(&mut app, 3);
    }

    #[test]
    fn empty_grid_no_materialization() {
        let mut app = minimal_worldgen_app();
        app.add_plugins(WorldgenPlugin);

        step_sim(&mut app, 5);

        let world = app.world_mut();
        let mut query = world.query::<&Materialized>();
        let count = query.iter(world).count();
        assert_eq!(count, 0, "empty grid should produce zero materializations");
    }
}
