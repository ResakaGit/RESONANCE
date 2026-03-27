//! Demo morfogénesis inferida MG-8E.
//! Tres biomas contrastantes; feno tipos emergen termodinámicamente.
//! `RESONANCE_MAP=morphogenesis_demo cargo run`.

use bevy::prelude::*;

use crate::entities::archetypes::{
    spawn_aquatic_organism, spawn_desert_creature, spawn_desert_plant, spawn_forest_plant,
};

/// Slug de mapa para dispatch condicional en plugins.
pub const MORPHOGENESIS_DEMO_SLUG: &str = "morphogenesis_demo";

/// Spawnea el escenario de morfogénesis: tres biomas + organismos contrastantes.
pub fn spawn_morphogenesis_demo_startup_system(mut commands: Commands) {
    // ── Océano profundo: 3 organismos acuáticos ──────────────────────────────
    spawn_aquatic_organism(&mut commands, Vec2::new(-20.0,  2.0));
    spawn_aquatic_organism(&mut commands, Vec2::new(-22.0,  0.0));
    spawn_aquatic_organism(&mut commands, Vec2::new(-20.0, -2.0));

    // ── Desierto abrasador: 3 plantas + 2 criaturas ──────────────────────────
    spawn_desert_plant(&mut commands, Vec2::new(20.0,  2.0));
    spawn_desert_plant(&mut commands, Vec2::new(22.0,  0.0));
    spawn_desert_plant(&mut commands, Vec2::new(20.0, -2.0));

    spawn_desert_creature(&mut commands, Vec2::new(30.0, -10.0));
    spawn_desert_creature(&mut commands, Vec2::new(32.0, -12.0));

    // ── Bosque templado: 3 plantas forestales ───────────────────────────────
    spawn_forest_plant(&mut commands, Vec2::new(20.0, 20.0));
    spawn_forest_plant(&mut commands, Vec2::new(22.0, 22.0));
    spawn_forest_plant(&mut commands, Vec2::new(18.0, 22.0));
}
