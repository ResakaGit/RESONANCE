use std::collections::HashMap;

use bevy::prelude::*;
use bevy::text::{Font, JustifyText, TextColor, TextFont, TextLayout};
#[cfg(feature = "bridge_optimizer")]
use bevy::ui::{BackgroundColor, Node, PositionType, Val};

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::ElementId;
#[cfg(feature = "bridge_optimizer")]
use crate::bridge::context_fill::BridgePhaseState;
#[cfg(feature = "bridge_optimizer")]
use crate::bridge::metrics::{BridgeMetricsSummary, hit_rate_quality_prefix};
use crate::layers::{BaseEnergy, OscillatorySignature, SpatialVolume};
use crate::runtime_platform::camera_controller_3d::{
    CameraRigTarget, MobaCameraBounds, MobaCameraConfig, MobaCameraState,
};
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;
use crate::simulation::Phase;
use crate::simulation::states::{GameState, PlayState};
use crate::worldgen::shape_color_inference_system;
use std::time::Duration;
use bevy::time::common_conditions::on_timer;

use crate::world::{
    DemoCloudSpawnerState, Scoreboard, ROUND_WORLD_ROSA_SLUG,
    demo_cloud_context_spawn_system, demo_cloud_motion_system, spawn_demo_clouds_startup_system,
    enforce_rosa_focus_system, enforce_round_world_rosa_focus_system,
    pin_rosa_lod_focus_system, round_world_rosa_pin_lod_focus_for_inference_system,
    spawn_demo_level_startup_system,
    spawn_round_world_rosa_startup_system, stabilize_rosa_energy_system,
    stabilize_rosa_growth_system, stabilize_round_world_rosa_energy_system,
};
use crate::worldgen::{ActiveMapName, Materialized};
use crate::worldgen::systems::startup::mark_play_state_active_system;

const COMPOUND_RING_RADIUS_MULT: f32 = 1.18;
const COMPOUND_RING_COLOR_DARKEN: f32 = 0.55;
const SEED_DEBUG_RING_RADIUS_MULT: f32 = 1.4;
const SEED_DEBUG_RING_COLOR: Color = Color::srgb(1.0, 0.2, 0.9);
/// Wireframe aproximado de esfera (segmentos); subir en entidades grandes.
const DEBUG_GIZMO_SPHERE_RESOLUTION: u32 = 36;
// `TextFont.font_size` no escala 1:1 con el radio del gizmo (son unidades distintas),
// así que calibramos con un multiplicador chico + clamp.
// Calibración: evitar glyph atlas demasiado chico (pixelado). Ajuste ~50% del tuning original.
const LABEL_FONT_MULT: f32 = 7.0;
const LABEL_FONT_MIN: f32 = 4.5;
const LABEL_FONT_MAX: f32 = 14.0;
// Reducimos el tamaño *percibido* escalando el Transform del texto.
// Esto mejora calidad (el atlas rasteriza a un tamaño mayor) sin volver a
// caer en el caso de pixelado por glyph atlas demasiado chico.
// Nuevo objetivo: bajar otro 50% el tamaño percibido manteniendo calidad.
const LABEL_TEXT_SCALE: f32 = 0.18;
const LABEL_Z_OFFSET: f32 = 10.0;

fn active_map_is_round_world_rosa(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == ROUND_WORLD_ROSA_SLUG)
}

/// Para demos narrativas (rosa), ocultamos overlays de debug ruidosos.
fn active_map_is_not_round_world_rosa(active: Option<Res<ActiveMapName>>) -> bool {
    active.map(|a| a.0 != ROUND_WORLD_ROSA_SLUG).unwrap_or(true)
}

/// Rosa lifecycle + nubes startup (no aplica a round_world_rosa).
fn active_map_is_default_flora_demo(active: Option<Res<ActiveMapName>>) -> bool {
    active
        .map(|a| a.0 != ROUND_WORLD_ROSA_SLUG)
        .unwrap_or(true)
}

/// Plugin de debug: gizmos, etiquetas elementales, scoreboard; demo mínima en Startup.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // `spawn_demo_*` usa `CameraRigTarget` + estado MOBA aunque `Camera3dPlugin` no corra (legacy2d).
        app.init_resource::<CameraRigTarget>()
            .init_resource::<MobaCameraState>()
            .init_resource::<MobaCameraConfig>()
            .init_resource::<MobaCameraBounds>()
            .init_resource::<DemoCloudSpawnerState>();

        app.add_systems(
            Startup,
            (
                spawn_round_world_rosa_startup_system
                    .after(mark_play_state_active_system)
                    .run_if(active_map_is_round_world_rosa),
                spawn_demo_level_startup_system
                    .after(mark_play_state_active_system)
                    .run_if(active_map_is_default_flora_demo),
                spawn_demo_clouds_startup_system
                    .after(spawn_demo_level_startup_system)
                    .run_if(active_map_is_default_flora_demo),
            ),
        );
        app.add_systems(
            Update,
            (
                // Rosa lifecycle (default): LOD focus, visibility filter, energy stabilizer.
                pin_rosa_lod_focus_system
                    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
                    .run_if(active_map_is_default_flora_demo)
                    .before(shape_color_inference_system),
                enforce_rosa_focus_system.run_if(active_map_is_default_flora_demo),
                stabilize_rosa_energy_system.run_if(active_map_is_default_flora_demo),
                // Round world rosa (dedicated planet demo).
                round_world_rosa_pin_lod_focus_for_inference_system
                    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
                    .run_if(active_map_is_round_world_rosa)
                    .before(shape_color_inference_system),
                enforce_round_world_rosa_focus_system.run_if(active_map_is_round_world_rosa),
                stabilize_round_world_rosa_energy_system.run_if(active_map_is_round_world_rosa),
                debug_seed_ring_round_world_system.run_if(active_map_is_round_world_rosa),
                debug_scoreboard_system,
                crate::world::demo_level::debug_rosa_inference_chain_system
                    .run_if(on_timer(Duration::from_secs_f32(2.0)))
                    .run_if(active_map_is_default_flora_demo),
                crate::world::demo_level::debug_botanical_seed_system
                    .run_if(on_timer(Duration::from_secs_f32(1.5))),
            ),
        );
        app.add_systems(
            FixedUpdate,
            (
                demo_cloud_context_spawn_system
                    .run_if(on_timer(Duration::from_secs_f32(1.0)))
                    .run_if(active_map_is_default_flora_demo),
                demo_cloud_motion_system.run_if(active_map_is_default_flora_demo),
            )
                .chain()
                .in_set(Phase::ThermodynamicLayer)
                .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
        // Rosa demo: fuerza biomasa + stage + detalle DESPUÉS del pipeline metabólico.
        app.add_systems(
            FixedUpdate,
            stabilize_rosa_growth_system
                .after(Phase::MorphologicalLayer)
                .run_if(active_map_is_default_flora_demo)
                .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
        #[cfg(feature = "bridge_optimizer")]
        app.add_systems(
            Update,
            (
                ensure_bridge_optimizer_debug_hud_system,
                sync_bridge_optimizer_debug_hud_system,
            )
                .chain(),
        );
    }
}

/// Ring de lectura: en `round_world_rosa` marcamos solo `flora_seed`.
fn debug_seed_ring_round_world_system(
    mut gizmos: Gizmos,
    query: Query<(&Name, &Transform, &SpatialVolume), Without<V6RuntimeEntity>>,
) {
    for (name, transform, volume) in &query {
        if !name.as_str().starts_with("flora_seed") {
            continue;
        }
        let iso = Isometry3d::from_translation(transform.translation);
        gizmos
            .sphere(
                iso,
                volume.radius * SEED_DEBUG_RING_RADIUS_MULT,
                SEED_DEBUG_RING_COLOR,
            )
            .resolution(DEBUG_GIZMO_SPHERE_RESOLUTION.saturating_add(10));
    }
}

#[derive(Resource, Default)]
struct DebugElementLabelCache {
    label_by_source: HashMap<Entity, Entity>,
    font: Option<Handle<Font>>,
}

#[derive(Component)]
struct DebugElementLabelText;

#[cfg(feature = "bridge_optimizer")]
#[derive(Component)]
struct BridgeOptimizerDebugHud;

#[cfg(feature = "bridge_optimizer")]
fn ensure_bridge_optimizer_debug_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hud_query: Query<Entity, With<BridgeOptimizerDebugHud>>,
) {
    if hud_query.iter().next().is_some() {
        return;
    }

    commands.spawn((
        BridgeOptimizerDebugHud,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            max_width: Val::Px(320.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.06, 0.04, 0.78)),
        Text::new("Bridge Optimizer"),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgba(0.85, 0.98, 0.88, 1.0)),
    ));
}

#[cfg(feature = "bridge_optimizer")]
fn sync_bridge_optimizer_debug_hud_system(
    summary: Option<Res<BridgeMetricsSummary>>,
    phase_state: Option<Res<BridgePhaseState>>,
    mut last_phase: Local<Option<crate::bridge::context_fill::BridgePhase>>,
    mut hud_query: Query<&mut Text, With<BridgeOptimizerDebugHud>>,
) {
    let phase_now = phase_state.as_ref().map(|p| p.phase);
    let phase_moved = phase_now != *last_phase;
    if phase_moved {
        *last_phase = phase_now;
    }

    let Some(summary) = summary else {
        let out = "BRIDGE OPT\n(BridgeConfigPlugin / métricas no cargadas)\n";
        for mut text in &mut hud_query {
            if text.0 != out {
                text.0 = out.into();
            }
        }
        return;
    };

    if !summary.is_changed() && !phase_moved {
        return;
    }

    let phase_str = phase_now
        .map(|p| match p {
            crate::bridge::context_fill::BridgePhase::Warmup => "Warmup",
            crate::bridge::context_fill::BridgePhase::Filling => "Filling",
            crate::bridge::context_fill::BridgePhase::Active => "Active",
        })
        .unwrap_or("n/a");

    let mut out = format!("BRIDGE OPT\nphase: {phase_str}\n");
    if summary.layers.is_empty() {
        out.push_str("(métricas cada ~60 ticks sim)\n");
    }
    for row in &summary.layers {
        let pfx = hit_rate_quality_prefix(row.hit_rate);
        out.push_str(&format!(
            "{pfx}{}: hit {:.0}% fill {:.0}%",
            row.name,
            row.hit_rate * 100.0,
            row.fill_level * 100.0
        ));
        if !row.recommendations.is_empty() {
            out.push_str(&format!("  | {}", row.recommendations.join("; ")));
        }
        out.push('\n');
    }

    for mut text in &mut hud_query {
        if text.0 != out {
            text.0 = out.clone();
        }
    }
}

fn debug_gizmos_system(
    mut gizmos: Gizmos,
    almanac: Res<AlchemicalAlmanac>,
    query: Query<
        (
            Option<&Name>,
            &Transform,
            &SpatialVolume,
            Option<&OscillatorySignature>,
            Option<&ElementId>,
            Option<&BaseEnergy>,
        ),
        Without<V6RuntimeEntity>,
    >,
) {
    for (name_opt, transform, volume, signature_opt, element_id_opt, _energy_opt) in query.iter() {
        let pos = transform.translation;
        let iso = Isometry3d::from_translation(pos);

        let color = if let Some(element_id) = element_id_opt {
            if let Some(def) = almanac.get(*element_id) {
                let base = Color::srgb(def.color.0, def.color.1, def.color.2);
                let radius = volume.radius;

                // Compuestos: esfera exterior más oscura (mismo criterio que el anillo 2D legacy).
                if def.is_compound {
                    gizmos
                        .sphere(
                            iso,
                            radius * COMPOUND_RING_RADIUS_MULT,
                            Color::srgb(
                                def.color.0 * COMPOUND_RING_COLOR_DARKEN,
                                def.color.1 * COMPOUND_RING_COLOR_DARKEN,
                                def.color.2 * COMPOUND_RING_COLOR_DARKEN,
                            ),
                        )
                        .resolution(DEBUG_GIZMO_SPHERE_RESOLUTION.saturating_add(8));
                }

                base
            } else {
                Color::srgb(0.5, 0.5, 0.5)
            }
        } else {
            if signature_opt.is_some() {
                Color::srgb(0.5, 0.5, 0.5)
            } else {
                Color::WHITE
            }
        };

        gizmos
            .sphere(iso, volume.radius, color)
            .resolution(DEBUG_GIZMO_SPHERE_RESOLUTION);

        // Resaltado dedicado: solo semillas botánicas.
        if name_opt.is_some_and(|n| n.as_str().starts_with("flora_seed")) {
            gizmos
                .sphere(
                    iso,
                    volume.radius * SEED_DEBUG_RING_RADIUS_MULT,
                    SEED_DEBUG_RING_COLOR,
                )
                .resolution(DEBUG_GIZMO_SPHERE_RESOLUTION.saturating_add(10));
        }
    }
}

fn debug_element_labels_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    almanac: Res<AlchemicalAlmanac>,
    mut cache: Local<DebugElementLabelCache>,
    query_sources: Query<
        (
            Entity,
            &Transform,
            &SpatialVolume,
            &ElementId,
            &OscillatorySignature,
            Option<&BaseEnergy>,
        ),
        (Without<DebugElementLabelText>, Without<Materialized>),
    >,
    mut query_texts: Query<
        (&mut Text2d, &mut Transform, &mut TextFont),
        With<DebugElementLabelText>,
    >,
) {
    // Carga de fuente 1 vez. Si no tenés el archivo en `fonts/`, el texto puede no renderizar,
    // pero la simulación no rompe (demo visual).
    if cache.font.is_none() {
        cache.font = Some(asset_server.load("fonts/FiraSans-Bold.ttf"));
    }
    let font_handle = cache.font.as_ref().unwrap().clone();

    // Limpieza: si la entidad source ya no existe, despawn el texto.
    let sources_set: HashMap<Entity, ()> = query_sources
        .iter()
        .map(|(e, _, _, _, _, _)| (e, ()))
        .collect();
    cache.label_by_source.retain(|src, text_ent| {
        if sources_set.contains_key(src) {
            true
        } else {
            commands.entity(*text_ent).despawn_recursive();
            false
        }
    });

    for (source_ent, transform, volume, element_id, signature, _energy_opt) in &query_sources {
        let label_ent = if let Some(label_ent) = cache.label_by_source.get(&source_ent) {
            *label_ent
        } else {
            let def = almanac.get(*element_id);
            let symbol = def.map(|d| d.symbol.as_str()).unwrap_or("?");
            let freq = signature.frequency_hz();
            let compound_flag = def.map(|d| d.is_compound).unwrap_or(false);

            let mut text = format!("{symbol}\n{freq:.0}Hz");
            if compound_flag {
                text.push_str("\nC");
            }

            let text_color = def
                .map(|d| TextColor(Color::srgb(d.color.0, d.color.1, d.color.2)))
                .unwrap_or_else(|| TextColor(Color::WHITE));

            // Redondeo para estabilizar el rasterizado del glyph atlas.
            let font_size = (volume.radius * LABEL_FONT_MULT)
                .clamp(LABEL_FONT_MIN, LABEL_FONT_MAX)
                .round();

            let text_ent = commands
                .spawn((
                    DebugElementLabelText,
                    Text2d::new(text),
                    TextFont {
                        font: font_handle.clone(),
                        font_size,
                        ..default()
                    },
                    text_color,
                    TextLayout::new_with_justify(JustifyText::Left),
                    Transform {
                        translation: transform.translation + Vec3::new(0.0, 0.0, LABEL_Z_OFFSET),
                        scale: Vec3::splat(LABEL_TEXT_SCALE),
                        ..default()
                    },
                ))
                .id();

            cache.label_by_source.insert(source_ent, text_ent);
            text_ent
        };

        if let Ok((mut text, mut label_transform, mut label_font)) = query_texts.get_mut(label_ent)
        {
            let def = almanac.get(*element_id);
            let symbol = def.map(|d| d.symbol.as_str()).unwrap_or("?");
            let freq = signature.frequency_hz();
            let compound_flag = def.map(|d| d.is_compound).unwrap_or(false);

            let mut new_text = format!("{symbol}\n{freq:.0}Hz");
            if compound_flag {
                new_text.push_str("\nC");
            }

            if text.0 != new_text {
                text.0 = new_text;
            }

            // Mantener proporcionalidad: recalcular font_size en cada frame
            label_font.font_size = (volume.radius * LABEL_FONT_MULT)
                .clamp(LABEL_FONT_MIN, LABEL_FONT_MAX)
                .round();
            label_transform.translation =
                transform.translation + Vec3::new(0.0, 0.0, LABEL_Z_OFFSET);
        }
    }
}

fn debug_scoreboard_system(scoreboard: Res<Scoreboard>, mut last_printed: Local<(u32, u32)>) {
    if scoreboard.red_points != last_printed.0 || scoreboard.blue_points != last_printed.1 {
        info!(
            "SCOREBOARD — Red: {} | Blue: {} (Kills R:{} B:{})",
            scoreboard.red_points,
            scoreboard.blue_points,
            scoreboard.red_kills,
            scoreboard.blue_kills
        );
        *last_printed = (scoreboard.red_points, scoreboard.blue_points);
    }
}
