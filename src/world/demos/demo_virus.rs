//! Demo: virus + células huésped.
//! 4 células en pool Aqua + 2 virus Ignis; observa parasitismo energético (inyector L8).
//! `RESONANCE_MAP=demo_virus cargo run`

use bevy::prelude::*;

use crate::blueprint::IdGenerator;
use crate::entities::archetypes::catalog::{spawn_celula, spawn_virus};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

pub const DEMO_VIRUS_SLUG: &str = "demo_virus";

/// 4 células huésped + 2 virus. Virus posicionados cerca del pool Ignis.
pub fn spawn_demo_virus_startup_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    layout: Res<SimWorldTransformParams>,
) {
    // Células huésped en el lado Aqua
    let cell_positions = [
        Vec2::new(-2.0, 0.5),
        Vec2::new(-2.0, -0.5),
        Vec2::new(-1.2, 0.0),
        Vec2::new(-1.5, 1.0),
    ];
    for pos in cell_positions {
        spawn_celula(&mut commands, &mut id_gen, pos, &layout);
    }

    // Virus en el lado Ignis (campo perturbador)
    let virus_positions = [Vec2::new(1.2, 0.2), Vec2::new(1.5, -0.3)];
    for pos in virus_positions {
        spawn_virus(&mut commands, &mut id_gen, pos, &layout);
    }

    info!("demo_virus: 4 células + 2 virus — observando inyección Ignis sobre huéspedes Aqua");
}
