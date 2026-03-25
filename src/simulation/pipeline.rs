//! Registro del schedule de simulación (`FixedUpdate`) y pipeline visual (`Update`).
//! Orden y `.chain()` idénticos al histórico `simulation_plugin.rs` (sprint Q5).
//!
//! **G9 (event ordering):** `SimulationClockSet → Phase::Input → … → Phase::MorphologicalLayer` en `.chain()`.
//! Ordena fases enteras; dentro de cada fase, subcadenas con `.chain()` / `.after()` (ver `physics`, `reactions`).

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::bridge::context_fill::bridge_phase_tick;
use crate::bridge::metrics::bridge_optimizer_enter_active_log_system;
use crate::bridge::{register_bridge_cache, CachePolicy, EvolutionSurrogateBridge};
use crate::blueprint::constants::EVOLUTION_SURROGATE_CACHE_CAPACITY;
use crate::runtime_platform::simulation_tick::{
    advance_simulation_clock_system, SimulationClockSet,
};
use crate::simulation::states::{GameState, PlayState};
use crate::simulation::{self, Phase};

/// Configura fases, reloj sim, Input, worldgen delta y cadenas por `Phase::ThermodynamicLayer` … `MorphologicalLayer`.
pub fn register_simulation_pipeline<S>(app: &mut App, schedule: S)
where
    S: ScheduleLabel + Clone,
{
    app.init_resource::<simulation::growth_budget::GrowthBudgetCursor>();
    app.init_resource::<simulation::env_scenario::EnvScenarioSnapshot>();
    app.init_resource::<simulation::evolution_surrogate::EvolutionSurrogateConfig>();
    app.init_resource::<simulation::evolution_surrogate::EvolutionSurrogateQueue>();
    app.init_resource::<simulation::evolution_surrogate::EvolutionSurrogateState>();
    if !app.world().contains_resource::<crate::bridge::BridgeCache<EvolutionSurrogateBridge>>() {
        register_bridge_cache::<EvolutionSurrogateBridge>(
            app,
            EVOLUTION_SURROGATE_CACHE_CAPACITY,
            CachePolicy::Lru,
        );
    }

    // `PrePhysics` mezcla worldgen (`GameState::Playing`, puede `Warmup`) y simulación (`Active` solo).
    let run_gameplay = in_state(GameState::Playing).and(in_state(PlayState::Active));
    app.configure_sets(
        schedule.clone(),
        (
            SimulationClockSet.run_if(run_gameplay.clone()),
            Phase::Input.run_if(run_gameplay.clone()),
            Phase::ThermodynamicLayer,
            Phase::AtomicLayer.run_if(run_gameplay.clone()),
            Phase::ChemicalLayer.run_if(run_gameplay.clone()),
            Phase::MetabolicLayer.run_if(run_gameplay.clone()),
            Phase::MorphologicalLayer.run_if(run_gameplay.clone()),
        )
            .chain(),
    );

    // Input: plataforma escribe voluntad antes que grimoire / capa2 (orden determinista).
    app.configure_sets(
        schedule.clone(),
        (
            simulation::InputChannelSet::PlatformWill,
            simulation::InputChannelSet::SimulationRest,
        )
            .chain()
            .in_set(Phase::Input),
    );

    // D1: Behavioral Intelligence — sub-phases with auto-deferred between Assess → Decide.
    app.configure_sets(
        schedule.clone(),
        (
            simulation::behavior::BehaviorSet::Assess,
            simulation::behavior::BehaviorSet::Decide,
        )
            .chain()
            .in_set(Phase::Input)
            .after(simulation::InputChannelSet::PlatformWill)
            .run_if(simulation::behavior::has_behavioral_agents),
    );

    app.add_systems(
        schedule.clone(),
        (
            advance_simulation_clock_system,
            bridge_phase_tick.after(advance_simulation_clock_system),
            bridge_optimizer_enter_active_log_system.after(bridge_phase_tick),
        )
            .chain()
            .in_set(SimulationClockSet),
    );

    #[cfg(not(feature = "v7_worldgen"))]
    crate::worldgen::systems::prephysics::register_prephysics_worldgen_through_delta(
        app,
        schedule.clone(),
    );
    #[cfg(feature = "v7_worldgen")]
    crate::worldgen::systems::prephysics::register_grimoire_and_spatial_index(
        app,
        schedule.clone(),
    );

    #[cfg(not(feature = "v7_worldgen"))]
    crate::worldgen::systems::prephysics::register_postphysics_nucleus_death_before_faction(
        app,
        schedule,
    );
}

/// Sistemas de derivación visual en `Update` (no `FixedUpdate`).
pub fn register_visual_derivation_pipeline(app: &mut App) {
    app.init_resource::<crate::worldgen::systems::visual::VisualDerivationStats>();
    app.init_resource::<crate::worldgen::ShapeInferenceFrameState>();
    app.add_systems(
        Update,
        (
            crate::worldgen::systems::performance::reset_visual_derivation_frame_system,
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
