//! SF-6: PropagationMode resource and NucleusEmissionState component.
//! Controls whether the field propagates instantly (Legacy) or via wavefront (WaveFront).

use bevy::prelude::*;

use crate::blueprint::equations::{
    DIFFUSION_BUDGET_MAX, DIFFUSION_CONDUCTIVITY_DEFAULT, PROPAGATION_SPEED_CELLS_PER_TICK,
    diffusion_delta, propagation_front_radius,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

// ─── Resource ────────────────────────────────────────────────────────────────

/// Controls field propagation algorithm.
/// Default: `Legacy` — unchanged instant-propagation (100% backward compatible).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropagationMode {
    #[default]
    Legacy,
    WaveFront,
}

// ─── Component ───────────────────────────────────────────────────────────────

/// Tracks the wave front radius for a single nucleus.
/// SparseSet: only entities with `EnergyNucleus`.
#[derive(Component, Debug, Clone, Copy, Default)]
#[component(storage = "SparseSet")]
pub struct NucleusEmissionState {
    /// Tick at which this nucleus began emitting.
    pub start_tick: u64,
    /// Front radius after the last propagation step.
    pub last_front_radius: f32,
}

impl NucleusEmissionState {
    /// Elapsed ticks since emission started.
    #[inline]
    pub fn elapsed_ticks(&self, current_tick: u64) -> u32 {
        current_tick.saturating_sub(self.start_tick) as u32
    }

    /// Current front radius based on elapsed ticks and propagation speed.
    #[inline]
    pub fn current_front(&self, current_tick: u64) -> f32 {
        propagation_front_radius(
            PROPAGATION_SPEED_CELLS_PER_TICK,
            self.elapsed_ticks(current_tick),
        )
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Auto-inserts `NucleusEmissionState` for nuclei that don't have it yet.
/// Phase::ThermodynamicLayer, before propagation.
pub fn insert_nucleus_emission_state_system(
    mut commands: Commands,
    clock: Res<SimulationClock>,
    nuclei: Query<
        Entity,
        (
            With<crate::worldgen::EnergyNucleus>,
            Without<NucleusEmissionState>,
        ),
    >,
) {
    for entity in &nuclei {
        commands.entity(entity).insert(NucleusEmissionState {
            start_tick: clock.tick_id,
            last_front_radius: 0.0,
        });
    }
}

/// Lateral diffusion between 4-connected neighbors.
/// Phase::ThermodynamicLayer, after `propagate_nuclei_system`.
/// No-op in `PropagationMode::Legacy`.
pub fn diffuse_propagation_system(
    mode: Option<Res<PropagationMode>>,
    mut grid: ResMut<EnergyFieldGrid>,
) {
    if mode.as_deref() != Some(&PropagationMode::WaveFront) {
        return;
    }
    if !grid.is_changed() {
        return;
    }

    let w = grid.width;
    let h = grid.height;
    let dt = 1.0_f32; // one tick
    let k = DIFFUSION_CONDUCTIVITY_DEFAULT;

    // Collect dirty cells (up to budget).
    let mut dirty_cells: Vec<(u32, u32)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if grid.is_cell_dirty(x, y) {
                dirty_cells.push((x, y));
                if dirty_cells.len() >= DIFFUSION_BUDGET_MAX {
                    break;
                }
            }
        }
        if dirty_cells.len() >= DIFFUSION_BUDGET_MAX {
            break;
        }
    }

    // Double-buffer: accumulate deltas, then apply (order-independent).
    let mut deltas: Vec<f32> = vec![0.0; (w * h) as usize];

    for (cx, cy) in &dirty_cells {
        let cx = *cx;
        let cy = *cy;
        let Some(source_qe) = grid.cell_xy(cx, cy).map(|c| c.accumulated_qe) else {
            continue;
        };

        // 4 neighbors.
        let neighbors: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (dx, dy) in neighbors {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let (nx, ny) = (nx as u32, ny as u32);
            let Some(target_qe) = grid.cell_xy(nx, ny).map(|c| c.accumulated_qe) else {
                continue;
            };
            let delta = diffusion_delta(source_qe, target_qe, k, dt);
            let src_idx = (cy * w + cx) as usize;
            let dst_idx = (ny * w + nx) as usize;
            deltas[src_idx] -= delta;
            deltas[dst_idx] += delta;
        }
    }

    // Apply deltas.
    for (cx, cy) in &dirty_cells {
        let cx = *cx;
        let cy = *cy;
        let idx = (cy * w + cx) as usize;
        if deltas[idx].abs() > 0.0 {
            if let Some(cell) = grid.cell_xy_mut(cx, cy) {
                cell.accumulated_qe = (cell.accumulated_qe + deltas[idx]).max(0.0);
            }
        }
    }
}
