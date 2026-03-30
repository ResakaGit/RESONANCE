//! Core data types: `EntitySlot` (flat entity) and `SimWorldFlat` (complete world).
//!
//! Zero heap allocation. Zero Bevy dependency. `repr(C)` for predictable layout.

use crate::blueprint::equations;
use crate::blueprint::equations::radial_field::{AXIAL, RADIAL};
use crate::blueprint::equations::codon_genome::{CodonGenome, CodonTable};
use crate::blueprint::equations::variable_genome::VariableGenome;

use super::constants::{GRID_CELLS, MAX_ENTITIES, QE_MIN_EXISTENCE};

/// Force computation strategy. Configurable per world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ForceStrategy {
    /// No particle forces (legacy behavior). Zero overhead.
    Disabled,
    /// Coulomb only (long-range electromagnetic).
    CoulombOnly,
    /// Coulomb + Lennard-Jones (full particle physics). Default.
    Full,
}

impl Default for ForceStrategy {
    fn default() -> Self { Self::Disabled } // disabled by default for backward compatibility
}
use super::events::EventBuffer;

// ─── EntitySlot ─────────────────────────────────────────────────────────────

/// Flat entity — all 14 layers packed into one `Copy` struct.
///
/// Layers that don't apply carry neutral values (0.0 / false).
/// L8 (Injector), L10 (Link), L11 (Tension), L13 (StructuralLink) are omitted:
/// they encode inter-entity relations, resolved via pair buffers in `ScratchPad`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EntitySlot {
    // ── identity ───────────────────────────
    pub entity_id:      u32,

    // ── L0  BaseEnergy ─────────────────────
    pub qe:             f32,

    // ── L1  SpatialVolume ──────────────────
    pub radius:         f32,

    // ── L2  OscillatorySignature ───────────
    pub frequency_hz:   f32,
    pub phase:          f32,

    // ── L3  FlowVector ─────────────────────
    pub velocity:       [f32; 2],
    pub dissipation:    f32,

    // ── L4  MatterCoherence ────────────────
    pub bond_energy:    f32,
    pub conductivity:   f32,

    // ── L5  AlchemicalEngine ───────────────
    pub engine_buffer:  f32,
    pub engine_max:     f32,
    pub input_valve:    f32,
    pub output_valve:   f32,

    // ── L6  AmbientPressure ────────────────
    pub pressure_dqe:   f32,
    pub viscosity:      f32,

    // ── L7  WillActuator ───────────────────
    pub will_intent:    [f32; 2],

    // ── L12 Homeostasis ────────────────────
    pub adapt_rate_hz:  f32,
    pub stability_band: f32,

    // ── Position (sim-plane) ─���─────────────
    pub position:       [f32; 2],

    // ── Genome (InferenceProfile) ──────────
    pub growth_bias:    f32,
    pub mobility_bias:  f32,
    pub branching_bias: f32,
    pub resilience:     f32,

    // ── Trophic state ──────────────────────
    pub satiation:      f32,

    // ── Epigenetic mask ────────────────────
    pub expression_mask: [f32; 4],

    // ── Internal energy field (AXIAL × RADIAL nodes)
    pub qe_field:   [[f32; RADIAL]; AXIAL],
    pub freq_field:  [[f32; RADIAL]; AXIAL],

    // ── L-1 ParticleCharge ──────────────────
    pub charge:         f32,
    pub particle_mass:  f32,

    // ── Flags (packed) ─────────────────────
    pub alive:          bool,
    pub archetype:      u8,
    pub matter_state:   u8,
    pub channeling:     bool,
    pub faction:        u8,
    pub trophic_class:  u8,
    pub field_converged: bool,
    pub _pad:           [u8; 1],
}

impl Default for EntitySlot {
    fn default() -> Self {
        // Safety: all-zero is valid — alive=false, all f32=0.0, all u8=0.
        // SAFETY: This is safe because EntitySlot is repr(C) and all-zeros is a valid state.
        // We avoid unsafe here by using explicit construction.
        Self {
            entity_id: 0, qe: 0.0, radius: 0.0,
            frequency_hz: 0.0, phase: 0.0,
            velocity: [0.0; 2], dissipation: 0.0,
            bond_energy: 0.0, conductivity: 0.0,
            engine_buffer: 0.0, engine_max: 0.0,
            input_valve: 0.0, output_valve: 0.0,
            pressure_dqe: 0.0, viscosity: 0.0,
            will_intent: [0.0; 2],
            adapt_rate_hz: 0.0, stability_band: 0.0,
            position: [0.0; 2],
            growth_bias: 0.0, mobility_bias: 0.0,
            branching_bias: 0.0, resilience: 0.0,
            satiation: 0.0,
            expression_mask: [1.0; 4], // fully expressed by default (ET-6)
            qe_field: [[0.0; RADIAL]; AXIAL],
            freq_field: [[0.0; RADIAL]; AXIAL],
            charge: 0.0, particle_mass: 1.0,
            alive: false, archetype: 0, matter_state: 0,
            channeling: false, faction: 0, trophic_class: 0,
            field_converged: false, _pad: [0; 1],
        }
    }
}

// ─── SimWorldFlat ��──────────────────────────────────────────────────────────

/// Complete world state. Fixed-size. Zero heap allocation.
///
/// `alive_mask` is a `u128` bitmask: bit `i` set ↔ `entities[i].alive == true`.
/// All mutation of entity liveness MUST go through `spawn()` / `kill()` to keep
/// `alive_mask` and `entity_count` consistent.
#[derive(Clone)]
pub struct SimWorldFlat {
    pub tick_id:         u64,
    pub seed:            u64,
    pub dt:              f32,
    pub entity_count:    u8,
    pub alive_mask:      u128,
    /// Particle force strategy. Disabled = legacy (no charge physics).
    pub force_strategy:  ForceStrategy,
    pub next_id:         u32,
    pub entities:        [EntitySlot; MAX_ENTITIES],
    /// Side-table: variable-length genomes per entity (cold data, DoD separation).
    /// Synced with entities[] by index. Only accessed during reproduction + metabolic inference.
    pub genomes:         [VariableGenome; MAX_ENTITIES],
    /// Side-table: codon genomes (PD-5). Cold data, DoD.
    pub codon_genomes:   [CodonGenome; MAX_ENTITIES],
    /// Side-table: genetic code per lineage (PD-5). Evolves with organism.
    pub codon_tables:    [CodonTable; MAX_ENTITIES],
    pub total_qe:        f32,
    pub nutrient_grid:   [f32; GRID_CELLS],
    pub irradiance_grid: [f32; GRID_CELLS],
    pub events:          EventBuffer,
}

impl SimWorldFlat {
    /// Create empty world with given seed and timestep.
    pub fn new(seed: u64, dt: f32) -> Self {
        Self {
            tick_id: 0,
            seed,
            dt,
            entity_count: 0,
            alive_mask: 0,
            force_strategy: ForceStrategy::default(),
            next_id: 0,
            entities: [EntitySlot::default(); MAX_ENTITIES],
            genomes: [VariableGenome::default(); MAX_ENTITIES],
            codon_genomes: [CodonGenome::default(); MAX_ENTITIES],
            codon_tables: [CodonTable::default(); MAX_ENTITIES],
            total_qe: 0.0,
            nutrient_grid: [0.0; GRID_CELLS],
            irradiance_grid: [0.0; GRID_CELLS],
            events: EventBuffer::new(),
        }
    }

    /// Spawn entity in first free slot. Returns slot index, or `None` if full.
    pub fn spawn(&mut self, mut slot: EntitySlot) -> Option<usize> {
        let idx = self.first_free_slot()?;
        slot.alive = true;
        slot.entity_id = self.next_id;
        self.next_id += 1;
        self.entities[idx] = slot;
        self.alive_mask |= 1 << idx;
        self.entity_count += 1;
        Some(idx)
    }

    /// Kill entity at index. Clears slot, updates bitmask.
    pub fn kill(&mut self, idx: usize) {
        if idx >= MAX_ENTITIES { return; }
        if self.alive_mask & (1 << idx) == 0 { return; }
        self.entities[idx] = EntitySlot::default();
        self.alive_mask &= !(1 << idx);
        self.entity_count = self.entity_count.saturating_sub(1);
    }

    /// First free slot via trailing zeros on inverted mask.
    pub fn first_free_slot(&self) -> Option<usize> {
        let free = !self.alive_mask;
        if free == 0 { return None; }
        let idx = free.trailing_zeros() as usize;
        if idx >= MAX_ENTITIES { None } else { Some(idx) }
    }

    /// Reap entities below death threshold.
    pub fn reap_dead(&mut self) {
        let mut mask = self.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            if self.entities[i].qe < QE_MIN_EXISTENCE {
                self.kill(i);
            }
        }
    }

    /// Recompute `total_qe` from live entities.
    pub fn update_total_qe(&mut self) {
        let mut sum = 0.0_f32;
        let mut mask = self.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            sum += self.entities[i].qe;
        }
        self.total_qe = sum;
    }

    /// Assert conservation invariant (debug builds only).
    pub fn assert_conservation(&self) {
        let mut mask = self.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            debug_assert!(
                equations::conservation::is_valid_qe(self.entities[i].qe),
                "INV-B2: entity {} has invalid qe={}", i, self.entities[i].qe,
            );
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EntitySlot ──────────────────────────────────────────────────────────

    #[test]
    fn entity_slot_default_is_dead() {
        let slot = EntitySlot::default();
        assert!(!slot.alive);
        assert_eq!(slot.qe, 0.0);
        assert_eq!(slot.entity_id, 0);
    }

    #[test]
    fn entity_slot_is_copy() {
        let a = EntitySlot::default();
        let b = a;
        assert_eq!(a.entity_id, b.entity_id);
    }

    #[test]
    fn entity_slot_size_is_reasonable() {
        let size = std::mem::size_of::<EntitySlot>();
        assert!(size <= 1200, "EntitySlot too large: {size} bytes");
        assert!(size >= 100, "EntitySlot too small: {size} bytes — fields missing?");
    }

    // ── SimWorldFlat ────────────────────────────────────────────────────────

    #[test]
    fn new_world_is_empty() {
        let w = SimWorldFlat::new(42, 0.05);
        assert_eq!(w.tick_id, 0);
        assert_eq!(w.seed, 42);
        assert_eq!(w.entity_count, 0);
        assert_eq!(w.alive_mask, 0);
        assert_eq!(w.next_id, 0);
    }

    #[test]
    fn spawn_sets_alive_mask() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        let idx = w.spawn(e).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(w.entity_count, 1);
        assert!(w.alive_mask & 1 != 0);
        assert!(w.entities[0].alive);
        assert_eq!(w.entities[0].entity_id, 0);
        assert_eq!(w.next_id, 1);
    }

    #[test]
    fn spawn_multiple_uses_sequential_slots() {
        let mut w = SimWorldFlat::new(0, 0.05);
        for i in 0..5 {
            let mut e = EntitySlot::default();
            e.qe = 10.0 * (i as f32 + 1.0);
            w.spawn(e).unwrap();
        }
        assert_eq!(w.entity_count, 5);
        assert_eq!(w.alive_mask, 0b11111);
        assert_eq!(w.next_id, 5);
    }

    #[test]
    fn spawn_returns_none_when_full() {
        let mut w = SimWorldFlat::new(0, 0.05);
        for _ in 0..MAX_ENTITIES {
            let mut e = EntitySlot::default();
            e.qe = 10.0;
            w.spawn(e).unwrap();
        }
        assert_eq!(w.entity_count, MAX_ENTITIES as u8);
        let mut e = EntitySlot::default();
        e.qe = 10.0;
        assert!(w.spawn(e).is_none());
    }

    #[test]
    fn kill_clears_slot_and_mask() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        w.spawn(e);
        assert_eq!(w.entity_count, 1);
        w.kill(0);
        assert_eq!(w.entity_count, 0);
        assert_eq!(w.alive_mask, 0);
        assert!(!w.entities[0].alive);
    }

    #[test]
    fn kill_out_of_bounds_is_noop() {
        let mut w = SimWorldFlat::new(0, 0.05);
        w.kill(99); // no panic
        assert_eq!(w.entity_count, 0);
    }

    #[test]
    fn spawn_reuses_killed_slot() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        w.spawn(e);
        w.spawn(e);
        w.kill(0);
        assert_eq!(w.entity_count, 1);
        let idx = w.spawn(e).unwrap();
        assert_eq!(idx, 0, "should reuse slot 0");
        assert_eq!(w.entity_count, 2);
    }

    #[test]
    fn first_free_slot_empty_world() {
        let w = SimWorldFlat::new(0, 0.05);
        assert_eq!(w.first_free_slot(), Some(0));
    }

    #[test]
    fn first_free_slot_full_world() {
        let mut w = SimWorldFlat::new(0, 0.05);
        for _ in 0..MAX_ENTITIES {
            let mut e = EntitySlot::default();
            e.qe = 10.0;
            w.spawn(e);
        }
        assert_eq!(w.first_free_slot(), None);
    }

    #[test]
    fn reap_dead_removes_starved_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        w.spawn(e);
        let mut e2 = EntitySlot::default();
        e2.qe = 0.001; // below QE_MIN_EXISTENCE
        w.spawn(e2);
        assert_eq!(w.entity_count, 2);
        w.reap_dead();
        assert_eq!(w.entity_count, 1);
        assert!(w.alive_mask & (1 << 0) != 0, "healthy entity survives");
        assert!(w.alive_mask & (1 << 1) == 0, "starved entity reaped");
    }

    #[test]
    fn update_total_qe_sums_alive() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 50.0;
        let mut e2 = EntitySlot::default();
        e2.qe = 30.0;
        w.spawn(e1);
        w.spawn(e2);
        w.update_total_qe();
        assert!((w.total_qe - 80.0).abs() < 1e-5);
    }

    #[test]
    fn update_total_qe_ignores_dead() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 50.0;
        w.spawn(e1);
        let mut e2 = EntitySlot::default();
        e2.qe = 30.0;
        w.spawn(e2);
        w.kill(1);
        w.update_total_qe();
        assert!((w.total_qe - 50.0).abs() < 1e-5);
    }

    #[test]
    fn assert_conservation_passes_for_valid_world() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        w.spawn(e);
        w.assert_conservation(); // should not panic
    }
}
