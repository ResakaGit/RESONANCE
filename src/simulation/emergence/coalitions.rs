//! ET-8: Dynamic Coalitions — alianzas con estabilidad Nash.

use bevy::prelude::*;

use crate::layers::{AlchemicalEngine, BaseEnergy};
use crate::blueprint::ids::WorldEntityId;
use crate::bridge::cache::{BridgeCache, CachedValue};
use crate::bridge::config::CoalitionBridge;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::blueprint::equations::emergence::coalitions as coalition_eq;

// ─── Constants ──────────────────────────────────────────────────────────────

pub use crate::blueprint::equations::emergence::coalitions::MAX_COALITION_MEMBERS;

// ─── Components ─────────────────────────────────────────────────────────────

/// Miembro de una coalición activa (SparseSet — coaliciones son transientes).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct CoalitionMember {
    pub coalition_id:      u32,
    pub role:              u8,   // 0=member, 1=leader
    pub join_tick:         u64,
    pub coordination_cost: f32,
}

// ─── Resource ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CoalitionEntry {
    pub coalition_id: u32,
    pub member_ids:   [u32; MAX_COALITION_MEMBERS as usize],
    pub member_count: u8,
    pub stability:    f32,
    pub formed_tick:  u64,
}

#[derive(Resource, Default, Debug)]
pub struct CoalitionRegistry {
    pub entries: Vec<CoalitionEntry>,
}

// ─── Event ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum CoalitionChange { Formed, Joined, Left, Dissolved }

#[derive(Event, Debug, Clone)]
pub struct CoalitionChangedEvent {
    pub coalition_id: u32,
    pub change_type:  CoalitionChange,
    pub entity:       Entity,
    pub tick_id:      u64,
}

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
pub struct CoalitionConfig {
    pub eval_interval:         u8,
    pub scale_factor:          f32,
    pub coordination_cost_rate: f32,
}

impl Default for CoalitionConfig {
    fn default() -> Self {
        Self { eval_interval: 10, scale_factor: 0.15, coordination_cost_rate: 0.02 }
    }
}

// ─── Systems ────────────────────────────────────────────────────────────────

/// Evalúa estabilidad Nash de coaliciones activas.
/// Phase::MetabolicLayer — after symbiosis_effect_system.
/// Cache crítico: CoalitionBridge Large(512) para evitar O(n²) cada tick.
pub fn coalition_stability_system(
    members: Query<(&WorldEntityId, &CoalitionMember, &BaseEnergy, &AlchemicalEngine)>,
    mut registry: ResMut<CoalitionRegistry>,
    mut cache: ResMut<BridgeCache<CoalitionBridge>>,
    mut events: EventWriter<CoalitionChangedEvent>,
    clock: Res<SimulationClock>,
    config: Res<CoalitionConfig>,
) {
    if clock.tick_id % config.eval_interval as u64 != 0 { return; }

    for entry in registry.entries.iter_mut() {
        let cache_key = entry.coalition_id as u64;

        if let Some(CachedValue::Scalar(cached)) = cache.lookup(cache_key) {
            entry.stability = cached;
            continue;
        }

        let n = entry.member_count as usize;
        let mut intake_with   = [0.0f32; MAX_COALITION_MEMBERS as usize];
        let mut intake_without = [0.0f32; MAX_COALITION_MEMBERS as usize];

        for (i, &mid) in entry.member_ids[..n].iter().enumerate() {
            if let Some((_, _cm, energy, engine)) = members.iter()
                .find(|(id, _, _, _)| id.0 == mid)
            {
                let base: f32 = engine.base_intake();
                let bonus = coalition_eq::coalition_intake_bonus(
                    base, entry.member_count, config.scale_factor,
                );
                intake_with[i]    = bonus - energy.qe() * config.coordination_cost_rate;
                intake_without[i] = base;
            }
        }

        let stability = coalition_eq::coalition_stability(&intake_with[..n], &intake_without[..n]);
        entry.stability = stability;
        cache.insert(cache_key, CachedValue::Scalar(stability));

        if stability < 0.0 {
            events.send(CoalitionChangedEvent {
                coalition_id: entry.coalition_id,
                change_type:  CoalitionChange::Dissolved,
                entity:       Entity::PLACEHOLDER,
                tick_id:      clock.tick_id,
            });
        }
    }
}

/// Aplica bonus de intake a miembros de coaliciones estables.
/// Phase::MetabolicLayer — after coalition_stability_system.
pub fn coalition_intake_bonus_system(
    mut members: Query<(&CoalitionMember, &mut AlchemicalEngine)>,
    registry: Res<CoalitionRegistry>,
    config: Res<CoalitionConfig>,
) {
    for (member, mut engine) in &mut members {
        let Some(entry) = registry.entries.iter()
            .find(|e| e.coalition_id == member.coalition_id) else { continue };
        if entry.stability < 0.0 { continue; }

        let boosted = coalition_eq::coalition_intake_bonus(
            engine.base_intake(), entry.member_count, config.scale_factor,
        );
        let net = (boosted - member.coordination_cost).max(engine.base_intake() * 0.5);
        if (engine.intake() - net).abs() > f32::EPSILON { engine.set_intake(net); }
    }
}
