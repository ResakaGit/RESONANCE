use bevy::prelude::*;

/// Entidad controlada por el jugador local (selector operativo de input).
#[derive(Component, Debug, Clone, Copy, Default)]
#[component(storage = "SparseSet")]
pub struct PlayerControlled;
