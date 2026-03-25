//! Utilidades puras: foco de cámara en el plano de simulación (2D coherente con Sprint 13 LOD).

use bevy::math::Vec2;
use bevy::prelude::{Camera, GlobalTransform, Query};

use crate::runtime_platform::core_math_agnostic::sim_plane_pos;

/// Posición en el plano XY o XZ según `use_xz_ground`, primera cámara con `is_active`.
#[inline]
pub(crate) fn active_camera_sim_plane(
    cameras: &Query<(&Camera, &GlobalTransform)>,
    use_xz_ground: bool,
) -> Option<Vec2> {
    cameras
        .iter()
        .find(|(cam, _)| cam.is_active)
        .map(|(_, gt)| sim_plane_pos(gt.translation(), use_xz_ground))
}
