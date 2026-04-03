//! Body plan layout cache: precomputed organ positions, directions, and symmetry mode.

use bevy::prelude::*;

use crate::blueprint::equations::SymmetryMode;
use crate::layers::organ::MAX_ORGANS_PER_ENTITY;

/// Cached body plan: positions + directions for up to MAX_ORGANS_PER_ENTITY organs.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BodyPlanLayout {
    positions: [Vec3; MAX_ORGANS_PER_ENTITY],
    directions: [Vec3; MAX_ORGANS_PER_ENTITY],
    symmetry: SymmetryMode,
    active_count: u8,
}

impl Default for BodyPlanLayout {
    fn default() -> Self {
        Self {
            positions: [Vec3::ZERO; MAX_ORGANS_PER_ENTITY],
            directions: [Vec3::Y; MAX_ORGANS_PER_ENTITY],
            symmetry: SymmetryMode::Bilateral,
            active_count: 0,
        }
    }
}

impl BodyPlanLayout {
    /// Build a layout from precomputed arrays.
    pub fn new(
        positions: [Vec3; MAX_ORGANS_PER_ENTITY],
        directions: [Vec3; MAX_ORGANS_PER_ENTITY],
        symmetry: SymmetryMode,
        active_count: u8,
    ) -> Self {
        Self {
            positions,
            directions,
            symmetry,
            active_count: active_count.min(MAX_ORGANS_PER_ENTITY as u8),
        }
    }

    #[inline]
    pub fn position(&self, index: usize) -> Vec3 {
        if index < self.active_count as usize {
            self.positions[index]
        } else {
            Vec3::ZERO
        }
    }

    #[inline]
    pub fn direction(&self, index: usize) -> Vec3 {
        if index < self.active_count as usize {
            self.directions[index]
        } else {
            Vec3::Y
        }
    }

    #[inline]
    pub fn symmetry(&self) -> SymmetryMode {
        self.symmetry
    }

    #[inline]
    pub fn active_count(&self) -> u8 {
        self.active_count
    }

    #[inline]
    pub fn as_position_slice(&self) -> &[Vec3] {
        &self.positions[..self.active_count as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_has_zero_active_count() {
        let layout = BodyPlanLayout::default();
        assert_eq!(layout.active_count(), 0);
        assert_eq!(layout.symmetry(), SymmetryMode::Bilateral);
        assert!(layout.as_position_slice().is_empty());
    }

    #[test]
    fn position_and_direction_out_of_range_return_defaults() {
        let layout = BodyPlanLayout::default();
        assert_eq!(layout.position(0), Vec3::ZERO);
        assert_eq!(layout.direction(0), Vec3::Y);
        assert_eq!(layout.position(99), Vec3::ZERO);
    }

    #[test]
    fn new_clamps_active_count_to_max() {
        let layout = BodyPlanLayout::new(
            [Vec3::ZERO; MAX_ORGANS_PER_ENTITY],
            [Vec3::Y; MAX_ORGANS_PER_ENTITY],
            SymmetryMode::Radial,
            255,
        );
        assert_eq!(layout.active_count(), MAX_ORGANS_PER_ENTITY as u8);
    }

    #[test]
    fn as_position_slice_returns_active_entries() {
        let mut positions = [Vec3::ZERO; MAX_ORGANS_PER_ENTITY];
        positions[0] = Vec3::X;
        positions[1] = Vec3::Y;
        positions[2] = Vec3::Z;
        let layout = BodyPlanLayout::new(
            positions,
            [Vec3::Y; MAX_ORGANS_PER_ENTITY],
            SymmetryMode::Bilateral,
            3,
        );
        let slice = layout.as_position_slice();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], Vec3::X);
        assert_eq!(slice[1], Vec3::Y);
        assert_eq!(slice[2], Vec3::Z);
    }
}
