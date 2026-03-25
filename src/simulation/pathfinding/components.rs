//! Componentes ECS para navegación con navmesh (alimentan Capa 7, no `Transform` directo).

use bevy::prelude::*;

/// Agente registrado para pathfinding (radio para llegada a waypoints; `speed` reservado p. ej. RVO futuro).
#[derive(Component, Debug, Clone, Copy)]
pub struct NavAgent {
    /// Reservado: tope de escala con intención; hoy la velocidad emerge de L3/L1/L6.
    #[allow(dead_code)]
    pub speed: f32,
    pub radius: f32,
}

impl NavAgent {
    pub fn new(radius: f32) -> Self {
        Self {
            speed: 0.0,
            radius: radius.max(0.01),
        }
    }
}

/// Polyline devuelta por `oxidized_navigation::query::find_path`.
#[derive(Component, Debug, Clone, Default)]
pub struct NavPath {
    pub waypoints: Vec<Vec3>,
    pub current_index: usize,
}

impl NavPath {
    pub fn clear(&mut self) {
        self.waypoints.clear();
        self.current_index = 0;
    }
}
