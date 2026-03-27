//! Radiation pressure: high-density cells push excess energy outward.
//! Stateless system operating on EnergyFieldGrid. No entity queries.
//!
//! Phase: ThermodynamicLayer, after dissipation, before materialization.

use bevy::prelude::*;

use crate::blueprint::constants::{
    RADIATION_PRESSURE_THRESHOLD_QE, RADIATION_PRESSURE_TRANSFER_RATE,
};
use crate::blueprint::equations::radiation_pressure_transfer;
use crate::worldgen::EnergyFieldGrid;

/// Applies non-linear outward pressure on cells exceeding the energy threshold.
/// Double-buffered: accumulates deltas then applies (order-independent, deterministic).
pub fn radiation_pressure_system(mut grid: ResMut<EnergyFieldGrid>) {
    let w = grid.width;
    let h = grid.height;
    let len = (w * h) as usize;
    if len == 0 {
        return;
    }

    let mut deltas = vec![0.0_f32; len];
    let mut any_change = false;

    for y in 0..h {
        for x in 0..w {
            let Some(cell) = grid.cell_xy(x, y) else { continue };
            if cell.accumulated_qe <= RADIATION_PRESSURE_THRESHOLD_QE {
                continue;
            }

            let neighbors = grid.neighbors4(x, y);
            let n_count = neighbors.iter().flatten().count() as u32;
            let transfer_per_neighbor = radiation_pressure_transfer(
                cell.accumulated_qe,
                RADIATION_PRESSURE_THRESHOLD_QE,
                RADIATION_PRESSURE_TRANSFER_RATE,
                n_count,
            );
            if transfer_per_neighbor <= 0.0 {
                continue;
            }

            let src_idx = y as usize * w as usize + x as usize;
            for neighbor in neighbors.iter().flatten() {
                let dst_idx = neighbor.1 as usize * w as usize + neighbor.0 as usize;
                deltas[src_idx] -= transfer_per_neighbor;
                deltas[dst_idx] += transfer_per_neighbor;
            }
            any_change = true;
        }
    }

    if !any_change {
        return;
    }

    for y in 0..h {
        for x in 0..w {
            let idx = y as usize * w as usize + x as usize;
            let d = deltas[idx];
            if d.abs() < 1e-6 {
                continue;
            }
            if let Some(cell) = grid.cell_xy_mut(x, y) {
                cell.accumulated_qe = (cell.accumulated_qe + d).max(0.0);
            }
            grid.mark_cell_dirty(x, y);
        }
    }
}
