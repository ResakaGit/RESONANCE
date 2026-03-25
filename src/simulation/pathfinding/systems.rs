//! Sistemas: eventos de goal → `NavPath`; no tocan `movement_system` ni `FlowVector`.

use bevy::prelude::*;
use oxidized_navigation::{NavMesh, NavMeshSettings, query};

use crate::events::PathRequestEvent;
use crate::runtime_platform::click_to_move::MoveTargetState;
use crate::simulation::PlayerControlled;
use crate::simulation::pathfinding::components::{NavAgent, NavPath};
use crate::simulation::pathfinding::constants::PATHFIND_POLYGON_SEARCH_RADIUS;
/// Emite [`PathRequestEvent`] cuando cambia el destino click-to-move (lazy recalculo).
pub fn emit_path_request_on_goal_change_system(
    move_target: Res<MoveTargetState>,
    mut last_goal: Local<Option<Vec2>>,
    mut writer: EventWriter<PathRequestEvent>,
) {
    if move_target.target_xz == *last_goal {
        return;
    }
    *last_goal = move_target.target_xz;
    let Some(goal_xz) = move_target.target_xz else {
        return;
    };
    writer.send(PathRequestEvent { goal_xz });
}

/// Sin destino: vacía rutas para no mezclar segmentos viejos con un nuevo click.
pub fn clear_nav_paths_when_no_move_target_system(
    move_target: Res<MoveTargetState>,
    mut q: Query<&mut NavPath, With<PlayerControlled>>,
) {
    if move_target.target_xz.is_some() {
        return;
    }
    for mut nav in &mut q {
        nav.clear();
    }
}

/// Calcula `NavPath` por agente con `NavAgent` + `PlayerControlled`.
#[allow(clippy::type_complexity)]
pub fn pathfinding_compute_system(
    move_target: Res<MoveTargetState>,
    mut path_events: EventReader<PathRequestEvent>,
    nav_mesh: Res<NavMesh>,
    nav_settings: Res<NavMeshSettings>,
    mut q: Query<(&Transform, &mut NavPath), (With<PlayerControlled>, With<NavAgent>)>,
    mut retry_stride: Local<u32>,
) {
    let Some(goal_xz) = move_target.target_xz else {
        return;
    };

    let mut event_fresh = false;
    for _ in path_events.read() {
        event_fresh = true;
    }

    let mut need_compute = event_fresh;
    if !need_compute {
        for (_, nav) in q.iter() {
            if nav.waypoints.is_empty() {
                need_compute = true;
                break;
            }
        }
    }

    if !need_compute {
        return;
    }

    let arc = nav_mesh.get();
    let Ok(guard) = arc.read() else {
        return;
    };
    if guard.tiles.is_empty() {
        return;
    }

    *retry_stride = retry_stride.wrapping_add(1);
    // Nuevo click → recalcular todos. Retry sin evento → solo agentes con ruta vacía (evita N×find_path/tick).
    // Throttle de reintentos si el mesh aún no está o el goal es inválido (~10 Hz con Fixed 30 Hz).
    const RETRY_STRIDE: u32 = 3;

    for (transform, mut nav) in &mut q {
        if !event_fresh && !nav.waypoints.is_empty() {
            continue;
        }
        if !event_fresh && nav.waypoints.is_empty() && *retry_stride % RETRY_STRIDE != 0 {
            continue;
        }
        let start = transform.translation;
        let end = Vec3::new(goal_xz.x, start.y, goal_xz.y);
        match query::find_path(
            &guard,
            &nav_settings,
            start,
            end,
            Some(PATHFIND_POLYGON_SEARCH_RADIUS),
            None,
        ) {
            Ok(waypoints) => {
                nav.waypoints = waypoints;
                nav.current_index = 0;
            }
            Err(_) => {
                nav.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::PathRequestEvent;
    use crate::runtime_platform::click_to_move::MoveTargetState;
    use crate::simulation::pathfinding::components::NavPath;
    use crate::simulation::PlayerControlled;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<PathRequestEvent>();
        app.init_resource::<MoveTargetState>();
        app
    }

    #[test]
    fn emit_path_request_fires_on_goal_change() {
        let mut app = test_app();
        app.add_systems(Update, emit_path_request_on_goal_change_system);
        app.world_mut().resource_mut::<MoveTargetState>().target_xz = Some(Vec2::new(10.0, 20.0));
        app.update();

        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<Events<PathRequestEvent>>()
            .drain()
            .collect();
        assert_eq!(events.len(), 1, "should emit PathRequestEvent on goal change");
        assert_eq!(events[0].goal_xz, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn emit_path_request_no_event_when_goal_unchanged() {
        let mut app = test_app();
        app.add_systems(Update, emit_path_request_on_goal_change_system);
        app.world_mut().resource_mut::<MoveTargetState>().target_xz = Some(Vec2::new(5.0, 5.0));
        app.update();
        // Drain first batch
        app.world_mut()
            .resource_mut::<Events<PathRequestEvent>>()
            .drain()
            .count();
        // Second update with same goal
        app.update();
        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<Events<PathRequestEvent>>()
            .drain()
            .collect();
        assert!(events.is_empty(), "should not re-emit when goal unchanged");
    }

    #[test]
    fn emit_path_request_no_event_when_no_target() {
        let mut app = test_app();
        app.add_systems(Update, emit_path_request_on_goal_change_system);
        // target_xz defaults to None
        app.update();
        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<Events<PathRequestEvent>>()
            .drain()
            .collect();
        assert!(events.is_empty(), "should not emit when target is None");
    }

    #[test]
    fn clear_nav_paths_when_no_target() {
        let mut app = test_app();
        app.add_systems(Update, clear_nav_paths_when_no_move_target_system);

        let e = app.world_mut().spawn((
            PlayerControlled,
            NavPath {
                waypoints: vec![Vec3::new(1.0, 0.0, 1.0), Vec3::new(2.0, 0.0, 2.0)],
                current_index: 0,
            },
        )).id();

        // target_xz = None (default)
        app.update();

        let nav = app.world().get::<NavPath>(e).unwrap();
        assert!(nav.waypoints.is_empty(), "paths should be cleared when no target");
    }

    #[test]
    fn clear_nav_paths_preserves_when_target_set() {
        let mut app = test_app();
        app.add_systems(Update, clear_nav_paths_when_no_move_target_system);
        app.world_mut().resource_mut::<MoveTargetState>().target_xz = Some(Vec2::new(5.0, 5.0));

        let e = app.world_mut().spawn((
            PlayerControlled,
            NavPath {
                waypoints: vec![Vec3::new(1.0, 0.0, 1.0)],
                current_index: 0,
            },
        )).id();

        app.update();

        let nav = app.world().get::<NavPath>(e).unwrap();
        assert!(!nav.waypoints.is_empty(), "paths should be preserved when target set");
    }
}
