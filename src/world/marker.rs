use bevy::prelude::*;

/// Recurso global: marcador de puntuación por facción.
#[derive(Resource, Default, Debug)]
pub struct Scoreboard {
    pub red_points: u32,
    pub blue_points: u32,
    pub red_kills: u32,
    pub blue_kills: u32,
}
