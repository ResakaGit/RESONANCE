//! Phase::ThermodynamicLayer batch systems — engine processing, irradiance.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::*;
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

/// External irradiance source (sunlight). Not entity-fed.
///
/// Axiom 5: energy conservation — irradiance is the ONLY external energy input.
/// Models a constant solar flux. Entities absorb from it via photosynthesis;
/// the grid itself is replenished externally (not from entity qe).
///
/// The grid represents available photon density per cell. Constant per tick
/// with slight spatial variation from seed.
pub fn irradiance_update(world: &mut SimWorldFlat) {
    // External source: constant solar flux. Not derived from entity energy.
    // Slight variation by cell index for spatial heterogeneity.
    // Axiom 4: seasonal variation — dissipation rate oscillates with time.
    let season = ((world.tick_id as f32) * SEASON_RATE).sin() * SEASON_AMPLITUDE + 1.0;
    let seasonal_flux = SOLAR_FLUX_BASE * season.max(0.1);

    for cell in 0..GRID_CELLS {
        let variation = ((cell as f32 * IRRADIANCE_VARIATION_FREQ).sin()
            * IRRADIANCE_VARIATION_AMP + 1.0).max(IRRADIANCE_VARIATION_MIN);
        world.irradiance_grid[cell] = seasonal_flux * variation;
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
    fn irradiance_grid_is_external_source() {
        let mut w = SimWorldFlat::new(0, 0.05);
        irradiance_update(&mut w);
        // All cells should have positive irradiance (solar flux)
        for cell in &w.irradiance_grid {
            assert!(*cell > 0.0, "external source should provide irradiance");
        }
    }

    #[test]
    fn irradiance_grid_independent_of_entities() {
        let mut w1 = SimWorldFlat::new(0, 0.05);
        let mut w2 = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 1000.0;
        w2.spawn(e); // w2 has entity, w1 doesn't
        irradiance_update(&mut w1);
        irradiance_update(&mut w2);
        // Grid should be identical — entities don't affect it (Axiom 5)
        assert_eq!(w1.irradiance_grid[0].to_bits(), w2.irradiance_grid[0].to_bits());
    }

    #[test]
    fn grid_cell_clamps_to_bounds() {
        assert_eq!(grid_cell([-5.0, -5.0]), 0);
        assert_eq!(grid_cell([100.0, 100.0]), (GRID_SIDE - 1) * GRID_SIDE + (GRID_SIDE - 1));
    }
}
