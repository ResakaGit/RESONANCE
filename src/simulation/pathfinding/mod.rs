//! Sprint G5 — pathfinding: navmesh (oxidized) alimenta `WillActuator` (L7), no `Transform`.
//!
//! Ver `docs/sprints/GAMEDEV_PATTERNS/SPRINT_G5_PATHFINDING.md`.

pub mod components;
pub mod constants;
pub mod core;
mod systems;

pub use components::{NavAgent, NavPath};
pub use systems::{
    clear_nav_paths_when_no_move_target_system, emit_path_request_on_goal_change_system,
    pathfinding_compute_system,
};
