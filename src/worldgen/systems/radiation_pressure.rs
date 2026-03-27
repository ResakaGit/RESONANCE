//! Radiation pressure: high-density cells push excess energy outward.
//! Transfer modulated by frequency alignment (Axiom 8): same-frequency cells
//! share easily, cross-frequency cells resist mixing. Biomes stay distinct.
//!
//! Stateless system operating on EnergyFieldGrid. No entity queries.
//! Phase: ThermodynamicLayer, after dissipation, before materialization.

use bevy::prelude::*;

use crate::blueprint::constants::nucleus_lifecycle::{
    radiation_pressure_threshold_qe, radiation_pressure_transfer_rate,
};
use crate::blueprint::equations::{
    radiation_pressure_transfer_coherent, PRESSURE_FREQUENCY_BANDWIDTH,
};
use crate::worldgen::EnergyFieldGrid;

/// Applies frequency-modulated outward pressure on cells exceeding threshold.
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
            if cell.accumulated_qe <= radiation_pressure_threshold_qe() {
                continue;
            }
            let source_freq = cell.dominant_frequency_hz;
            let source_qe = cell.accumulated_qe;

            let neighbors = grid.neighbors4(x, y);
            let n_count = neighbors.iter().flatten().count() as u32;

            let src_idx = y as usize * w as usize + x as usize;
            for neighbor in neighbors.iter().flatten() {
                let (nx, ny) = *neighbor;
                let target_freq = grid.cell_xy(nx, ny)
                    .map(|c| c.dominant_frequency_hz)
                    .unwrap_or(0.0);
                let transfer = radiation_pressure_transfer_coherent(
                    source_qe,
                    target_freq,
                    source_freq,
                    radiation_pressure_threshold_qe(),
                    radiation_pressure_transfer_rate(),
                    PRESSURE_FREQUENCY_BANDWIDTH,
                    n_count,
                );
                if transfer > 0.0 {
                    let dst_idx = ny as usize * w as usize + nx as usize;
                    deltas[src_idx] -= transfer;
                    deltas[dst_idx] += transfer;
                    any_change = true;
                }
            }
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
