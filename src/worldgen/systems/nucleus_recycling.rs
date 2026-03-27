//! Nucleus recycling: dead matter nutrients → new energy sources.
//! When nutrient density exceeds threshold in a cell without a nucleus nearby,
//! a new nucleus spawns there. This closes the energy cycle:
//! nucleus → field → entities → death → nutrients → nucleus.
//!
//! Phase: [`Phase::MorphologicalLayer`], after abiogenesis.

use bevy::prelude::*;

use crate::blueprint::constants::{
    NUCLEUS_RECYCLING_EMISSION_RATE, NUCLEUS_RECYCLING_NUTRIENT_THRESHOLD,
    NUCLEUS_RECYCLING_RADIUS, NUCLEUS_RECYCLING_RESERVOIR_QE, NUCLEUS_RECYCLING_SCAN_BUDGET,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::worldgen::{
    EnergyFieldGrid, EnergyNucleus, NucleusReservoir, NutrientFieldGrid, PropagationDecay,
};

/// Round-robin cursor for nutrient grid scanning.
#[derive(Resource, Debug, Default)]
pub struct NucleusRecyclingCursor {
    pub next_cell: usize,
}

/// Scans nutrient grid for cells with high nutrient density. Spawns a finite
/// nucleus there, draining nutrients to fuel it. Max 1 nucleus per tick.
pub fn nucleus_recycling_system(
    mut commands: Commands,
    nutrient_grid: Option<ResMut<NutrientFieldGrid>>,
    energy_grid: Option<Res<EnergyFieldGrid>>,
    layout: Res<SimWorldTransformParams>,
    existing_nuclei: Query<&Transform, With<EnergyNucleus>>,
    mut cursor: ResMut<NucleusRecyclingCursor>,
) {
    let Some(mut nutrients) = nutrient_grid else { return };
    let Some(grid) = energy_grid else { return };

    let total_cells = (nutrients.width * nutrients.height) as usize;
    if total_cells == 0 {
        return;
    }

    let scan = total_cells.min(NUCLEUS_RECYCLING_SCAN_BUDGET);
    let xz = layout.use_xz_ground;

    for _ in 0..scan {
        let idx = cursor.next_cell % total_cells;
        cursor.next_cell = (cursor.next_cell + 1) % total_cells.max(1);

        let w = nutrients.width;
        let cx = (idx % w as usize) as u32;
        let cy = (idx / w as usize) as u32;

        let Some(cell) = nutrients.cell_xy(cx, cy) else { continue };
        let avg_nutrient = (cell.carbon_norm + cell.nitrogen_norm
            + cell.phosphorus_norm + cell.water_norm) * 0.25;

        if avg_nutrient < NUCLEUS_RECYCLING_NUTRIENT_THRESHOLD {
            continue;
        }

        // Don't spawn if existing nucleus is already nearby.
        let Some(world_pos) = grid.world_pos(cx, cy) else { continue };
        let too_close = existing_nuclei.iter().any(|tr| {
            let npos = if xz {
                crate::math_types::Vec2::new(tr.translation.x, tr.translation.z)
            } else {
                    crate::math_types::Vec2::new(tr.translation.x, tr.translation.y)
            };
            npos.distance(world_pos) < NUCLEUS_RECYCLING_RADIUS * 2.0
        });
        if too_close {
            continue;
        }

        // Determine frequency from dominant field frequency at this cell.
        let freq = grid.cell_xy(cx, cy)
            .map(|c| c.dominant_frequency_hz)
            .filter(|f| *f > 0.0)
            .unwrap_or(85.0);

        // Drain nutrients to fuel the new nucleus.
        if let Some(ncell) = nutrients.cell_xy_mut(cx, cy) {
            ncell.carbon_norm = (ncell.carbon_norm * 0.3).max(0.0);
            ncell.nitrogen_norm = (ncell.nitrogen_norm * 0.3).max(0.0);
            ncell.phosphorus_norm = (ncell.phosphorus_norm * 0.3).max(0.0);
            ncell.water_norm = (ncell.water_norm * 0.5).max(0.0);
        }

        // Spawn recycled nucleus with finite reservoir.
        let translation = if xz {
            Vec3::new(world_pos.x, 0.0, world_pos.y)
        } else {
            Vec3::new(world_pos.x, world_pos.y, 0.0)
        };
        commands.spawn((
            EnergyNucleus::new(freq, NUCLEUS_RECYCLING_EMISSION_RATE, NUCLEUS_RECYCLING_RADIUS, PropagationDecay::InverseLinear),
            NucleusReservoir { qe: NUCLEUS_RECYCLING_RESERVOIR_QE },
            Transform::from_translation(translation),
            GlobalTransform::default(),
        ));

        // Max 1 recycled nucleus per tick.
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recycling_constants_valid() {
        assert!(NUCLEUS_RECYCLING_NUTRIENT_THRESHOLD > 0.0);
        assert!(NUCLEUS_RECYCLING_NUTRIENT_THRESHOLD <= 1.0);
        assert!(NUCLEUS_RECYCLING_EMISSION_RATE > 0.0);
        assert!(NUCLEUS_RECYCLING_RESERVOIR_QE > 0.0);
        assert!(NUCLEUS_RECYCLING_RADIUS > 0.0);
        assert!(NUCLEUS_RECYCLING_SCAN_BUDGET > 0);
    }
}
