use bevy::prelude::*;

use crate::blueprint::constants::{
    MORPHO_ADAPTATION_RATE, MORPHO_REBALANCE_THRESHOLD, MORPHO_TARGET_TEMPERATURE,
    WOLFF_SEDENTARY_SPEED,
};
use crate::blueprint::equations;
use crate::layers::{
    BaseEnergy, BehaviorIntent, BehaviorMode, FlowVector, InferenceProfile, MatterCoherence,
    PendingMorphRebuild, SpatialVolume,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

const MORPHO_TICK_INTERVAL: u64 = 16;

/// Run condition: every 16 ticks.
pub fn every_16_ticks(clock: Res<SimulationClock>) -> bool {
    clock.tick_id % MORPHO_TICK_INTERVAL == 0
}

/// S1: Environmental pressure nudges InferenceProfile biases (Bergmann + Allen).
pub fn morphology_environmental_pressure_system(
    mut query: Query<
        (&Transform, &BaseEnergy, &mut InferenceProfile),
        With<SpatialVolume>,
    >,
    field: Res<EnergyFieldGrid>,
) {
    for (transform, energy, mut profile) in &mut query {
        if energy.qe() <= 0.0 {
            continue;
        }
        let pos = transform.translation.truncate();
        let Some(cell) = field.cell_at(pos) else {
            continue;
        };
        let t_env = cell.temperature;

        let bergmann = equations::bergmann_radius_pressure(t_env, MORPHO_TARGET_TEMPERATURE);
        let allen = equations::allen_appendage_pressure(t_env, MORPHO_TARGET_TEMPERATURE);

        let new_growth = (profile.growth_bias + bergmann.clamp(-MORPHO_ADAPTATION_RATE, MORPHO_ADAPTATION_RATE))
            .clamp(0.0, 1.0);
        let new_branching = (profile.branching_bias + allen.clamp(-MORPHO_ADAPTATION_RATE, MORPHO_ADAPTATION_RATE))
            .clamp(0.0, 1.0);

        if (profile.growth_bias - new_growth).abs() > f32::EPSILON {
            profile.growth_bias = new_growth;
        }
        if (profile.branching_bias - new_branching).abs() > f32::EPSILON {
            profile.branching_bias = new_branching;
        }
    }
}

/// S2: Use-driven adaptation — moving entities strengthen bonds (Wolff's law).
pub fn morphology_use_adaptation_system(
    mut query: Query<(
        &FlowVector,
        &mut MatterCoherence,
        Option<&BehaviorIntent>,
    )>,
) {
    for (flow, mut coherence, behavior) in &mut query {
        let speed = flow.speed();
        let is_active = speed > WOLFF_SEDENTARY_SPEED
            || behavior.is_some_and(|b| matches!(
                b.mode,
                BehaviorMode::Hunt { .. } | BehaviorMode::Flee { .. }
            ));
        let load_history = if is_active { speed.min(1.0) } else { 0.0 };
        let new_bond = equations::use_driven_bone_density(load_history, coherence.bond_energy_eb());
        if (coherence.bond_energy_eb() - new_bond).abs() > f32::EPSILON {
            coherence.set_bond_energy_eb(new_bond);
        }
    }
}

/// S3: Trigger organ re-inference when InferenceProfile deviates beyond threshold.
pub fn morphology_organ_rebalance_system(
    mut commands: Commands,
    query: Query<
        (Entity, &InferenceProfile),
        (Changed<InferenceProfile>, Without<PendingMorphRebuild>),
    >,
) {
    const MID: f32 = 0.5;
    for (entity, profile) in &query {
        let max_deviation = (profile.growth_bias - MID)
            .abs()
            .max((profile.branching_bias - MID).abs())
            .max((profile.mobility_bias - MID).abs());
        if max_deviation > MORPHO_REBALANCE_THRESHOLD {
            commands.entity(entity).insert(PendingMorphRebuild);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::{BaseEnergy, SpatialVolume, FlowVector, MatterCoherence, InferenceProfile};
    use crate::worldgen::EnergyFieldGrid;
    use crate::blueprint::constants::{DEFAULT_GRID_DIMS, DEFAULT_GRID_ORIGIN};
    use crate::worldgen::FIELD_CELL_SIZE;

    fn minimal_app_with_field() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let grid = EnergyFieldGrid::new(DEFAULT_GRID_DIMS, DEFAULT_GRID_DIMS, FIELD_CELL_SIZE, DEFAULT_GRID_ORIGIN);
        app.insert_resource(grid);
        app.init_resource::<SimulationClock>();
        app
    }

    #[test]
    fn bergmann_cold_increases_growth_bias() {
        let mut app = minimal_app_with_field();
        app.add_systems(Update, morphology_environmental_pressure_system);

        // Cold cell: temperature = 50 (well below MORPHO_TARGET_TEMPERATURE 300)
        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            if let Some(cell) = grid.cell_xy_mut(0, 0) {
                cell.temperature = 50.0;
            }
        }

        let origin = DEFAULT_GRID_ORIGIN;
        let half_cell = FIELD_CELL_SIZE * 0.5;
        let pos = Vec3::new(origin.x + half_cell, origin.y + half_cell, 0.0);

        let initial_growth = 0.5;
        let entity = app.world_mut().spawn((
            Transform::from_translation(pos),
            BaseEnergy::new(100.0),
            SpatialVolume::new(1.0),
            InferenceProfile::new(initial_growth, 0.5, 0.5, 0.5),
        )).id();

        app.update();

        let profile = app.world().get::<InferenceProfile>(entity).unwrap();
        assert!(profile.growth_bias > initial_growth, "cold should increase growth_bias: {}", profile.growth_bias);
    }

    #[test]
    fn bergmann_hot_no_pressure() {
        let mut app = minimal_app_with_field();
        app.add_systems(Update, morphology_environmental_pressure_system);

        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            if let Some(cell) = grid.cell_xy_mut(0, 0) {
                cell.temperature = 500.0;
            }
        }

        let origin = DEFAULT_GRID_ORIGIN;
        let half_cell = FIELD_CELL_SIZE * 0.5;
        let pos = Vec3::new(origin.x + half_cell, origin.y + half_cell, 0.0);

        let initial_growth = 0.5;
        let entity = app.world_mut().spawn((
            Transform::from_translation(pos),
            BaseEnergy::new(100.0),
            SpatialVolume::new(1.0),
            InferenceProfile::new(initial_growth, 0.5, 0.5, 0.5),
        )).id();

        app.update();

        let profile = app.world().get::<InferenceProfile>(entity).unwrap();
        assert!((profile.growth_bias - initial_growth).abs() < 0.01, "hot should not change growth_bias: {}", profile.growth_bias);
    }

    #[test]
    fn allen_cold_reduces_branching() {
        let mut app = minimal_app_with_field();
        app.add_systems(Update, morphology_environmental_pressure_system);

        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            if let Some(cell) = grid.cell_xy_mut(0, 0) {
                cell.temperature = 50.0;
            }
        }

        let origin = DEFAULT_GRID_ORIGIN;
        let half_cell = FIELD_CELL_SIZE * 0.5;
        let pos = Vec3::new(origin.x + half_cell, origin.y + half_cell, 0.0);

        let initial_branching = 0.5;
        let entity = app.world_mut().spawn((
            Transform::from_translation(pos),
            BaseEnergy::new(100.0),
            SpatialVolume::new(1.0),
            InferenceProfile::new(0.5, 0.5, initial_branching, 0.5),
        )).id();

        app.update();

        let profile = app.world().get::<InferenceProfile>(entity).unwrap();
        assert!(profile.branching_bias < initial_branching, "cold should reduce branching_bias: {}", profile.branching_bias);
    }

    #[test]
    fn wolff_running_entity_increases_bond() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, morphology_use_adaptation_system);

        let initial_bond = 100.0;
        let entity = app.world_mut().spawn((
            FlowVector::new(Vec2::new(5.0, 0.0), 0.1),
            MatterCoherence::new(crate::layers::MatterState::Solid, initial_bond, 0.5),
        )).id();

        app.update();

        let coherence = app.world().get::<MatterCoherence>(entity).unwrap();
        assert!(coherence.bond_energy_eb() > initial_bond, "running should increase bond: {}", coherence.bond_energy_eb());
    }

    #[test]
    fn wolff_sedentary_decreases_bond() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, morphology_use_adaptation_system);

        let initial_bond = 100.0;
        let entity = app.world_mut().spawn((
            FlowVector::new(Vec2::ZERO, 0.1),
            MatterCoherence::new(crate::layers::MatterState::Solid, initial_bond, 0.5),
        )).id();

        app.update();

        let coherence = app.world().get::<MatterCoherence>(entity).unwrap();
        assert!(coherence.bond_energy_eb() < initial_bond, "sedentary should decrease bond: {}", coherence.bond_energy_eb());
    }

    #[test]
    fn organ_rebalance_triggered_on_profile_change() {
        let mut app = minimal_app_with_field();

        // S1 changes profile, S3 detects and inserts marker
        app.add_systems(Update, (
            morphology_environmental_pressure_system,
            morphology_organ_rebalance_system,
        ).chain());

        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            if let Some(cell) = grid.cell_xy_mut(0, 0) {
                cell.temperature = 50.0;
            }
        }

        let origin = DEFAULT_GRID_ORIGIN;
        let half_cell = FIELD_CELL_SIZE * 0.5;
        let pos = Vec3::new(origin.x + half_cell, origin.y + half_cell, 0.0);

        let entity = app.world_mut().spawn((
            Transform::from_translation(pos),
            BaseEnergy::new(100.0),
            SpatialVolume::new(1.0),
            InferenceProfile::new(0.5, 0.5, 0.5, 0.5),
        )).id();

        // Run multiple updates so the profile accumulates enough delta for the threshold
        for _ in 0..20 {
            app.update();
        }

        let has_rebuild = app.world().get::<PendingMorphRebuild>(entity).is_some();
        assert!(has_rebuild, "organ rebalance should be triggered after profile changes accumulate");
    }
}
