//! Rig orbital/chase legacy (Sprint 09). Conservado para referencia u opt-in;
//! el binario demo usa [`super::moba_camera`] por defecto.

use bevy::prelude::*;

use crate::runtime_platform::input_capture::IntentBuffer;
use crate::simulation::PlayerControlled;

/// Configuración del rig de cámara orbital/chase (legacy).
#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraRigConfig {
    pub distance: f32,
    pub height_offset: f32,
    pub pitch_min_rad: f32,
    pub pitch_max_rad: f32,
    pub yaw_sensitivity_rad: f32,
    pub pitch_sensitivity_rad: f32,
    pub smoothing_hz: f32,
    pub orbit_requires_primary_action: bool,
}

impl Default for CameraRigConfig {
    fn default() -> Self {
        Self {
            distance: 22.0,
            height_offset: 3.0,
            pitch_min_rad: -1.1,
            pitch_max_rad: -0.15,
            yaw_sensitivity_rad: 1.8,
            pitch_sensitivity_rad: 1.2,
            smoothing_hz: 8.0,
            orbit_requires_primary_action: true,
        }
    }
}

/// Estado interno del rig orbital (legacy).
#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraRigState {
    pub yaw_rad: f32,
    pub pitch_rad: f32,
}

impl Default for CameraRigState {
    fn default() -> Self {
        Self {
            yaw_rad: 0.0,
            pitch_rad: -0.45,
        }
    }
}

use super::CameraRigTarget;

/// Prioridad: `CameraRigTarget.entity` si está viva; si no, primer `PlayerControlled`.
pub fn resolve_camera_target_entity(
    target: &CameraRigTarget,
    player_query: &Query<Entity, With<PlayerControlled>>,
) -> Option<Entity> {
    if let Some(explicit) = target.entity {
        return Some(explicit);
    }
    player_query.iter().next()
}

/// Actualiza transform de cámara orbital (legacy). No usar en paralelo con MOBA rig.
pub fn update_camera_rig_system(
    time: Res<Time>,
    config: Res<CameraRigConfig>,
    mut state: ResMut<CameraRigState>,
    target: Res<CameraRigTarget>,
    snapshot: Option<Res<IntentBuffer>>,
    player_query: Query<Entity, With<PlayerControlled>>,
    target_transform_query: Query<&GlobalTransform>,
    mut camera_query: Query<&mut Transform, With<super::CameraRig>>,
) {
    let Some(target_entity) = resolve_camera_target_entity(&target, &player_query) else {
        return;
    };

    let Ok(target_tf) = target_transform_query.get(target_entity) else {
        return;
    };

    let dt = time.delta_secs().max(0.0);
    if let Some(buffer) = snapshot {
        let allow_orbit =
            !config.orbit_requires_primary_action || buffer.last_snapshot.primary_action();
        if allow_orbit {
            state.yaw_rad -= buffer.last_snapshot.movement_xy.x * config.yaw_sensitivity_rad * dt;
            state.pitch_rad +=
                buffer.last_snapshot.movement_xy.y * config.pitch_sensitivity_rad * dt;
            state.pitch_rad = state
                .pitch_rad
                .clamp(config.pitch_min_rad, config.pitch_max_rad);
        }
    }

    let target_pos = target_tf.translation() + Vec3::Y * config.height_offset.max(0.0);
    let orbit_rot = Quat::from_euler(EulerRot::YXZ, state.yaw_rad, state.pitch_rad, 0.0);
    let desired_pos = target_pos + orbit_rot * Vec3::new(0.0, 0.0, config.distance.max(0.1));
    let desired_tf = Transform::from_translation(desired_pos).looking_at(target_pos, Vec3::Y);
    let alpha = 1.0 - (-config.smoothing_hz.max(0.0) * dt).exp();

    for mut camera_tf in &mut camera_query {
        if camera_tf.translation != desired_tf.translation {
            camera_tf.translation = camera_tf.translation.lerp(desired_tf.translation, alpha);
        }
        if camera_tf.rotation != desired_tf.rotation {
            camera_tf.rotation = camera_tf.rotation.slerp(desired_tf.rotation, alpha);
        }
    }
}
