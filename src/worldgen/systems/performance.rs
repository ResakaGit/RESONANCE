//! LOD, presupuestos por tick/frame y cache de materialización worldgen (wiring ECS).

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;

use crate::simulation::PlayerControlled;
use crate::worldgen::contracts::MaterializationResult;

/// Global worldgen performance settings (tunable; tests may insert low values).
///
/// Budgets calibrated for grids ≤128×128 at 60 Hz. For larger grids, scale
/// proportionally or insert a `WorldgenPerfSettings` with higher values.
#[derive(Resource, Clone, Debug)]
pub struct WorldgenPerfSettings {
    /// Beyond this distance no materialization or cell visuals.
    pub lod_materialization_cull_distance: f32,
    /// Update period (ticks) for mid LOD band (30–80 m).
    pub lod_mid_period: u64,
    /// Update period (ticks) for far LOD band (>80 m).
    pub lod_far_period: u64,
    /// Max entities materialized per tick (ECS spawn).
    pub max_material_spawn_per_tick: u32,
    /// Max entities dematerialized per tick (ECS despawn).
    pub max_material_despawn_per_tick: u32,
    /// Max field grid propagation writes per tick.
    pub max_propagation_cell_writes_per_tick: u32,
    /// Max visual derivations (mesh rebuild) per frame.
    pub max_visual_derivation_per_frame: u32,
    /// Shape/morph mesh rebuild period for mid LOD band.
    pub shape_rebuild_mid_period: u64,
    /// Shape/morph mesh rebuild period for far LOD band.
    pub shape_rebuild_far_period: u64,
}

/// Calibrated budgets for grid ≤128×128 (~16K cells) at 60 Hz.
///
/// Near band ≈ pi*30^2 ≈ 2800 visible cells; budgets protect against spikes
/// without throttling normal operation. For larger grids, insert scaled values.
impl Default for WorldgenPerfSettings {
    fn default() -> Self {
        Self {
            lod_materialization_cull_distance: 150.0,
            lod_mid_period: 4,
            lod_far_period: 16,
            max_material_spawn_per_tick: 64,
            max_material_despawn_per_tick: 64,
            max_propagation_cell_writes_per_tick: 256,
            max_visual_derivation_per_frame: 128,
            shape_rebuild_mid_period: 2,
            shape_rebuild_far_period: 8,
        }
    }
}

/// LOD focus + monotonic simulation tick counter (FixedUpdate).
#[derive(Resource, Default, Debug)]
pub struct WorldgenLodContext {
    pub focus_world: Option<Vec2>,
    pub sim_tick: u64,
}

/// Grid-parallel cache: fingerprint + result per linear index.
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
