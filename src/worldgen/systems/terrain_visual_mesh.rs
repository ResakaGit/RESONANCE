//! IWG-4 — ECS wiring for terrain mesh generation + sync to `Mesh3d`.
//!
//! Pattern: FixedUpdate writes `TerrainMeshResource` → Update reads and syncs to render.

use bevy::prelude::*;

use crate::blueprint::equations::inferred_world_geometry::build_terrain_visuals;
use crate::topology::{TerrainField, generate_terrain_mesh};
use crate::worldgen::EnergyFieldGrid;

/// Marks the entity that carries the terrain mesh.
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct TerrainMeshEntity;

/// Terrain mesh generated in FixedUpdate, pending sync to Mesh3d.
#[derive(Resource, Default)]
pub struct TerrainMeshResource {
    pub mesh: Option<Mesh>,
    pub dirty: bool,
}

/// Generates terrain mesh from TerrainField + EnergyFieldGrid.
///
/// Phase: `MorphologicalLayer` (FixedUpdate). Only runs when either resource changed.
pub fn terrain_mesh_generation_system(
    terrain: Option<Res<TerrainField>>,
    grid: Option<Res<EnergyFieldGrid>>,
    mut terrain_mesh_res: ResMut<TerrainMeshResource>,
) {
    let Some(ref terrain) = terrain else {
        return;
    };
    let Some(ref grid) = grid else {
        return;
    };
    if !terrain.is_changed() && !grid.is_changed() {
        return;
    }

    let visuals = build_terrain_visuals(grid, terrain);
    let mesh = generate_terrain_mesh(terrain, &visuals);
    terrain_mesh_res.mesh = mesh;
    terrain_mesh_res.dirty = true;
}

/// Syncs generated terrain mesh to Mesh3d + MeshMaterial3d (Update schedule).
pub fn terrain_mesh_sync_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_mesh_res: ResMut<TerrainMeshResource>,
    existing: Query<Entity, With<TerrainMeshEntity>>,
) {
    if !terrain_mesh_res.dirty {
        return;
    }
    let Some(mesh) = terrain_mesh_res.mesh.take() else {
        terrain_mesh_res.dirty = false;
        return;
    };

    let mesh_handle = meshes.add(mesh);

    if let Some(entity) = existing.iter().next() {
        commands.entity(entity).insert(Mesh3d(mesh_handle));
    } else {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        });
        commands.spawn((
            TerrainMeshEntity,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::IDENTITY,
        ));
    }

    terrain_mesh_res.dirty = false;
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
        let res = TerrainMeshResource::default();
        assert!(!res.dirty);
        assert!(res.mesh.is_none());
    }

    #[test]
    fn generation_produces_mesh_and_sets_dirty() {
        let mut app = App::new();
        app.insert_resource(make_grid(4, 4));
        app.insert_resource(make_terrain(4, 4));
        app.init_resource::<TerrainMeshResource>();
        app.add_systems(Update, terrain_mesh_generation_system);

        app.update();

        let res = app.world().resource::<TerrainMeshResource>();
        assert!(res.dirty);
        assert!(res.mesh.is_some());
    }

    #[test]
    fn generation_no_terrain_returns_early() {
        let mut app = App::new();
        app.insert_resource(make_grid(4, 4));
        // No TerrainField inserted.
        app.init_resource::<TerrainMeshResource>();
        app.add_systems(Update, terrain_mesh_generation_system);

        app.update();

        let res = app.world().resource::<TerrainMeshResource>();
        assert!(!res.dirty);
        assert!(res.mesh.is_none());
    }

    #[test]
    fn generation_no_grid_returns_early() {
        let mut app = App::new();
        app.insert_resource(make_terrain(4, 4));
        // No EnergyFieldGrid inserted.
        app.init_resource::<TerrainMeshResource>();
        app.add_systems(Update, terrain_mesh_generation_system);

        app.update();

        let res = app.world().resource::<TerrainMeshResource>();
        assert!(!res.dirty);
        assert!(res.mesh.is_none());
    }
}
