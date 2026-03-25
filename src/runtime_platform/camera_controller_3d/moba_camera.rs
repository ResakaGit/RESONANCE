//! Cámara MOBA: pan libre, lock al héroe, zoom y edge scroll.
//!
//! Matemática pura en `crate::blueprint::equations` (`moba_*`).

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::blueprint::equations::{
    moba_camera_offset_from_pitch, moba_clamp_focus_xz, moba_zoom_horizontal_delta,
};
use crate::runtime_platform::core_math_agnostic::normalize_or_zero;
use crate::runtime_platform::intent_projection_3d::CameraBasisForSim;
use crate::simulation::PlayerControlled;

use super::CameraRigTarget;
use super::constants::{DEFAULT_MOBA_PITCH_DEG, DEFAULT_MOBA_ZOOM_HORIZONTAL};
use super::legacy_orbital::resolve_camera_target_entity;

/// Modo de control de cámara (MOBA estándar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Free,
    Locked { target: Entity },
}

/// Configuración inmutable de comportamiento (tuning).
#[derive(Resource, Debug, Clone)]
pub struct MobaCameraConfig {
    pub mode: CameraMode,
    pub pan_speed: f32,
    pub edge_scroll_speed: f32,
    pub edge_scroll_margin: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub zoom_speed: f32,
    /// Ángulo cámara→foco respecto al plano horizontal (grados).
    pub pitch_deg_from_horizontal: f32,
}

impl Default for MobaCameraConfig {
    fn default() -> Self {
        Self {
            mode: CameraMode::Free,
            pan_speed: 38.0,
            edge_scroll_speed: 42.0,
            edge_scroll_margin: 18.0,
            zoom_min: 8.0,
            zoom_max: 95.0,
            zoom_speed: 4.5,
            pitch_deg_from_horizontal: DEFAULT_MOBA_PITCH_DEG,
        }
    }
}

/// Estado mutable del rig (foco + zoom).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MobaCameraState {
    pub focus_xz: Vec2,
    pub focus_y: f32,
    pub zoom_horizontal: f32,
}

impl Default for MobaCameraState {
    fn default() -> Self {
        Self {
            focus_xz: Vec2::ZERO,
            focus_y: 0.0,
            zoom_horizontal: DEFAULT_MOBA_ZOOM_HORIZONTAL,
        }
    }
}

/// Límites del plano XZ para el punto de mira (foco).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MobaCameraBounds {
    pub min_xz: Vec2,
    pub max_xz: Vec2,
}

impl Default for MobaCameraBounds {
    fn default() -> Self {
        Self {
            min_xz: Vec2::splat(-160.0),
            max_xz: Vec2::splat(160.0),
        }
    }
}

/// Si el target bloqueado dejó de existir, volver a libre.
pub fn moba_camera_guard_locked_target_system(
    mut config: ResMut<MobaCameraConfig>,
    globals: Query<&GlobalTransform>,
) {
    let CameraMode::Locked { target } = config.mode else {
        return;
    };
    if globals.get(target).is_err() {
        config.mode = CameraMode::Free;
    }
}

/// Space / Y: alterna Free ↔ Locked al héroe resuelto por `CameraRigTarget`.
pub fn moba_camera_lock_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<MobaCameraConfig>,
    rig_target: Res<CameraRigTarget>,
    players: Query<Entity, With<PlayerControlled>>,
) {
    let toggle = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyY);
    if !toggle {
        return;
    }

    match config.mode {
        CameraMode::Free => {
            if let Some(e) = resolve_camera_target_entity(&rig_target, &players) {
                config.mode = CameraMode::Locked { target: e };
            }
        }
        CameraMode::Locked { .. } => {
            config.mode = CameraMode::Free;
        }
    }
}

/// En modo Locked, el foco copia al héroe (XZ + altura).
pub fn moba_camera_follow_locked_system(
    config: Res<MobaCameraConfig>,
    mut state: ResMut<MobaCameraState>,
    globals: Query<&GlobalTransform>,
) {
    let CameraMode::Locked { target } = config.mode else {
        return;
    };
    let Ok(tf) = globals.get(target) else {
        return;
    };
    let t = tf.translation();
    let next_xz = Vec2::new(t.x, t.z);
    let next_y = t.y;
    if state.focus_xz != next_xz {
        state.focus_xz = next_xz;
    }
    if state.focus_y != next_y {
        state.focus_y = next_y;
    }
}

/// Pan en plano XZ relativo a la base de cámara congelada (WASD + bordes).
pub fn moba_camera_pan_free_system(
    config: Res<MobaCameraConfig>,
    mut state: ResMut<MobaCameraState>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    basis: Res<CameraBasisForSim>,
    time: Res<Time>,
) {
    if !matches!(config.mode, CameraMode::Free) {
        return;
    }

    let dt = time.delta_secs().max(0.0);
    let mut dir = Vec2::ZERO;
    let f = basis.0.forward_xz;
    let r = basis.0.right_xz;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir += f;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir -= f;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir -= r;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir += r;
    }

    let mut speed = config.pan_speed;

    if let Ok(w) = windows.get_single() {
        if let Some(cursor) = w.cursor_position() {
            let width = w.width().max(1.0);
            let height = w.height().max(1.0);
            let m = config.edge_scroll_margin.max(0.0);
            let mut edge = Vec2::ZERO;
            if cursor.x <= m {
                edge -= r;
            }
            if cursor.x >= width - m {
                edge += r;
            }
            if cursor.y <= m {
                edge += f;
            }
            if cursor.y >= height - m {
                edge -= f;
            }
            let edge = normalize_or_zero(edge);
            if edge.length_squared() > 0.0 {
                dir += edge;
                speed = config.edge_scroll_speed;
            }
        }
    }

    let dir = normalize_or_zero(dir);
    if dir.length_squared() <= 0.0 {
        return;
    }

    let delta = dir * speed * dt;
    let next = state.focus_xz + delta;
    if next != state.focus_xz {
        state.focus_xz = next;
    }
}

/// Rueda: zoom en distancia horizontal con pitch fijo.
pub fn moba_camera_zoom_system(
    config: Res<MobaCameraConfig>,
    mut state: ResMut<MobaCameraState>,
    mut scroll: EventReader<MouseWheel>,
) {
    let mut lines = 0.0_f32;
    for ev in scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => lines += ev.y,
            MouseScrollUnit::Pixel => lines += ev.y * 0.01,
        }
    }
    if lines.abs() < f32::EPSILON {
        return;
    }
    let next = moba_zoom_horizontal_delta(
        state.zoom_horizontal,
        lines,
        config.zoom_speed,
        config.zoom_min,
        config.zoom_max,
    );
    if next != state.zoom_horizontal {
        state.zoom_horizontal = next;
    }
}

/// Aplica límites de arena al foco.
pub fn moba_camera_clamp_focus_system(
    bounds: Res<MobaCameraBounds>,
    mut state: ResMut<MobaCameraState>,
) {
    let clamped = moba_clamp_focus_xz(
        Vec3::new(state.focus_xz.x, state.focus_y, state.focus_xz.y),
        bounds.min_xz,
        bounds.max_xz,
    );
    if clamped.x != state.focus_xz.x || clamped.z != state.focus_xz.y {
        state.focus_xz = Vec2::new(clamped.x, clamped.z);
    }
    if clamped.y != state.focus_y {
        state.focus_y = clamped.y;
    }
}

/// Escribe `Transform` de la cámara desde foco + offset geométrico.
pub fn moba_camera_apply_transform_system(
    config: Res<MobaCameraConfig>,
    state: Res<MobaCameraState>,
    mut cameras: Query<&mut Transform, With<super::CameraRig>>,
) {
    let focus = Vec3::new(state.focus_xz.x, state.focus_y, state.focus_xz.y);
    let offset =
        moba_camera_offset_from_pitch(config.pitch_deg_from_horizontal, state.zoom_horizontal);
    let cam_pos = focus + offset;
    let desired = Transform::from_translation(cam_pos).looking_at(focus, Vec3::Y);

    for mut tf in &mut cameras {
        if tf.translation != desired.translation {
            tf.translation = desired.translation;
        }
        if tf.rotation != desired.rotation {
            tf.rotation = desired.rotation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_platform::input_capture::{
        InputCapturePlugin, IntentBuffer, MobaIntentCaptureOverride,
    };

    fn camera_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<MouseWheel>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<CameraBasisForSim>()
            .init_resource::<MobaCameraConfig>()
            .init_resource::<MobaCameraState>()
            .init_resource::<MobaCameraBounds>()
            .init_resource::<CameraRigTarget>()
            .add_systems(
                Update,
                (
                    moba_camera_guard_locked_target_system,
                    moba_camera_lock_toggle_system,
                    moba_camera_follow_locked_system,
                    moba_camera_pan_free_system,
                    moba_camera_zoom_system,
                    moba_camera_clamp_focus_system,
                    moba_camera_apply_transform_system,
                )
                    .chain(),
            );
        app
    }

    #[test]
    fn lock_toggle_y_switches_mode_and_target() {
        let mut app = camera_test_app();
        let hero = app.world_mut().spawn(()).id();
        app.world_mut()
            .entity_mut(hero)
            .insert((PlayerControlled, Transform::default()));
        app.world_mut()
            .insert_resource(CameraRigTarget { entity: Some(hero) });

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::KeyY);
        }
        app.update();
        assert_eq!(
            app.world().resource::<MobaCameraConfig>().mode,
            CameraMode::Locked { target: hero }
        );

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.clear();
            input.release(KeyCode::KeyY);
        }
        app.update();
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::KeyY);
        }
        app.update();
        assert_eq!(
            app.world().resource::<MobaCameraConfig>().mode,
            CameraMode::Free
        );
    }

    #[test]
    fn lock_toggle_space_switches_mode() {
        let mut app = camera_test_app();
        let hero = app.world_mut().spawn(()).id();
        app.world_mut()
            .entity_mut(hero)
            .insert((PlayerControlled, Transform::default()));
        app.world_mut()
            .insert_resource(CameraRigTarget { entity: Some(hero) });

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::Space);
        }
        app.update();

        let mode = app.world().resource::<MobaCameraConfig>().mode;
        assert_eq!(mode, CameraMode::Locked { target: hero });
    }

    #[test]
    fn stale_locked_target_drops_to_free() {
        let mut app = camera_test_app();
        let dead = app.world_mut().spawn(()).id();
        app.world_mut().resource_mut::<MobaCameraConfig>().mode =
            CameraMode::Locked { target: dead };
        app.world_mut().despawn(dead);
        app.update();

        assert_eq!(
            app.world().resource::<MobaCameraConfig>().mode,
            CameraMode::Free
        );
    }

    #[test]
    fn capture_suppresses_wasd_when_moba_routing_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(MobaIntentCaptureOverride {
                suppress_wasd_in_movement_intent: true,
                primary_action_uses_left_shift: false,
            })
            .add_plugins(InputCapturePlugin);

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::KeyW);
        }
        app.update();
        let snap = app.world().resource::<IntentBuffer>().last_snapshot;
        assert_eq!(snap.movement_xy, Vec2::ZERO);
    }
}
