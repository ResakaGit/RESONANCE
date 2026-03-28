//! MorphologicalPlugin — Phase::MorphologicalLayer systems.
//!
//! Extracted from `pipeline.rs` in sprint Q5.
//! Pure registrar: no state, no resources. Ordering preserved exactly.
//!
//! Domains: shape optimization (MG-4), surface rugosity (MG-7), albedo (MG-5),
//! inference_growth, allometric_growth, env_scenario, evolution_surrogate,
//! organ_lifecycle, reproduction, abiogenesis, morpho_adaptation (D8), bridge_metrics.

use bevy::prelude::*;

use crate::bridge::metrics::bridge_metrics_collect_system;
use crate::simulation::{self, Phase};
use crate::simulation::post::faction_identity_system;
use crate::simulation::states::{GameState, PlayState};
use crate::worldgen::ActiveMapName;
use crate::worldgen::map_config::ROUND_WORLD_ROSA_MAP_SLUG;

#[inline]
fn not_round_world_rosa(active: Option<Res<ActiveMapName>>) -> bool {
    active.map(|a| a.0 != ROUND_WORLD_ROSA_MAP_SLUG).unwrap_or(true)
}

/// Registers all Phase::MorphologicalLayer systems.
pub struct MorphologicalPlugin;

impl Plugin for MorphologicalPlugin {
    fn build(&self, app: &mut App) {
        let run_gameplay = in_state(GameState::Playing).and(in_state(PlayState::Active));

        // MG-4: shape optimization — after DAG step, before downstream morphological systems.
        app.add_systems(
            FixedUpdate,
            simulation::morphogenesis::shape_optimization_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::morphogenesis::entropy_constraint_system),
        );

        // MG-7: surface rugosity — after shape optimization, before albedo inference.
        app.add_systems(
            FixedUpdate,
            simulation::morphogenesis::surface_rugosity_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::morphogenesis::shape_optimization_system),
        );

        // MG-5: albedo inference — after surface rugosity (MG-7 enlazado).
        app.add_systems(
            FixedUpdate,
            simulation::morphogenesis::albedo_inference_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::morphogenesis::surface_rugosity_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::inference_growth::cleanup_orphan_growth_intent_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::growth_budget::growth_budget_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::inference_growth::growth_intent_inference_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::inference_growth::cleanup_orphan_growth_intent_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::allometric_growth::allometric_growth_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                // Importante: mantener dependencia intra-capa solamente.
                // No agregar `.before(faction_identity_system)` para evitar ciclos entre
                // `MetabolicLayer` y `MorphologicalLayer` (ya ordenadas por `.chain()`).
                .after(simulation::inference_growth::growth_intent_inference_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::env_scenario::effective_viability_init_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::growth_budget::growth_budget_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::env_scenario::organ_viability_with_env_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::env_scenario::effective_viability_init_system)
                .before(simulation::organ_lifecycle::lifecycle_stage_inference_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::evolution_surrogate::evolution_surrogate_enqueue_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::env_scenario::organ_viability_with_env_system)
                .before(simulation::evolution_surrogate::evolution_surrogate_tick_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::evolution_surrogate::evolution_surrogate_tick_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::env_scenario::organ_viability_with_env_system)
                .before(simulation::organ_lifecycle::lifecycle_stage_inference_system),
        );

        app.add_systems(
            FixedUpdate,
            (
                simulation::organ_lifecycle::lifecycle_stage_init_system,
                simulation::organ_lifecycle::lifecycle_stage_inference_system,
            )
                .chain()
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::growth_budget::growth_budget_system)
                .after(simulation::allometric_growth::allometric_growth_system),
        );

        // AD-2: Internal field diffusion (before split detection).
        app.add_systems(
            FixedUpdate,
            simulation::lifecycle::internal_field_diffusion::internal_field_diffusion_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::allometric_growth::allometric_growth_system),
        );

        // AD-4: Axiomatic split — replaces reproduction_spawn_system (AD-5).
        // Division occurs when internal field valley reaches qe ≤ 0 (Axiom 1).
        app.add_systems(
            FixedUpdate,
            simulation::lifecycle::axiomatic_split::axiomatic_split_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::lifecycle::internal_field_diffusion::internal_field_diffusion_system),
        );

        app.add_systems(
            FixedUpdate,
            simulation::abiogenesis::abiogenesis_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .run_if(not_round_world_rosa)
                .after(simulation::lifecycle::axiomatic_split::axiomatic_split_system),
        );

        // Nucleus recycling: nutrient accumulation → new finite nucleus.
        app.init_resource::<crate::worldgen::systems::nucleus_recycling::NucleusRecyclingCursor>();
        app.add_systems(
            FixedUpdate,
            crate::worldgen::systems::nucleus_recycling::nucleus_recycling_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::abiogenesis::abiogenesis_system),
        );

        // Awakening: inert entities gain behavioral capabilities when coherence threshold met.
        app.add_systems(
            FixedUpdate,
            simulation::awakening::awakening_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(crate::worldgen::systems::nucleus_recycling::nucleus_recycling_system),
        );

        // ET-6: Epigenetic adaptation — environment modulates gene expression before constructal.
        app.add_systems(
            FixedUpdate,
            simulation::emergence::epigenetic_adaptation::epigenetic_adaptation_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::morphogenesis::albedo_inference_system),
        );

        // Constructal body plan — infer appendage count from thermodynamic cost minimization.
        // After albedo (MG-5) + epigenetics so all morph params are current; before lifecycle stage.
        app.add_systems(
            FixedUpdate,
            simulation::lifecycle::constructal_body_plan_system
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::emergence::epigenetic_adaptation::epigenetic_adaptation_system),
        );

        // D8: Morphological Adaptation — Bergmann/Allen/Wolff, every 16 ticks.
        app.add_systems(
            FixedUpdate,
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
            FixedUpdate,
            bridge_metrics_collect_system
                .after(faction_identity_system)
                .after(simulation::abiogenesis::abiogenesis_system)
                .in_set(Phase::MorphologicalLayer)
                .run_if(run_gameplay),
        );
    }
}
