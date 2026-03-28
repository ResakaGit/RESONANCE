//! AD-1: Internal energy distribution across entity body (8 axial nodes).
//!
//! Diffusion between nodes creates emergent gradients.
//! Valleys (qe ≤ 0) trigger division. Peaks shape morphology.
//! Axiom 1: entity = energy distribution, not energy point.

use bevy::prelude::*;

/// Internal energy field: 8 nodes along the body axis.
/// Diffusion + dissipation create valleys. Valleys at zero → entity splits.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct InternalEnergyField {
    pub nodes: [f32; 8],
}

impl InternalEnergyField {
    /// Uniform distribution from total qe.
    pub fn uniform(total_qe: f32) -> Self {
        let per_node = (total_qe / 8.0).max(0.0);
        Self { nodes: [per_node; 8] }
    }

    /// Total energy across all nodes.
    #[inline]
    pub fn total(&self) -> f32 {
        self.nodes.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_distributes_evenly() {
        let f = InternalEnergyField::uniform(80.0);
        for n in &f.nodes {
            assert!((*n - 10.0).abs() < 1e-5);
        }
    }

    #[test]
    fn total_matches_sum() {
        let f = InternalEnergyField::uniform(100.0);
        assert!((f.total() - 100.0).abs() < 1e-4);
    }

    #[test]
    fn zero_energy_all_zeros() {
        let f = InternalEnergyField::uniform(0.0);
        assert_eq!(f.total(), 0.0);
    }
}
