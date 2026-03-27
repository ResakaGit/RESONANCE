//! IWG-5 — Water surface detection + mesh generation.
//!
//! Pattern: FixedUpdate writes `WaterMeshResource` → Update reads and syncs to Mesh3d.

use bevy::prelude::*;

use crate::blueprint::constants::inferred_world_geometry::{
    WATER_MIN_CELLS, WATER_SUBDIVISIONS,
};
use crate::blueprint::equations::inferred_world_geometry::{
    build_water_mesh, water_surface_height,
};
use crate::layers::MatterState;
use crate::topology::TerrainField;
use crate::worldgen::EnergyFieldGrid;

/// Water mesh generated in FixedUpdate, pending sync to Mesh3d.
#[derive(Resource, Default)]
pub struct WaterMeshResource {
    pub mesh:  Option<Mesh>,
    pub dirty: bool,
}

/// Marks the entity carrying the water surface mesh.
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct WaterMeshEntity;

/// Detects liquid regions and generates water surface mesh.
///
/// Phase: `MorphologicalLayer` (FixedUpdate), after `terrain_mesh_generation_system`.
pub fn water_surface_system(
    field: Option<Res<EnergyFieldGrid>>,
    terrain: Option<Res<TerrainField>>,
    mut water_res: ResMut<WaterMeshResource>,
) {
    let Some(ref field) = field else {
        return;
    };
    let Some(ref terrain) = terrain else {
        return;
    };
    if !field.is_changed() && !terrain.is_changed() {
        return;
    }

    let w = field.width;
    let h = field.height;

    // Collect liquid cells: index + terrain altitude.
    let mut liquid_heights: Vec<f32> = Vec::with_capacity((w as usize * h as usize) / 4);
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    let mut min_terrain_h = f32::MAX;

    for cy in 0..h {
        for cx in 0..w {
            let idx = cy as usize * w as usize + cx as usize;
            let Some(cell) = field.cell_xy(cx, cy) else {
                continue;
            };
            if cell.matter_state != MatterState::Liquid {
                continue;
            }
            let alt = terrain.altitude.get(idx).copied().unwrap_or(0.0);
            liquid_heights.push(alt);
            if alt < min_terrain_h {
                min_terrain_h = alt;
            }

            let world_x = terrain.origin.x + cx as f32 * terrain.cell_size;
            let world_z = terrain.origin.y + cy as f32 * terrain.cell_size;
            let world_x_end = world_x + terrain.cell_size;
            let world_z_end = world_z + terrain.cell_size;

            if world_x < min_x { min_x = world_x; }
            if world_x_end > max_x { max_x = world_x_end; }
            if world_z < min_z { min_z = world_z; }
            if world_z_end > max_z { max_z = world_z_end; }
        }
    }

    if (liquid_heights.len() as u32) < WATER_MIN_CELLS {
        if water_res.mesh.is_some() || water_res.dirty {
            water_res.mesh = None;
            water_res.dirty = true;
        }
        return;
    }

    if min_terrain_h == f32::MAX {
        min_terrain_h = 0.0;
    }

    let water_h = water_surface_height(&liquid_heights, min_terrain_h);

    let bounds_min = Vec3::new(min_x, 0.0, min_z);
    let bounds_max = Vec3::new(max_x, 0.0, max_z);

    let mesh = build_water_mesh(
        bounds_min,
        bounds_max,
        water_h,
        WATER_SUBDIVISIONS,
        &terrain.altitude,
        terrain.height,
        terrain.width,
    );

    water_res.mesh = Some(mesh);
    water_res.dirty = true;
}

/// Syncs generated water mesh to Mesh3d + MeshMaterial3d (Update schedule).
pub fn water_mesh_sync_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_res: ResMut<WaterMeshResource>,
    existing: Query<Entity, With<WaterMeshEntity>>,
) {
    if !water_res.dirty {
        return;
    }

    let Some(mesh) = water_res.mesh.take() else {
        // No mesh — despawn existing water entity if any.
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
        water_res.dirty = false;
        return;
    };

    let mesh_handle = meshes.add(mesh);

    if let Some(entity) = existing.iter().next() {
        commands.entity(entity).insert(Mesh3d(mesh_handle));
    } else {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.4, 0.7, 0.7),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.3,
            metallic: 0.1,
            ..default()
        });
        commands.spawn((
            WaterMeshEntity,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::IDENTITY,
        ));
    }

    water_res.dirty = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    fn make_grid(w: u32, h: u32) -> EnergyFieldGrid {
        EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO)
    }

    fn make_terrain(w: u32, h: u32) -> TerrainField {
        TerrainField::new(w, h, 1.0, Vec2::ZERO, 0)
    }

    #[test]
    fn resource_default_not_dirty() {
        let res = WaterMeshResource::default();
        assert!(!res.dirty);
        assert!(res.mesh.is_none());
    }

    #[test]
    fn no_liquid_cells_no_mesh() {
        let mut app = App::new();
        app.insert_resource(make_grid(4, 4));
        app.insert_resource(make_terrain(4, 4));
        app.init_resource::<WaterMeshResource>();
        app.add_systems(Update, water_surface_system);

        app.update();

        let res = app.world().resource::<WaterMeshResource>();
        // All cells default to Solid → no liquid → no mesh.
        assert!(res.mesh.is_none());
    }

    #[test]
    fn liquid_cells_produce_mesh() {
        let mut app = App::new();
        let mut grid = make_grid(4, 4);
        // Set enough cells to Liquid state.
        for cy in 0..4u32 {
            for cx in 0..4u32 {
                if let Some(cell) = grid.cell_xy_mut(cx, cy) {
                    cell.matter_state = MatterState::Liquid;
                }
            }
        }
        app.insert_resource(grid);
        app.insert_resource(make_terrain(4, 4));
        app.init_resource::<WaterMeshResource>();
        app.add_systems(Update, water_surface_system);

        app.update();

        let res = app.world().resource::<WaterMeshResource>();
        assert!(res.dirty);
        assert!(res.mesh.is_some());
    }

    #[test]
    fn below_min_cells_no_mesh() {
        let mut app = App::new();
        let mut grid = make_grid(4, 4);
        // Set only 2 liquid cells — below WATER_MIN_CELLS threshold.
        if let Some(cell) = grid.cell_xy_mut(0, 0) {
            cell.matter_state = MatterState::Liquid;
        }
        if let Some(cell) = grid.cell_xy_mut(1, 0) {
            cell.matter_state = MatterState::Liquid;
        }
        app.insert_resource(grid);
        app.insert_resource(make_terrain(4, 4));
        app.init_resource::<WaterMeshResource>();
        app.add_systems(Update, water_surface_system);

        app.update();

        let res = app.world().resource::<WaterMeshResource>();
        assert!(res.mesh.is_none());
    }

    #[test]
    fn no_terrain_returns_early() {
        let mut app = App::new();
        app.insert_resource(make_grid(4, 4));
        // No TerrainField inserted.
        app.init_resource::<WaterMeshResource>();
        app.add_systems(Update, water_surface_system);

        app.update();

        let res = app.world().resource::<WaterMeshResource>();
        assert!(!res.dirty);
        assert!(res.mesh.is_none());
    }

    #[test]
    fn no_grid_returns_early() {
        let mut app = App::new();
        app.insert_resource(make_terrain(4, 4));
        // No EnergyFieldGrid inserted.
        app.init_resource::<WaterMeshResource>();
        app.add_systems(Update, water_surface_system);

        app.update();

        let res = app.world().resource::<WaterMeshResource>();
        assert!(!res.dirty);
        assert!(res.mesh.is_none());
    }
}
