use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::IdGenerator;
use crate::blueprint::constants::QE_MIN_EXISTENCE;
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
use crate::world::demo_clouds::DemoCloudAnchor;
use crate::worldgen::systems::performance::{WorldgenLodContext, WorldgenPerfSettings};
use crate::worldgen::systems::startup::StartupNucleus;
use crate::worldgen::{
    EnergyFieldGrid, EnergyVisual, Materialized, WorldArchetype, materialize_cell_at_time,
};

/// Mes de 30 días en segundos (calendario demo, coherente con `t_bio`).
const SECONDS_PER_MONTH: f32 = 30.0 * 24.0 * 60.0 * 60.0;

/// Reloj biológico: 1 hora por segundo de simulación (germinación visible en ~30s).
const DEMO_BIO_SECS_PER_SIM_SEC: f32 = 60.0 * 60.0;

/// Turbo del delta radial en `allometric_growth_system` (por tick, /60 del reloj bio).
const DEMO_ALLOMETRIC_GROWTH_MULTIPLIER: f32 = DEMO_BIO_SECS_PER_SIM_SEC / 60.0;

/// Zoom para ver la planta completa con espacio alrededor.
const DEMO_CLOSE_ZOOM_HORIZONTAL: f32 = 2.5;
const DEMO_MIN_ZOOM_HORIZONTAL: f32 = 0.3;

/// Piso de qe: rescate suave si fotosíntesis/Liebig caen.
const ROSA_QE_FLOOR: f32 = QE_MIN_EXISTENCE * 8000.0;
const ROSA_QE_INJECT_CAP: f32 = 6.0;

/// Perfil de inferencia enriquecido para manifest de órganos rico (pétalos, espinas, hojas).
const ROSA_GROWTH_BIAS: f32 = 0.93;
const ROSA_BRANCHING_BIAS: f32 = 0.94;
const ROSA_RESILIENCE: f32 = 0.52;

/// Marcador para la rosa foco de la demo.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RosaLifecycleFocus;

/// Acopla la rosa al pipeline GF1: gradiente V7, materialización, detalle máximo.
fn attach_rosa_to_field(
    commands: &mut Commands,
    rosa: Entity,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    interference_t: f32,
) {
    let Some((cx, cy)) = grid.cell_coords(Vec2::ZERO) else {
        return;
    };
    let Some(cell) = grid.cell_xy(cx, cy) else {
        return;
    };
    let field_archetype = materialize_cell_at_time(cell, almanac, interference_t, grid.cell_size, None)
        .map(|r| r.archetype)
        .unwrap_or(WorldArchetype::TerraSolid);

    commands.entity(rosa).insert((
        Name::new("flora_rosa_focus"),
        RosaLifecycleFocus,
        Materialized {
            cell_x: cx as i32,
            cell_y: cy as i32,
            archetype: field_archetype,
        },
        QuantizedPrecision(1.0),
        CapabilitySet::new(
            CapabilitySet::GROW
                | CapabilitySet::BRANCH
                | CapabilitySet::ROOT
                | CapabilitySet::PHOTOSYNTH
                | CapabilitySet::REPRODUCE,
        ),
        // Semilla: radius mínimo para que growth_progress empiece en 0%.
        crate::layers::SpatialVolume::new(0.01),
        crate::layers::AllometricRadiusAnchor::new(0.01),
        LifecycleStageCache {
            stage: LifecycleStage::Dormant,
            ticks_in_stage: 0,
            candidate_stage: None,
            candidate_ticks: 0,
        },
        InferenceProfile::new(ROSA_GROWTH_BIAS, 0.0, ROSA_BRANCHING_BIAS, ROSA_RESILIENCE),
        // Nutrientes abundantes para fotosíntesis sostenida.
        crate::layers::NutrientProfile::new(200.0, 200.0, 200.0, 200.0),
        // Fallback visual para shape_color_inference_system (requiere &EnergyVisual).
        EnergyVisual {
            color: Color::srgb(0.25, 0.6, 0.2),
            scale: 1.0,
            emission: 0.0,
            opacity: 1.0,
        },
    ));
    commands.entity(rosa).remove::<V6RuntimeEntity>();
}

/// Demo: **Rosa Lifecycle** — una sola rosa desde semilla hasta reproducción.
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

/// Startup: spawnea la rosa, acopla al campo V7, configura cámara y ambiente.
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
    materialized_q: Query<(Entity, Option<&Name>), With<Materialized>>,
    nuclei_q: Query<Entity, With<StartupNucleus>>,
    mut perf: ResMut<WorldgenPerfSettings>,
    mut env_snapshot: ResMut<EnvScenarioSnapshot>,
) {
    commands.insert_resource(AllometricGrowthTimeScale {
        growth_multiplier: DEMO_ALLOMETRIC_GROWTH_MULTIPLIER,
    });

    let rosa = spawn_demo_flora(&mut commands, &mut id_gen, *profile);
    commands.entity(rosa).insert(DemoCloudAnchor);

    if profile.enables_visual_3d() {
        // Ambiente benévolo para organ viability.
        *env_snapshot = EnvScenarioSnapshot {
            food_density_t: 0.92,
            predation_pressure_t: 0.06,
            temperature_t: 0.55,
            medium_density_t: 0.55,
        };
        // Rebuild de malla a máxima cadencia.
        perf.shape_rebuild_mid_period = 1;
        perf.shape_rebuild_far_period = 1;

        if let Some(g) = grid.as_deref() {
            attach_rosa_to_field(
                &mut commands,
                rosa,
                g,
                almanac.as_ref(),
                time.elapsed_secs(),
            );
        }

        // Ocultar tiles de terreno y núcleos — solo la rosa visible.
        for (entity, name_opt) in &materialized_q {
            let is_flora = name_opt.is_some_and(|n| n.as_str().starts_with("flora_"));
            if entity != rosa && !is_flora {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
        for entity in &nuclei_q {
            commands.entity(entity).insert(Visibility::Hidden);
        }

        camera_target.entity = Some(rosa);
        moba_config.zoom_min = DEMO_MIN_ZOOM_HORIZONTAL;
        // 70% más lento para control fino a escala flora.
        moba_config.pan_speed *= 0.3;
        moba_config.edge_scroll_speed *= 0.3;
        moba_state.focus_xz = Vec2::ZERO;
        moba_state.focus_y = DEFAULT_SIM_STANDING_Y;
        moba_state.zoom_horizontal = DEMO_CLOSE_ZOOM_HORIZONTAL;
    } else {
        camera_target.entity = None;
    }
}

/// Ancla LOD al centro de la rosa para que shape inference la trate como Near.
pub fn pin_rosa_lod_focus_system(
    grid: Option<Res<EnergyFieldGrid>>,
    mut lod: ResMut<WorldgenLodContext>,
) {
    let Some(grid) = grid.as_deref() else {
        return;
    };
    let Some((cx, cy)) = grid.cell_coords(Vec2::ZERO) else {
        return;
    };
    let Some(center) = grid.world_pos(cx, cy) else {
        return;
    };
    lod.focus_world = Some(center);
}

/// Oculta todo excepto la rosa foco: quita V6RuntimeEntity de no-rosa (evita esferas del render bridge)
/// y oculta tiles materializadas + núcleos.
pub fn enforce_rosa_focus_system(
    mut commands: Commands,
    materialized_q: Query<
        (Entity, Option<&Visibility>, Option<&RosaLifecycleFocus>),
        With<Materialized>,
    >,
    nuclei_q: Query<(Entity, Option<&Visibility>), With<StartupNucleus>>,
    runtime_q: Query<
        Entity,
        (With<crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity>,
         Without<RosaLifecycleFocus>),
    >,
) {
    for (entity, vis_opt, focus_opt) in &materialized_q {
        if focus_opt.is_none() && !matches!(vis_opt, Some(Visibility::Hidden)) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    for (entity, vis_opt) in &nuclei_q {
        if !matches!(vis_opt, Some(Visibility::Hidden)) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    // Quitar V6RuntimeEntity → render bridge no crea esferas nuevas.
    // Ocultar → Visibility propagates to children (Mesh3d spheres).
    for entity in &runtime_q {
        commands.entity(entity).remove::<crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity>();
        commands.entity(entity).insert(Visibility::Hidden);
    }
}

/// Rescate: mantiene energía, biomasa, lifecycle y detalle para la rosa foco.
/// Corre en Update (para energía/LOD) y en FixedUpdate.after(MetabolicLayer) (para biomasa/stage).
pub fn stabilize_rosa_energy_system(
    mut energy_q: Query<&mut BaseEnergy, With<RosaLifecycleFocus>>,
) {
    for mut energy in &mut energy_q {
        let qe = energy.qe();
        if qe < ROSA_QE_FLOOR {
            let deficit = ROSA_QE_FLOOR - qe;
            energy.inject(deficit.min(ROSA_QE_INJECT_CAP));
        }
    }
}

/// Mantiene detalle máximo y fuerza rebuild periódico para que el mesh refleje el crecimiento.
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
        if prec.0 < 1.0 {
            prec.0 = 1.0;
        }
    }
    // Rebuild periódico: cada tick fuerza re-inferencia para reflejar crecimiento.
    for entity in &rebuild_q {
        commands.entity(entity).insert(
            crate::worldgen::shape_inference::PendingGrowthMorphRebuild,
        );
    }
}

/// Telemetría de diagnóstico: traza la cadena completa de inferencia de la rosa.
pub fn debug_rosa_inference_chain_system(
    q: Query<(
        Entity,
        Option<&LifecycleStageCache>,
        Option<&GrowthBudget>,
        Option<&InferenceProfile>,
        Option<&CapabilitySet>,
        Option<&EnergyVisual>,
        Option<&Materialized>,
        Option<&crate::layers::BaseEnergy>,
        Option<&crate::rendering::quantized_color::QuantizedPrecision>,
        Option<&crate::worldgen::shape_inference::ShapeInferred>,
    ), With<RosaLifecycleFocus>>,
) {
    for (entity, stage, growth, profile, caps, visual, mat, energy, prec, shape_inf) in &q {
        let stage_str = stage.map(|s| format!("{:?} t={}", s.stage, s.ticks_in_stage)).unwrap_or("NONE".into());
        let growth_str = growth.map(|g| format!("bio={:.3} eff={:.2}", g.biomass_available, g.efficiency)).unwrap_or("NONE".into());
        let profile_str = profile.map(|p| format!("g={:.2} b={:.2}", p.growth_bias, p.branching_bias)).unwrap_or("NONE".into());
        let caps_str = caps.map(|c| format!("flags=0x{:02X} grow={} branch={} repro={}", c.flags, c.can_grow(), c.flags & CapabilitySet::BRANCH != 0, c.can_reproduce())).unwrap_or("NONE".into());
        let visual_str = if visual.is_some() { "YES" } else { "NONE" };
        let mat_str = mat.map(|m| format!("({},{})", m.cell_x, m.cell_y)).unwrap_or("NONE".into());
        let qe_str = energy.map(|e| format!("{:.0}", e.qe())).unwrap_or("?".into());
        let prec_str = prec.map(|p| format!("{:.2}", p.0)).unwrap_or("NONE".into());
        let shape_str = if shape_inf.is_some() { "YES" } else { "NO" };

        info!(
            "[ROSA {entity:?}] stage={stage_str} | growth={growth_str} | profile={profile_str} | caps={caps_str} | visual={visual_str} | mat={mat_str} | qe={qe_str} | prec={prec_str} | shape_inferred={shape_str}"
        );
    }
}

/// Telemetría flora_*.
pub fn debug_botanical_seed_system(
    sim_elapsed: Option<Res<SimulationElapsed>>,
    q: Query<(
        &Name,
        &BaseEnergy,
        &MatterCoherence,
        &SpatialVolume,
        Option<&GrowthBudget>,
        Option<&InferenceProfile>,
    )>,
) {
    let sim_secs = sim_elapsed.map(|e| e.secs).unwrap_or(0.0);
    let bio_months = (sim_secs * DEMO_BIO_SECS_PER_SIM_SEC) / SECONDS_PER_MONTH;

    for (name, energy, matter, volume, budget_opt, profile_opt) in &q {
        if !name.as_str().starts_with("flora_") {
            continue;
        }
        let budget_str = if let Some(b) = budget_opt {
            format!("Bio={:.3} Eff={:.2}", b.biomass_available, b.efficiency)
        } else {
            "None".into()
        };
        let profile_str = if let Some(p) = profile_opt {
            format!("g={:.1} b={:.1} r={:.1}", p.growth_bias, p.branching_bias, p.resilience)
        } else {
            "None".into()
        };
        info!(
            "[{}] t_bio={:.1}m | QE={:.0} | Bonds={:.0} | Rad={:.3} | {} | {}",
            name.as_str(),
            bio_months,
            energy.qe,
            matter.bond_energy_eb,
            volume.radius,
            budget_str,
            profile_str,
        );
    }
}

#[cfg(test)]
mod demo_bio_clock_tests {
    use super::{DEMO_ALLOMETRIC_GROWTH_MULTIPLIER, DEMO_BIO_SECS_PER_SIM_SEC, SECONDS_PER_MONTH};

    #[test]
    fn t_bio_one_sim_second_is_one_hour() {
        let bio_hours = DEMO_BIO_SECS_PER_SIM_SEC / (60.0 * 60.0);
        assert!(
            (bio_hours - 1.0).abs() < 1e-2,
            "esperado ~1 hora/s de sim, got {bio_hours}"
        );
    }

    #[test]
    fn allometric_turbo_is_one_sixtieth_of_bio_secs_per_sim_sec() {
        let scaled = DEMO_ALLOMETRIC_GROWTH_MULTIPLIER * 60.0;
        let rel = (scaled - DEMO_BIO_SECS_PER_SIM_SEC).abs() / DEMO_BIO_SECS_PER_SIM_SEC;
        assert!(
            rel < 1e-5,
            "×60 del turbo alométrico debe igualar el reloj bio, rel_err={rel}"
        );
    }
}
