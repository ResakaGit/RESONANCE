//! Sistemas Fog of War (G12): stamp en grid + máscara por equipo sobre entidades con L9.
//!
//! Si falta [`crate::world::FogOfWarGrid`], los sistemas no hacen nada (herramientas/tests mínimos).
//! En el juego completo el grid se inserta en bootstrap + `init_fog_of_war_from_energy_field_system`.

use bevy::prelude::*;

use crate::layers::vision_fog::{FogHiddenMask, VisionBlocker, VisionFogAnchor, VisionProvider};
use crate::layers::{Faction, MobaIdentity};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::world::fog_of_war::{FogOfWarGrid, NUM_FOG_TEAMS, faction_for_fog_team};

const PLANE_MOVE_EPS2: f32 = 1e-5;

fn should_hide_from_viewer(entity_faction: Faction, viewer_faction: Faction) -> bool {
    if entity_faction == viewer_faction {
        return false;
    }
    true
}

/// Actualiza refcount del grid cuando un proveedor se mueve o spawnea.
pub fn fog_of_war_provider_system(
    layout: Res<SimWorldTransformParams>,
    fog: Option<ResMut<FogOfWarGrid>>,
    mut q: Query<(&Transform, &VisionProvider, &mut VisionFogAnchor)>,
) {
    let Some(mut fog) = fog else {
        return;
    };
    let xz = layout.use_xz_ground;
    let mut stamped = false;
    for (tf, prov, mut anchor) in &mut q {
        let team = prov.team() as usize;
        if team >= NUM_FOG_TEAMS {
            continue;
        }
        let plane = sim_plane_pos(tf.translation, xz);
        if !plane.is_finite() {
            continue;
        }
        let r = prov.max_radius();
        if anchor.has_last {
            let d = plane - anchor.last_plane;
            if d.length_squared() < PLANE_MOVE_EPS2 {
                continue;
            }
            fog.unstamp_disk(team, anchor.last_plane, r);
        }
        fog.stamp_disk(team, plane, r);
        anchor.last_plane = plane;
        anchor.has_last = true;
        stamped = true;
    }
    if stamped {
        fog.bump_stamp_generation();
    }
}

/// Marca enemigos (distinta facción PvP que el observador del equipo) como ocultos si la celda no es visible.
pub fn fog_visibility_mask_system(
    layout: Res<SimWorldTransformParams>,
    fog: Option<Res<FogOfWarGrid>>,
    mut commands: Commands,
    q: Query<
        (Entity, &Transform, &MobaIdentity, Option<&FogHiddenMask>),
        (Without<VisionProvider>, Without<VisionBlocker>),
    >,
) {
    let Some(fog) = fog else {
        return;
    };
    let xz = layout.use_xz_ground;
    for (entity, tf, id, prev) in &q {
        let plane = sim_plane_pos(tf.translation, xz);
        let cell = fog.world_to_cell(plane);
        let mut mask = 0u8;
        for t in 0..NUM_FOG_TEAMS {
            let Some(vf) = faction_for_fog_team(t as u8) else {
                continue;
            };
            let hide = if should_hide_from_viewer(id.faction(), vf) {
                match cell {
                    Some((cx, cy)) => !fog.is_visible(t, cx, cy),
                    None => true,
                }
            } else {
                false
            };
            if hide {
                mask |= 1u8 << t;
            }
        }

        let old = prev.map(|p| p.0).unwrap_or(0);
        if old == mask {
            continue;
        }
        if mask == 0 {
            commands.entity(entity).remove::<FogHiddenMask>();
        } else {
            commands.entity(entity).insert(FogHiddenMask(mask));
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y;
    use crate::world::fog_of_war::FogOfWarGrid;
    use bevy::math::Vec2;

    #[test]
    fn provider_move_updates_visibility_and_mask() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams {
            use_xz_ground: true,
            standing_y: DEFAULT_SIM_STANDING_Y,
            ..default()
        });
        app.insert_resource(FogOfWarGrid::aligned_with_energy_field(
            &crate::worldgen::EnergyFieldGrid::new(32, 32, 2.0, Vec2::new(-32.0, -32.0)),
        ));
        app.add_systems(
            Update,
            (fog_of_war_provider_system, fog_visibility_mask_system).chain(),
        );

        let hero = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, DEFAULT_SIM_STANDING_Y, 0.0),
                VisionProvider::new(8.0, 0.05, 0),
                VisionFogAnchor::default(),
            ))
            .id();

        let enemy = app
            .world_mut()
            .spawn((
                Transform::from_xyz(20.0, DEFAULT_SIM_STANDING_Y, 0.0),
                MobaIdentity {
                    faction: Faction::Blue,
                    relational_tags: vec![],
                    critical_multiplier: 1.0,
                },
            ))
            .id();

        app.update();
        assert!(
            app.world()
                .entity(enemy)
                .get::<FogHiddenMask>()
                .is_some_and(|m| m.hidden_from_team(0)),
            "Blue lejos no debería ser visible para visión Red"
        );

        app.world_mut().entity_mut(hero).insert(Transform::from_xyz(
            18.0,
            DEFAULT_SIM_STANDING_Y,
            0.0,
        ));
        app.update();
        assert!(
            app.world().entity(enemy).get::<FogHiddenMask>().is_none(),
            "Al acercar el proveedor Red, el enemigo debería revelarse al equipo 0"
        );
    }

    #[test]
    fn fog_systems_skip_without_grid_resource() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams {
            use_xz_ground: true,
            standing_y: DEFAULT_SIM_STANDING_Y,
            ..default()
        });
        app.add_systems(
            Update,
            (fog_of_war_provider_system, fog_visibility_mask_system).chain(),
        );
        let e = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, DEFAULT_SIM_STANDING_Y, 0.0),
                VisionProvider::new(4.0, 0.05, 0),
                VisionFogAnchor::default(),
                MobaIdentity {
                    faction: Faction::Blue,
                    relational_tags: vec![],
                    critical_multiplier: 1.0,
                },
            ))
            .id();
        app.update();
        assert!(app.world().entity(e).get::<FogHiddenMask>().is_none());
    }
}
