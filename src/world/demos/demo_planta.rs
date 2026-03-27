//! Demo: planta fotosintética.
//! 2 plantas en campo Terra+Aqua; observa fotosíntesis, crecimiento morfogenético y ramificación.
//! `RESONANCE_MAP=demo_planta cargo run`

use bevy::prelude::*;

use crate::blueprint::IdGenerator;
use crate::entities::archetypes::catalog::spawn_planta_demo;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

pub const DEMO_PLANTA_SLUG: &str = "demo_planta";

/// 2 plantas en zonas de suelo Terra, con acceso a lluvia Aqua.
pub fn spawn_demo_planta_startup_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    layout: Res<SimWorldTransformParams>,
) {
    let positions = [
        Vec2::new(-2.5, -1.5),
        Vec2::new( 2.5, -1.5),
    ];
    for pos in positions {
        spawn_planta_demo(&mut commands, &mut id_gen, pos, &layout);
    }
    info!("demo_planta: 2 plantas — observando fotosíntesis + crecimiento Terra/Aqua");
}
