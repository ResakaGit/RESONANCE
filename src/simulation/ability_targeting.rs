//! Targeting MOBA (punto / estado) + tick de canalización hacia pending de grimorio.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::equations;
use crate::events::{AbilityCastEvent, GrimoireProjectileCastPending, GrimoireSelfBuffCastPending};
use crate::layers::{
    AbilityTarget, AlchemicalEngine, Channeling, Grimoire, SpatialVolume, TargetingMode,
    WillActuator,
};
use crate::runtime_platform::click_to_move::core::{GroundPlaneConfig, resolve_click_target_with};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::hud::{MinimapScreenRect, minimap_cursor_blocks_primary_pick};
use crate::simulation::PlayerControlled;
use crate::simulation::grimoire_enqueue::enqueue_grimoire_cast_intent;
use crate::simulation::time_compat::simulation_delta_secs;

/// Estado transitorio: esperando click de suelo / unidad.
#[derive(Debug, Clone)]
pub struct ActiveTargeting {
    pub caster: Entity,
    pub slot_index: usize,
    pub mode: TargetingMode,
}

#[derive(Resource, Default)]
pub struct TargetingState {
    pub active: Option<ActiveTargeting>,
}

/// Click confirma Point / Area / Unit (MVP: mismo raycast que click-to-move).
pub fn ability_point_target_pick_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    minimap_screen: Option<Res<MinimapScreenRect>>,
    layout: Res<SimWorldTransformParams>,
    mut targeting: ResMut<TargetingState>,
    mut cast_ev: EventWriter<AbilityCastEvent>,
    mut pending_proj: EventWriter<GrimoireProjectileCastPending>,
    mut pending_self: EventWriter<GrimoireSelfBuffCastPending>,
    heroes: Query<
        (
            Entity,
            &Transform,
            &SpatialVolume,
            &WillActuator,
            &Grimoire,
            &AlchemicalEngine,
        ),
        With<PlayerControlled>,
    >,
    almanac: Res<AlchemicalAlmanac>,
) {
    let Some(active) = targeting.active.as_ref() else {
        return;
    };
    let needs_point = matches!(
        active.mode,
        TargetingMode::PointTarget { .. }
            | TargetingMode::AreaTarget { .. }
            | TargetingMode::UnitTarget { .. }
    );
    if !needs_point {
        return;
    }

    let range = match &active.mode {
        TargetingMode::PointTarget { range } => *range,
        TargetingMode::AreaTarget { range, .. } => *range,
        TargetingMode::UnitTarget { range } => *range,
        _ => return,
    };

    let Some(window) = windows.iter().next() else {
        return;
    };
    let Some((camera, camera_tf)) = camera_q.iter().next() else {
        return;
    };

    let cursor = window.cursor_position();
    let block_minimap = minimap_screen
        .as_ref()
        .map(|s| minimap_cursor_blocks_primary_pick(cursor, s))
        .unwrap_or(false);
    let click = mouse.just_pressed(MouseButton::Left) && !block_minimap;
    let ground = GroundPlaneConfig {
        ground_y: layout.standing_y,
    };
    let hit_xz = resolve_click_target_with(click, cursor, ground, |c| {
        let ray = camera.viewport_to_world(camera_tf, c).ok()?;
        Some((ray.origin, ray.direction.as_vec3()))
    });
    let Some(hit) = hit_xz else {
        return;
    };

    let point3 = if layout.use_xz_ground {
        Vec3::new(hit.x, layout.standing_y, hit.y)
    } else {
        Vec3::new(hit.x, hit.y, 0.0)
    };

    let caster_ent = active.caster;
    let slot_index = active.slot_index;
    let Ok((_, transform, vol, actuator, grimoire, engine)) = heroes.get(caster_ent) else {
        if targeting.active.is_some() {
            targeting.active = None;
        }
        return;
    };

    let caster_plane = sim_plane_pos(transform.translation, layout.use_xz_ground);
    let target_plane = sim_plane_pos(point3, layout.use_xz_ground);
    if !equations::ability_point_in_cast_range(caster_plane, target_plane, range) {
        return;
    }

    if targeting.active.is_some() {
        targeting.active = None;
    }

    let _ = enqueue_grimoire_cast_intent(
        caster_ent,
        slot_index,
        grimoire,
        engine,
        transform,
        vol,
        actuator,
        AbilityTarget::Point(point3),
        &layout,
        &almanac,
        &mut cast_ev,
        &mut pending_proj,
        &mut pending_self,
    );
}

/// Al terminar `Channeling`, encola el pending (misma ruta que Input).
pub fn channeling_grimoire_emit_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut commands: Commands,
    mut done: Local<Vec<(Entity, Channeling)>>,
    mut ch_q: Query<(Entity, &mut Channeling)>,
    heroes: Query<
        (
            &Transform,
            &SpatialVolume,
            &WillActuator,
            &Grimoire,
            &AlchemicalEngine,
        ),
        With<PlayerControlled>,
    >,
    layout: Res<SimWorldTransformParams>,
    almanac: Res<AlchemicalAlmanac>,
    mut cast_ev: EventWriter<AbilityCastEvent>,
    mut pending_proj: EventWriter<GrimoireProjectileCastPending>,
    mut pending_self: EventWriter<GrimoireSelfBuffCastPending>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    done.clear();

    for (entity, mut ch) in &mut ch_q {
        ch.remaining_secs -= dt;
        if ch.remaining_secs <= 0.0 {
            done.push((entity, ch.clone()));
        }
    }

    for (entity, ch) in done.drain(..) {
        commands.entity(entity).remove::<Channeling>();
        let Ok((transform, vol, actuator, grimoire, engine)) = heroes.get(entity) else {
            continue;
        };
        let _ = enqueue_grimoire_cast_intent(
            entity,
            ch.slot_index,
            grimoire,
            engine,
            transform,
            vol,
            actuator,
            ch.target,
            &layout,
            &almanac,
            &mut cast_ev,
            &mut pending_proj,
            &mut pending_self,
        );
    }
}
