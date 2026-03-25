use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::IdGenerator;
use crate::entities::archetypes::spawn_rosa;
use crate::layers::{
    BaseEnergy, CapabilitySet, GrowthBudget, InferenceProfile, LifecycleStage,
    LifecycleStageCache, MatterCoherence, SpatialVolume,
};
use crate::rendering::quantized_color::QuantizedPrecision;
use crate::runtime_platform::camera_controller_3d::{
    CameraRigTarget, MobaCameraConfig, MobaCameraState,
};
use crate::runtime_platform::compat_2d3d::{RenderCompatProfile, SimWorldTransformParams};
use crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y;
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;
use crate::runtime_platform::simulation_tick::SimulationElapsed;
use crate::simulation::allometric_growth::AllometricGrowthTimeScale;
use crate::simulation::env_scenario::EnvScenarioSnapshot;
use crate::worldgen::systems::performance::{WorldgenLodContext, WorldgenPerfSettings};
use crate::worldgen::systems::startup::StartupNucleus;
use crate::worldgen::{
    EnergyFieldGrid, EnergyVisual, Materialized, WorldArchetype, materialize_cell_at_time,
};

// ── Constantes ──────────────────────────────────────────────────────────

/// Mes de 30 días en segundos.
const SECONDS_PER_MONTH: f32 = 30.0 * 24.0 * 60.0 * 60.0;

/// 1 hora biológica por segundo de simulación.
const DEMO_BIO_SECS_PER_SIM_SEC: f32 = 60.0 * 60.0;

/// Multiplicador allométrico por tick (bio_secs / 60 Hz).
const DEMO_ALLOMETRIC_GROWTH_MULTIPLIER: f32 = DEMO_BIO_SECS_PER_SIM_SEC / 60.0;

/// Cámara.
const DEMO_ZOOM: f32 = 2.5;
const DEMO_ZOOM_MIN: f32 = 0.3;

/// Marcador para la rosa foco.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RosaLifecycleFocus;

// ── Startup ─────────────────────────────────────────────────────────────

/// Acopla la rosa al pipeline visual (Materialized + EnergyVisual + detalle).
fn attach_rosa_to_field(
    commands: &mut Commands,
    rosa: Entity,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    t: f32,
) {
    let Some((cx, cy)) = grid.cell_coords(Vec2::ZERO) else { return };
    let Some(cell) = grid.cell_xy(cx, cy) else { return };
    let archetype = materialize_cell_at_time(cell, almanac, t, grid.cell_size, None)
        .map(|r| r.archetype)
        .unwrap_or(WorldArchetype::TerraSolid);

    commands.entity(rosa).insert((
        Name::new("flora_rosa"),
        RosaLifecycleFocus,
        Materialized { cell_x: cx as i32, cell_y: cy as i32, archetype },
        QuantizedPrecision(1.0),
        CapabilitySet::new(
            CapabilitySet::GROW | CapabilitySet::BRANCH | CapabilitySet::ROOT
            | CapabilitySet::PHOTOSYNTH | CapabilitySet::REPRODUCE,
        ),
        LifecycleStageCache {
            stage: LifecycleStage::Dormant,
            ticks_in_stage: 0,
            candidate_stage: None,
            candidate_ticks: 0,
        },
        InferenceProfile::new(0.93, 0.0, 0.94, 0.52),
        EnergyVisual { color: Color::srgb(0.25, 0.6, 0.2), scale: 1.0, emission: 0.0, opacity: 1.0 },
    ));
    commands.entity(rosa).remove::<V6RuntimeEntity>();
}

/// Spawnea la rosa.
pub fn spawn_demo_flora(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    profile: RenderCompatProfile,
) -> Entity {
    if !profile.enables_visual_3d() {
        commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(0.05))));
    }
    let layout = SimWorldTransformParams::from_profile(profile);
    spawn_rosa(commands, id_gen, Vec2::ZERO, &layout)
}

/// Startup system: rosa + cámara + ambiente.
pub fn spawn_demo_level_startup_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    profile: Res<RenderCompatProfile>,
    grid: Option<Res<EnergyFieldGrid>>,
    almanac: Res<AlchemicalAlmanac>,
    time: Res<Time>,
    mut camera_target: ResMut<CameraRigTarget>,
    mut moba_config: ResMut<MobaCameraConfig>,
    mut moba_state: ResMut<MobaCameraState>,
    materialized_q: Query<Entity, With<Materialized>>,
    nuclei_q: Query<Entity, With<StartupNucleus>>,
    mut perf: ResMut<WorldgenPerfSettings>,
    mut env_snapshot: ResMut<EnvScenarioSnapshot>,
) {
    commands.insert_resource(AllometricGrowthTimeScale {
        growth_multiplier: DEMO_ALLOMETRIC_GROWTH_MULTIPLIER,
    });

    let rosa = spawn_demo_flora(&mut commands, &mut id_gen, *profile);

    if !profile.enables_visual_3d() {
        camera_target.entity = None;
        return;
    }

    // Ambiente benévolo.
    *env_snapshot = EnvScenarioSnapshot {
        food_density_t: 0.92,
        predation_pressure_t: 0.06,
        temperature_t: 0.55,
        medium_density_t: 0.55,
    };
    perf.shape_rebuild_mid_period = 1;
    perf.shape_rebuild_far_period = 1;

    // Acoplar al campo V7.
    if let Some(g) = grid.as_deref() {
        attach_rosa_to_field(&mut commands, rosa, g, almanac.as_ref(), time.elapsed_secs());
    }

    // Ocultar todo lo que no sea la rosa.
    for entity in &materialized_q {
        if entity != rosa {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    for entity in &nuclei_q {
        commands.entity(entity).insert(Visibility::Hidden);
    }

    // Cámara.
    camera_target.entity = Some(rosa);
    moba_config.zoom_min = DEMO_ZOOM_MIN;
    moba_config.pan_speed *= 0.3;
    moba_config.edge_scroll_speed *= 0.3;
    moba_state.focus_xz = Vec2::ZERO;
    moba_state.focus_y = DEFAULT_SIM_STANDING_Y;
    moba_state.zoom_horizontal = DEMO_ZOOM;
}

// ── Sistemas runtime ────────────────────────────────────────────────────

/// Ancla LOD al centro para que la rosa esté siempre en banda Near.
pub fn pin_rosa_lod_focus_system(
    grid: Option<Res<EnergyFieldGrid>>,
    mut lod: ResMut<WorldgenLodContext>,
) {
    let Some(grid) = grid.as_deref() else { return };
    let Some((cx, cy)) = grid.cell_coords(Vec2::ZERO) else { return };
    let Some(center) = grid.world_pos(cx, cy) else { return };
    lod.focus_world = Some(center);
}

/// Oculta todo excepto la rosa — tiles, núcleos, runtime entities.
pub fn enforce_rosa_focus_system(
    mut commands: Commands,
    materialized_q: Query<(Entity, Option<&Visibility>, Option<&RosaLifecycleFocus>), With<Materialized>>,
    nuclei_q: Query<(Entity, Option<&Visibility>), With<StartupNucleus>>,
    runtime_q: Query<Entity, (With<V6RuntimeEntity>, Without<RosaLifecycleFocus>)>,
) {
    for (entity, vis, focus) in &materialized_q {
        if focus.is_none() && !matches!(vis, Some(Visibility::Hidden)) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    for (entity, vis) in &nuclei_q {
        if !matches!(vis, Some(Visibility::Hidden)) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    for entity in &runtime_q {
        commands.entity(entity).remove::<V6RuntimeEntity>();
        commands.entity(entity).insert(Visibility::Hidden);
    }
}

/// Mantiene detalle máximo y fuerza rebuild para reflejar crecimiento.
pub fn stabilize_rosa_growth_system(
    mut commands: Commands,
    mut prec_q: Query<&mut QuantizedPrecision, With<RosaLifecycleFocus>>,
    rebuild_q: Query<
        Entity,
        (With<RosaLifecycleFocus>,
         With<crate::worldgen::shape_inference::ShapeInferred>,
         Without<crate::worldgen::shape_inference::PendingGrowthMorphRebuild>),
    >,
) {
    for mut prec in &mut prec_q {
        if prec.0 < 1.0 { prec.0 = 1.0; }
    }
    for entity in &rebuild_q {
        commands.entity(entity).insert(crate::worldgen::shape_inference::PendingGrowthMorphRebuild);
    }
}

/// Telemetría flora_*.
pub fn debug_botanical_seed_system(
    sim_elapsed: Option<Res<SimulationElapsed>>,
    q: Query<(&Name, &BaseEnergy, &MatterCoherence, &SpatialVolume, Option<&GrowthBudget>, Option<&InferenceProfile>)>,
) {
    let sim_secs = sim_elapsed.map(|e| e.secs).unwrap_or(0.0);
    let bio_months = (sim_secs * DEMO_BIO_SECS_PER_SIM_SEC) / SECONDS_PER_MONTH;
    for (name, energy, _matter, volume, budget, profile) in &q {
        if !name.as_str().starts_with("flora_") { continue; }
        let b = budget.map(|g| format!("Bio={:.3} Eff={:.2}", g.biomass_available, g.efficiency)).unwrap_or("None".into());
        let p = profile.map(|p| format!("g={:.1} b={:.1}", p.growth_bias, p.branching_bias)).unwrap_or("None".into());
        info!("[{}] t={:.1}m | qe={:.0} | r={:.3} | {b} | {p}", name.as_str(), bio_months, energy.qe, volume.radius);
    }
}

#[cfg(test)]
mod tests {
    use super::{DEMO_ALLOMETRIC_GROWTH_MULTIPLIER, DEMO_BIO_SECS_PER_SIM_SEC};

    #[test]
    fn bio_clock_one_hour_per_sim_second() {
        let hours = DEMO_BIO_SECS_PER_SIM_SEC / 3600.0;
        assert!((hours - 1.0).abs() < 1e-2);
    }

    #[test]
    fn growth_multiplier_consistent_with_bio_clock() {
        let scaled = DEMO_ALLOMETRIC_GROWTH_MULTIPLIER * 60.0;
        let rel = (scaled - DEMO_BIO_SECS_PER_SIM_SEC).abs() / DEMO_BIO_SECS_PER_SIM_SEC;
        assert!(rel < 1e-5);
    }
}
