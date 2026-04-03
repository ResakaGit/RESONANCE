//! Lineage tracking — parentesco determinista para árboles filogenéticos.
//! Lineage tracking — deterministic ancestry for phylogenetic trees.
//!
//! `LineageId` identifica de forma única cada genoma en la historia evolutiva.
//! `TrackedGenome` envuelve `GenomeBlob` con metadata sin modificar el original.
//! Zero Bevy. Copy. Stack-allocated.

use crate::batch::genome::GenomeBlob;

// ─── Hashing (determinista, sin pérdida de bits) ────────────────────────────

/// FNV-1a 64-bit sobre u64 parts. Determinista, sin truncamiento float.
/// FNV-1a 64-bit over u64 parts. Deterministic, no float truncation.
#[inline]
fn hash_u64_parts(parts: &[u64]) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let mut h = FNV_OFFSET;
    for &p in parts {
        h ^= p;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

// ─── LineageId ──────────────────────────────────────────────────────────────

/// Identificador único de linaje. Determinista: f(parent, child_index, generation).
/// Unique lineage identifier. Deterministic: f(parent, child_index, generation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineageId(pub u64);

impl LineageId {
    /// Id para entidad sin padre (abiogénesis / semilla inicial).
    /// Id for entity with no parent (abiogenesis / initial seed).
    #[inline]
    pub fn root(seed: u64, slot_index: u8) -> Self {
        Self(hash_u64_parts(&[seed, slot_index as u64]))
    }

    /// Id para offspring derivado de un padre.
    /// Id for offspring derived from a parent.
    #[inline]
    pub fn child(parent: LineageId, child_index: u8, generation: u32) -> Self {
        Self(hash_u64_parts(&[
            parent.0,
            child_index as u64,
            generation as u64,
        ]))
    }
}

// ─── TrackedGenome ──────────────────────────────────────────────────────────

/// Genoma + metadata de linaje. `GenomeBlob` intacto (22 bytes). Metadata envolvente.
/// Genome + lineage metadata. `GenomeBlob` unchanged (22 bytes). Wrapping metadata.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackedGenome {
    pub genome: GenomeBlob,
    pub lineage_id: LineageId,
    pub parent_id: Option<LineageId>,
    pub birth_gen: u32,
}

impl TrackedGenome {
    /// Crea un tracked genome raíz (sin padre).
    /// Creates a root tracked genome (no parent).
    #[inline]
    pub fn root(genome: GenomeBlob, seed: u64, slot: u8) -> Self {
        Self {
            genome,
            lineage_id: LineageId::root(seed, slot),
            parent_id: None,
            birth_gen: 0,
        }
    }

    /// Crea un tracked genome hijo.
    /// Creates a child tracked genome.
    #[inline]
    pub fn child(genome: GenomeBlob, parent: LineageId, child_index: u8, generation: u32) -> Self {
        Self {
            genome,
            lineage_id: LineageId::child(parent, child_index, generation),
            parent_id: Some(parent),
            birth_gen: generation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── LineageId ──

    #[test]
    fn lineage_root_deterministic_same_inputs() {
        assert_eq!(LineageId::root(42, 0), LineageId::root(42, 0));
    }

    #[test]
    fn lineage_root_different_seeds_differ() {
        assert_ne!(LineageId::root(42, 0), LineageId::root(99, 0));
    }

    #[test]
    fn lineage_root_different_slots_differ() {
        assert_ne!(LineageId::root(42, 0), LineageId::root(42, 1));
    }

    #[test]
    fn lineage_child_deterministic_same_inputs() {
        let parent = LineageId::root(42, 0);
        assert_eq!(
            LineageId::child(parent, 0, 10),
            LineageId::child(parent, 0, 10),
        );
    }

    #[test]
    fn lineage_child_different_parents_differ() {
        let a = LineageId::root(42, 0);
        let b = LineageId::root(99, 0);
        assert_ne!(LineageId::child(a, 0, 10), LineageId::child(b, 0, 10));
    }

    #[test]
    fn lineage_child_different_generations_differ() {
        let parent = LineageId::root(42, 0);
        assert_ne!(
            LineageId::child(parent, 0, 10),
            LineageId::child(parent, 0, 20),
        );
    }

    #[test]
    fn lineage_child_different_index_differ() {
        let parent = LineageId::root(42, 0);
        assert_ne!(
            LineageId::child(parent, 0, 10),
            LineageId::child(parent, 1, 10),
        );
    }

    // ── TrackedGenome ──

    #[test]
    fn tracked_root_has_no_parent() {
        let g = GenomeBlob::default();
        let t = TrackedGenome::root(g, 42, 0);
        assert!(t.parent_id.is_none());
        assert_eq!(t.birth_gen, 0);
    }

    #[test]
    fn tracked_child_has_parent() {
        let g = GenomeBlob::default();
        let parent = LineageId::root(42, 0);
        let t = TrackedGenome::child(g, parent, 0, 5);
        assert_eq!(t.parent_id, Some(parent));
        assert_eq!(t.birth_gen, 5);
    }

    #[test]
    fn tracked_preserves_genome_bitwise() {
        let g = GenomeBlob::random(12345);
        let t = TrackedGenome::root(g, 42, 0);
        assert_eq!(t.genome, g);
    }
}
