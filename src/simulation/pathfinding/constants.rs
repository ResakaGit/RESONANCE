//! Ajustes del sprint G5 (navmesh + follow). Centralizados; no inline mágicos en sistemas.

/// Radio de búsqueda de polígono start/end en unidades mundo (`oxidized_navigation::query::find_path`).
pub const PATHFIND_POLYGON_SEARCH_RADIUS: f32 = 6.0;

/// Escala del radio del agente para considerar waypoint alcanzado (además de `ClickToMoveConfig.arrival_epsilon`).
pub const PATHFOLLOW_REACH_RADIUS_FACTOR: f32 = 0.4;
