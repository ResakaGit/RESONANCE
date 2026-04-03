//! ET-11: Multi-Scale Information — MultiscaleSignalGrid resource + aggregation system.
//!
//! STATUS: IMPLEMENTED, NOT REGISTERED. System complete, no plugin wires it.
//! No consumers read MultiscaleSignalGrid yet.
//! To activate: register in MetabolicPlugin after ecology_dynamics.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::multiscale as multiscale_eq;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

// ─── Constants ──────────────────────────────────────────────────────────────

pub const MULTISCALE_LOCAL_SIZE: usize = 32 * 32; // 1024 cells
pub const MULTISCALE_REGIONAL_SIZE: usize = 8 * 8; // 64 regions (4×4 cells each)
pub const MULTISCALE_UPDATE_INTERVAL: u64 = 8;

// ─── Resource ───────────────────────────────────────────────────────────────

/// Resource: señales pre-agregadas en 3 escalas espaciales.
#[derive(Resource, Debug)]
pub struct MultiscaleSignalGrid {
    pub local: Vec<f32>,
    pub regional: Vec<f32>,
    pub global: f32,
    pub last_updated: u64,
}

impl Default for MultiscaleSignalGrid {
    fn default() -> Self {
        Self {
            local: vec![0.0; MULTISCALE_LOCAL_SIZE],
            regional: vec![0.0; MULTISCALE_REGIONAL_SIZE],
            global: 0.0,
            last_updated: 0,
        }
    }
}

impl MultiscaleSignalGrid {
    /// Señal local en el índice de celda dado.
    pub fn local_at(&self, cell_idx: usize) -> f32 {
        self.local.get(cell_idx).copied().unwrap_or(0.0)
    }
    /// Señal regional para la región que contiene `cell_idx`.
    pub fn regional_at(&self, region_idx: usize) -> f32 {
        self.regional.get(region_idx).copied().unwrap_or(0.0)
    }
    /// Convierte un índice de celda (32×32 grid) al índice de región (8×8 grid).
    pub fn cell_to_region(cell_idx: u32) -> usize {
        let x = (cell_idx % 32) / 4;
        let y = (cell_idx / 32) / 4;
        (y * 8 + x) as usize
    }
}

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
pub struct MultiscaleConfig {
    pub update_interval: u64,
}

impl Default for MultiscaleConfig {
    fn default() -> Self {
        Self {
            update_interval: MULTISCALE_UPDATE_INTERVAL,
        }
    }
}

// ─── System ─────────────────────────────────────────────────────────────────

/// Agrega señales de EnergyFieldGrid en tres escalas.
/// Phase::MetabolicLayer — runs every N ticks (throttled).
pub fn multiscale_aggregation_system(
    field: Res<EnergyFieldGrid>,
    mut ms: ResMut<MultiscaleSignalGrid>,
    clock: Res<SimulationClock>,
    config: Res<MultiscaleConfig>,
) {
    if clock.tick_id % config.update_interval != 0 {
        return;
    }

    // Local: copiar qe por celda
    for (i, v) in ms.local.iter_mut().enumerate() {
        *v = field.cell_qe(i);
    }

    // Regional: media de 4×4 bloques
    for ry in 0..8usize {
        for rx in 0..8usize {
            let region_idx = ry * 8 + rx;
            let mut sum = 0.0f32;
            let mut count = 0u32;
            for dy in 0..4usize {
                for dx in 0..4usize {
                    let cx = (rx * 4 + dx) as u32;
                    let cy = (ry * 4 + dy) as u32;
                    if cx < field.width && cy < field.height {
                        sum += field.cell_qe((cy as usize) * field.width as usize + cx as usize);
                        count += 1;
                    }
                }
            }
            if let Some(v) = ms.regional.get_mut(region_idx) {
                *v = if count > 0 { sum / count as f32 } else { 0.0 };
            }
        }
    }

    // Global: media de todas las regiones
    let n = ms.regional.len();
    ms.global = if n > 0 {
        ms.regional.iter().sum::<f32>() / n as f32
    } else {
        0.0
    };
    ms.last_updated = clock.tick_id;

    let _ = multiscale_eq::aggregate_signal; // ensure eq module is used
}
