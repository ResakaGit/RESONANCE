//! Phase::ThermodynamicLayer batch systems — engine processing, irradiance.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::{GRID_CELLS, GRID_SIDE, MAX_ENTITIES};
use crate::blueprint::equations;

/// L5 AlchemicalEngine: intake energy into buffer per tick.
///
/// `intake = engine_intake_allometric(valve_in, dt, qe, buffer, max, radius)`.
/// Drain from qe, deposit into engine_buffer.
pub fn engine_processing(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        if e.engine_max <= 0.0 { continue; }
        let intake = equations::engine_intake_allometric(
            e.input_valve, dt, e.qe, e.engine_buffer, e.engine_max, e.radius,
        );
        let clamped = intake.min(e.qe).min(e.engine_max - e.engine_buffer);
        if clamped <= 0.0 { continue; }
        e.qe -= clamped;
        e.engine_buffer += clamped;
    }
}

/// Fill irradiance grid from alive entities' energy contribution.
///
/// Each alive entity adds a fraction of its qe to the grid cell it occupies.
/// Grid is zeroed each tick (transient field).
pub fn irradiance_update(world: &mut SimWorldFlat) {
    // Reset grid each tick
    world.irradiance_grid = [0.0; GRID_CELLS];
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &world.entities[i];
        let cell = grid_cell(e.position);
        if cell < GRID_CELLS {
            world.irradiance_grid[cell] += e.qe * 0.01;
        }
    }
}

/// Map world position to flat grid index.
#[inline]
pub fn grid_cell(position: [f32; 2]) -> usize {
    let cx = (position[0].max(0.0) as usize).min(GRID_SIDE - 1);
    let cy = (position[1].max(0.0) as usize).min(GRID_SIDE - 1);
    cy * GRID_SIDE + cx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn spawn(w: &mut SimWorldFlat, qe: f32, engine_max: f32) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 1.0;
        e.engine_max = engine_max;
        e.engine_buffer = 0.0;
        e.input_valve = 1.0;
        e.output_valve = 1.0;
        w.spawn(e).unwrap()
    }

    #[test]
    fn engine_processing_transfers_qe_to_buffer() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0, 50.0);
        let qe_before = w.entities[idx].qe;
        engine_processing(&mut w);
        assert!(w.entities[idx].engine_buffer > 0.0, "buffer should receive energy");
        assert!(w.entities[idx].qe < qe_before, "qe should decrease");
        let total = w.entities[idx].qe + w.entities[idx].engine_buffer;
        assert!((total - qe_before).abs() < 1e-4, "energy conserved: {total} vs {qe_before}");
    }

    #[test]
    fn engine_processing_skips_no_engine() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        e.engine_max = 0.0; // no engine
        w.spawn(e);
        engine_processing(&mut w);
        assert_eq!(w.entities[0].engine_buffer, 0.0);
        assert_eq!(w.entities[0].qe, 100.0);
    }

    #[test]
    fn engine_processing_does_not_overflow_buffer() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 1000.0, 10.0);
        w.entities[idx].engine_buffer = 9.5; // near cap
        engine_processing(&mut w);
        assert!(w.entities[idx].engine_buffer <= 10.0 + 1e-5);
    }

    #[test]
    fn irradiance_grid_populated_from_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 200.0;
        e.position = [3.0, 5.0];
        w.spawn(e);
        irradiance_update(&mut w);
        let cell = grid_cell([3.0, 5.0]);
        assert!(w.irradiance_grid[cell] > 0.0, "cell should have irradiance");
    }

    #[test]
    fn irradiance_grid_zeroed_each_tick() {
        let mut w = SimWorldFlat::new(0, 0.05);
        w.irradiance_grid[0] = 999.0;
        irradiance_update(&mut w);
        assert_eq!(w.irradiance_grid[0], 0.0, "grid should reset");
    }

    #[test]
    fn grid_cell_clamps_to_bounds() {
        assert_eq!(grid_cell([-5.0, -5.0]), 0);
        assert_eq!(grid_cell([100.0, 100.0]), (GRID_SIDE - 1) * GRID_SIDE + (GRID_SIDE - 1));
    }
}
