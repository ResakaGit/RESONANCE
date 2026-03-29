//! Planetary formation: gravity, fusion, angular momentum.
//!
//! Three stateless systems that transform the energy field:
//! 1. Gravity: high-mass cells attract energy from neighbors (anti-diffusion).
//! 2. Fusion: plasma-density cells emit bonus energy (mass→energy).
//! 3. Angular momentum: radial gravity transfers deflect tangentially.
//!
//! All rates are tick-rate-independent via dt normalization.
//! Phase: ThermodynamicLayer, before day_night (gravity shapes the field first).

use bevy::prelude::*;

use crate::blueprint::equations::planetary_formation as pf;
use crate::worldgen::EnergyFieldGrid;

/// Reference tick rate for dt normalization.
const REFERENCE_HZ: f32 = 60.0;

/// Combined system: gravity + fusion + angular momentum in one grid pass.
/// Single double-buffered delta array for all three effects (conservation-safe).
pub fn planetary_formation_system(
    mut grid: ResMut<EnergyFieldGrid>,
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
) {
    let w = grid.width;
    let h = grid.height;
    let len = (w * h) as usize;
    if len == 0 { return; }

    let dt = fixed.as_ref().map(|f| f.delta_secs()).unwrap_or_else(|| time.delta_secs());
    let dt_ratio = dt * REFERENCE_HZ;

    let mut deltas = vec![0.0_f32; len];
    let mut any_change = false;

    // --- Pass 1: Gravity + angular momentum ---
    // Find center of mass for angular momentum direction.
    let mut com_x = 0.0_f64;
    let mut com_y = 0.0_f64;
    let mut total_mass = 0.0_f64;
    for y in 0..h {
        for x in 0..w {
            if let Some(cell) = grid.cell_xy(x, y) {
                let m = cell.accumulated_qe as f64;
                com_x += x as f64 * m;
                com_y += y as f64 * m;
                total_mass += m;
            }
        }
    }
    if total_mass > 0.0 {
        com_x /= total_mass;
        com_y /= total_mass;
    } else {
        com_x = w as f64 * 0.5;
        com_y = h as f64 * 0.5;
    }

    for y in 0..h {
        for x in 0..w {
            let Some(cell) = grid.cell_xy(x, y) else { continue };
            let source_qe = cell.accumulated_qe;
            if source_qe <= 0.0 { continue; }

            let src_idx = y as usize * w as usize + x as usize;
            let neighbors = grid.neighbors4(x, y);
            let n_count = neighbors.iter().flatten().count() as u32;

            for (ni, neighbor) in neighbors.iter().enumerate() {
                let Some((nx, ny)) = *neighbor else { continue };
                let neighbor_qe = grid.cell_xy(nx, ny)
                    .map(|c| c.accumulated_qe).unwrap_or(0.0);

                // Gravitational transfer: attract toward higher mass.
                let radial = pf::gravitational_transfer(source_qe, neighbor_qe, n_count) * dt_ratio;
                if radial <= 0.0 { continue; }

                let dst_idx = ny as usize * w as usize + nx as usize;

                // Angular momentum: deflect fraction to perpendicular neighbor.
                let tangential = pf::tangential_deflection(radial);
                let radial_net = radial - tangential;

                // Radial component: neighbor → source (gravity pull).
                deltas[dst_idx] -= radial_net;
                deltas[src_idx] += radial_net;

                // Tangential component: neighbor → perpendicular neighbor.
                // Perpendicular = next neighbor in the ring (90° rotation).
                let perp_idx = (ni + 1) % 4;
                if let Some((px, py)) = neighbors[perp_idx] {
                    let perp_cell_idx = py as usize * w as usize + px as usize;
                    deltas[dst_idx] -= tangential;
                    deltas[perp_cell_idx] += tangential;
                }

                any_change = true;
            }
        }
    }

    // --- Pass 2: Fusion ---
    for y in 0..h {
        for x in 0..w {
            if let Some(cell) = grid.cell_xy(x, y) {
                let bonus = pf::fusion_release(cell.accumulated_qe) * dt_ratio;
                if bonus > 0.01 {
                    let idx = y as usize * w as usize + x as usize;
                    deltas[idx] += bonus;
                    any_change = true;
                }
            }
        }
    }

    if !any_change { return; }

    // --- Apply deltas ---
    for y in 0..h {
        for x in 0..w {
            let idx = y as usize * w as usize + x as usize;
            let d = deltas[idx];
            if d.abs() < 1e-6 { continue; }
            if let Some(cell) = grid.cell_xy_mut(x, y) {
                cell.accumulated_qe = (cell.accumulated_qe + d).max(0.0);
            }
            grid.mark_cell_dirty(x, y);
        }
    }
}
