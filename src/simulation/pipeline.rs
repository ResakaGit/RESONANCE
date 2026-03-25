//! Registro del schedule de simulación (`FixedUpdate`) y pipeline visual (`Update`).
//! Orden y `.chain()` idénticos al histórico `simulation_plugin.rs` (sprint Q5).
//!
//! **G9 (event ordering):** `SimulationClockSet → Phase::Input → … → Phase::MorphologicalLayer` en `.chain()`.
//! Ordena fases enteras; dentro de cada fase, subcadenas con `.chain()` / `.after()` (ver `physics`, `reactions`).

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::blueprint::almanac_hot_reload_system;
use crate::bridge::context_fill::bridge_phase_tick;
use crate::bridge::metrics::{
    bridge_metrics_collect_system, bridge_optimizer_enter_active_log_system,
};
use crate::bridge::{register_bridge_cache, CachePolicy, EvolutionSurrogateBridge};
use crate::blueprint::constants::EVOLUTION_SURROGATE_CACHE_CAPACITY;
use crate::eco::climate::{climate_config_hot_reload_system, climate_tick_system};
use crate::eco::eco_boundaries_system;
use crate::runtime_platform::simulation_tick::{
    advance_simulation_clock_system, SimulationClockSet,
};
use crate::simulation::physics;
use crate::simulation::post::faction_identity_system;
use crate::simulation::reactions;
use crate::simulation::states::{GameState, PlayState};
use crate::simulation::{self, Phase};
use crate::topology::config::terrain_config_loader_system;
use crate::worldgen::ActiveMapName;
use crate::worldgen::map_config::ROUND_WORLD_ROSA_MAP_SLUG;

#[inline]
fn not_round_world_rosa(active: Option<Res<ActiveMapName>>) -> bool {
    active.map(|a| a.0 != ROUND_WORLD_ROSA_MAP_SLUG).unwrap_or(true)
}
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
    )
    .add_systems(
        schedule.clone(),
        almanac_hot_reload_system.in_set(simulation::InputChannelSet::SimulationRest),
    )
    .add_systems(
        schedule.clone(),
        simulation::element_layer2::ensure_element_id_component_system
            .in_set(simulation::InputChannelSet::SimulationRest),
    )
    .add_systems(
        schedule.clone(),
        simulation::element_layer2::derive_frequency_from_element_id_system
            .in_set(simulation::InputChannelSet::SimulationRest)
            .after(simulation::element_layer2::ensure_element_id_component_system),
    )
    .add_systems(
        schedule.clone(),
        simulation::element_layer2::sync_element_id_from_frequency_system
            .in_set(simulation::InputChannelSet::SimulationRest)
            .after(simulation::element_layer2::derive_frequency_from_element_id_system),
    );

    app.add_systems(
        schedule.clone(),
        // Hotkeys primero: en el mismo tick Q+click, el targeting ya está armado antes del pick.
        (
            simulation::input::grimoire_cast_intent_system,
            simulation::ability_targeting::ability_point_target_pick_system,
        )
            .chain()
            .in_set(simulation::InputChannelSet::SimulationRest)
            .before(simulation::element_layer2::derive_frequency_from_element_id_system),
    );

    // D5: Sensory Perception — runs before D1 Assess so SensoryAwareness is ready.
    app.init_resource::<simulation::sensory_perception::SensoryScanCursor>();
    app.add_systems(
        schedule.clone(),
        (
            simulation::sensory_perception::sensory_frequency_scan_system,
            simulation::sensory_perception::sensory_threat_memory_system,
            simulation::sensory_perception::sensory_awareness_event_system,
        )
            .chain()
            .in_set(Phase::Input)
            .after(simulation::InputChannelSet::PlatformWill)
            .before(simulation::behavior::BehaviorSet::Assess)
            .run_if(simulation::behavior::has_behavioral_agents),
    );

    // D1: Behavioral Intelligence systems
    app.add_systems(
        schedule.clone(),
        (
            simulation::behavior::behavior_cooldown_tick_system,
            simulation::behavior::behavior_assess_needs_system,
            simulation::behavior::behavior_evaluate_threats_system,
        )
            .chain()
            .in_set(simulation::behavior::BehaviorSet::Assess),
    );
    app.add_systems(
        schedule.clone(),
        (
            simulation::behavior::behavior_decision_system,
            simulation::behavior::behavior_will_bridge_system,
        )
            .chain()
            .in_set(simulation::behavior::BehaviorSet::Decide),
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

    app.init_resource::<simulation::sensory::AttentionGrid>();

    app.add_systems(
        schedule.clone(),
        (
            terrain_config_loader_system,
            climate_config_hot_reload_system,
            climate_tick_system,
        )
            .chain()
            .in_set(Phase::ThermodynamicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .before(crate::worldgen::systems::terrain::terrain_mutation_system)
            .before(eco_boundaries_system)
            .before(simulation::containment::containment_system),
    )
    .add_systems(
        schedule.clone(),
        simulation::sensory::attention_convergence_system
            .in_set(Phase::ThermodynamicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
    )
    .add_systems(
        schedule.clone(),
        (
            simulation::containment::containment_system,
            simulation::structural_runtime::structural_constraint_system,
            simulation::containment::contained_thermal_transfer_system,
            simulation::pre_physics::reset_resonance_overlay_system,
            simulation::pre_physics::resonance_link_system,
            simulation::pre_physics::sync_injector_projected_qe_system,
            simulation::pre_physics::engine_processing_system,
            simulation::photosynthesis::irradiance_update_system,
            simulation::pre_physics::perception_system,
        )
            .chain()
            .in_set(Phase::ThermodynamicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(crate::worldgen::systems::visual::flush_pending_energy_visual_rebuild_system),
    );

    physics::register_physics_phase_systems(app, schedule.clone());
    reactions::register_reactions_phase_systems(app, schedule.clone());

    app.init_resource::<simulation::trophic::TrophicScanCursor>();

    app.add_systems(
        schedule.clone(),
        (
            simulation::growth_budget::growth_budget_system,
            simulation::metabolic_stress::metabolic_stress_death_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .before(faction_identity_system),
    );

    // D2: Trophic & Predation — 4 systems chained after growth_budget.
    app.add_systems(
        schedule.clone(),
        (
            simulation::trophic::trophic_satiation_decay_system,
            simulation::trophic::trophic_herbivore_forage_system,
            simulation::trophic::trophic_predation_attempt_system,
            simulation::trophic::trophic_decomposer_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(run_gameplay.clone())
            .after(simulation::growth_budget::growth_budget_system)
            .before(faction_identity_system),
    );

    // D6: Social & Communication — 3 systems chained after trophic.
    app.add_systems(
        schedule.clone(),
        (
            simulation::social_communication::social_pack_formation_system,
            simulation::social_communication::social_pack_cohesion_system,
            simulation::social_communication::social_dominance_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(run_gameplay.clone())
            .after(simulation::trophic::trophic_decomposer_system)
            .before(faction_identity_system),
    );

    // D9: Ecological Dynamics — census + carrying capacity + succession, after D6 social.
    app.init_resource::<simulation::ecology_dynamics::PopulationCensus>();
    app.init_resource::<simulation::ecology_dynamics::ReproductionPressureField>();
    app.init_resource::<simulation::ecology_dynamics::SuccessionState>();
    app.add_systems(
        schedule.clone(),
        (
            simulation::ecology_dynamics::census_system,
            simulation::ecology_dynamics::carrying_capacity_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(run_gameplay.clone())
            .run_if(simulation::ecology_dynamics::every_census_interval)
            .after(simulation::social_communication::social_dominance_system)
            .before(faction_identity_system),
    );
    app.add_systems(
        schedule.clone(),
        simulation::ecology_dynamics::succession_system
            .in_set(Phase::MetabolicLayer)
            .run_if(run_gameplay.clone())
            .run_if(simulation::ecology_dynamics::every_succession_interval)
            .after(simulation::ecology_dynamics::carrying_capacity_system)
            .before(faction_identity_system),
    );

    // MG-3/6: DAG metabólico — step → constraint → ledger.
    // Orden: metabolic_stress → step → constraint → ledger → faction_identity.
    app.add_systems(
        schedule.clone(),
        (
            simulation::morphogenesis::metabolic_graph_step_system,
            simulation::morphogenesis::entropy_constraint_system,
            simulation::morphogenesis::entropy_ledger_system,
        )
            .chain()
            .in_set(Phase::MetabolicLayer)
            .run_if(run_gameplay.clone())
            .after(simulation::metabolic_stress::metabolic_stress_death_system)
            .before(faction_identity_system),
    );

    #[cfg(not(feature = "v7_worldgen"))]
    crate::worldgen::systems::prephysics::register_postphysics_nucleus_death_before_faction(
        app,
        schedule.clone(),
    );

    // MG-4: shape optimization — after DAG step, before downstream morphological systems.
    app.add_systems(
        schedule.clone(),
        simulation::morphogenesis::shape_optimization_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::morphogenesis::entropy_constraint_system),
    );

    // MG-7: surface rugosity — after shape optimization, before albedo inference.
    app.add_systems(
        schedule.clone(),
        simulation::morphogenesis::surface_rugosity_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::morphogenesis::shape_optimization_system),
    );

    // MG-5: albedo inference — after surface rugosity (MG-7 enlazado).
    app.add_systems(
        schedule.clone(),
        simulation::morphogenesis::albedo_inference_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::morphogenesis::surface_rugosity_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::inference_growth::cleanup_orphan_growth_intent_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::growth_budget::growth_budget_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::inference_growth::growth_intent_inference_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::inference_growth::cleanup_orphan_growth_intent_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::allometric_growth::allometric_growth_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            // Importante: mantener dependencia intra-capa solamente.
            // No agregar `.before(faction_identity_system)` para evitar ciclos entre
            // `MetabolicLayer` y `MorphologicalLayer` (ya ordenadas por `.chain()`).
            .after(simulation::inference_growth::growth_intent_inference_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::env_scenario::effective_viability_init_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::growth_budget::growth_budget_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::env_scenario::organ_viability_with_env_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::env_scenario::effective_viability_init_system)
            .before(simulation::organ_lifecycle::lifecycle_stage_inference_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::evolution_surrogate::evolution_surrogate_enqueue_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::env_scenario::organ_viability_with_env_system)
            .before(simulation::evolution_surrogate::evolution_surrogate_tick_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::evolution_surrogate::evolution_surrogate_tick_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::env_scenario::organ_viability_with_env_system)
            .before(simulation::organ_lifecycle::lifecycle_stage_inference_system),
    );

    app.add_systems(
        schedule.clone(),
        (
            simulation::organ_lifecycle::lifecycle_stage_init_system,
            simulation::organ_lifecycle::lifecycle_stage_inference_system,
        )
            .chain()
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .after(simulation::growth_budget::growth_budget_system)
            .after(simulation::allometric_growth::allometric_growth_system),
    );

    app.add_systems(
        schedule.clone(),
        (
            simulation::reproduction::reproduction_cooldown_tick_system,
            simulation::reproduction::reproduction_spawn_system,
        )
            .chain()
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .run_if(not_round_world_rosa)
            .after(simulation::allometric_growth::allometric_growth_system),
    );

    app.add_systems(
        schedule.clone(),
        simulation::abiogenesis::abiogenesis_system
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
            .run_if(not_round_world_rosa)
            .after(simulation::reproduction::reproduction_spawn_system),
    );

    // D8: Morphological Adaptation — Bergmann/Allen/Wolff, every 16 ticks.
    app.add_systems(
        schedule.clone(),
        (
            simulation::morpho_adaptation::morphology_environmental_pressure_system,
            simulation::morpho_adaptation::morphology_use_adaptation_system,
            simulation::morpho_adaptation::morphology_organ_rebalance_system,
        )
            .chain()
            .in_set(Phase::MorphologicalLayer)
            .run_if(run_gameplay.clone())
            .run_if(simulation::morpho_adaptation::every_16_ticks)
            .after(simulation::morphogenesis::albedo_inference_system),
    );

    app.add_systems(
        schedule,
        bridge_metrics_collect_system
            .after(faction_identity_system)
            .after(simulation::abiogenesis::abiogenesis_system)
            .in_set(Phase::MorphologicalLayer)
            .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
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
