use std::collections::{BTreeSet, HashMap};

use bevy::prelude::{Entity, IVec2, Vec2, Vec3};

use crate::runtime_platform::contracts::{Pose2, Pose3, SpatialCandidatePair};

/// Pose de entrada agnóstica 2D/3D para broadphase V6.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpatialPose {
    Pose2(Pose2),
    Pose3(Pose3),
}

impl SpatialPose {
    pub fn with_radius(self, radius: f32) -> Self {
        match self {
            Self::Pose2(pose) => Self::Pose2(Pose2::new(pose.position, radius)),
            Self::Pose3(pose) => Self::Pose3(Pose3::new(pose.position, radius)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BroadphaseEntry2D {
    pub entity: Entity,
    pub position: Vec2,
    pub radius: f32,
}

impl BroadphaseEntry2D {
    pub fn new(entity: Entity, position: Vec2, radius: f32) -> Self {
        Self {
            entity,
            position,
            radius: radius.max(0.0),
        }
    }
}

/// Contrato de broadphase para backends espaciales V6.
pub trait SpatialBroadphase {
    fn clear(&mut self);
    fn insert(&mut self, pose: SpatialPose, entity: Entity, radius: f32);
    fn candidate_pairs(&self) -> Vec<SpatialCandidatePair>;
}

/// Backend 2D legacy sobre grid uniforme.
///
/// Política de bordes:
/// - `cell = floor(position / cell_size)` para ambos ejes.
/// - Cada inserción cae en una celda principal.
/// - El chequeo de candidatos consulta vecindad 3x3 alrededor de esa celda.
#[derive(Debug)]
pub struct Grid2DSpatialBroadphase {
    cell_size: f32,
    cells: HashMap<IVec2, Vec<BroadphaseEntry2D>>,
    max_indexed_radius: f32,
}

impl Grid2DSpatialBroadphase {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(0.000_1),
            cells: HashMap::new(),
            max_indexed_radius: 0.0,
        }
    }

    pub fn insert_entry(&mut self, entry: BroadphaseEntry2D) {
        let key = self.cell_for(entry.position);
        self.max_indexed_radius = self.max_indexed_radius.max(entry.radius.max(0.0));
        self.cells.entry(key).or_default().push(entry);
    }

    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<BroadphaseEntry2D> {
        let mut result = Vec::new();
        if !center.is_finite() || !radius.is_finite() || radius < 0.0 {
            return result;
        }
        let cell = self.cell_for(center);
        let effective_radius = radius + self.max_indexed_radius;
        let search = (effective_radius / self.cell_size).ceil().max(1.0) as i32;

        for dy in -search..=search {
            for dx in -search..=search {
                let key = IVec2::new(cell.x + dx, cell.y + dy);
                if let Some(entries) = self.cells.get(&key) {
                    for entry in entries {
                        if entry.position.distance(center) <= radius + entry.radius {
                            result.push(*entry);
                        }
                    }
                }
            }
        }

        result.sort_by_key(|entry| entry.entity.to_bits());
        result
    }

    /// Pares solapados con orden canónico estable (misma semántica que `candidate_pairs`).
    pub fn overlapping_pairs_canonical(&self) -> Vec<(BroadphaseEntry2D, BroadphaseEntry2D)> {
        let mut entity_to_entry: HashMap<u64, BroadphaseEntry2D> = HashMap::new();
        for entries in self.cells.values() {
            for e in entries {
                entity_to_entry.insert(e.entity.to_bits(), *e);
            }
        }

        self.candidate_pairs()
            .into_iter()
            .filter_map(|p| {
                let a = entity_to_entry.get(&p.a.to_bits())?;
                let b = entity_to_entry.get(&p.b.to_bits())?;
                Some((*a, *b))
            })
            .collect()
    }

    /// Compat legacy: preserva semántica antigua (no usar en hot path determinista).
    pub fn overlapping_pairs_legacy(&self) -> Vec<(BroadphaseEntry2D, BroadphaseEntry2D)> {
        let mut result = Vec::new();

        for (cell, entries) in &self.cells {
            let neighbors = self.neighbor_keys(*cell);
            for key in neighbors {
                if let Some(other_entries) = self.cells.get(&key) {
                    for a in entries {
                        for b in other_entries {
                            if a.entity >= b.entity {
                                continue;
                            }
                            let dist = a.position.distance(b.position);
                            if dist < a.radius + b.radius {
                                result.push((*a, *b));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    fn cell_for(&self, position: Vec2) -> IVec2 {
        IVec2::new(
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    fn neighbor_keys(&self, cell: IVec2) -> [IVec2; 9] {
        let mut keys = [cell; 9];
        let mut i = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                keys[i] = IVec2::new(cell.x + dx, cell.y + dy);
                i += 1;
            }
        }
        keys
    }
}

impl SpatialBroadphase for Grid2DSpatialBroadphase {
    fn clear(&mut self) {
        self.cells.clear();
        self.max_indexed_radius = 0.0;
    }

    fn insert(&mut self, pose: SpatialPose, entity: Entity, radius: f32) {
        if let SpatialPose::Pose2(pose2) = pose.with_radius(radius) {
            self.insert_entry(BroadphaseEntry2D::new(entity, pose2.position, pose2.radius));
        }
    }

    fn candidate_pairs(&self) -> Vec<SpatialCandidatePair> {
        let mut pairs = BTreeSet::<(u64, u64)>::new();

        let mut cells: Vec<(IVec2, Vec<BroadphaseEntry2D>)> = self
            .cells
            .iter()
            .map(|(cell, entries)| (*cell, entries.clone()))
            .collect();
        cells.sort_by_key(|(cell, _)| (cell.x, cell.y));

        for (cell, mut entries) in cells {
            entries.sort_by_key(|entry| entry.entity.to_bits());

            let mut neighbors: Vec<IVec2> = self.neighbor_keys(cell).into_iter().collect();
            neighbors.sort_by_key(|n| (n.x, n.y));

            for neighbor in neighbors {
                let Some(other_entries) = self.cells.get(&neighbor) else {
                    continue;
                };

                let mut other_sorted = other_entries.clone();
                other_sorted.sort_by_key(|entry| entry.entity.to_bits());

                for a in &entries {
                    for b in &other_sorted {
                        if a.entity >= b.entity {
                            continue;
                        }

                        let dist = a.position.distance(b.position);
                        if dist < a.radius + b.radius {
                            pairs.insert((a.entity.to_bits(), b.entity.to_bits()));
                        }
                    }
                }
            }
        }

        pairs
            .into_iter()
            .map(|(a, b)| SpatialCandidatePair::new(Entity::from_bits(a), Entity::from_bits(b)))
            .collect()
    }
}

/// Stub 3D inicial para mantener contrato estable de Sprint 06.
#[derive(Debug, Default)]
pub struct Grid3DStubBroadphase {
    entries: Vec<(Entity, Vec3, f32)>,
}

impl SpatialBroadphase for Grid3DStubBroadphase {
    fn clear(&mut self) {
        self.entries.clear();
    }

    fn insert(&mut self, pose: SpatialPose, entity: Entity, radius: f32) {
        if let SpatialPose::Pose3(pose3) = pose.with_radius(radius) {
            self.entries.push((entity, pose3.position, pose3.radius));
        }
    }

    fn candidate_pairs(&self) -> Vec<SpatialCandidatePair> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_pairs_are_stable_for_same_entities() {
        let mut grid = Grid2DSpatialBroadphase::new(5.0);
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let e3 = Entity::from_raw(3);

        grid.insert(
            SpatialPose::Pose2(Pose2::new(Vec2::new(1.0, 1.0), 2.0)),
            e1,
            2.0,
        );
        grid.insert(
            SpatialPose::Pose2(Pose2::new(Vec2::new(2.0, 1.0), 2.0)),
            e3,
            2.0,
        );
        grid.insert(
            SpatialPose::Pose2(Pose2::new(Vec2::new(1.5, 1.0), 2.0)),
            e2,
            2.0,
        );

        let first = grid.candidate_pairs();
        let second = grid.candidate_pairs();

        assert_eq!(first, second);
        assert_eq!(first.len(), 3);
        assert_eq!(first[0], SpatialCandidatePair::new(e1, e2));
        assert_eq!(first[1], SpatialCandidatePair::new(e1, e3));
        assert_eq!(first[2], SpatialCandidatePair::new(e2, e3));
    }

    #[test]
    fn legacy_overlap_retains_previous_pair_detection() {
        let mut grid = Grid2DSpatialBroadphase::new(5.0);
        let a = Entity::from_raw(10);
        let b = Entity::from_raw(11);

        grid.insert_entry(BroadphaseEntry2D::new(a, Vec2::new(0.0, 0.0), 1.0));
        grid.insert_entry(BroadphaseEntry2D::new(b, Vec2::new(1.0, 0.0), 1.0));

        let overlaps = grid.overlapping_pairs_legacy();
        assert!(!overlaps.is_empty());
        assert!(
            overlaps
                .iter()
                .any(|(left, right)| left.entity == a && right.entity == b)
        );

        let canonical = grid.overlapping_pairs_canonical();
        assert!(!canonical.is_empty());
        assert!(
            canonical
                .iter()
                .any(|(left, right)| left.entity == a && right.entity == b)
        );
    }

    #[test]
    fn query_radius_reaches_far_cells_when_radius_is_large() {
        let mut grid = Grid2DSpatialBroadphase::new(5.0);
        let near = Entity::from_raw(1);
        let far = Entity::from_raw(2);
        grid.insert_entry(BroadphaseEntry2D::new(near, Vec2::new(0.0, 0.0), 0.5));
        grid.insert_entry(BroadphaseEntry2D::new(far, Vec2::new(24.0, 0.0), 0.5));

        let hits = grid.query_radius(Vec2::ZERO, 30.0);
        assert!(hits.iter().any(|entry| entry.entity == near));
        assert!(hits.iter().any(|entry| entry.entity == far));
    }
}
