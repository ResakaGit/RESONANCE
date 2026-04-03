//! Demo **mundo redondo + atmósfera (solo visual)** y una rosa en el polo.
//!
//! - **Simulación:** plano XZ en [`DEFAULT_SIM_STANDING_Y`](crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y)
//!   (V7, núcleos, morfología igual que otros mapas).
//! - **Render:** esfera “planeta” bajo el polo, cascarón atmosférico semitransparente.
//! - **Forma de la rosa (full3d):** misma tubería que celdas V7 — [`Materialized`] en la celda del campo bajo `(0,0)` en XZ,
//!   `shape_color_inference_system` + `growth_morphology_system` infieren GF1 desde gradiente de `EnergyFieldGrid` y `GrowthBudget`.
//! - **Tuning solo-demo (sin tocar blueprint):** foco LOD anclado al centro de esa celda (rebuild GF1 en banda Near),
//!   `WorldgenPerfSettings` con cadencia de malla Mid/Far = 1, `EnvScenarioSnapshot` benévolo para `EffectiveOrganViability`,
//!   `LifecycleStageCache` + `InferenceProfile` explícitos en la entidad foco (EPI3 / órganos desde el primer frame útil).
//!
//! Mapa: [`ROUND_WORLD_ROSA_MAP_SLUG`](crate::worldgen::map_config::ROUND_WORLD_ROSA_MAP_SLUG) → `assets/maps/round_world_rosa.ron`.

use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::IdGenerator;
use crate::blueprint::constants::QE_MIN_EXISTENCE;
use crate::entities::archetypes::spawn_rosa;
use crate::layers::{
    BaseEnergy, CapabilitySet, GrowthBudget, InferenceProfile, LifecycleStage, LifecycleStageCache,
};
use crate::rendering::quantized_color::QuantizedPrecision;
use crate::runtime_platform::camera_controller_3d::{
    CameraRigTarget, MobaCameraConfig, MobaCameraState,
};
use crate::runtime_platform::compat_2d3d::{RenderCompatProfile, SimWorldTransformParams};
use crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y;
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;
use crate::simulation::allometric_growth::AllometricGrowthTimeScale;
use crate::simulation::env_scenario::EnvScenarioSnapshot;
use crate::worldgen::systems::performance::{WorldgenLodContext, WorldgenPerfSettings};
use crate::worldgen::systems::startup::StartupNucleus;
use crate::worldgen::{EnergyFieldGrid, Materialized, WorldArchetype, materialize_cell_at_time};

/// Slug de mapa / [`crate::worldgen::ActiveMapName`] (fuente única: `map_config`).
pub use crate::worldgen::map_config::ROUND_WORLD_ROSA_MAP_SLUG as ROUND_WORLD_ROSA_SLUG;

const SECONDS_PER_MONTH: f32 = 30.0 * 24.0 * 60.0 * 60.0;
const DEMO_BIO_SECS_PER_SIM_SEC: f32 = 3.0 * SECONDS_PER_MONTH;
const DEMO_ALLOMETRIC_GROWTH_MULTIPLIER: f32 = DEMO_BIO_SECS_PER_SIM_SEC / 60.0;
const DEMO_CLOSE_ZOOM_HORIZONTAL: f32 = 1.0;
const DEMO_MIN_ZOOM_HORIZONTAL: f32 = 0.35;
/// Piso de qe (múltiplo de existencia mínima): rescate suave si fotosíntesis/Liebig caen; no modula forma a propósito.
const ROSA_FOCUS_QE_FLOOR: f32 = QE_MIN_EXISTENCE * 8000.0;
const ROSA_FOCUS_QE_INJECT_CAP: f32 = 6.0;

/// ρ = 1 → máximo detalle permitido en `derive_geometry_influence` (LOD GF1 desde inferencia).
const POLE_ROSA_SHAPE_PRECISION: f32 = 1.0;

/// Perfil de inferencia EA2+ ligeramente por encima del preset `ROSA` para manifest de órganos más rico (misma tubería pura).
const ROSA_DEMO_GROWTH_BIAS: f32 = 0.93;
const ROSA_DEMO_MOBILITY_BIAS: f32 = 0.0;
const ROSA_DEMO_BRANCHING_BIAS: f32 = 0.94;
const ROSA_DEMO_RESILIENCE: f32 = 0.52;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RoundWorldRosaFocus;

/// Acopla la rosa al mismo pipeline que `shape_inference` de celdas: gradiente V7 en la celda bajo el polo sim.
fn attach_round_world_rosa_field_shape(
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
    let field_archetype =
        materialize_cell_at_time(cell, almanac, interference_t, grid.cell_size, None)
            .map(|r| r.archetype)
            .unwrap_or(WorldArchetype::TerraSolid);

    commands.entity(rosa).insert((
        Name::new("flora_rosa_focus"),
        RoundWorldRosaFocus,
        Materialized {
            cell_x: cx as i32,
            cell_y: cy as i32,
            // Arquetipo desde interferencia V7 en la celda del polo (tinte paleta + coherencia con materialización).
            archetype: field_archetype,
        },
        QuantizedPrecision(POLE_ROSA_SHAPE_PRECISION),
        // Preserva preset ROSA en L0/L1/L4; añade fotosíntesis para Liebig + irradiancia real.
        CapabilitySet::new(
            CapabilitySet::GROW
                | CapabilitySet::BRANCH
                | CapabilitySet::ROOT
                | CapabilitySet::PHOTOSYNTH
                | CapabilitySet::REPRODUCE,
        ),
        // `resolve_organ_manifest` exige `Some(LifecycleStageCache)`; sin esto GF1 cae a tubo simple sin EPI3.
        LifecycleStageCache {
            stage: LifecycleStage::Reproductive,
            ticks_in_stage: 64,
            candidate_stage: None,
            candidate_ticks: 0,
        },
        GrowthBudget::new(3.0, 0, 0.9),
        InferenceProfile::new(
            ROSA_DEMO_GROWTH_BIAS,
            ROSA_DEMO_MOBILITY_BIAS,
            ROSA_DEMO_BRANCHING_BIAS,
            ROSA_DEMO_RESILIENCE,
        ),
    ));
    commands.entity(rosa).remove::<V6RuntimeEntity>();
}

/// Spawnea rosa en el origen del plano sim; en 3D añade planeta + atmósfera.
pub fn spawn_round_world_rosa_demo(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
    id_gen: &mut IdGenerator,
    profile: RenderCompatProfile,
) -> Entity {
    if !profile.enables_visual_3d() {
        commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(0.05))));
    }

    let layout = SimWorldTransformParams::from_profile(profile);
    spawn_rosa(commands, id_gen, Vec2::ZERO, &layout)
}

pub fn spawn_round_world_rosa_startup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    let rosa = spawn_round_world_rosa_demo(
        &mut commands,
        meshes.as_mut(),
        materials.as_mut(),
        &mut id_gen,
        *profile,
    );
    if profile.enables_visual_3d() {
        // Sandbox ambiental suave → `organ_viability_score` no aplasta el manifest en la única entidad viva.
        *env_snapshot = EnvScenarioSnapshot {
            food_density_t: 0.92,
            predation_pressure_t: 0.06,
            temperature_t: 0.55,
            medium_density_t: 0.55,
        };
        // Si el héroe/jugador está lejos del polo, LOD Mid/Far diezma rebuilds de malla; 1 = misma cadencia que Near.
        perf.shape_rebuild_mid_period = 1;
        perf.shape_rebuild_far_period = 1;

        if let Some(g) = grid.as_deref() {
            attach_round_world_rosa_field_shape(
                &mut commands,
                rosa,
                g,
                almanac.as_ref(),
                time.elapsed_secs(),
            );
        }
        // Modo microscopio: solo linaje flora_* visible.
        for (entity, name_opt) in &materialized_q {
            let is_flora = name_opt.is_some_and(|n| n.as_str().starts_with("flora_"));
            if entity != rosa && !is_flora {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
        // Ocultamos marcadores de núcleo para lectura de organismo a ~1m.
        for entity in &nuclei_q {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        camera_target.entity = Some(rosa);
        moba_config.zoom_min = DEMO_MIN_ZOOM_HORIZONTAL;
        moba_state.focus_xz = Vec2::ZERO;
        moba_state.focus_y = DEFAULT_SIM_STANDING_Y;
        moba_state.zoom_horizontal = DEMO_CLOSE_ZOOM_HORIZONTAL;
    } else {
        camera_target.entity = None;
    }
}

/// Ancla el foco LOD al centro de la celda bajo `(0,0)` en XZ para que `shape_rebuild_tick_active` trate el polo como Near
/// aunque exista `PlayerControlled` lejos (FixedUpdate ya refrescó foco desde el jugador).
pub fn round_world_rosa_pin_lod_focus_for_inference_system(
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

/// Filtro continuo de visibilidad para demo rosa: deja solo linaje `flora_*`.
pub fn enforce_round_world_rosa_focus_system(
    mut commands: Commands,
    materialized_q: Query<
        (
            Entity,
            Option<&Name>,
            Option<&Visibility>,
            Option<&RoundWorldRosaFocus>,
        ),
        With<Materialized>,
    >,
    nuclei_q: Query<(Entity, Option<&Visibility>), With<StartupNucleus>>,
) {
    for (entity, name_opt, vis_opt, focus_opt) in &materialized_q {
        let is_flora = name_opt.is_some_and(|n| n.as_str().starts_with("flora_"));
        let is_focus = focus_opt.is_some();
        if (!is_flora || !is_focus) && !matches!(vis_opt, Some(Visibility::Hidden)) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    for (entity, vis_opt) in &nuclei_q {
        if !matches!(vis_opt, Some(Visibility::Hidden)) {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
}

/// Rescate mínimo: no impone banda ni drena; si la sim baja el qe por debajo del piso, inyecta poco a poco.
pub fn stabilize_round_world_rosa_energy_system(
    mut q: Query<&mut BaseEnergy, With<RoundWorldRosaFocus>>,
) {
    for mut energy in &mut q {
        let qe = energy.qe();
        if qe < ROSA_FOCUS_QE_FLOOR {
            let deficit = ROSA_FOCUS_QE_FLOOR - qe;
            energy.inject(deficit.min(ROSA_FOCUS_QE_INJECT_CAP));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_matches_map_config() {
        assert_eq!(ROUND_WORLD_ROSA_SLUG, "round_world_rosa");
    }
}
