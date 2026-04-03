//! Demo: animal herbívoro autónomo.
//! 1 animal en sabana Terra + 3 plantas como fuente trófica.
//! Observa: BehaviorIntent, homeostasis, TrophicState, voluntad (L7) y ciclo hambre/saciedad.
//! `RESONANCE_MAP=demo_animal cargo run`

use bevy::prelude::*;

use crate::blueprint::IdGenerator;
use crate::entities::archetypes::catalog::{spawn_animal_demo, spawn_planta_demo};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

pub const DEMO_ANIMAL_SLUG: &str = "demo_animal";

/// 1 herbívoro + 3 plantas dispersas en la sabana.
pub fn spawn_demo_animal_startup_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    layout: Res<SimWorldTransformParams>,
) {
    // Plantas como fuente trófica
    let plant_positions = [
        Vec2::new(-5.0, 2.0),
        Vec2::new(3.0, -3.0),
        Vec2::new(0.0, 5.0),
    ];
    for pos in plant_positions {
        spawn_planta_demo(&mut commands, &mut id_gen, pos, &layout);
    }

    // Animal herbívoro — comienza con saciedad baja (0.3) para activar búsqueda
    spawn_animal_demo(&mut commands, &mut id_gen, Vec2::new(0.0, 0.0), &layout);

    info!("demo_animal: 1 herbívoro + 3 plantas — observando BehaviorIntent + ciclo trófico");
}
