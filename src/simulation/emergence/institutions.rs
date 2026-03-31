//! ET-14: Institutions — coordinación colectiva que trasciende individuos.
//!
//! STATUS: IMPLEMENTED, NOT REGISTERED. Systems are complete but no plugin
//! wires them into the schedule. No consumers read InstitutionRegistry yet.
//! To activate: register in MetabolicPlugin after coalition_intake_bonus_system.

use bevy::prelude::*;

use crate::layers::BaseEnergy;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::blueprint::equations::emergence::institutions as inst_eq;

// ─── Constants ──────────────────────────────────────────────────────────────

pub const MAX_INSTITUTION_MEMBERS: usize = 16;
pub const MAX_INSTITUTION_RULES: usize = 4;

// ─── Components ─────────────────────────────────────────────────────────────

/// Miembro de una institución activa (SparseSet — miembros son subconjunto pequeño).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct InstitutionMember {
    pub institution_id: u32,
    pub contribution:   f32,
    pub join_tick:      u64,
    pub compliance:     f32,   // [0,1]
}

// ─── Resource ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InstitutionEntry {
    pub institution_id:  u32,
    pub rule_hashes:     [u32; MAX_INSTITUTION_RULES],
    pub rule_count:      u8,
    pub member_count:    u16,
    pub stability:       f32,
    pub surplus_per_tick: f32,
    pub admin_cost:      f32,
    pub founded_tick:    u64,
}

/// Resource: catálogo de instituciones activas (no Entity — trasciende individuos).
#[derive(Resource, Default, Debug)]
pub struct InstitutionRegistry {
    pub entries: Vec<InstitutionEntry>,
}

// ─── Events ─────────────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone)]
pub struct InstitutionFormedEvent {
    pub institution_id: u32,
    pub founder:        Entity,
    pub tick_id:        u64,
}

#[derive(Event, Debug, Clone)]
pub struct InstitutionDissolvedEvent {
    pub institution_id: u32,
    pub tick_id:        u64,
}

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
pub struct InstitutionConfig {
    pub eval_interval:          u8,
    pub enforcement_efficiency: f32,
    pub penalty_rate:           f32,
    pub detection_probability:  f32,
}

impl Default for InstitutionConfig {
    fn default() -> Self {
        Self {
            eval_interval: 20,
            enforcement_efficiency: 5.0,
            penalty_rate: 2.0,
            detection_probability: 0.7,
        }
    }
}

// ─── Systems ────────────────────────────────────────────────────────────────

/// Evalúa estabilidad de instituciones y disuelve las no viables.
/// Phase::MorphologicalLayer — after coalition_intake_bonus_system.
pub fn institution_stability_system(
    mut registry: ResMut<InstitutionRegistry>,
    mut events: EventWriter<InstitutionDissolvedEvent>,
    clock: Res<SimulationClock>,
    config: Res<InstitutionConfig>,
) {
    if clock.tick_id % config.eval_interval as u64 != 0 { return; }

    registry.entries.retain(|entry| {
        let compliance_rate = if entry.member_count > 0 { 0.8 } else { 0.0 };
        let stability = inst_eq::institution_stability(
            compliance_rate, config.enforcement_efficiency, entry.admin_cost,
        );
        if stability < 0.0 {
            events.send(InstitutionDissolvedEvent {
                institution_id: entry.institution_id,
                tick_id: clock.tick_id,
            });
            false
        } else {
            true
        }
    });
}

/// Distribuye surplus institucional entre miembros compliance.
/// Phase::MetabolicLayer — after institution_stability_system.
pub fn institution_distribution_system(
    mut members: Query<(&InstitutionMember, &mut BaseEnergy)>,
    registry: Res<InstitutionRegistry>,
) {
    for (member, mut energy) in &mut members {
        let Some(entry) = registry.entries.iter()
            .find(|e| e.institution_id == member.institution_id) else { continue };
        if member.compliance < 0.5 { continue; }

        let total_contrib: f32 = entry.surplus_per_tick;
        let share = inst_eq::allocation_share(member.contribution, total_contrib.max(f32::EPSILON));
        let bonus = entry.surplus_per_tick * share;
        if bonus > 0.0 {
            let new_qe = energy.qe() + bonus;
            if (energy.qe() - new_qe).abs() > f32::EPSILON { energy.set_qe(new_qe); }
        }
    }
}
