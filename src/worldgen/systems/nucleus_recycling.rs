//! Nucleus recycling: dead matter nutrients → new energy sources.
//! When nutrient density exceeds threshold in a cell without a nucleus nearby,
//! a new nucleus spawns there. This closes the energy cycle:
//! nucleus → field → entities → death → nutrients → nucleus.
//!
//! Conservation: the recycled nucleus's reservoir is drained FROM the grid.
//! No energy is created — it is concentrated from a zone into a point source.
//! Second Law (Axiom 4): conversion efficiency < 1.0.
//!
//! Phase: [`Phase::MorphologicalLayer`], after abiogenesis.

use bevy::prelude::*;

use crate::blueprint::constants::{
    nucleus_recycling_nutrient_threshold, NUCLEUS_RECYCLING_SCAN_BUDGET,
};
use crate::blueprint::equations::derived_thresholds as dt;
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
/// nucleus there, draining energy from the zone to fuel it. Max 1 nucleus per tick.
pub fn nucleus_recycling_system(
    mut commands: Commands,
    nutrient_grid: Option<ResMut<NutrientFieldGrid>>,
    mut energy_grid: Option<ResMut<EnergyFieldGrid>>,
    layout: Res<SimWorldTransformParams>,
    existing_nuclei: Query<&Transform, With<EnergyNucleus>>,
    mut cursor: ResMut<NucleusRecyclingCursor>,
) {
    let Some(mut nutrients) = nutrient_grid else { return };
    let Some(ref mut grid) = energy_grid else { return };

    let total_cells = (nutrients.width * nutrients.height) as usize;
    if total_cells == 0 {
        return;
    }

    let scan = total_cells.min(NUCLEUS_RECYCLING_SCAN_BUDGET);
    let xz = layout.use_xz_ground;
    let nutrient_threshold = nucleus_recycling_nutrient_threshold();

    for _ in 0..scan {
        let idx = cursor.next_cell % total_cells;
        cursor.next_cell = (cursor.next_cell + 1) % total_cells.max(1);

        let w = nutrients.width;
        let cx = (idx % w as usize) as u32;
        let cy = (idx / w as usize) as u32;

        let Some(cell) = nutrients.cell_xy(cx, cy) else { continue };
        let avg_nutrient = (cell.carbon_norm + cell.nitrogen_norm
            + cell.phosphorus_norm + cell.water_norm) * 0.25;

        if avg_nutrient < nutrient_threshold {
            continue;
        }

        let Some(world_pos) = grid.world_pos(cx, cy) else { continue };

        // Estimate propagation radius for proximity check (before draining).
        let center_qe = grid.cell_xy(cx, cy).map(|c| c.accumulated_qe).unwrap_or(0.0);
        let est_radius = dt::recycled_propagation_radius(center_qe * 4.0);

        let too_close = existing_nuclei.iter().any(|tr| {
            let npos = if xz {
                crate::math_types::Vec2::new(tr.translation.x, tr.translation.z)
            } else {
                crate::math_types::Vec2::new(tr.translation.x, tr.translation.y)
            };
            npos.distance(world_pos) < est_radius * 2.0
        });
        if too_close {
            continue;
        }

        // Determine frequency from dominant field frequency at this cell.
        let freq = grid.cell_xy(cx, cy)
            .map(|c| c.dominant_frequency_hz)
            .filter(|f| *f > 0.0)
            .unwrap_or(crate::blueprint::constants::ABIOGENESIS_FLORA_PEAK_HZ);

        // Drain nutrients proportional to dissipation ratios (Axiom 4).
        let mineral_ret = dt::nutrient_retention_mineral();
        let water_ret = dt::nutrient_retention_water();
        if let Some(ncell) = nutrients.cell_xy_mut(cx, cy) {
            ncell.carbon_norm = (ncell.carbon_norm * mineral_ret).max(0.0);
            ncell.nitrogen_norm = (ncell.nitrogen_norm * mineral_ret).max(0.0);
            ncell.phosphorus_norm = (ncell.phosphorus_norm * mineral_ret).max(0.0);
            ncell.water_norm = (ncell.water_norm * water_ret).max(0.0);
        }

        // Harvest energy from grid zone (conservation: energy moves, not created).
        let harvest_r = dt::recycling_harvest_radius_cells() as i32;
        let drain_frac = dt::recycling_drain_fraction();
        let gw = grid.width as i32;
        let gh = grid.height as i32;
        let mut harvested_qe = 0.0_f32;

        for dy in -harvest_r..=harvest_r {
            for dx in -harvest_r..=harvest_r {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= gw || ny >= gh { continue; }
                if let Some(cell) = grid.cell_xy_mut(nx as u32, ny as u32) {
                    let drained = cell.accumulated_qe * drain_frac;
                    if drained > 0.01 {
                        cell.accumulated_qe -= drained;
                        harvested_qe += drained;
                    }
                }
                grid.mark_cell_dirty(nx as u32, ny as u32);
            }
        }

        // Apply conversion efficiency (Axiom 4: Second Law loss).
        let reservoir = harvested_qe * dt::recycling_conversion_efficiency();
        if reservoir < dt::self_sustaining_qe_min() {
            continue; // Too little energy to form a viable nucleus.
        }

        // Derive emission and radius from reservoir (axiom-derived scaling).
        let emission = dt::recycled_emission_rate(reservoir);
        let radius = dt::recycled_propagation_radius(reservoir);

        let translation = if xz {
            Vec3::new(world_pos.x, 0.0, world_pos.y)
        } else {
            Vec3::new(world_pos.x, world_pos.y, 0.0)
        };
        commands.spawn((
            EnergyNucleus::new(freq, emission, radius, PropagationDecay::InverseLinear),
            NucleusReservoir { qe: reservoir },
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
    fn recycling_threshold_derived_and_valid() {
        let t = nucleus_recycling_nutrient_threshold();
        assert!(t > 0.0 && t <= 1.0, "threshold={t}");
    }

    #[test]
    fn recycled_nucleus_scales_with_energy() {
        let small = dt::recycled_emission_rate(50.0);
        let large = dt::recycled_emission_rate(500.0);
        assert!(large > small);
        assert!(small > 0.0);
    }

    #[test]
    fn conversion_efficiency_subunit() {
        let e = dt::recycling_conversion_efficiency();
        assert!(e > 0.0 && e < 1.0);
    }

    #[test]
    fn minimum_reservoir_check() {
        assert!(dt::self_sustaining_qe_min() > 0.0);
    }
}
