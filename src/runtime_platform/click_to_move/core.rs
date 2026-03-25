use bevy::prelude::*;

/// Responsabilidad pura:
/// - Resolver target de click sobre plano.
/// - Resolver intención de movimiento hacia target.
/// Sin acceso a ECS/resources/comandos.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundPlaneConfig {
    pub ground_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveArrivalPolicy {
    pub arrival_epsilon: f32,
    pub radius_factor: f32,
}

impl Default for MoveArrivalPolicy {
    fn default() -> Self {
        Self {
            arrival_epsilon: 0.25,
            radius_factor: 0.25,
        }
    }
}

/// Proyecta un rayo de cámara al plano horizontal `y = ground_y`.
pub fn project_ray_to_ground(origin: Vec3, dir: Vec3, ground_y: f32) -> Option<Vec2> {
    if dir.y.abs() <= f32::EPSILON {
        return None;
    }
    let t = (ground_y - origin.y) / dir.y;
    if t < 0.0 {
        return None;
    }
    let hit = origin + dir * t;
    Some(Vec2::new(hit.x, hit.z))
}

/// HoF: convierte cursor -> rayo -> target de click.
///
/// `cursor_to_ray` define el adapter de plataforma (Bevy o tests).
pub fn resolve_click_target_with<F>(
    click_just_pressed: bool,
    cursor_pos: Option<Vec2>,
    config: GroundPlaneConfig,
    cursor_to_ray: F,
) -> Option<Vec2>
where
    F: FnOnce(Vec2) -> Option<(Vec3, Vec3)>,
{
    if !click_just_pressed {
        return None;
    }
    let cursor = cursor_pos?;
    let (origin, dir) = cursor_to_ray(cursor)?;
    project_ray_to_ground(origin, dir, config.ground_y)
}

/// HoF: actualiza un target opcional sin conocer storage externo.
pub fn map_target_state_with<F>(
    prev_target: Option<Vec2>,
    click_just_pressed: bool,
    cursor_pos: Option<Vec2>,
    config: GroundPlaneConfig,
    resolver: F,
) -> Option<Vec2>
where
    F: FnOnce(bool, Option<Vec2>, GroundPlaneConfig) -> Option<Vec2>,
{
    resolver(click_just_pressed, cursor_pos, config).or(prev_target)
}

/// Resultado puro de navegación hacia target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveIntentDecision {
    pub movement_intent: Vec2,
    pub next_target: Option<Vec2>,
}

/// Resuelve intención hacia target con regla de llegada estable.
pub fn resolve_move_intent_to_target(
    current_xz: Vec2,
    target_xz: Option<Vec2>,
    entity_radius: f32,
    policy: MoveArrivalPolicy,
) -> MoveIntentDecision {
    let Some(target) = target_xz else {
        return MoveIntentDecision {
            movement_intent: Vec2::ZERO,
            next_target: None,
        };
    };

    let delta = target - current_xz;
    let stop_distance =
        policy.arrival_epsilon.max(0.0) + entity_radius.max(0.0) * policy.radius_factor.max(0.0);
    if delta.length() <= stop_distance {
        return MoveIntentDecision {
            movement_intent: Vec2::ZERO,
            next_target: None,
        };
    }

    MoveIntentDecision {
        movement_intent: delta.normalize_or_zero(),
        next_target: Some(target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_click_target_with_projects_valid_hit() {
        let result = resolve_click_target_with(
            true,
            Some(Vec2::new(120.0, 40.0)),
            GroundPlaneConfig { ground_y: 0.0 },
            |_| Some((Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, -1.0, 0.0))),
        );
        assert_eq!(result, Some(Vec2::ZERO));
    }

    #[test]
    fn map_target_state_with_keeps_previous_when_no_new_click() {
        let prev = Some(Vec2::new(3.0, 4.0));
        let next = map_target_state_with(
            prev,
            false,
            None,
            GroundPlaneConfig { ground_y: 0.0 },
            |_, _, _| None,
        );
        assert_eq!(next, prev);
    }

    #[test]
    fn resolve_move_intent_to_target_clears_target_on_arrival() {
        let decision = resolve_move_intent_to_target(
            Vec2::new(0.0, 0.0),
            Some(Vec2::new(0.1, 0.1)),
            0.4,
            MoveArrivalPolicy {
                arrival_epsilon: 0.25,
                radius_factor: 0.25,
            },
        );
        assert_eq!(decision.movement_intent, Vec2::ZERO);
        assert!(decision.next_target.is_none());
    }
}
