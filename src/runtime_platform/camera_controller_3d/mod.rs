//! Controlador de cámara 3D: rig MOBA (default) + módulo orbital legacy.

mod constants;
pub mod legacy_orbital;
mod moba_camera;

pub use constants::{DEFAULT_MOBA_PITCH_DEG, DEFAULT_MOBA_ZOOM_HORIZONTAL};
pub use legacy_orbital::{CameraRigConfig, CameraRigState, update_camera_rig_system};
pub use moba_camera::{
    CameraMode, MobaCameraBounds, MobaCameraConfig, MobaCameraState,
    moba_camera_apply_transform_system, moba_camera_clamp_focus_system,
    moba_camera_follow_locked_system, moba_camera_guard_locked_target_system,
    moba_camera_lock_toggle_system, moba_camera_pan_free_system, moba_camera_zoom_system,
};

use bevy::prelude::*;

use crate::runtime_platform::input_capture::{
    MobaIntentCaptureOverride, MobaKeyboardRouting3d, V6InputCaptureSet,
};
use crate::runtime_platform::intent_projection_3d::CameraBasisForSim;

/// Habilita/deshabilita runtime del controlador de cámara.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Camera3dEnabled(pub bool);

/// Target explícito opcional para seguimiento / lock MOBA.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CameraRigTarget {
    pub entity: Option<Entity>,
}

/// Marca de cámara gestionada por el plugin 3D.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CameraRig;

/// Plugin de cámara 3D (MOBA por defecto).
pub struct Camera3dPlugin;

impl Plugin for Camera3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<bevy::input::mouse::MouseWheel>()
            .init_resource::<CameraBasisForSim>()
            // También sin `InputCapturePlugin` (tests mínimos): el sync necesita el recurso.
            .init_resource::<MobaIntentCaptureOverride>()
            .init_resource::<Camera3dEnabled>()
            .init_resource::<MobaCameraConfig>()
            .init_resource::<MobaCameraState>()
            .init_resource::<MobaCameraBounds>()
            .insert_resource(MobaKeyboardRouting3d {
                suppress_wasd_in_movement_intent: true,
                primary_action_uses_left_shift: true,
            })
            .init_resource::<CameraRigTarget>()
            .add_systems(
                PreUpdate,
                sync_moba_intent_capture_override_system.before(V6InputCaptureSet),
            )
            .add_systems(
                Update,
                (
                    ensure_camera_entity_system,
                    moba_camera_guard_locked_target_system,
                    moba_camera_lock_toggle_system,
                    moba_camera_follow_locked_system,
                    moba_camera_pan_free_system,
                    moba_camera_zoom_system,
                    moba_camera_clamp_focus_system,
                    moba_camera_apply_transform_system,
                    refresh_camera_basis_for_sim_system.after(moba_camera_apply_transform_system),
                )
                    .chain()
                    .run_if(v6_camera_3d_enabled),
            );
    }
}

/// Congela base XZ para el tick fijo tras actualizar el rig.
fn refresh_camera_basis_for_sim_system(
    mut basis: ResMut<CameraBasisForSim>,
    primary: Query<&Transform, (With<Camera3d>, With<CameraRig>)>,
    fallback: Query<&Transform, With<Camera3d>>,
) {
    let tf = primary.iter().next().or_else(|| fallback.iter().next());
    basis.0 = tf
        .map(crate::runtime_platform::intent_projection_3d::CameraBasisXZ::from_transform)
        .unwrap_or_else(crate::runtime_platform::intent_projection_3d::CameraBasisXZ::identity);
}

fn v6_camera_3d_enabled(enabled: Res<Camera3dEnabled>) -> bool {
    enabled.0
}

/// Política MOBA en captura: solo aplica si la cámara 3D está habilitada (evita “sin WASD ni pan”).
fn sync_moba_intent_capture_override_system(
    mut effective: ResMut<MobaIntentCaptureOverride>,
    routing: Option<Res<MobaKeyboardRouting3d>>,
    cam: Res<Camera3dEnabled>,
) {
    let cam_on = cam.0;
    let next = match routing.as_ref() {
        Some(r) => MobaIntentCaptureOverride {
            suppress_wasd_in_movement_intent: r.suppress_wasd_in_movement_intent && cam_on,
            primary_action_uses_left_shift: r.primary_action_uses_left_shift && cam_on,
        },
        None => MobaIntentCaptureOverride::default(),
    };
    if *effective != next {
        *effective = next;
    }
}

/// Garantiza que exista una cámara 3D gestionada por el rig.
pub fn ensure_camera_entity_system(
    mut commands: Commands,
    rig_query: Query<Entity, With<CameraRig>>,
    state: Res<MobaCameraState>,
    config: Res<MobaCameraConfig>,
) {
    if rig_query.iter().next().is_some() {
        return;
    }

    let focus = Vec3::new(state.focus_xz.x, state.focus_y, state.focus_xz.y);
    let offset = crate::blueprint::equations::moba_camera_offset_from_pitch(
        config.pitch_deg_from_horizontal,
        state.zoom_horizontal,
    );
    let cam_pos = focus + offset;
    let tf = Transform::from_translation(cam_pos).looking_at(focus, Vec3::Y);

    commands.spawn((Camera3d::default(), tf, CameraRig));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_platform::input_capture::InputCapturePlugin;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::input::InputPlugin)
            .add_plugins(Camera3dPlugin)
            .insert_resource(Camera3dEnabled(true));
        app
    }

    #[test]
    fn disabled_flag_prevents_camera_spawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(Camera3dPlugin);

        app.update();
        let world = app.world_mut();
        let mut query = world.query::<&CameraRig>();
        let count = query.iter(world).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn camera_disabled_clears_moba_intent_capture_override() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(InputCapturePlugin)
            .add_plugins(Camera3dPlugin)
            .insert_resource(Camera3dEnabled(false));
        app.update();
        let o = app.world().resource::<MobaIntentCaptureOverride>();
        assert!(!o.suppress_wasd_in_movement_intent);
        assert!(!o.primary_action_uses_left_shift);
    }

    #[test]
    fn explicit_target_despawned_is_guarded_without_panics() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(Transform::default()).id();
        app.world_mut().insert_resource(CameraRigTarget {
            entity: Some(entity),
        });
        app.world_mut().despawn(entity);

        app.update();
        let world = app.world_mut();
        let mut query = world.query::<&CameraRig>();
        let count = query.iter(world).count();
        assert_eq!(count, 1);
    }
}
