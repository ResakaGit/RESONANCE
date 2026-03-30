//! Population census — snapshot poblacional por generación.
//! Population census — per-generation population snapshot.
//!
//! `EntitySnapshot` captura el estado de una entidad al final de una evaluación.
//! `PopulationCensus` agrupa todos los snapshots de una generación.
//! Stateless capture: pure function over `SimWorldFlat` → owned data.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::lineage::LineageId;
use crate::batch::MAX_ENTITIES;

// ─── EntitySnapshot ─────────────────────────────────────────────────────────

/// Estado inmutable de una entidad al final de una evaluación. Copy, stack-allocated.
/// Immutable entity state at the end of an evaluation. Copy, stack-allocated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntitySnapshot {
    pub lineage_id:     LineageId,
    pub world_index:    u16,
    pub slot_index:     u8,
    pub archetype:      u8,
    pub alive:          bool,
    pub qe:             f32,
    pub radius:         f32,
    pub frequency_hz:   f32,
    pub growth_bias:    f32,
    pub mobility_bias:  f32,
    pub branching_bias: f32,
    pub resilience:     f32,
    pub trophic_class:  u8,
    pub age_ticks:      u16,
}

impl EntitySnapshot {
    /// Extrae snapshot desde un EntitySlot del batch simulator.
    /// Extracts snapshot from a batch simulator EntitySlot.
    #[inline]
    pub fn from_slot(
        slot: &EntitySlot,
        lineage_id: LineageId,
        world_index: u16,
        slot_index: u8,
    ) -> Self {
        Self {
            lineage_id,
            world_index,
            slot_index,
            archetype:      slot.archetype,
            alive:          slot.alive,
            qe:             slot.qe,
            radius:         slot.radius,
            frequency_hz:   slot.frequency_hz,
            growth_bias:    slot.growth_bias,
            mobility_bias:  slot.mobility_bias,
            branching_bias: slot.branching_bias,
            resilience:     slot.resilience,
            trophic_class:  slot.trophic_class,
            age_ticks:      0, // Batch EntitySlot no rastrea edad; se infiere del lineage si se necesita
        }
    }
}

// ─── PopulationCensus ───────────────────────────────────────────────────────

/// Censo completo de una generación. Owned, heap-allocated (snapshots × N_worlds × 64).
/// Complete census of one generation. Owned, heap-allocated.
#[derive(Clone, Debug)]
pub struct PopulationCensus {
    pub generation: u32,
    pub snapshots:  Vec<EntitySnapshot>,
}

impl PopulationCensus {
    /// Captura censo desde una colección de mundos.
    /// Captures census from a collection of worlds.
    pub fn capture(generation: u32, worlds: &[SimWorldFlat]) -> Self {
        let capacity = worlds.len() * MAX_ENTITIES;
        let mut snapshots = Vec::with_capacity(capacity);
        for (wi, world) in worlds.iter().enumerate() {
            let mut mask = world.alive_mask;
            while mask != 0 {
                let si = mask.trailing_zeros() as usize;
                mask &= mask - 1;
                let slot = &world.entities[si];
                let lineage_id = LineageId::root(world.seed, si as u8);
                snapshots.push(EntitySnapshot::from_slot(
                    slot, lineage_id, wi as u16, si as u8,
                ));
            }
        }
        Self { generation, snapshots }
    }

    /// Iterador sobre entidades vivas.
    /// Iterator over alive entities.
    #[inline]
    pub fn alive(&self) -> impl Iterator<Item = &EntitySnapshot> {
        self.snapshots.iter().filter(|s| s.alive)
    }

    /// Conteo de entidades vivas.
    /// Count of alive entities.
    #[inline]
    pub fn alive_count(&self) -> usize {
        self.snapshots.iter().filter(|s| s.alive).count()
    }

    /// Distribución de un campo (HOF: recibe accessor closure).
    /// Distribution of a field (HOF: receives accessor closure).
    #[inline]
    pub fn distribution<F: Fn(&EntitySnapshot) -> f32>(&self, f: F) -> Vec<f32> {
        self.alive().map(f).collect()
    }

    /// Media de un campo sobre entidades vivas.
    /// Mean of a field over alive entities.
    pub fn mean<F: Fn(&EntitySnapshot) -> f32>(&self, f: F) -> f32 {
        let (sum, count) = self.alive().fold((0.0_f32, 0_u32), |(s, c), snap| {
            (s + f(snap), c + 1)
        });
        if count == 0 { 0.0 } else { sum / count as f32 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::SimWorldFlat;

    fn world_with_alive(n: u8, seed: u64) -> SimWorldFlat {
        let mut w = SimWorldFlat::new(seed, 0.05);
        for i in 0..n {
            let mut slot = EntitySlot::default();
            slot.qe = 100.0 + i as f32;
            slot.radius = 1.0 + i as f32 * 0.1;
            slot.frequency_hz = 100.0;
            slot.growth_bias = 0.5;
            slot.mobility_bias = 0.3;
            slot.branching_bias = 0.2;
            slot.resilience = 0.4;
            slot.archetype = 2; // fauna
            w.spawn(slot);
        }
        w
    }

    #[test]
    fn snapshot_from_slot_preserves_fields() {
        let mut slot = EntitySlot::default();
        slot.qe = 42.0;
        slot.radius = 2.5;
        slot.frequency_hz = 150.0;
        slot.alive = true;
        slot.archetype = 1;
        let snap = EntitySnapshot::from_slot(&slot, LineageId::root(0, 0), 0, 0);
        assert_eq!(snap.qe, 42.0);
        assert_eq!(snap.radius, 2.5);
        assert_eq!(snap.frequency_hz, 150.0);
        assert!(snap.alive);
        assert_eq!(snap.archetype, 1);
    }

    #[test]
    fn snapshot_dead_slot_alive_false() {
        let slot = EntitySlot::default(); // alive = false
        let snap = EntitySnapshot::from_slot(&slot, LineageId::root(0, 0), 0, 0);
        assert!(!snap.alive);
    }

    #[test]
    fn census_capture_counts_alive_only() {
        let w = world_with_alive(3, 42);
        let census = PopulationCensus::capture(0, &[w]);
        assert_eq!(census.alive_count(), 3);
        assert_eq!(census.snapshots.len(), 3); // capture only scans alive_mask
    }

    #[test]
    fn census_capture_multiple_worlds() {
        let w1 = world_with_alive(2, 42);
        let w2 = world_with_alive(4, 99);
        let census = PopulationCensus::capture(5, &[w1, w2]);
        assert_eq!(census.alive_count(), 6);
        assert_eq!(census.generation, 5);
    }

    #[test]
    fn census_distribution_returns_values() {
        let w = world_with_alive(3, 42);
        let census = PopulationCensus::capture(0, &[w]);
        let qe_dist = census.distribution(|s| s.qe);
        assert_eq!(qe_dist.len(), 3);
        assert!(qe_dist.iter().all(|&v| v >= 100.0));
    }

    #[test]
    fn census_mean_correct_average() {
        let w = world_with_alive(3, 42);
        let census = PopulationCensus::capture(0, &[w]);
        // qe = 100, 101, 102 → mean = 101
        let m = census.mean(|s| s.qe);
        assert!((m - 101.0).abs() < 0.01);
    }

    #[test]
    fn census_mean_empty_returns_zero() {
        let census = PopulationCensus { generation: 0, snapshots: vec![] };
        assert_eq!(census.mean(|s| s.qe), 0.0);
    }

    #[test]
    fn census_alive_iterator_filters_dead() {
        let mut w = world_with_alive(3, 42);
        w.kill(0); // kill first entity
        let census = PopulationCensus::capture(0, &[w]);
        // capture only scanned alive_mask, so dead entity is not in snapshots
        assert_eq!(census.alive_count(), 2);
    }
}
