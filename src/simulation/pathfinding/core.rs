//! Matemática pura de seguimiento de waypoints (plano XZ).
//! Sin ECS: reusable desde tests y desde sistemas (Capa 7 vía `movement_intent`).

use bevy::prelude::{Vec2, Vec3};

/// Resultado de un paso de steering hacia el polyline del navmesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathFollowStep {
    /// Dirección normalizada en XZ hacia el waypoint activo, o cero si no hay ruta útil.
    pub movement_xz: Vec2,
    /// Índice del waypoint al que apuntamos tras posibles skips por radio de llegada.
    pub next_index: usize,
    /// `true` cuando no quedan waypoints (ruta consumida).
    pub path_finished: bool,
}

/// Avanza waypoints ya alcanzados (radio `reach_radius`) y devuelve dirección al siguiente.
pub fn path_follow_step_xz(
    agent_pos_xz: Vec2,
    waypoints: &[Vec3],
    mut index: usize,
    reach_radius: f32,
) -> PathFollowStep {
    let r = reach_radius.max(0.0);

    while index < waypoints.len() {
        let wp = waypoints[index];
        let wp_xz = Vec2::new(wp.x, wp.z);
        let delta = wp_xz - agent_pos_xz;
        if delta.length() <= r {
            index += 1;
            continue;
        }
        return PathFollowStep {
            movement_xz: delta.normalize_or_zero(),
            next_index: index,
            path_finished: false,
        };
    }

    PathFollowStep {
        movement_xz: Vec2::ZERO,
        next_index: index,
        path_finished: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;

    #[test]
    fn path_follow_zero_waypoints_finishes_immediately() {
        let step = path_follow_step_xz(Vec2::ZERO, &[], 0, 0.5);
        assert_eq!(step.movement_xz, Vec2::ZERO);
        assert!(step.path_finished);
        assert_eq!(step.next_index, 0);
    }

    #[test]
    fn path_follow_points_toward_first_waypoint() {
        let wps = [Vec3::new(10.0, 0.0, 0.0)];
        let step = path_follow_step_xz(Vec2::ZERO, &wps, 0, 0.25);
        assert!((step.movement_xz - Vec2::X).length() < 1e-5);
        assert!(!step.path_finished);
        assert_eq!(step.next_index, 0);
    }

    #[test]
    fn path_follow_skips_waypoint_inside_reach_radius() {
        let wps = [Vec3::new(1.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)];
        let step = path_follow_step_xz(Vec2::new(0.9, 0.0), &wps, 0, 0.25);
        assert!((step.movement_xz - Vec2::X).length() < 1e-4);
        assert_eq!(step.next_index, 1);
        assert!(!step.path_finished);
    }

    #[test]
    fn path_follow_finishes_after_last_reached() {
        let wps = [Vec3::new(5.0, 0.0, 0.0)];
        let step = path_follow_step_xz(Vec2::new(5.0, 0.0), &wps, 0, 0.5);
        assert_eq!(step.movement_xz, Vec2::ZERO);
        assert!(step.path_finished);
        assert_eq!(step.next_index, 1);
    }
}
