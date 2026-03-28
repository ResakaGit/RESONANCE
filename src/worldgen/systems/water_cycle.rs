//! Water cycle: evaporation from hot cells, precipitation on cool cells.
//!
//! Hot cells (high qe) lose water_norm → cool cells gain it.
//! Conservation: total water is preserved (transfer, not creation).
//! Rate derived from DISSIPATION_LIQUID (liquid→gas phase transition, Axiom 4).
//!
//! Phase: ThermodynamicLayer, after day_night (temperature drives the cycle).

use bevy::prelude::*;

use crate::blueprint::equations::derived_thresholds as dt;
use crate::worldgen::{EnergyFieldGrid, NutrientFieldGrid};

/// Minimum water_norm to trigger evaporation (below = negligible moisture).
const WATER_MIN_THRESHOLD: f32 = dt::DISSIPATION_LIQUID; // 0.02
/// Minimum delta to apply (avoids float churn on negligible changes).
const DELTA_EPSILON: f32 = 1e-5;

/// Transfers water from hot cells to their coolest neighbor.
/// Conservation: water moved, not created/destroyed (Axiom 5).
pub fn water_cycle_system(
    energy_grid: Option<Res<EnergyFieldGrid>>,
    mut nutrient_grid: Option<ResMut<NutrientFieldGrid>>,
) {
    let Some(grid) = energy_grid else { return };
    let Some(ref mut nutrients) = nutrient_grid else { return };
    if grid.width != nutrients.width || grid.height != nutrients.height { return; }

    let w = grid.width as usize;
    let h = grid.height as usize;
    let total = w * h;
    let evap_rate = dt::DISSIPATION_LIQUID;

    // Double-buffered water deltas (conservation-safe).
    let mut deltas = vec![0.0_f32; total];

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let cell_qe = grid.cell_xy(x as u32, y as u32)
                .map(|c| c.accumulated_qe).unwrap_or(0.0);
            let cell_water = nutrients.cell_xy(x as u32, y as u32)
                .map(|c| c.water_norm).unwrap_or(0.0);

            if cell_water < WATER_MIN_THRESHOLD || cell_qe < 1.0 { continue; }

            // Evaporation amount: proportional to energy and current water.
            let evap = cell_water * evap_rate * (cell_qe / dt::DENSITY_SCALE).min(1.0);
            if evap < DELTA_EPSILON { continue; }

            // Find coolest neighbor (precipitation target).
            let neighbors = grid.neighbors4(x as u32, y as u32);
            let mut coolest_idx = idx;
            let mut coolest_qe = cell_qe;
            for n in neighbors.iter().flatten() {
                let nidx = n.1 as usize * w + n.0 as usize;
                let nqe = grid.cell_xy(n.0, n.1)
                    .map(|c| c.accumulated_qe).unwrap_or(f32::MAX);
                if nqe < coolest_qe {
                    coolest_qe = nqe;
                    coolest_idx = nidx;
                }
            }

            // Only transfer if there's a temperature gradient.
            if coolest_idx == idx { continue; }

            deltas[idx] -= evap;
            deltas[coolest_idx] += evap;
        }
    }

    // Apply deltas.
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if deltas[idx].abs() < DELTA_EPSILON { continue; }
            if let Some(cell) = nutrients.cell_xy_mut(x as u32, y as u32) {
                cell.water_norm = (cell.water_norm + deltas[idx]).clamp(0.0, 1.0);
            }
        }
    }
}
