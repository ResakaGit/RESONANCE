//! ET-13: Geological Time LOD — física comprimida para escalas geológicas.

use bevy::prelude::*;

use crate::layers::BaseEnergy;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::blueprint::equations::emergence::geological_lod as lod_eq;

// ─── Constants ──────────────────────────────────────────────────────────────

pub const LOD_TICK_COMPRESSIONS: [u32; 4] = [1, 10, 100, 1000];

// ─── Resource ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct PopulationGroup {
    pub entity_ids:     [u32; 8],
    pub entity_count:   u8,
    pub mean_qe:        f32,
    pub mean_intake:    f32,
    pub mean_diss:      f32,
}

#[derive(Resource, Debug)]
pub struct GeologicalLODState {
    pub current_lod:        u8,
    pub tick_compression:   u32,
    pub performance_budget: f32,
    pub aggregate_groups:   Vec<PopulationGroup>,
}

impl Default for GeologicalLODState {
    fn default() -> Self {
        Self {
            current_lod:        0,
            tick_compression:   1,
            performance_budget: 10_000.0,
            aggregate_groups:   Vec::new(),
        }
    }
}

/// Marker: entidad actualmente comprimida en LOD > 0 (SparseSet).
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct LODCompressed {
    pub lod_level: u8,
    pub group_idx: u32,
}

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone)]
pub struct GeologicalLODConfig {
    pub performance_budget: f32,
    pub variance_factor:    f32,
}

impl Default for GeologicalLODConfig {
    fn default() -> Self { Self { performance_budget: 10_000.0, variance_factor: 0.1 } }
}

// ─── Systems ────────────────────────────────────────────────────────────────

/// Determina el nivel de LOD basado en carga actual y ajusta tick_compression.
/// Phase::MorphologicalLayer — runs every 1000 ticks.
pub fn geological_lod_update_system(
    agents: Query<&BaseEnergy, Without<LODCompressed>>,
    mut lod_state: ResMut<GeologicalLODState>,
    config: Res<GeologicalLODConfig>,
    clock: Res<SimulationClock>,
) {
    if clock.tick_id % 1000 != 0 { return; }

    let entity_count = agents.iter().count() as u32;
    let new_lod = lod_eq::optimal_lod_level(entity_count, 1000, config.performance_budget);
    lod_state.current_lod = new_lod;
    lod_state.tick_compression = LOD_TICK_COMPRESSIONS[new_lod as usize];
}
