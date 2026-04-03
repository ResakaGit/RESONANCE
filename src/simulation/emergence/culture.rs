//! ET-3 + AC-3: Cultural Transmission — sistema de imitación + frecuency affinity.
//!
//! AC-3 extends imitation with oscillatory affinity: same-band entities imitate
//! each other more readily. Cross-band entities have suppressed imitation gain.

use bevy::prelude::*;

use crate::blueprint::constants::CULTURE_COHERENCE_IMITATION_BONUS_CAP;
use crate::blueprint::equations::emergence::culture as culture_eq;
use crate::layers::{BaseEnergy, BehavioralAgent, OscillatorySignature};
use crate::runtime_platform::simulation_tick::{SimulationClock, SimulationElapsed};
use crate::world::SpatialIndex;

// ─── Types ──────────────────────────────────────────────────────────────────

pub const MAX_MEMES: usize = 4;

/// Comportamiento transmisible: identificado por hash u32, sin String.
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct MemeEntry {
    pub behavior_hash: u32,
    pub estimated_fitness: f32,
    pub adoption_tick: u64,
    pub spread_count: u8,
}

/// Capa T1-3: CulturalMemory — comportamientos aprendidos por imitación.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CulturalMemory {
    pub memes: [MemeEntry; MAX_MEMES],
    pub meme_count: u8,
    pub imitation_radius: f32,
    pub imitation_prob: f32,
}

impl Default for CulturalMemory {
    fn default() -> Self {
        Self {
            memes: [MemeEntry::default(); MAX_MEMES],
            meme_count: 0,
            imitation_radius: 10.0,
            imitation_prob: 0.3,
        }
    }
}

impl CulturalMemory {
    pub fn memes_active(&self) -> &[MemeEntry] {
        &self.memes[..self.meme_count as usize]
    }
}

// ─── Event ──────────────────────────────────────────────────────────────────

/// Emitido cuando una entidad adopta un comportamiento por imitación.
#[derive(Event, Debug, Clone)]
pub struct MemeAdoptedEvent {
    pub adopter: Entity,
    pub source: Entity,
    pub behavior_hash: u32,
    pub tick_id: u64,
}

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
pub struct CultureConfig {
    pub adoption_cost: f32,
    pub observation_uncertainty: f32,
}

impl Default for CultureConfig {
    fn default() -> Self {
        Self {
            adoption_cost: 1.0,
            observation_uncertainty: 0.2,
        }
    }
}

// ─── System ─────────────────────────────────────────────────────────────────

/// Propaga comportamientos entre entidades en rango de imitación.
/// AC-3: imitation probability weighted by oscillatory affinity (Axiom 6×8).
/// Phase::Input — after ET-2 theory_of_mind_update_system.
pub fn cultural_transmission_system(
    mut imitators: Query<
        (
            Entity,
            &Transform,
            &mut CulturalMemory,
            &BaseEnergy,
            Option<&OscillatorySignature>,
        ),
        With<BehavioralAgent>,
    >,
    spatial: Res<SpatialIndex>,
    clock: Res<SimulationClock>,
    elapsed: Option<Res<SimulationElapsed>>,
    mut events: EventWriter<MemeAdoptedEvent>,
    config: Res<CultureConfig>,
) {
    let t = elapsed.map(|e| e.secs).unwrap_or(0.0);

    // Snapshot all cultural data before mutation pass (avoids query aliasing).
    let snapshot: Vec<(Entity, [MemeEntry; MAX_MEMES], u8, f32, f32, f32)> = imitators
        .iter()
        .map(|(e, _t, cm, en, osc)| {
            let (freq, phase) = osc
                .map(|o| (o.frequency_hz(), o.phase()))
                .unwrap_or((0.0, 0.0));
            (e, cm.memes, cm.meme_count, en.qe(), freq, phase)
        })
        .collect();

    for (imitator_entity, transform, mut culture, energy, imitator_osc) in &mut imitators {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let nearby = spatial.query_radius(pos, culture.imitation_radius);
        let (imitator_freq, imitator_phase) = imitator_osc
            .map(|o| (o.frequency_hz(), o.phase()))
            .unwrap_or((0.0, 0.0));

        for entry in &nearby {
            let target_entity = entry.entity;
            if target_entity == imitator_entity {
                continue;
            }

            let Some(&(_, t_memes, t_count, t_qe, t_freq, t_phase)) = snapshot
                .iter()
                .find(|(e, _, _, _, _, _)| *e == target_entity)
            else {
                continue;
            };

            // AC-3: oscillatory affinity gates imitation — same-band entities imitate easier.
            let affinity = culture_eq::frequency_imitation_affinity(
                imitator_freq,
                imitator_phase,
                t_freq,
                t_phase,
                t,
            );
            // Proxy: pairwise affinity used as a stand-in for group coherence.
            // A model that resonates with the observer is likely part of a coherent group.
            let coherence_bonus = culture_eq::group_coherence_imitation_bonus(
                affinity,
                CULTURE_COHERENCE_IMITATION_BONUS_CAP,
            );

            if !culture_eq::should_imitate_with_affinity(
                energy.qe(),
                t_qe,
                config.adoption_cost,
                config.observation_uncertainty,
                affinity,
                coherence_bonus,
            ) {
                continue;
            }

            let t_active = &t_memes[..t_count as usize];
            let best = t_active
                .iter()
                .filter(|m| {
                    !culture
                        .memes_active()
                        .iter()
                        .any(|own| own.behavior_hash == m.behavior_hash)
                })
                .max_by(|a, b| {
                    a.estimated_fitness
                        .partial_cmp(&b.estimated_fitness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            let Some(best_meme) = best else { continue };
            if culture.meme_count as usize >= MAX_MEMES {
                continue;
            }

            let idx = culture.meme_count as usize;
            culture.memes[idx] = MemeEntry {
                behavior_hash: best_meme.behavior_hash,
                estimated_fitness: best_meme.estimated_fitness,
                adoption_tick: clock.tick_id,
                spread_count: 0,
            };
            culture.meme_count += 1;
            events.send(MemeAdoptedEvent {
                adopter: imitator_entity,
                source: target_entity,
                behavior_hash: best_meme.behavior_hash,
                tick_id: clock.tick_id,
            });
        }
    }
}
