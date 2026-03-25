use bevy::prelude::*;

use crate::layers::SpatialVolume;
use crate::layers::WillActuator;
use crate::runtime_platform::click_to_move::core::{
    MoveArrivalPolicy, resolve_move_intent_to_target,
};
use crate::runtime_platform::click_to_move::{ClickToMoveConfig, MoveTargetState};
use crate::runtime_platform::contracts::{IntentSnapshot, WillIntent3D};
use crate::runtime_platform::core_math_agnostic::{flatten_xz, normalize_or_zero};
use crate::runtime_platform::input_capture::IntentBuffer;
use crate::simulation::PlayerControlled;
use crate::simulation::pathfinding::constants::PATHFOLLOW_REACH_RADIUS_FACTOR;
use crate::simulation::pathfinding::core::path_follow_step_xz;
use crate::simulation::pathfinding::{NavAgent, NavPath};

/// Base ortonormal del plano jugable XZ derivada de cámara.
///
/// Convención: sistema de mano derecha, eje Y hacia arriba.
#[derive(Debug, Clone, Copy)]
pub struct CameraBasisXZ {
    pub forward_xz: Vec2,
    pub right_xz: Vec2,
}

/// Base de cámara congelada para el tick de simulación (no lee Transform en FixedUpdate).
#[derive(Resource, Clone, Copy, Debug)]
pub struct CameraBasisForSim(pub CameraBasisXZ);

impl Default for CameraBasisForSim {
    fn default() -> Self {
        Self(CameraBasisXZ::identity())
    }
}

/// Intención proyectada lista para aplicación al dominio (Capa 7).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ProjectedWillIntent {
    pub movement_intent: Vec2,
    pub channeling_ability: bool,
}

impl CameraBasisXZ {
    /// Base identidad: avanzar = +Z, derecha = +X.
    pub fn identity() -> Self {
        Self {
            forward_xz: Vec2::Y,
            right_xz: Vec2::X,
        }
    }

    /// Deriva la base desde el `Transform` de cámara proyectando a XZ.
    pub fn from_transform(transform: &Transform) -> Self {
        let world_forward = transform.forward().as_vec3();
        let world_right = transform.right().as_vec3();

        let forward_xz =
            normalize_or_zero(flatten_xz(Vec3::new(world_forward.x, 0.0, world_forward.z)));
        let right_xz = normalize_or_zero(flatten_xz(Vec3::new(world_right.x, 0.0, world_right.z)));

        let safe_forward = if forward_xz.length_squared() > 0.0 {
            forward_xz
        } else {
            Vec2::Y
        };
        let safe_right = if right_xz.length_squared() > 0.0 {
            right_xz
        } else {
            Vec2::X
        };

        Self {
            forward_xz: safe_forward,
            right_xz: safe_right,
        }
    }
}

/// Función pura del Sprint 03.
///
/// Convierte `IntentSnapshot` local a cámara en intención de mundo para XZ.
pub fn project_intent(snapshot: IntentSnapshot, basis: CameraBasisXZ) -> WillIntent3D {
    let axis = snapshot.movement_xy;
    let magnitude = axis.length().clamp(0.0, 1.0);

    if magnitude <= 0.0 {
        return WillIntent3D::zero();
    }

    let world_xz = basis.right_xz * axis.x + basis.forward_xz * axis.y;
    if world_xz.length_squared() <= 0.0 {
        return WillIntent3D::zero();
    }

    WillIntent3D::new(world_xz, magnitude)
}

/// Adapter puro: proyecta input + base de cámara a intención en mundo.
pub fn project_intent_to_resource_system(
    snapshot: Res<IntentBuffer>,
    basis: Res<CameraBasisForSim>,
    mut projected: ResMut<ProjectedWillIntent>,
) {
    let intent = project_intent(snapshot.last_snapshot, basis.0);
    projected.movement_intent = intent.direction_xz * intent.magnitude;
    projected.channeling_ability = snapshot.last_snapshot.primary_action();
}

/// Apply de dominio: materializa intención proyectada sobre Capa 7.
///
/// Prioridad: teclado/gamepad > polyline navmesh > línea recta al target (fallback).
#[allow(clippy::type_complexity)]
pub fn apply_projected_intent_to_will_system(
    projected: Res<ProjectedWillIntent>,
    config: Res<ClickToMoveConfig>,
    move_target: Res<MoveTargetState>,
    mut will_query: Query<
        (
            &Transform,
            Option<&SpatialVolume>,
            &mut WillActuator,
            Option<&mut NavPath>,
            Option<&NavAgent>,
        ),
        With<PlayerControlled>,
    >,
) {
    const KEYBOARD_DEADZONE_SQ: f32 = 1e-6;

    for (transform, volume_opt, mut will, nav_path_opt, nav_agent_opt) in &mut will_query {
        let keyboard = projected.movement_intent;

        if keyboard.length_squared() > KEYBOARD_DEADZONE_SQ {
            will.set_movement_intent(keyboard);
            will.set_channeling_ability(projected.channeling_ability);
            continue;
        }

        if let (Some(mut nav), Some(agent)) = (nav_path_opt, nav_agent_opt) {
            if !nav.waypoints.is_empty() {
                let current_xz = Vec2::new(transform.translation.x, transform.translation.z);
                let reach = config.arrival_epsilon + agent.radius * PATHFOLLOW_REACH_RADIUS_FACTOR;
                let step =
                    path_follow_step_xz(current_xz, &nav.waypoints, nav.current_index, reach);
                nav.current_index = step.next_index;
                if step.path_finished {
                    // No tocar `MoveTargetState` global: otras unidades `PlayerControlled` pueden seguir en ruta.
                    nav.clear();
                }
                will.set_movement_intent(step.movement_xz);
                will.set_channeling_ability(projected.channeling_ability);
                continue;
            }
        }

        let mut next_movement = Vec2::ZERO;
        if let Some(target_xz) = move_target.target_xz {
            let current_xz = Vec2::new(transform.translation.x, transform.translation.z);
            let decision = resolve_move_intent_to_target(
                current_xz,
                Some(target_xz),
                volume_opt.map(|v| v.radius).unwrap_or(0.0),
                MoveArrivalPolicy {
                    arrival_epsilon: config.arrival_epsilon,
                    radius_factor: 0.25,
                },
            );
            // No propagar `decision.next_target` al recurso global: varias unidades pueden compartir destino.
            next_movement = decision.movement_intent;
        }

        will.set_movement_intent(next_movement);
        will.set_channeling_ability(projected.channeling_ability);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::WillActuator;
    use crate::simulation::PlayerControlled;
    use crate::simulation::pathfinding::{NavAgent, NavPath};
    use bevy::prelude::{Vec3, World};

    fn approx_vec2(a: Vec2, b: Vec2) {
        assert!((a - b).length() < 1e-4, "{a:?} != {b:?}");
    }

    #[test]
    fn project_intent_identity_basis_cardinals() {
        let basis = CameraBasisXZ::identity();

        let forward = project_intent(IntentSnapshot::new(Vec2::Y, 0, None), basis);
        approx_vec2(forward.direction_xz, Vec2::Y);
        assert_eq!(forward.magnitude, 1.0);

        let right = project_intent(IntentSnapshot::new(Vec2::X, 0, None), basis);
        approx_vec2(right.direction_xz, Vec2::X);
        assert_eq!(right.magnitude, 1.0);
    }

    #[test]
    fn project_intent_rotated_basis_90_deg() {
        // Yaw +90°: forward local (+Z) termina en +X global.
        let basis = CameraBasisXZ {
            forward_xz: Vec2::X,
            right_xz: Vec2::NEG_Y,
        };

        let projected = project_intent(IntentSnapshot::new(Vec2::Y, 0, None), basis);
        approx_vec2(projected.direction_xz, Vec2::X);
        assert_eq!(projected.magnitude, 1.0);
    }

    #[test]
    fn project_intent_zero_is_guarded() {
        let projected = project_intent(IntentSnapshot::default(), CameraBasisXZ::identity());
        assert_eq!(projected, WillIntent3D::zero());
    }

    #[test]
    fn apply_system_writes_will_actuator_from_projected_resource() {
        let mut world = World::new();
        world.insert_resource(ClickToMoveConfig::default());
        world.insert_resource(MoveTargetState::default());
        world.insert_resource(ProjectedWillIntent {
            movement_intent: Vec2::new(0.5, -0.25),
            channeling_ability: true,
        });
        let entity = world
            .spawn((
                WillActuator::default(),
                PlayerControlled,
                Transform::default(),
                NavPath::default(),
                NavAgent::new(0.5),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_projected_intent_to_will_system);
        schedule.run(&mut world);

        let will = world
            .get::<WillActuator>(entity)
            .expect("missing WillActuator");
        assert_eq!(will.movement_intent(), Vec2::new(0.5, -0.25));
        assert!(will.channeling_ability());
    }

    #[test]
    fn apply_system_click_to_move_stops_inside_arrival_epsilon() {
        let mut world = World::new();
        world.insert_resource(ClickToMoveConfig {
            ground_y: 0.0,
            arrival_epsilon: 0.5,
        });
        world.insert_resource(MoveTargetState {
            target_xz: Some(Vec2::new(0.2, 0.1)),
        });
        world.insert_resource(ProjectedWillIntent::default());
        let entity = world
            .spawn((
                WillActuator::default(),
                PlayerControlled,
                Transform::default(),
                NavPath::default(),
                NavAgent::new(0.5),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_projected_intent_to_will_system);
        schedule.run(&mut world);

        let will = world
            .get::<WillActuator>(entity)
            .expect("missing WillActuator");
        assert_eq!(will.movement_intent(), Vec2::ZERO);
        let target = world.resource::<MoveTargetState>();
        assert_eq!(
            target.target_xz,
            Some(Vec2::new(0.2, 0.1)),
            "el goal global persiste hasta un nuevo click"
        );
    }

    #[test]
    fn apply_system_prefers_nav_path_when_waypoints_exist() {
        let mut world = World::new();
        world.insert_resource(ClickToMoveConfig::default());
        world.insert_resource(MoveTargetState {
            target_xz: Some(Vec2::new(100.0, 0.0)),
        });
        world.insert_resource(ProjectedWillIntent::default());
        let entity = world
            .spawn((
                WillActuator::default(),
                PlayerControlled,
                Transform::from_xyz(0.0, 0.0, 0.0),
                NavPath {
                    waypoints: vec![Vec3::new(4.0, 0.0, 0.0)],
                    current_index: 0,
                },
                NavAgent::new(0.25),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_projected_intent_to_will_system);
        schedule.run(&mut world);

        let will = world
            .get::<WillActuator>(entity)
            .expect("missing WillActuator");
        assert!(
            (will.movement_intent() - Vec2::X).length() < 1e-5,
            "expected steering toward waypoint, got {:?}",
            will.movement_intent()
        );
        let target = world.resource::<MoveTargetState>();
        assert!(target.target_xz.is_some());
    }
}
