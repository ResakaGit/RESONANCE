//! Estado energético persistente per-órgano — sobrevive rebuild del OrganManifest.
//! Persistent per-organ energy state — survives OrganManifest rebuild.

use bevy::prelude::*;

/// Maximum organs per entity (mirrors MAX_ORGANS_PER_ENTITY).
pub const MAX_ORGAN_SLOTS: usize = 12;

/// Persistent per-organ energy state. Each slot is an energy packet with
/// physical properties. Behavior derives from density, not from role.
///
/// Estado energético persistente per-órgano. Cada slot es un paquete de
/// energía con propiedades físicas. El comportamiento deriva de densidad.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct OrganEnergySlots {
    /// Energy per organ slot.
    pub qe: [f32; MAX_ORGAN_SLOTS],
    /// Volume per organ slot (determines density = qe/volume).
    pub volume: [f32; MAX_ORGAN_SLOTS],
    /// Bond energy per organ slot (structural rigidity).
    pub bond_energy: [f32; MAX_ORGAN_SLOTS],
    /// Number of active slots.
    pub len: u8,
}

impl Default for OrganEnergySlots {
    fn default() -> Self {
        Self {
            qe: [0.0; MAX_ORGAN_SLOTS],
            volume: [0.0; MAX_ORGAN_SLOTS],
            bond_energy: [0.0; MAX_ORGAN_SLOTS],
            len: 0,
        }
    }
}

impl OrganEnergySlots {
    /// Density of organ at index. Returns 0 if invalid.
    #[inline]
    pub fn density(&self, idx: usize) -> f32 {
        if idx >= self.len as usize || self.volume[idx] <= 0.0 {
            return 0.0;
        }
        self.qe[idx] / self.volume[idx]
    }

    /// Total energy across all active slots.
    #[inline]
    pub fn total_qe(&self) -> f32 {
        self.qe.iter().take(self.len as usize).sum()
    }

    /// Total density across all active slots.
    #[inline]
    pub fn total_density(&self) -> f32 {
        (0..self.len as usize).map(|i| self.density(i)).sum()
    }
}
