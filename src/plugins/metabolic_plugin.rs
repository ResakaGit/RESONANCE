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
                simulation::metabolic::basal_drain::basal_drain_system,
                simulation::metabolic::senescence_death::senescence_death_system,
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

        // SF-1: Observability — metrics snapshot after ecology dynamics.
        app.init_resource::<simulation::observability::SimulationMetricsSnapshot>();
        app.init_resource::<simulation::observability::SimulationEcologySnapshot>();
        app.init_resource::<simulation::observability::SimulationHealthDashboard>();
        app.add_systems(
            FixedUpdate,
            simulation::observability::metrics_snapshot_system
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::ecology_dynamics::succession_system)
                .before(faction_identity_system),
        );

        // CE: Culture observation — after metrics_snapshot, before faction_identity.
        app.add_systems(
            FixedUpdate,
            simulation::culture_observation::culture_observation_system
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .run_if(simulation::culture_observation::every_culture_observation_interval)
                .after(simulation::observability::metrics_snapshot_system)
                .before(faction_identity_system),
        );

        // ET-4: Infrastructure — persistent field modification + intake bonus.
        app.init_resource::<simulation::emergence::infrastructure::InfrastructureGrid>();
        app.init_resource::<simulation::emergence::infrastructure::InfrastructureConfig>();
        app.add_event::<simulation::emergence::infrastructure::InfrastructureInvestEvent>();
        app.add_systems(
            FixedUpdate,
            (
                simulation::emergence::infrastructure::infrastructure_update_system,
                simulation::emergence::infrastructure::infrastructure_intake_bonus_system,
            )
                .chain()
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::trophic::trophic_decomposer_system)
                .before(faction_identity_system),
        );

        // AC-5: Cooperation Emergence — Nash alliance detection after trophic.
        app.add_systems(
            FixedUpdate,
            simulation::cooperation::cooperation_evaluation_system
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::trophic::trophic_decomposer_system)
                .before(faction_identity_system),
        );

        // ET-5: Symbiosis effects — mutualism/parasitism drain/benefit.
        app.add_systems(
            FixedUpdate,
            simulation::emergence::symbiosis_effect::symbiosis_effect_system
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::cooperation::cooperation_evaluation_system)
                .before(faction_identity_system),
        );

        // ET-9: Niche adaptation — character displacement under competitive pressure.
        app.add_systems(
            FixedUpdate,
            simulation::emergence::niche_adaptation::niche_adaptation_system
                .in_set(Phase::MetabolicLayer)
                .run_if(run_gameplay.clone())
                .after(simulation::cooperation::cooperation_evaluation_system)
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
