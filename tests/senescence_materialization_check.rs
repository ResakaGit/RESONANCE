//! Verifies that materialization_incremental_system inserts SenescenceProfile.

use bevy::prelude::*;
use resonance::layers::{BaseEnergy, MatterState, SenescenceProfile};
use resonance::runtime_platform::simulation_tick::SimulationClock;
use resonance::worldgen::{EnergyFieldGrid, Materialized};
use resonance::worldgen::systems::materialization_delta::materialization_incremental_system;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;

#[test]
fn incremental_materialization_inserts_senescence_profile() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimWorldTransformParams>();
    app.insert_resource(SimulationClock { tick_id: 42 });

    // Create a small grid with one cell that has energy.
    let mut grid = EnergyFieldGrid::new(4, 4, 2.0, bevy::math::Vec2::ZERO);
    if let Some(cell) = grid.cell_xy_mut(1, 1) {
        cell.accumulated_qe = 100.0;
        cell.dominant_frequency_hz = 85.0;
        cell.matter_state = MatterState::Solid;
    }
    grid.mark_cell_dirty(1, 1);
    app.insert_resource(grid);

    app.add_systems(Update, materialization_incremental_system);
    app.update();

    // The system should have spawned an entity for cell (1,1).
    let world = app.world_mut();
    let mut q = world.query::<(&Materialized, &BaseEnergy, Option<&SenescenceProfile>)>();
    let results: Vec<_> = q.iter(world).collect();

    assert!(!results.is_empty(), "should have spawned at least one materialized entity");

    let (mat, energy, sen) = results[0];
    assert_eq!(mat.cell_x, 1);
    assert_eq!(mat.cell_y, 1);
    assert!(energy.qe() > 0.0);
    assert!(sen.is_some(), "SenescenceProfile should be present on materialized entity");

    let profile = sen.unwrap();
    assert_eq!(profile.tick_birth, 42, "tick_birth should match SimulationClock.tick_id");
    assert_eq!(profile.max_viable_age, 5_000);
}
