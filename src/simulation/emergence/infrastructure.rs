//! ET-4: Infrastructure — modificación persistente del campo de energía.

use bevy::prelude::*;

use crate::layers::AlchemicalEngine;
use crate::worldgen::EnergyFieldGrid;
use crate::blueprint::equations::emergence::infrastructure as infra_eq;

// ─── Constants ──────────────────────────────────────────────────────────────

pub const MAX_INFRASTRUCTURE_DELTA: f32 = 100.0;
pub const INFRASTRUCTURE_MIN_ACTIVE_DELTA: f32 = 0.1;

// ─── Resource ───────────────────────────────────────────────────────────────

/// Resource: mapa de modificaciones de infraestructura por celda.
#[derive(Resource, Debug)]
pub struct InfrastructureGrid {
    pub modifications:       Vec<f32>,
    pub decay_rate:          f32,
    pub amplification_factor: f32,
}

impl Default for InfrastructureGrid {
    fn default() -> Self {
        Self {
            modifications: vec![0.0f32; 32 * 32],
            decay_rate: 0.001,
            amplification_factor: 0.002,
        }
    }
}

impl InfrastructureGrid {
    pub fn cell_delta(&self, cell_idx: u32) -> f32 {
        self.modifications.get(cell_idx as usize).copied().unwrap_or(0.0)
    }
    pub fn add_modification(&mut self, cell_idx: u32, delta: f32) {
        if let Some(cell) = self.modifications.get_mut(cell_idx as usize) {
            *cell = (*cell + delta).min(MAX_INFRASTRUCTURE_DELTA);
        }
    }
}

// ─── Event ──────────────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone)]
pub struct InfrastructureInvestEvent {
    pub investor:    Entity,
    pub cell_idx:    u32,
    pub qe_invested: f32,
    pub tick_id:     u64,
}

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
pub struct InfrastructureConfig {
    pub modification_rate: f32,
}

impl Default for InfrastructureConfig {
    fn default() -> Self { Self { modification_rate: 0.05 } }
}

// ─── Systems ────────────────────────────────────────────────────────────────

/// Aplica inversiones y decae modificaciones existentes.
/// Phase::MetabolicLayer — after node_control, before victory_check.
pub fn infrastructure_update_system(
    mut infra: ResMut<InfrastructureGrid>,
    mut events: EventReader<InfrastructureInvestEvent>,
    config: Res<InfrastructureConfig>,
) {
    for ev in events.read() {
        let delta = infra_eq::field_modification_delta(ev.qe_invested, config.modification_rate);
        infra.add_modification(ev.cell_idx, delta);
    }
    let decay = infra.decay_rate;
    for cell in infra.modifications.iter_mut() {
        *cell = infra_eq::field_modification_decay(*cell, decay);
    }
}

/// Aplica amplificación de intake a entidades en celdas con infraestructura.
/// Phase::MetabolicLayer — after infrastructure_update_system.
pub fn infrastructure_intake_bonus_system(
    mut agents: Query<(&Transform, &mut AlchemicalEngine)>,
    infra: Res<InfrastructureGrid>,
    field: Res<EnergyFieldGrid>,
) {
    for (transform, mut engine) in &mut agents {
        let cell_idx = field.world_to_cell_idx(transform.translation.x, transform.translation.z);
        if cell_idx == u32::MAX { continue; }
        let delta = infra.cell_delta(cell_idx);
        if delta < INFRASTRUCTURE_MIN_ACTIVE_DELTA { continue; }
        let amp = infra_eq::infrastructure_intake_amplifier(delta, infra.amplification_factor);
        let boosted = engine.base_intake() * amp;
        if (engine.intake() - boosted).abs() > f32::EPSILON { engine.set_intake(boosted); }
    }
}
