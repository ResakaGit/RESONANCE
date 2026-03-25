//! `TerrainField`: grid SoA alineado con `EnergyFieldGrid` (misma convención mundo↔celda).

use bevy::math::Vec2;
use bevy::prelude::{Reflect, Resource};

use crate::topology::contracts::{TerrainSample, TerrainType};

/// Estado global del terreno por celda — no es capa ECS; es resource hermano del campo energético.
#[derive(Clone, Debug, Resource, Reflect)]
pub struct TerrainField {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    pub seed: u64,
    /// Contador de versión: mutaciones runtime lo incrementan para invalidar caches.
    pub generation: u32,
    pub altitude: Vec<f32>,
    pub slope: Vec<f32>,
    pub aspect: Vec<f32>,
    pub drainage: Vec<Vec2>,
    pub drainage_accumulation: Vec<f32>,
    pub terrain_type: Vec<TerrainType>,
}

impl TerrainField {
    pub fn new(width: u32, height: u32, cell_size: f32, origin: Vec2, seed: u64) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let cell_size = if cell_size.is_finite() {
            cell_size.max(0.001)
        } else {
            1.0
        };
        let n = width as usize * height as usize;
        Self {
            width,
            height,
            cell_size,
            origin,
            seed,
            generation: 0,
            altitude: vec![0.0; n],
            slope: vec![0.0; n],
            aspect: vec![0.0; n],
            drainage: vec![Vec2::ZERO; n],
            drainage_accumulation: vec![0.0; n],
            terrain_type: vec![TerrainType::Plain; n],
        }
    }

    #[inline]
    pub fn total_cells(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[inline]
    pub fn is_valid(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Índice row-major `y * width + x`, coherente con `EnergyFieldGrid::index_of`.
    /// **Invariante:** llamar solo si `is_valid(x, y)`; si no, el índice puede aliasar otra celda.
    #[inline]
    pub fn cell_index(&self, x: u32, y: u32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    /// Misma semántica que `EnergyFieldGrid::cell_coords`.
    pub fn world_to_cell(&self, world_pos: Vec2) -> Option<(u32, u32)> {
        if !world_pos.is_finite() {
            return None;
        }
        let rel = world_pos - self.origin;
        if rel.x < 0.0 || rel.y < 0.0 {
            return None;
        }
        let x = (rel.x / self.cell_size).floor() as i32;
        let y = (rel.y / self.cell_size).floor() as i32;
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some((x as u32, y as u32))
    }

    /// Centro de la celda en mundo — misma fórmula que `EnergyFieldGrid::world_pos`.
    /// Panic si `(x,y)` está fuera del grid (mejor que devolver un punto “fantasma”).
    #[inline]
    pub fn cell_to_world(&self, x: u32, y: u32) -> Vec2 {
        assert!(
            self.is_valid(x, y),
            "cell_to_world: coordenadas fuera del grid"
        );
        Vec2::new(
            self.origin.x + (x as f32 + 0.5) * self.cell_size,
            self.origin.y + (y as f32 + 0.5) * self.cell_size,
        )
    }

    pub fn sample_at(&self, x: u32, y: u32) -> TerrainSample {
        assert!(self.is_valid(x, y), "sample_at: coordenadas fuera del grid");
        let i = self.cell_index(x, y);
        TerrainSample {
            altitude: self.altitude[i],
            slope: self.slope[i],
            aspect: self.aspect[i],
            drainage: self.drainage[i],
            drainage_accumulation: self.drainage_accumulation[i],
            terrain_type: self.terrain_type[i],
        }
    }

    pub fn sample_at_world(&self, pos: Vec2) -> Option<TerrainSample> {
        let (x, y) = self.world_to_cell(pos)?;
        Some(self.sample_at(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    use crate::worldgen::EnergyFieldGrid;

    #[test]
    fn new_100x100_has_10k_cells() {
        let f = TerrainField::new(100, 100, 2.0, Vec2::ZERO, 42);
        assert_eq!(f.total_cells(), 10_000);
        assert_eq!(f.altitude.len(), 10_000);
        assert_eq!(f.terrain_type.len(), 10_000);
    }

    #[test]
    fn world_to_cell_example_from_sprint() {
        let f = TerrainField::new(100, 100, 2.0, Vec2::ZERO, 0);
        assert_eq!(f.world_to_cell(Vec2::new(5.0, 5.0)), Some((2, 2)));
    }

    #[test]
    fn world_to_cell_outside_returns_none() {
        let f = TerrainField::new(10, 10, 1.0, Vec2::ZERO, 0);
        assert!(f.world_to_cell(Vec2::new(-0.1, 1.0)).is_none());
        assert!(f.world_to_cell(Vec2::new(10.1, 1.0)).is_none());
        assert!(f.world_to_cell(Vec2::new(1.0, 10.1)).is_none());
    }

    #[test]
    fn world_to_cell_non_finite_returns_none() {
        let f = TerrainField::new(4, 4, 1.0, Vec2::ZERO, 0);
        assert!(f.world_to_cell(Vec2::new(f32::NAN, 0.0)).is_none());
        assert!(f.world_to_cell(Vec2::new(0.0, f32::INFINITY)).is_none());
    }

    #[test]
    fn world_to_cell_matches_energy_field_grid() {
        let w = 11u32;
        let h = 7u32;
        let cs = 1.25f32;
        let origin = Vec2::new(-3.0, 2.0);
        let t = TerrainField::new(w, h, cs, origin, 0);
        let e = EnergyFieldGrid::new(w, h, cs, origin);
        for pos in [
            Vec2::new(0.0, 2.0),
            Vec2::new(-2.9, 2.1),
            Vec2::new(5.0, 6.25),
        ] {
            assert_eq!(t.world_to_cell(pos), e.cell_coords(pos));
        }
    }

    #[test]
    fn cell_to_world_is_cell_center() {
        let f = TerrainField::new(8, 8, 2.0, Vec2::new(-8.0, -8.0), 0);
        let center = f.cell_to_world(3, 4);
        assert_eq!(center.x, -8.0 + (3.5 * 2.0));
        assert_eq!(center.y, -8.0 + (4.5 * 2.0));
        let back = f.world_to_cell(center);
        assert_eq!(back, Some((3, 4)));
    }

    #[test]
    fn sample_at_matches_parallel_arrays() {
        let mut f = TerrainField::new(4, 4, 1.0, Vec2::ZERO, 0);
        let i = f.cell_index(2, 1);
        f.altitude[i] = 12.5;
        f.slope[i] = 3.0;
        f.aspect[i] = 90.0;
        f.drainage[i] = Vec2::new(1.0, -1.0);
        f.drainage_accumulation[i] = 77.0;
        f.terrain_type[i] = TerrainType::Valley;
        let s = f.sample_at(2, 1);
        assert_eq!(s.altitude, 12.5);
        assert_eq!(s.slope, 3.0);
        assert_eq!(s.aspect, 90.0);
        assert_eq!(s.drainage, Vec2::new(1.0, -1.0));
        assert_eq!(s.drainage_accumulation, 77.0);
        assert_eq!(s.terrain_type, TerrainType::Valley);
    }

    #[test]
    fn cell_index_corners() {
        let f = TerrainField::new(10, 20, 1.0, Vec2::ZERO, 0);
        assert_eq!(f.cell_index(0, 0), 0);
        assert_eq!(f.cell_index(1, 0), 1);
        assert_eq!(f.cell_index(0, 1), 10);
    }
}
