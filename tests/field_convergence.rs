//! Worldgen field convergence test.
//! Verifies that diffusion on a small grid converges to steady state
//! within bounded iterations (no infinite loops, no NaN, energy conserved).

use bevy::math::Vec2;
use resonance::blueprint::equations::{DIFFUSION_CONDUCTIVITY_DEFAULT, diffusion_delta};
use resonance::worldgen::EnergyFieldGrid;

/// Manual diffusion pass on a grid (pure, no Bevy).
/// Returns the number of cells whose qe changed by more than `epsilon`.
fn diffuse_one_pass(grid: &mut EnergyFieldGrid, w: u32, h: u32, epsilon: f32) -> usize {
    let k = DIFFUSION_CONDUCTIVITY_DEFAULT;
    let dt = 1.0_f32;

    // Collect deltas (order-independent double-buffer).
    let len = (w * h) as usize;
    let mut deltas = vec![0.0_f32; len];

    for y in 0..h {
        for x in 0..w {
            let Some(source_qe) = grid.cell_xy(x, y).map(|c| c.accumulated_qe) else {
                continue;
            };
            let neighbors: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            for (dx, dy) in neighbors {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let (nx, ny) = (nx as u32, ny as u32);
                let Some(target_qe) = grid.cell_xy(nx, ny).map(|c| c.accumulated_qe) else {
                    continue;
                };
                let delta = diffusion_delta(source_qe, target_qe, k, dt);
                let src_idx = (y * w + x) as usize;
                let dst_idx = (ny * w + nx) as usize;
                deltas[src_idx] -= delta;
                deltas[dst_idx] += delta;
            }
        }
    }

    // Apply deltas and count changed cells.
    let mut changed = 0;
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if deltas[idx].abs() > epsilon {
                if let Some(cell) = grid.cell_xy_mut(x, y) {
                    cell.accumulated_qe = (cell.accumulated_qe + deltas[idx]).max(0.0);
                    changed += 1;
                }
            }
        }
    }
    changed
}

#[test]
fn diffusion_converges_within_bounded_iterations() {
    let w = 8_u32;
    let h = 8_u32;
    let mut grid = EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO);

    // Seed center cell with high energy.
    if let Some(cell) = grid.cell_xy_mut(4, 4) {
        cell.accumulated_qe = 1000.0;
    }

    let max_iterations = 500;
    let convergence_epsilon = 1e-4;

    let total_qe_before = grid.total_qe();

    let mut converged_at = None;
    for i in 0..max_iterations {
        let changed = diffuse_one_pass(&mut grid, w, h, convergence_epsilon);
        if changed == 0 {
            converged_at = Some(i);
            break;
        }
    }

    assert!(
        converged_at.is_some(),
        "diffusion did not converge within {max_iterations} iterations"
    );

    // Energy conservation: total qe should be preserved (within float tolerance).
    let total_qe_after = grid.total_qe();
    assert!(
        (total_qe_before - total_qe_after).abs() < 1.0,
        "energy not conserved: before={total_qe_before} after={total_qe_after}"
    );

    // No NaN or negative values.
    for y in 0..h {
        for x in 0..w {
            let qe = grid.cell_xy(x, y).map(|c| c.accumulated_qe).unwrap_or(0.0);
            assert!(qe.is_finite(), "NaN at ({x},{y})");
            assert!(qe >= 0.0, "negative qe at ({x},{y}): {qe}");
        }
    }
}

#[test]
fn diffusion_early_gradient_decreases_from_source() {
    let w = 32_u32;
    let h = 32_u32;
    let mut grid = EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO);

    // Single source at center of a larger grid.
    if let Some(cell) = grid.cell_xy_mut(16, 16) {
        cell.accumulated_qe = 10000.0;
    }

    // Run only a few passes — before energy reaches boundaries, gradient is monotonic.
    for _ in 0..5 {
        diffuse_one_pass(&mut grid, w, h, 1e-6);
    }

    // Check that energy decreases along cardinal direction from center.
    let center_qe = grid.cell_xy(16, 16).map(|c| c.accumulated_qe).unwrap_or(0.0);
    let near_qe = grid.cell_xy(17, 16).map(|c| c.accumulated_qe).unwrap_or(0.0);
    let far_qe = grid.cell_xy(19, 16).map(|c| c.accumulated_qe).unwrap_or(0.0);
    let edge_qe = grid.cell_xy(22, 16).map(|c| c.accumulated_qe).unwrap_or(0.0);

    assert!(center_qe > 0.0, "center should have energy");
    assert!(center_qe >= near_qe, "center={center_qe} >= near={near_qe}");
    assert!(near_qe >= far_qe, "near={near_qe} >= far={far_qe}");
    assert!(far_qe >= edge_qe, "far={far_qe} >= edge={edge_qe}");
}

#[test]
fn diffusion_total_energy_conserved_across_passes() {
    let w = 8_u32;
    let h = 8_u32;
    let mut grid = EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO);

    // Seed two cells with different energy levels.
    if let Some(cell) = grid.cell_xy_mut(2, 2) {
        cell.accumulated_qe = 500.0;
    }
    if let Some(cell) = grid.cell_xy_mut(5, 5) {
        cell.accumulated_qe = 300.0;
    }

    let total_before = grid.total_qe();

    for _ in 0..50 {
        diffuse_one_pass(&mut grid, w, h, 1e-6);
    }

    let total_after = grid.total_qe();
    assert!(
        (total_before - total_after).abs() < 1.0,
        "energy not conserved: before={total_before} after={total_after}"
    );
}

#[test]
fn uniform_field_stays_unchanged() {
    let w = 8_u32;
    let h = 8_u32;
    let mut grid = EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO);

    // Uniform energy across all cells.
    for y in 0..h {
        for x in 0..w {
            if let Some(cell) = grid.cell_xy_mut(x, y) {
                cell.accumulated_qe = 100.0;
            }
        }
    }

    let changed = diffuse_one_pass(&mut grid, w, h, 1e-6);
    assert_eq!(changed, 0, "uniform field should produce zero deltas");
}
