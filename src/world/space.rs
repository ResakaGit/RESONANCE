use bevy::ecs::world::FromWorld;
use bevy::prelude::*;

use crate::layers::SpatialVolume;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::contracts::Pose2;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::spatial_index_backend::{
    BroadphaseEntry2D, Grid2DSpatialBroadphase, SpatialBroadphase, SpatialPose,
};

#[derive(Clone, Copy)]
pub struct SpatialEntry {
    pub entity: Entity,
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Resource)]
pub struct SpatialIndex {
    backend: Grid2DSpatialBroadphase,
}

impl SpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            backend: Grid2DSpatialBroadphase::new(cell_size),
        }
    }

    pub fn clear(&mut self) {
        self.backend.clear();
    }

    pub fn insert(&mut self, entry: SpatialEntry) {
        self.backend.insert(
            SpatialPose::Pose2(Pose2::new(entry.position, entry.radius)),
            entry.entity,
            entry.radius,
        );
    }

    pub fn overlapping_pairs(&self) -> Vec<(SpatialEntry, SpatialEntry)> {
        self.backend
            .overlapping_pairs_canonical()
            .into_iter()
            .map(|(a, b)| (to_spatial_entry(a), to_spatial_entry(b)))
            .collect()
    }

    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<SpatialEntry> {
        self.backend
            .query_radius(center, radius)
            .into_iter()
            .map(to_spatial_entry)
            .collect()
    }
}

impl FromWorld for SpatialIndex {
    fn from_world(_world: &mut World) -> Self {
        SpatialIndex::new(5.0)
    }
}

fn rebuild_spatial_index(
    index: &mut SpatialIndex,
    query: &Query<(Entity, &Transform, &SpatialVolume)>,
    use_xz_ground: bool,
) {
    index.clear();
    for (entity, transform, volume) in query {
        index.insert(SpatialEntry {
            entity,
            position: sim_plane_pos(transform.translation, use_xz_ground),
            radius: volume.radius,
        });
    }
}

/// Índice al inicio de PrePhysics (antes de contención / worldgen que leen pares).
pub fn update_spatial_index_system(
    mut index: ResMut<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    query: Query<(Entity, &Transform, &SpatialVolume)>,
) {
    rebuild_spatial_index(&mut index, &query, layout.use_xz_ground);
}

/// Segundo refresh tras integrar `Transform` en Physics (tensión + interferencia ven posición actual).
pub fn update_spatial_index_after_move_system(
    mut index: ResMut<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    query: Query<(Entity, &Transform, &SpatialVolume)>,
) {
    rebuild_spatial_index(&mut index, &query, layout.use_xz_ground);
}

fn to_spatial_entry(entry: BroadphaseEntry2D) -> SpatialEntry {
    SpatialEntry {
        entity: entry.entity,
        position: entry.position,
        radius: entry.radius,
    }
}
