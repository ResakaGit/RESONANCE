//! Demo: célula eucariota mínima.
//! 3 células en campo Aqua; observa ciclo metabólico, homeostasis y reproducción.
//! `RESONANCE_MAP=demo_celula cargo run`

use bevy::prelude::*;

use crate::blueprint::IdGenerator;
use crate::entities::archetypes::catalog::spawn_celula;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

pub const DEMO_CELULA_SLUG: &str = "demo_celula";

/// Spawnea 3 células en posiciones simétricas alrededor del núcleo Aqua.
pub fn spawn_demo_celula_startup_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    layout: Res<SimWorldTransformParams>,
) {
    let positions = [
        Vec2::new(-0.6, 0.0),
        Vec2::new(0.6, 0.0),
        Vec2::new(0.0, 0.8),
    ];
    for pos in positions {
        spawn_celula(&mut commands, &mut id_gen, pos, &layout);
    }
    info!("demo_celula: 3 células spawneadas — observando ciclo metabólico + homeostasis Aqua");
}
