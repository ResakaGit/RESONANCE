//! LOD, presupuestos por tick/frame y cache de materialización worldgen (wiring ECS).

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;

use crate::simulation::PlayerControlled;
use crate::worldgen::contracts::MaterializationResult;

/// Ajustes globales (tunable; tests pueden insertar valores bajos).
#[derive(Resource, Clone, Debug)]
pub struct WorldgenPerfSettings {
    /// Más allá de esta distancia no hay materialización ni visual de celdas.
    pub lod_materialization_cull_distance: f32,
    pub lod_mid_period: u64,
    pub lod_far_period: u64,
    pub max_material_spawn_per_tick: u32,
    pub max_material_despawn_per_tick: u32,
    pub max_propagation_cell_writes_per_tick: u32,
    pub max_visual_derivation_per_frame: u32,
    /// Cadencia de recálculo de malla inferida (shape/morph) en banda media.
    pub shape_rebuild_mid_period: u64,
    /// Cadencia de recálculo de malla inferida (shape/morph) en banda lejana.
    pub shape_rebuild_far_period: u64,
}

impl Default for WorldgenPerfSettings {
    fn default() -> Self {
        Self {
            lod_materialization_cull_distance: 150.0,
            lod_mid_period: 4,
            lod_far_period: 16,
            max_material_spawn_per_tick: u32::MAX,
            max_material_despawn_per_tick: u32::MAX,
            max_propagation_cell_writes_per_tick: u32::MAX,
            max_visual_derivation_per_frame: u32::MAX,
            // Near: siempre. Mid/Far: recálculo temporalmente diezmado para look más recortado.
            shape_rebuild_mid_period: 2,
            shape_rebuild_far_period: 8,
        }
    }
}

/// Foco LOD + contador monótono de tick de simulación (FixedUpdate).
#[derive(Resource, Default, Debug)]
pub struct WorldgenLodContext {
    pub focus_world: Option<Vec2>,
    pub sim_tick: u64,
}

/// Cache paralelo al grid: firma + resultado por índice lineal.
#[derive(Resource, Default, Debug)]
pub struct MaterializationCellCache(pub Vec<Option<(u64, MaterializationResult)>>);

#[derive(Resource, Default, Debug)]
pub struct MatBudgetCounters {
    pub spawns_this_tick: u32,
    pub despawns_this_tick: u32,
}

#[derive(Resource, Default, Debug)]
pub struct MatCacheStats {
    pub hits: u64,
    pub misses: u64,
}

#[derive(Resource, Debug)]
pub struct PropagationWriteBudget {
    pub remaining: u32,
}

impl Default for PropagationWriteBudget {
    fn default() -> Self {
        Self {
            remaining: u32::MAX,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct VisualDerivationFrameState {
    pub processed_this_frame: u32,
}

pub fn worldgen_sim_tick_advance_system(mut lod: ResMut<WorldgenLodContext>) {
    lod.sim_tick = lod.sim_tick.wrapping_add(1);
}

pub fn worldgen_perf_reset_dirty_system(mut grid: ResMut<crate::worldgen::EnergyFieldGrid>) {
    grid.clear_dirty();
}

pub fn worldgen_propagation_budget_reset_system(
    settings: Res<WorldgenPerfSettings>,
    mut budget: ResMut<PropagationWriteBudget>,
) {
    budget.remaining = settings.max_propagation_cell_writes_per_tick;
}

pub fn worldgen_mat_budget_reset_system(mut counters: ResMut<MatBudgetCounters>) {
    counters.spawns_this_tick = 0;
    counters.despawns_this_tick = 0;
}

pub fn worldgen_lod_refresh_system(
    mut lod: ResMut<WorldgenLodContext>,
    layout: Res<SimWorldTransformParams>,
    player: Query<&Transform, With<PlayerControlled>>,
) {
    if let Some(tf) = player.iter().next() {
        let p = sim_plane_pos(tf.translation, layout.use_xz_ground);
        lod.focus_world = if p.is_finite() { Some(p) } else { None };
    } else {
        lod.focus_world = None;
    }
}

pub fn sync_materialization_cache_len_system(
    grid: Res<crate::worldgen::EnergyFieldGrid>,
    mut cache: ResMut<MaterializationCellCache>,
) {
    let len = grid.width as usize * grid.height as usize;
    if cache.0.len() != len {
        cache.0.resize(len, None);
    }
}

pub fn reset_visual_derivation_frame_system(mut frame: ResMut<VisualDerivationFrameState>) {
    frame.processed_this_frame = 0;
}
