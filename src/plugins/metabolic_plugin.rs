//! MetabolicPlugin — Phase::MetabolicLayer systems.
//!
//! Extracted from `pipeline.rs` in sprint Q5.
//! Pure registrar: no state, no resources. Ordering preserved exactly.
//!
//! Domains: growth_budget, metabolic_stress, trophic (D2), social (D6),
//! ecology (D9), morphogenesis DAG (MG-3/6).

use bevy::prelude::*;

use crate::simulation::{self, Phase};
use crate::simulation::post::faction_identity_system;
use crate::simulation::states::{GameState, PlayState};

/// Registers all Phase::MetabolicLayer systems.
pub struct MetabolicPlugin;

impl Plugin for MetabolicPlugin {
    fn build(&self, app: &mut App) {
        let run_gameplay = in_state(GameState::Playing).and(in_state(PlayState::Active));

        app.init_resource::<simulation::trophic::TrophicScanCursor>();

        app.add_systems(
            FixedUpdate,
            (
                simulation::growth_budget::growth_budget_system,
                simulation::metabolic_stress::metabolic_stress_death_system,
            )
                .chain()
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .before(faction_identity_system),
        );

        // D2: Trophic & Predation — 4 systems chained after growth_budget.
        app.add_systems(
            FixedUpdate,
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
            FixedUpdate,
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
            FixedUpdate,
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
            FixedUpdate,
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
            FixedUpdate,
            (
                simulation::morphogenesis::metabolic_graph_step_system,
                simulation::morphogenesis::entropy_constraint_system,
                simulation::morphogenesis::entropy_ledger_system,
            )
                .chain()
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay)
                .after(simulation::metabolic_stress::metabolic_stress_death_system)
                .before(faction_identity_system),
        );
    }
}
