//! Incremental materialization: only dirty cells are processed per tick.

use bevy::prelude::*;

use crate::layers::{BaseEnergy, MatterCoherence, OscillatorySignature, SenescenceProfile, SpatialVolume};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::worldgen::constants::{
    MATERIALIZED_COLLIDER_RADIUS_FACTOR, MATERIALIZED_MIN_COLLIDER_RADIUS,
    MATERIALIZED_SPAWN_BOND_ENERGY, MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
};
use crate::worldgen::{EnergyFieldGrid, Materialized};

/// Max entities spawned or despawned per tick (prevents frame drops).
const DELTA_SPAWN_BUDGET: usize = 50;

/// Processes only dirty cells from [`EnergyFieldGrid`] per tick.
///
/// For each dirty cell: spawns an entity if the cell should be materialized and has none;
/// despawns if the cell no longer qualifies and an entity exists.
/// Budget-limited to [`DELTA_SPAWN_BUDGET`] changes per tick.
pub fn materialization_incremental_system(
    mut commands: Commands,
    mut grid: ResMut<EnergyFieldGrid>,
    layout: Res<SimWorldTransformParams>,
    materialized_query: Query<(), With<Materialized>>,
    clock: Res<SimulationClock>,
) {
    if !grid.any_dirty() {
        return;
    }

    let cell_size = grid.cell_size;
    let width = grid.width;

    // Collect dirty indices within budget. We cannot borrow grid mutably for the iterator
    // and immutably for cell reads at the same time, so we gather indices first.
    let dirty: Vec<usize> = grid
        .drain_dirty_budgeted(DELTA_SPAWN_BUDGET)
        .collect();

    for idx in dirty {
        let x = (idx % width as usize) as u32;
        let y = (idx / width as usize) as u32;

        let world_pos = match grid.world_pos(x, y) {
            Some(p) => p,
            None => continue,
        };

        let should_materialize = {
            let Some(cell) = grid.cell_xy(x, y) else { continue };
            cell.accumulated_qe > 0.0
        };

        let existing = grid.cell_xy(x, y).and_then(|c| c.materialized_entity);

        match (should_materialize, existing) {
            (true, None) => {
                let Some(cell) = grid.cell_xy(x, y) else { continue };
                let id = commands
                    .spawn((
                        Materialized {
                            cell_x: x as i32,
                            cell_y: y as i32,
                            archetype: crate::worldgen::WorldArchetype::TerraSolid,
                        },
                        BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                        OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                        SpatialVolume::new(
                            (cell_size * MATERIALIZED_COLLIDER_RADIUS_FACTOR)
                                .max(MATERIALIZED_MIN_COLLIDER_RADIUS),
                        ),
                        MatterCoherence::new(
                            cell.matter_state,
                            MATERIALIZED_SPAWN_BOND_ENERGY,
                            MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                        ),
                        layout.materialized_tile_transform(world_pos),
                        GlobalTransform::default(),
                        Sprite::default(),
                        SenescenceProfile {
                            tick_birth: clock.tick_id,
                            senescence_coeff: crate::blueprint::constants::senescence_coeff_materialized(),
                            max_viable_age: crate::blueprint::constants::senescence_max_age_materialized(),
                            strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
                        },
                    ))
                    .id();
                let Some(cell_mut) = grid.cell_xy_mut(x, y) else { continue };
                cell_mut.materialized_entity = Some(id);
            }
            (false, Some(e)) => {
                if materialized_query.get(e).is_ok() {
                    commands.entity(e).despawn();
                }
                let Some(cell_mut) = grid.cell_xy_mut(x, y) else { continue };
                cell_mut.materialized_entity = None;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::worldgen::field_grid::EnergyFieldGrid;

    #[test]
    fn drain_dirty_budgeted_yields_marked_indices_and_clears_them() {
        let mut grid = EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO);
        grid.mark_cell_dirty(1, 0);
        grid.mark_cell_dirty(3, 0);
        grid.mark_cell_dirty(5, 0);

        let drained: Vec<usize> = grid.drain_dirty_budgeted(10).collect();
        assert_eq!(drained.len(), 3);
        assert!(!grid.any_dirty());
    }

    #[test]
    fn drain_dirty_budgeted_respects_budget() {
        let mut grid = EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO);
        for x in 0..8u32 {
            grid.mark_cell_dirty(x, 0);
        }
        let drained: Vec<usize> = grid.drain_dirty_budgeted(3).collect();
        assert_eq!(drained.len(), 3);
        // Remaining 5 cells should still be dirty.
        assert!(grid.any_dirty());
    }

    #[test]
    fn drain_dirty_budgeted_empty_grid_yields_nothing() {
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
        let drained: Vec<usize> = grid.drain_dirty_budgeted(50).collect();
        assert!(drained.is_empty());
    }
}
