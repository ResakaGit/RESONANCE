//! Sistemas ECS de eco-boundaries (actualización de `EcoBoundaryField` en PrePhysics).

use bevy::prelude::*;

use crate::eco::boundary_field::EcoBoundaryField;
use crate::worldgen::{EnergyFieldGrid, WorldgenLodContext};

/// Compara `grid.generation` con el cache; respeta `BOUNDARY_RECOMPUTE_COOLDOWN` vía `sim_tick`.
pub fn eco_boundaries_system(
    grid: Res<EnergyFieldGrid>,
    mut field: ResMut<EcoBoundaryField>,
    lod: Res<WorldgenLodContext>,
) {
    field.recompute_if_needed(&grid, lod.sim_tick);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::MatterState;
    use bevy::math::Vec2;
    use bevy::prelude::{App, MinimalPlugins};

    #[test]
    fn eco_boundaries_system_ejecuta_sin_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO));
        app.insert_resource(EcoBoundaryField::default());
        app.insert_resource(WorldgenLodContext::default());
        app.add_systems(Update, eco_boundaries_system);
        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            grid.generation = 1;
            for y in 0..2 {
                for x in 0..2 {
                    let c = grid.cell_xy_mut(x, y).unwrap();
                    c.accumulated_qe = 3.0;
                    c.temperature = 1.2;
                    c.matter_state = MatterState::Liquid;
                    c.dominant_frequency_hz = 250.0;
                }
            }
        }
        app.update();
        assert_eq!(app.world().resource::<EcoBoundaryField>().markers.len(), 4);
    }

    #[test]
    fn sistema_no_recomputa_si_generacion_igual_entre_ticks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO));
        app.insert_resource(EcoBoundaryField::default());
        app.insert_resource(WorldgenLodContext {
            focus_world: None,
            sim_tick: 5,
        });
        app.add_systems(Update, eco_boundaries_system);
        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            grid.generation = 1;
            for y in 0..2 {
                for x in 0..2 {
                    let c = grid.cell_xy_mut(x, y).unwrap();
                    c.accumulated_qe = 3.0;
                    c.temperature = 1.2;
                    c.matter_state = MatterState::Liquid;
                    c.dominant_frequency_hz = 250.0;
                }
            }
        }
        app.update();
        let snap = app.world().resource::<EcoBoundaryField>().markers.clone();
        app.update();
        assert_eq!(app.world().resource::<EcoBoundaryField>().markers, snap);
    }

    #[test]
    fn sistema_respeta_cooldown_entre_generaciones() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO));
        app.insert_resource(EcoBoundaryField::default());
        app.insert_resource(WorldgenLodContext {
            focus_world: None,
            sim_tick: 10,
        });
        app.add_systems(Update, eco_boundaries_system);
        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            grid.generation = 1;
            for y in 0..2 {
                for x in 0..2 {
                    let c = grid.cell_xy_mut(x, y).unwrap();
                    c.accumulated_qe = 3.0;
                    c.temperature = 1.2;
                    c.matter_state = MatterState::Liquid;
                    c.dominant_frequency_hz = 250.0;
                }
            }
        }
        app.update();
        assert_eq!(
            app.world()
                .resource::<EcoBoundaryField>()
                .last_seen_grid_generation,
            1
        );

        app.world_mut().resource_mut::<EnergyFieldGrid>().generation = 2;
        app.update();
        assert_eq!(
            app.world()
                .resource::<EcoBoundaryField>()
                .last_seen_grid_generation,
            1,
            "same sim_tick → cooldown blocks even if generation increases"
        );

        app.world_mut()
            .resource_mut::<WorldgenLodContext>()
            .sim_tick = 12;
        app.update();
        assert_eq!(
            app.world()
                .resource::<EcoBoundaryField>()
                .last_seen_grid_generation,
            2
        );
    }
}
