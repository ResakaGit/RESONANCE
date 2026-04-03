use bevy::prelude::*;

use crate::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolParentLink};

// ── EC-8B: Pool Archetypes ────────────────────────────────────────────────────

/// Ambient pool: zone/region with distributable energy.
pub fn spawn_environment_pool(
    commands: &mut Commands,
    pool: f32,
    capacity: f32,
    intake_rate: f32,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            EnergyPool::new(pool, capacity, intake_rate, 0.001),
            Transform::from_translation(position),
        ))
        .id()
}

/// Competitive organism: extracts energy from a parent pool.
pub fn spawn_competitor(
    commands: &mut Commands,
    parent: Entity,
    extraction_type: ExtractionType,
    primary_param: f32,
    initial_qe: f32,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            BaseEnergy::new(initial_qe),
            PoolParentLink::new(parent, extraction_type, primary_param),
            Transform::from_translation(position),
        ))
        .id()
}

/// Intermediate pool (Matryoshka): pool that also extracts from a parent.
pub fn spawn_sub_pool(
    commands: &mut Commands,
    parent: Entity,
    extraction_type: ExtractionType,
    fitness: f32,
    pool_capacity: f32,
    intake_rate: f32,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            EnergyPool::new(pool_capacity * 0.5, pool_capacity, intake_rate, 0.001),
            PoolParentLink::new(parent, extraction_type, fitness),
            Transform::from_translation(position),
        ))
        .id()
}

#[cfg(test)]
mod pool_archetype_tests {
    use super::{spawn_competitor, spawn_environment_pool, spawn_sub_pool};
    use crate::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolParentLink};
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn spawn_environment_pool_has_energy_pool_and_transform() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_environment_pool(&mut commands, 5000.0, 10000.0, 100.0, Vec3::ZERO);
        drop(commands);
        app.update();
        let e = app.world().entity(entity);
        assert!(e.contains::<EnergyPool>(), "missing EnergyPool");
        assert!(e.contains::<Transform>(), "missing Transform");
        assert!(!e.contains::<BaseEnergy>(), "should not have BaseEnergy");
    }

    #[test]
    fn spawn_competitor_has_base_energy_and_pool_parent_link() {
        let mut app = test_app();
        let parent = app
            .world_mut()
            .spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01))
            .id();
        let mut commands = app.world_mut().commands();
        let entity = spawn_competitor(
            &mut commands,
            parent,
            ExtractionType::Competitive,
            0.5,
            100.0,
            Vec3::ZERO,
        );
        drop(commands);
        app.update();
        let e = app.world().entity(entity);
        assert!(e.contains::<BaseEnergy>(), "missing BaseEnergy");
        assert!(e.contains::<PoolParentLink>(), "missing PoolParentLink");
        assert!(e.contains::<Transform>(), "missing Transform");
    }

    #[test]
    fn spawn_sub_pool_has_energy_pool_and_pool_parent_link() {
        let mut app = test_app();
        let root = app
            .world_mut()
            .spawn(EnergyPool::new(5000.0, 10000.0, 200.0, 0.001))
            .id();
        let mut commands = app.world_mut().commands();
        let entity = spawn_sub_pool(
            &mut commands,
            root,
            ExtractionType::Competitive,
            0.5,
            2000.0,
            40.0,
            Vec3::ZERO,
        );
        drop(commands);
        app.update();
        let e = app.world().entity(entity);
        assert!(e.contains::<EnergyPool>(), "missing EnergyPool");
        assert!(e.contains::<PoolParentLink>(), "missing PoolParentLink");
    }

    #[test]
    fn spawn_environment_pool_values_correct() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_environment_pool(
            &mut commands,
            5000.0,
            10000.0,
            100.0,
            Vec3::new(1.0, 2.0, 0.0),
        );
        drop(commands);
        app.update();
        let pool = app.world().entity(entity).get::<EnergyPool>().unwrap();
        assert!((pool.pool() - 5000.0).abs() < 1e-3);
        assert!((pool.capacity() - 10000.0).abs() < 1e-3);
        assert!((pool.intake_rate() - 100.0).abs() < 1e-3);
    }
}
