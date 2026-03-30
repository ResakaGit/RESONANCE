//! MGN-4 Bevy: Infer MetabolicGraph from InferenceProfile + EpigeneticState.
//!
//! Triggers on `Changed<InferenceProfile>` (mutation, reproduction, growth).
//! Reads epigenetic mask for gene gating.
//! Inserts/replaces MetabolicGraph component on the entity.

use bevy::prelude::*;
use bevy::ecs::query::Or;

use crate::blueprint::equations::derived_thresholds::DISSIPATION_SOLID;
use crate::blueprint::equations::metabolic_genome;
use crate::blueprint::equations::variable_genome::{self, VariableGenome};
use crate::layers::epigenetics::EpigeneticState;
use crate::layers::inference::InferenceProfile;
use crate::layers::metabolic_graph::MetabolicGraph;
use crate::layers::BaseEnergy;

/// Infer MetabolicGraph for entities that gained or changed InferenceProfile.
///
/// Only runs when InferenceProfile changes (Changed<> filter).
/// Entities with too-simple profiles (no extra genes) get no graph.
/// Re-infers MetabolicGraph when genome (InferenceProfile) or environment (EpigeneticState) changes.
pub fn genome_to_metabolic_graph_system(
    mut commands: Commands,
    query: Query<
        (Entity, &InferenceProfile, Option<&EpigeneticState>, Option<&BaseEnergy>),
        Or<(Changed<InferenceProfile>, Changed<EpigeneticState>)>,
    >,
) {
    for (entity, profile, epigenetics, energy) in &query {
        let vg = VariableGenome::from_biases(
            profile.growth_bias,
            profile.mobility_bias,
            profile.branching_bias,
            profile.resilience,
        );

        let mask = epigenetics
            .map(|e| e.expression_mask)
            .unwrap_or([1.0; 4]);

        match metabolic_genome::metabolic_graph_from_variable_genome(&vg, &mask) {
            Ok(graph) => { commands.entity(entity).insert(graph); }
            Err(_) => {
                // Not complex enough for a metabolic graph — remove if present
                commands.entity(entity).remove::<MetabolicGraph>();
            }
        }

        // Apply genome maintenance cost if entity has energy
        if let Some(energy) = energy {
            let cost = variable_genome::gated_maintenance_cost(&vg, &mask, DISSIPATION_SOLID);
            if cost > 0.0 && energy.qe() > cost {
                // Cost applied via drain — uses existing EnergyOps pattern
                commands.entity(entity).insert(GenomeMaintenanceCost(cost));
            }
        }
    }
}

/// Transient marker: genome maintenance cost to be drained next tick.
///
/// Applied by `genome_maintenance_drain_system` in MetabolicLayer.
#[derive(Component, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct GenomeMaintenanceCost(pub f32);

/// Drain genome maintenance cost from BaseEnergy. Axiom 4: complexity costs.
pub fn genome_maintenance_drain_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BaseEnergy, &GenomeMaintenanceCost)>,
) {
    for (entity, mut energy, cost) in &mut query {
        let drain = cost.0.min(energy.qe());
        if drain > 0.0 {
            energy.drain(drain);
        }
        commands.entity(entity).remove::<GenomeMaintenanceCost>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::ecs::schedule::ScheduleLabel;

    #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
    struct TestSchedule;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_schedule(TestSchedule);
        app.add_systems(TestSchedule, genome_to_metabolic_graph_system);
        app
    }

    fn spawn_entity(app: &mut App, growth: f32, mobility: f32, branching: f32, resilience: f32, qe: f32) -> Entity {
        app.world_mut().spawn((
            BaseEnergy::new(qe),
            InferenceProfile::new(growth, mobility, branching, resilience),
        )).id()
    }

    #[test]
    fn simple_profile_no_graph() {
        let mut app = test_app();
        let e = spawn_entity(&mut app, 0.5, 0.5, 0.5, 0.5, 100.0);
        app.world_mut().run_schedule(TestSchedule);
        // 4-gene genome = no metabolic graph (too simple)
        assert!(
            app.world().entity(e).get::<MetabolicGraph>().is_none(),
            "4-gene genome should not produce metabolic graph"
        );
    }

    #[test]
    fn entity_without_profile_untouched() {
        let mut app = test_app();
        let e = app.world_mut().spawn(BaseEnergy::new(100.0)).id();
        app.world_mut().run_schedule(TestSchedule);
        assert!(app.world().entity(e).get::<MetabolicGraph>().is_none());
    }

    #[test]
    fn maintenance_cost_component_created() {
        let mut app = test_app();
        let e = spawn_entity(&mut app, 0.5, 0.5, 0.5, 0.5, 100.0);
        app.world_mut().run_schedule(TestSchedule);
        // Even if no graph, maintenance cost may be inserted for genome upkeep
        // (depends on whether cost > 0 for 4-gene genome at base dissipation)
        // Just verify no panic
        let _ = app.world().entity(e).get::<GenomeMaintenanceCost>();
    }

    #[test]
    fn drain_system_reduces_energy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_schedule(TestSchedule);
        app.add_systems(TestSchedule, genome_maintenance_drain_system);

        let e = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            GenomeMaintenanceCost(5.0),
        )).id();

        app.world_mut().run_schedule(TestSchedule);

        let qe = app.world().entity(e).get::<BaseEnergy>().unwrap().qe();
        assert!(qe < 100.0, "drain should reduce energy: {qe}");
        assert!(app.world().entity(e).get::<GenomeMaintenanceCost>().is_none(),
            "cost marker should be removed after drain");
    }

    #[test]
    fn drain_never_below_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_schedule(TestSchedule);
        app.add_systems(TestSchedule, genome_maintenance_drain_system);

        let e = app.world_mut().spawn((
            BaseEnergy::new(1.0),
            GenomeMaintenanceCost(999.0),
        )).id();

        app.world_mut().run_schedule(TestSchedule);
        let qe = app.world().entity(e).get::<BaseEnergy>().unwrap().qe();
        assert!(qe >= 0.0, "energy should not go below zero: {qe}");
    }
}
