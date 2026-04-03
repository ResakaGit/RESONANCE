//! Season transitions and nucleus lifecycle: resources, pure math, event-driven systems.

use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::events::{DeathEvent, SeasonChangeEvent, WorldgenMutationEvent};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::worldgen::EnergyNucleus;
use crate::worldgen::map_config::{MapConfig, NucleusDelta, SeasonPreset};

pub use crate::worldgen::constants::SEASON_TRANSITION_TICKS;

use super::super::startup::StartupNucleus;

/// Snapshot de frecuencias por núcleo para detectar `NucleusModified` sin spam en seasons.
#[derive(Resource, Default, Debug)]
pub struct NucleusFreqTrack(pub HashMap<Entity, f32>);

// --- Funciones puras (testeables, stateless) ---

/// Núcleo objetivo tras aplicar deltas del preset (sin interpolación).
pub fn nucleus_target_from_delta(base: EnergyNucleus, delta: &NucleusDelta) -> EnergyNucleus {
    let df = delta.frequency_hz_delta.unwrap_or(0.0);
    let de = delta.emission_rate_delta.unwrap_or(0.0);
    let dr = delta.propagation_radius_delta.unwrap_or(0.0);
    EnergyNucleus::new(
        base.frequency_hz() + df,
        base.emission_rate_qe_s() + de,
        base.propagation_radius() + dr,
        base.decay(),
    )
}

/// Interpolación lineal entre dos núcleos; `decay` se mantiene desde `start`.
pub fn lerp_nucleus(start: EnergyNucleus, end: EnergyNucleus, t: f32) -> EnergyNucleus {
    let t = if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.0
    };
    EnergyNucleus::new(
        start.frequency_hz() + (end.frequency_hz() - start.frequency_hz()) * t,
        start.emission_rate_qe_s() + (end.emission_rate_qe_s() - start.emission_rate_qe_s()) * t,
        start.propagation_radius() + (end.propagation_radius() - start.propagation_radius()) * t,
        start.decay(),
    )
}

/// Resuelve `SeasonPreset` por nombre en `MapConfig`.
pub fn find_season_preset<'a>(config: &'a MapConfig, name: &str) -> Option<&'a SeasonPreset> {
    config.seasons.iter().find(|s| s.name == name)
}

fn nucleus_display_name(config_name: &str) -> String {
    format!("nucleus::{config_name}")
}

/// Spawnea un núcleo con el mismo bundle que el mapa; `StartupNucleus` omite runtime.
pub fn spawn_runtime_nucleus(
    commands: &mut Commands,
    name: &str,
    nucleus: EnergyNucleus,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let transform = if layout.use_xz_ground {
        Transform::from_xyz(pos.x, layout.standing_y, pos.y)
    } else {
        Transform::from_xyz(pos.x, pos.y, 0.0)
    };
    commands
        .spawn((Name::new(nucleus_display_name(name)), nucleus, transform))
        .id()
}

// --- Estado de transición de season (Resource) ---

#[derive(Debug, Clone)]
struct SeasonLerpEntry {
    entity: Entity,
    start: EnergyNucleus,
    target: EnergyNucleus,
}

#[derive(Resource, Debug, Default)]
pub struct SeasonTransition {
    pub preset_name: String,
    pub tick_index: u32,
    pub total_ticks: u32,
    entries: Vec<SeasonLerpEntry>,
}

impl SeasonTransition {
    pub fn is_active(&self) -> bool {
        !self.preset_name.is_empty()
            && self.tick_index < self.total_ticks
            && !self.entries.is_empty()
    }
}

// --- Sistemas ---

pub fn worldgen_nucleus_death_notify_system(
    mut deaths: EventReader<DeathEvent>,
    layout: Res<SimWorldTransformParams>,
    mut track: ResMut<NucleusFreqTrack>,
    mut mutation: EventWriter<WorldgenMutationEvent>,
    nuclei: Query<&EnergyNucleus>,
    transforms: Query<&Transform>,
) {
    for ev in deaths.read() {
        if nuclei.get(ev.entity).is_err() {
            continue;
        }
        track.0.remove(&ev.entity);
        let pos = transforms
            .get(ev.entity)
            .map(|t| sim_plane_pos(t.translation, layout.use_xz_ground))
            .unwrap_or(Vec2::ZERO);
        mutation.send(WorldgenMutationEvent::NucleusDestroyed {
            entity: ev.entity,
            position: pos,
        });
    }
}

/// Inicializa tracking de frecuencia para nuevos núcleos.
pub fn worldgen_nucleus_freq_seed_system(
    mut track: ResMut<NucleusFreqTrack>,
    query: Query<(Entity, &EnergyNucleus), Added<EnergyNucleus>>,
) {
    for (e, n) in &query {
        track.0.entry(e).or_insert(n.frequency_hz());
    }
}

pub fn worldgen_runtime_nucleus_created_system(
    layout: Res<SimWorldTransformParams>,
    mut query: Query<(Entity, &Transform), (Added<EnergyNucleus>, Without<StartupNucleus>)>,
    mut mutation: EventWriter<WorldgenMutationEvent>,
) {
    for (entity, tf) in &mut query {
        mutation.send(WorldgenMutationEvent::NucleusCreated {
            entity,
            position: sim_plane_pos(tf.translation, layout.use_xz_ground),
        });
    }
}

/// Inicia transición gradual al recibir `SeasonChangeEvent`.
pub fn season_change_begin_system(
    mut events: EventReader<SeasonChangeEvent>,
    config: Res<MapConfig>,
    mut transition: ResMut<SeasonTransition>,
    query: Query<(Entity, &Name, &EnergyNucleus)>,
) {
    for ev in events.read() {
        let Some(preset) = find_season_preset(&config, &ev.preset_name) else {
            continue;
        };
        let mut entries = Vec::new();
        for delta in &preset.nucleus_deltas {
            let wanted = nucleus_display_name(&delta.nucleus_name);
            for (entity, name, nucleus) in &query {
                if name.as_str() == wanted {
                    let target = nucleus_target_from_delta(*nucleus, delta);
                    entries.push(SeasonLerpEntry {
                        entity,
                        start: *nucleus,
                        target,
                    });
                }
            }
        }
        if entries.is_empty() {
            continue;
        }
        transition.preset_name = preset.name.clone();
        transition.tick_index = 0;
        transition.total_ticks = SEASON_TRANSITION_TICKS;
        transition.entries = entries;
    }
}

/// Avanza un paso de interpolación por tick de simulación.
pub fn season_transition_tick_system(
    mut transition: ResMut<SeasonTransition>,
    mut nuclei: Query<&mut EnergyNucleus>,
    mut mutation: EventWriter<WorldgenMutationEvent>,
) {
    if !transition.is_active() {
        return;
    }
    let total = transition.total_ticks.max(1) as f32;
    let t = ((transition.tick_index + 1) as f32 / total).clamp(0.0, 1.0);
    for entry in &transition.entries {
        if let Ok(mut n) = nuclei.get_mut(entry.entity) {
            let next = lerp_nucleus(entry.start, entry.target, t);
            if *n != next {
                *n = next;
            }
        }
    }
    transition.tick_index += 1;
    if transition.tick_index >= transition.total_ticks {
        mutation.send(WorldgenMutationEvent::SeasonApplied {
            preset_name: transition.preset_name.clone(),
        });
        *transition = SeasonTransition::default();
    }
}

/// Emite `NucleusModified` con old/new reales; durante `SeasonTransition` activa solo actualiza el track.
pub fn worldgen_nucleus_freq_changed_notify_system(
    mut track: ResMut<NucleusFreqTrack>,
    transition: Res<SeasonTransition>,
    query: Query<(Entity, &EnergyNucleus), Changed<EnergyNucleus>>,
    mut mutation: EventWriter<WorldgenMutationEvent>,
) {
    if transition.is_active() {
        for (e, n) in &query {
            track.0.insert(e, n.frequency_hz());
        }
        return;
    }
    for (e, n) in &query {
        let prev = track.0.insert(e, n.frequency_hz());
        if let Some(old_freq) = prev {
            if (old_freq - n.frequency_hz()).abs() > 0.01 {
                mutation.send(WorldgenMutationEvent::NucleusModified {
                    entity: e,
                    old_freq,
                    new_freq: n.frequency_hz(),
                });
            }
        }
    }
}
