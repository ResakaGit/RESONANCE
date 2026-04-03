use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
#[cfg(feature = "bridge_optimizer")]
use bevy::ui::{BackgroundColor, Node, PositionType, Val};
use std::time::Duration;

#[cfg(feature = "bridge_optimizer")]
use crate::bridge::context_fill::BridgePhaseState;
#[cfg(feature = "bridge_optimizer")]
use crate::bridge::metrics::{BridgeMetricsSummary, hit_rate_quality_prefix};
use crate::layers::SpatialVolume;
use crate::runtime_platform::camera_controller_3d::{
    CameraRigTarget, MobaCameraBounds, MobaCameraConfig, MobaCameraState,
};
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;
use crate::simulation::Phase;
use crate::simulation::states::{GameState, PlayState};
use crate::worldgen::shape_color_inference_system;

use crate::simulation::lifecycle::transition_to_active_system;
use crate::world::{
    COMPETITION_ARENA_SLUG, DEMO_ANIMAL_SLUG, DEMO_CELULA_SLUG, DEMO_PLANTA_SLUG, DEMO_VIRUS_SLUG,
    DemoCloudSpawnerState, INFERRED_WORLD_SLUG, ROUND_WORLD_ROSA_SLUG, SIGNAL_DEMO_SLUG,
    Scoreboard, demo_cloud_context_spawn_system, demo_cloud_motion_system,
    enforce_rosa_focus_system, enforce_round_world_rosa_focus_system,
    ensure_demo_metrics_hud_system, pin_rosa_lod_focus_system,
    round_world_rosa_pin_lod_focus_for_inference_system, spawn_competition_demo_startup_system,
    spawn_demo_animal_startup_system, spawn_demo_celula_startup_system,
    spawn_demo_clouds_startup_system, spawn_demo_level_startup_system,
    spawn_demo_planta_startup_system, spawn_demo_virus_startup_system,
    spawn_inferred_world_startup_system, spawn_round_world_rosa_startup_system,
    spawn_signal_demo_startup_system, stabilize_rosa_growth_system,
    stabilize_round_world_rosa_energy_system, sync_demo_metrics_hud_system,
};
use crate::worldgen::ActiveMapName;

const SEED_DEBUG_RING_RADIUS_MULT: f32 = 1.4;
const SEED_DEBUG_RING_COLOR: Color = Color::srgb(1.0, 0.2, 0.9);
const DEBUG_GIZMO_SPHERE_RESOLUTION: u32 = 36;

fn active_map_is_round_world_rosa(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == ROUND_WORLD_ROSA_SLUG)
}

fn active_map_is_competition_arena(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == COMPETITION_ARENA_SLUG)
}

fn active_map_is_inferred_world(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == INFERRED_WORLD_SLUG)
}

fn active_map_is_signal_demo(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == SIGNAL_DEMO_SLUG)
}

fn active_map_is_demo_celula(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == DEMO_CELULA_SLUG)
}

fn active_map_is_demo_virus(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == DEMO_VIRUS_SLUG)
}

fn active_map_is_demo_planta(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == DEMO_PLANTA_SLUG)
}

fn active_map_is_demo_animal(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| a.0 == DEMO_ANIMAL_SLUG)
}

fn active_map_is_catalog_demo(active: Option<Res<ActiveMapName>>) -> bool {
    active.is_some_and(|a| {
        a.0 == DEMO_CELULA_SLUG
            || a.0 == DEMO_VIRUS_SLUG
            || a.0 == DEMO_PLANTA_SLUG
            || a.0 == DEMO_ANIMAL_SLUG
    })
}

fn active_map_is_default_flora_demo(active: Option<Res<ActiveMapName>>) -> bool {
    active
        .map(|a| {
            a.0 != ROUND_WORLD_ROSA_SLUG
                && a.0 != COMPETITION_ARENA_SLUG
                && a.0 != INFERRED_WORLD_SLUG
                && a.0 != SIGNAL_DEMO_SLUG
                && a.0 != DEMO_CELULA_SLUG
                && a.0 != DEMO_VIRUS_SLUG
                && a.0 != DEMO_PLANTA_SLUG
                && a.0 != DEMO_ANIMAL_SLUG
        })
        .unwrap_or(true)
}

/// Plugin de demo: startup de rosa + sistemas runtime mínimos.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRigTarget>()
            .init_resource::<MobaCameraState>()
            .init_resource::<MobaCameraConfig>()
            .init_resource::<MobaCameraBounds>()
            .init_resource::<DemoCloudSpawnerState>();

        // ── Startup ──
        app.add_systems(
            Startup,
            (
                spawn_round_world_rosa_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_round_world_rosa),
                spawn_demo_level_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_default_flora_demo),
                spawn_demo_clouds_startup_system
                    .after(spawn_demo_level_startup_system)
                    .run_if(active_map_is_default_flora_demo),
                spawn_competition_demo_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_competition_arena),
                spawn_inferred_world_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_inferred_world),
                spawn_signal_demo_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_signal_demo),
                spawn_demo_celula_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_demo_celula),
                spawn_demo_virus_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_demo_virus),
                spawn_demo_planta_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_demo_planta),
                spawn_demo_animal_startup_system
                    .after(transition_to_active_system)
                    .run_if(active_map_is_demo_animal),
            ),
        );

        // ── Update ──
        app.add_systems(
            Update,
            (
                // Default rosa lifecycle.
                pin_rosa_lod_focus_system
                    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
                    .run_if(active_map_is_default_flora_demo)
                    .before(shape_color_inference_system),
                enforce_rosa_focus_system.run_if(active_map_is_default_flora_demo),
                // Round world rosa.
                round_world_rosa_pin_lod_focus_for_inference_system
                    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active)))
                    .run_if(active_map_is_round_world_rosa)
                    .before(shape_color_inference_system),
                enforce_round_world_rosa_focus_system.run_if(active_map_is_round_world_rosa),
                stabilize_round_world_rosa_energy_system.run_if(active_map_is_round_world_rosa),
                debug_seed_ring_round_world_system.run_if(active_map_is_round_world_rosa),
                // Telemetría.
                debug_scoreboard_system,
                crate::world::demo_level::debug_botanical_seed_system
                    .run_if(on_timer(Duration::from_secs_f32(2.0))),
                // Catalog demo metrics HUD.
                ensure_demo_metrics_hud_system.run_if(active_map_is_catalog_demo),
                sync_demo_metrics_hud_system.run_if(active_map_is_catalog_demo),
            ),
        );

        // ── FixedUpdate ──
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
        app.add_systems(
            FixedUpdate,
            stabilize_rosa_growth_system
                .after(Phase::MetabolicLayer)
                .before(Phase::MorphologicalLayer)
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

// ── Sistemas auxiliares (mínimos) ───────────────────────────────────────

fn debug_seed_ring_round_world_system(
    mut gizmos: Gizmos,
    query: Query<(&Name, &Transform, &SpatialVolume), Without<V6RuntimeEntity>>,
) {
    for (name, transform, volume) in &query {
        if !name.as_str().starts_with("flora_") {
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

fn debug_scoreboard_system(scoreboard: Res<Scoreboard>, mut last: Local<(u32, u32)>) {
    if scoreboard.red_points != last.0 || scoreboard.blue_points != last.1 {
        info!(
            "SCOREBOARD — Red: {} | Blue: {}",
            scoreboard.red_points, scoreboard.blue_points
        );
        *last = (scoreboard.red_points, scoreboard.blue_points);
    }
}

// ── Bridge optimizer HUD (feature-gated) ────────────────────────────────

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
        for mut text in &mut hud_query {
            let out = "BRIDGE OPT\n(no metrics)\n";
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
    for row in &summary.layers {
        let pfx = hit_rate_quality_prefix(row.hit_rate);
        out.push_str(&format!(
            "{pfx}{}: hit {:.0}% fill {:.0}%\n",
            row.name,
            row.hit_rate * 100.0,
            row.fill_level * 100.0
        ));
    }
    for mut text in &mut hud_query {
        if text.0 != out {
            text.0 = out.clone();
        }
    }
}
