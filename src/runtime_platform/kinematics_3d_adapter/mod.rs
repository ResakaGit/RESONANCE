use bevy::prelude::*;

use crate::layers::FlowVector;
use crate::runtime_platform::core_math_agnostic::vec2_to_xz;

/// Marca explícita para runtime V6.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct V6RuntimeEntity;

/// Habilita cinemática 3D por entidad.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct V6Kinematic3D;

/// Política configurable del adapter 2D->3D.
#[derive(Resource, Debug, Clone, Copy)]
pub struct V6KinematicPolicy {
    /// Modo para resolver el eje Y durante la traducción XZ.
    pub y_mode: V6YPolicy,
    /// Si está activo, aplica a todas las entidades V6 (aunque no tengan tag `V6Kinematic3D`).
    pub enable_for_all_v6: bool,
}

impl Default for V6KinematicPolicy {
    fn default() -> Self {
        Self {
            y_mode: V6YPolicy::KeepCurrent,
            enable_for_all_v6: false,
        }
    }
}

/// Estrategia para el componente vertical al aplicar movimiento.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum V6YPolicy {
    /// Conserva altura actual de la entidad.
    KeepCurrent,
    /// Fuerza una altura fija.
    Fixed(f32),
}

/// Convierte un flujo 2D del plano de simulación a delta en mundo 3D.
#[inline]
pub fn flow_to_world_delta(flow: Vec2, dt: f32) -> Vec3 {
    if dt <= 0.0 {
        return Vec3::ZERO;
    }
    vec2_to_xz(flow) * dt
}

/// Aplica delta 3D sobre un `Transform` sin side effects extra.
#[inline]
pub fn apply_to_transform(transform: &mut Transform, delta: Vec3, y_mode: V6YPolicy) {
    transform.translation += delta;
    match y_mode {
        V6YPolicy::KeepCurrent => {}
        V6YPolicy::Fixed(y) => {
            transform.translation.y = y;
        }
    }
}

/// Sistema opt-in del Sprint 05:
/// - lee `FlowVector`
/// - escribe solo `Transform` de entidades V6
/// - activa por tag `V6Kinematic3D` o flag global de política
pub fn apply_v6_kinematics_3d_system(
    time: Res<Time>,
    policy: Res<V6KinematicPolicy>,
    mut query: Query<(&FlowVector, &mut Transform, Option<&V6Kinematic3D>), With<V6RuntimeEntity>>,
) {
    let dt = time.delta_secs();
    for (flow, mut transform, per_entity_tag) in &mut query {
        if !policy.enable_for_all_v6 && per_entity_tag.is_none() {
            continue;
        }
        let delta = flow_to_world_delta(flow.velocity(), dt);
        apply_to_transform(&mut transform, delta, policy.y_mode);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_vec3(a: Vec3, b: Vec3) {
        assert!((a - b).length() < 1e-6, "{a:?} != {b:?}");
    }

    #[test]
    fn flow_to_world_delta_dt_zero_returns_zero() {
        let delta = flow_to_world_delta(Vec2::new(3.0, 4.0), 0.0);
        assert_eq!(delta, Vec3::ZERO);
    }

    #[test]
    fn flow_to_world_delta_zero_velocity_returns_zero() {
        let delta = flow_to_world_delta(Vec2::ZERO, 0.5);
        assert_eq!(delta, Vec3::ZERO);
    }

    #[test]
    fn flow_to_world_delta_diagonal_unit_maps_to_xz() {
        let flow = Vec2::new(1.0, 1.0).normalize();
        let delta = flow_to_world_delta(flow, 1.0);
        approx_vec3(delta, Vec3::new(flow.x, 0.0, flow.y));
    }

    #[test]
    fn apply_to_transform_fixed_y_overrides_vertical_axis() {
        let mut transform = Transform::from_translation(Vec3::new(1.0, 5.0, 2.0));
        apply_to_transform(
            &mut transform,
            Vec3::new(0.5, 9.0, -1.0),
            V6YPolicy::Fixed(3.0),
        );
        approx_vec3(transform.translation, Vec3::new(1.5, 3.0, 1.0));
    }
}
