//! ET-12: Continental Drift — modificación tectónica del EnergyFieldGrid.
//!
//! STATUS: IMPLEMENTED, NOT REGISTERED. System complete, no plugin wires it.
//! No consumers read TectonicGrid modifications yet.
//! To activate: register in ThermodynamicPlugin or a dedicated GeologyPlugin.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::tectonics as tectonic_eq;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

// ─── Constants ──────────────────────────────────────────────────────────────

pub const MAX_PLATES: usize = 4;
pub const TECTONIC_EVAL_INTERVAL: u64 = 200;
pub const TECTONIC_STRESS_THRESHOLD: f32 = 50.0;

// ─── Resource ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct TectonicPlate {
    pub id: u32,
    pub drift_velocity: f32,
    pub contact_length: f32,
    pub accumulated_stress: f32,
    pub center_cell: u32,
}

#[derive(Resource, Debug)]
pub struct TectonicGrid {
    pub plates: [TectonicPlate; MAX_PLATES],
    pub plate_count: u8,
    pub friction_coeff: f32,
    pub depth_factor: f32,
    pub eruption_efficiency: f32,
}

impl Default for TectonicGrid {
    fn default() -> Self {
        Self {
            plates: [TectonicPlate {
                id: 0,
                drift_velocity: 0.0,
                contact_length: 0.0,
                accumulated_stress: 0.0,
                center_cell: 0,
            }; MAX_PLATES],
            plate_count: 0,
            friction_coeff: 0.1,
            depth_factor: 1.0,
            eruption_efficiency: 0.5,
        }
    }
}

// ─── Event ──────────────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone)]
pub struct TectonicEvent {
    pub epicenter_cell: u32,
    pub amplitude: f32,
    pub is_constructive: bool,
    pub tick_id: u64,
}

// ─── System ─────────────────────────────────────────────────────────────────

/// Acumula estrés tectónico y emite eventos de liberación.
/// Phase::MorphologicalLayer — modifica el EnergyFieldGrid lentamente.
pub fn tectonic_drift_system(
    mut tectonic: ResMut<TectonicGrid>,
    mut field: ResMut<EnergyFieldGrid>,
    mut events: EventWriter<TectonicEvent>,
    clock: Res<SimulationClock>,
) {
    if clock.tick_id % TECTONIC_EVAL_INTERVAL != 0 {
        return;
    }

    let friction = tectonic.friction_coeff;
    let depth = tectonic.depth_factor;
    let efficiency = tectonic.eruption_efficiency;
    let n = tectonic.plate_count as usize;

    for i in 0..n {
        let plate = &mut tectonic.plates[i];
        let stress_delta =
            tectonic_eq::boundary_stress(plate.drift_velocity, plate.contact_length, friction);
        plate.accumulated_stress += stress_delta;

        if plate.accumulated_stress > TECTONIC_STRESS_THRESHOLD {
            let released = plate.accumulated_stress;
            plate.accumulated_stress = 0.0;
            let amplitude = tectonic_eq::seismic_amplitude(released, depth);

            // Deterministc epicenter from tick + plate id
            let seed = clock.tick_id ^ (plate.id as u64).wrapping_mul(2654435761);
            let epicenter = (seed % (field.width * field.height) as u64) as u32;
            let is_constructive = (seed & 1) == 0;

            events.send(TectonicEvent {
                epicenter_cell: epicenter,
                amplitude,
                is_constructive,
                tick_id: clock.tick_id,
            });

            // Apply delta to nearby cells
            for dist_sq in 0..9u32 {
                let cell_idx = epicenter.saturating_add(dist_sq);
                if cell_idx >= field.width * field.height {
                    break;
                }
                let dist = (dist_sq as f32).sqrt();
                let delta = tectonic_eq::seismic_qe_delta(amplitude, dist, is_constructive);
                if delta.abs() > 0.01 {
                    if is_constructive {
                        let uplift = tectonic_eq::volcanic_qe_uplift(delta, efficiency);
                        if let Some(cell) = field.cell_linear(cell_idx as usize) {
                            let new_qe = cell.accumulated_qe + uplift;
                            let _ = new_qe; // actual mutation via drain_cell with neg delta
                        }
                        field.drain_cell(cell_idx, -delta.abs() * efficiency);
                    } else {
                        let erosion = tectonic_eq::tectonic_erosion(
                            field.cell_qe(cell_idx as usize),
                            delta.abs(),
                        );
                        field.drain_cell(cell_idx, erosion);
                    }
                }
            }
        }
    }
}
