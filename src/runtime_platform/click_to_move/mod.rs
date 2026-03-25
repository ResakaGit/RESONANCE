use bevy::math::primitives::Cuboid;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub mod core;

use core::{GroundPlaneConfig, map_target_state_with, resolve_click_target_with};

use crate::runtime_platform::hud::{MinimapScreenRect, minimap_cursor_blocks_primary_pick};

/// Configuración del plano navegable y tolerancia de llegada.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ClickToMoveConfig {
    pub ground_y: f32,
    pub arrival_epsilon: f32,
}

impl Default for ClickToMoveConfig {
    fn default() -> Self {
        Self {
            ground_y: 0.0,
            arrival_epsilon: 0.25,
        }
    }
}

/// Estado de navegación por click (destino opcional).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MoveTargetState {
    pub target_xz: Option<Vec2>,
}

/// Marcador visual del destino click-to-move.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MoveTargetMarker;

/// Plugin para locomoción click-to-move en plano XZ.
pub struct ClickToMovePlugin;

impl Plugin for ClickToMovePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClickToMoveConfig>()
            .init_resource::<MoveTargetState>()
            .add_systems(
                Update,
                (
                    capture_click_to_move_target_system,
                    sync_target_marker_system,
                ),
            );
    }
}

/// Captura click izquierdo y fija destino en plano navegable.
pub fn capture_click_to_move_target_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    config: Res<ClickToMoveConfig>,
    minimap_screen: Option<Res<MinimapScreenRect>>,
    mut target: ResMut<MoveTargetState>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some((camera, camera_tf)) = camera_query.iter().next() else {
        return;
    };

    let click_pressed = mouse.just_pressed(MouseButton::Left);
    let cursor_pos = window.cursor_position();
    let block_minimap = minimap_screen
        .as_ref()
        .map(|s| minimap_cursor_blocks_primary_pick(cursor_pos, s))
        .unwrap_or(false);
    let click_for_world = click_pressed && !block_minimap;
    let ground = GroundPlaneConfig {
        ground_y: config.ground_y,
    };
    let previous = target.target_xz;
    let next = map_target_state_with(
        previous,
        click_for_world,
        cursor_pos,
        ground,
        |pressed, cursor, cfg| {
            resolve_click_target_with(pressed, cursor, cfg, |cursor_screen| {
                let ray = camera.viewport_to_world(camera_tf, cursor_screen).ok()?;
                Some((ray.origin, ray.direction.as_vec3()))
            })
        },
    );
    target.target_xz = next;
}

/// Sincroniza/crea el marcador visual de destino (solo capa visual).
pub fn sync_target_marker_system(
    mut commands: Commands,
    target: Res<MoveTargetState>,
    mut marker_query: Query<(Entity, &mut Transform), With<MoveTargetMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(target_xz) = target.target_xz else {
        return;
    };

    let marker_pos = Vec3::new(target_xz.x, 0.05, target_xz.y);
    if let Some((_, mut marker_tf)) = marker_query.iter_mut().next() {
        marker_tf.translation = marker_pos;
        return;
    }

    let mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(0.35, 0.1, 0.35))));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.95, 0.35),
        emissive: Color::srgb(0.12, 0.45, 0.16).into(),
        ..default()
    });

    commands.spawn((
        MoveTargetMarker,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(marker_pos),
    ));
}

#[cfg(test)]
mod tests {
    use super::core::project_ray_to_ground;
    use bevy::prelude::*;

    #[test]
    fn project_ray_to_ground_returns_hit_on_valid_intersection() {
        let origin = Vec3::new(0.0, 10.0, 0.0);
        let dir = Vec3::new(1.0, -2.0, 0.0).normalize();
        let hit = project_ray_to_ground(origin, dir, 0.0).expect("expected ground hit");
        assert!(hit.x > 0.0);
        assert!(hit.y.abs() < 1e-6);
    }

    #[test]
    fn project_ray_to_ground_returns_none_when_parallel() {
        let origin = Vec3::new(0.0, 5.0, 0.0);
        let dir = Vec3::X;
        assert_eq!(project_ray_to_ground(origin, dir, 0.0), None);
    }
}
