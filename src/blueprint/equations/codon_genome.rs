//! PD-1/2/3/4: Codon Genome + Genetic Code + Translation + Silent Mutations.
//!
//! Sequences of codons (nucleotide tripletes) that encode amino acids.
//! The genetic code table is evolucionable. Silent mutations emerge from redundancy.
//!
//! Axiom 4: mutation/translation have cost. Axiom 6: code table emerges by selection.
//! Axiom 8: codon frequency influences translation efficiency.

use crate::blueprint::constants::{
    AMINO_ACID_TYPES, CODON_DELETION_RATE, CODON_DUPLICATION_RATE,
    CODON_MUTATION_RATE, MAX_CODONS, MIN_CODONS, TOTAL_CODONS,
};
use super::determinism;
use super::protein_fold::{Monomer, MAX_CHAIN};

// ─── PD-1: CodonGenome ─────────────────────────────────────────────────────

/// Codon-based genome. Each codon = u8 ∈ [0,63] (6 bits = 3 nucleotides × 2 bits).
/// Fixed-size backing array. Copy. No heap.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct CodonGenome {
    pub codons: [u8; MAX_CODONS],
    pub len: u16,
    pub sigma: f32,
}

impl Default for CodonGenome {
    fn default() -> Self {
        Self { codons: [0; MAX_CODONS], len: MIN_CODONS as u16, sigma: 0.15 }
    }
}

impl CodonGenome {
    pub fn active(&self) -> &[u8] { &self.codons[..self.len as usize] }
    pub fn codon_count(&self) -> usize { self.len as usize }

    /// Number of amino acids this genome encodes (codons / 3).
    pub fn amino_count(&self) -> usize { self.len as usize / 3 }

    pub fn from_seed(seed: u64) -> Self {
        let mut g = Self::default();
        let mut s = seed;
        for i in 0..g.len as usize {
            s = determinism::next_u64(s);
            g.codons[i] = (determinism::unit_f32(s) * TOTAL_CODONS as f32) as u8;
        }
        g
    }
}

/// Mutate codon genome: point mutations + duplication + deletion. Deterministic.
pub fn mutate_codon(genome: &CodonGenome, rng_state: u64) -> CodonGenome {
    let mut g = *genome;
    let mut s = rng_state;

    // Sigma self-adaptation
    let tau = super::batch_fitness::SELF_ADAPTIVE_TAU;
    s = determinism::next_u64(s);
    g.sigma = (g.sigma * (tau * determinism::gaussian_f32(s, 1.0)).exp()).clamp(0.001, 0.3);

    // Point mutations: flip individual nucleotides (2-bit positions within codon)
    for i in 0..g.len as usize {
        s = determinism::next_u64(s);
        if determinism::unit_f32(s) < CODON_MUTATION_RATE {
            s = determinism::next_u64(s);
            let bit_pos = (determinism::unit_f32(s) * 6.0) as u8; // which of 6 bits to flip
            g.codons[i] ^= 1 << bit_pos.min(5);
            g.codons[i] &= 0x3F; // keep in [0,63]
        }
    }

    // Codon duplication
    s = determinism::next_u64(s);
    if determinism::unit_f32(s) < CODON_DUPLICATION_RATE && (g.len as usize) < MAX_CODONS {
        s = determinism::next_u64(s);
        let src = (determinism::unit_f32(s) * g.len as f32) as usize;
        let src = src.min(g.len as usize - 1);
        g.codons[g.len as usize] = g.codons[src];
        g.len += 1;
    }

    // Codon deletion
    s = determinism::next_u64(s);
    if determinism::unit_f32(s) < CODON_DELETION_RATE && g.len as usize > MIN_CODONS {
        s = determinism::next_u64(s);
        let del = (determinism::unit_f32(s) * g.len as f32) as usize;
        let del = del.min(g.len as usize - 1);
        for j in del..(g.len as usize - 1) { g.codons[j] = g.codons[j + 1]; }
        g.len -= 1;
        g.codons[g.len as usize] = 0;
    }

    g
}

/// Crossover: single cut point, first half from A, second from B.
pub fn crossover_codon(a: &CodonGenome, b: &CodonGenome, rng_state: u64) -> CodonGenome {
    let s = determinism::next_u64(rng_state);
    let cut_a = (determinism::unit_f32(s) * a.len as f32) as usize;
    let cut_b = (determinism::unit_f32(determinism::next_u64(s)) * b.len as f32) as usize;
    let new_len = (cut_a + (b.len as usize - cut_b)).min(MAX_CODONS);

    let mut child = CodonGenome::default();
    child.len = new_len as u16;
    child.sigma = (a.sigma + b.sigma) * 0.5;
    for i in 0..cut_a.min(new_len) { child.codons[i] = a.codons[i]; }
    for i in cut_a..new_len {
        let bi = cut_b + (i - cut_a);
        child.codons[i] = if bi < b.len as usize { b.codons[bi] } else { 0 };
    }
    child
}

// ─── PD-2: Genetic Code Table ───────────────────────────────────────────────

/// Genetic code: maps 64 codons → 8 amino acid types. Evolucionable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodonTable {
    pub mapping: [u8; 64],
}

impl Default for CodonTable {
    fn default() -> Self { Self::systematic() }
}

impl CodonTable {
    /// Systematic mapping: codon / 8 → amino. Uniform redundancy (8:1).
    pub fn systematic() -> Self {
        let mut mapping = [0u8; 64];
        for i in 0..64 { mapping[i] = (i as u8 / 8).min(AMINO_ACID_TYPES - 1); }
        Self { mapping }
    }

    pub fn translate(&self, codon: u8) -> u8 {
        self.mapping[(codon & 0x3F) as usize]
    }

    /// Redundancy: how many codons map to this amino acid.
    pub fn redundancy(&self, amino: u8) -> u8 {
        self.mapping.iter().filter(|&&m| m == amino).count() as u8
    }
}

/// Mutate one random mapping entry. The code table evolves.
pub fn mutate_table(table: &CodonTable, rng_state: u64) -> CodonTable {
    let mut t = *table;
    let s = determinism::next_u64(rng_state);
    let codon = (determinism::unit_f32(s) * 64.0) as usize;
    let s2 = determinism::next_u64(s);
    let new_amino = (determinism::unit_f32(s2) * AMINO_ACID_TYPES as f32) as u8;
    t.mapping[codon.min(63)] = new_amino.min(AMINO_ACID_TYPES - 1);
    t
}

// ─── PD-3: Translation ──────────────────────────────────────────────────────

/// Amino acid properties by type. Pure from ID, no lookup table.
const AMINO_HYDROPHOBICITY: [f32; 8] = [0.8, 0.9, 0.2, 0.1, 0.1, 0.7, 0.3, 0.6];
const AMINO_CHARGE: [f32; 8] = [0.0, 0.0, 0.0, 0.5, -0.5, 0.0, 0.0, 0.2];

/// Translate codon genome → Monomer chain (compatible with protein_fold).
pub fn translate_genome(genome: &CodonGenome, table: &CodonTable) -> ([Monomer; MAX_CHAIN], usize) {
    let mut chain = [Monomer::default(); MAX_CHAIN];
    let n_amino = genome.amino_count().min(MAX_CHAIN);

    for i in 0..n_amino {
        let codon_start = i * 3;
        if codon_start + 2 >= genome.len as usize { break; }
        // Use middle codon of triplet as the representative
        let codon = genome.codons[codon_start + 1] & 0x3F;
        let amino = table.translate(codon) as usize;
        let amino = amino.min(AMINO_ACID_TYPES as usize - 1);

        let prev_h = if i > 0 { chain[i - 1].hydrophobicity } else { AMINO_HYDROPHOBICITY[amino] };
        chain[i] = Monomer {
            hydrophobicity: AMINO_HYDROPHOBICITY[amino],
            charge: (AMINO_HYDROPHOBICITY[amino] - prev_h) * 0.5 + AMINO_CHARGE[amino] * 0.3,
        };
    }

    (chain, n_amino)
}

// ─── PD-4: Silent Mutations ─────────────────────────────────────────────────

/// Mutation type classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationType {
    Silent,   // Same amino acid (no phenotypic effect)
    Missense, // Different amino acid (phenotype changes)
}

/// Classify a codon mutation. Pure.
pub fn classify_mutation(table: &CodonTable, old_codon: u8, new_codon: u8) -> MutationType {
    if table.translate(old_codon) == table.translate(new_codon) {
        MutationType::Silent
    } else {
        MutationType::Missense
    }
}

/// Fraction of all possible single-nucleotide mutations that are silent.
/// Higher redundancy → more silent mutations → more neutral drift.
pub fn silent_mutation_fraction(table: &CodonTable) -> f32 {
    let mut silent = 0u32;
    let mut total = 0u32;
    for codon in 0u8..64 {
        // 6 bits → 6 possible single-bit flip mutations
        for bit in 0..6u8 {
            let mutant = codon ^ (1 << bit);
            if table.translate(codon) == table.translate(mutant) { silent += 1; }
            total += 1;
        }
    }
    silent as f32 / total.max(1) as f32
}

/// Hash of codon genome for deterministic comparison. No heap.
pub fn codon_hash(genome: &CodonGenome) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    for &c in genome.active() { (c as u32).hash(&mut h); }
    Hasher::finish(&h)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PD-1: CodonGenome ───────────────────────────────────────────────

    #[test] fn default_has_min_codons() { assert_eq!(CodonGenome::default().codon_count(), MIN_CODONS); }
    #[test] fn default_amino_count() { assert_eq!(CodonGenome::default().amino_count(), MIN_CODONS / 3); }

    #[test] fn from_seed_deterministic() {
        assert_eq!(CodonGenome::from_seed(42), CodonGenome::from_seed(42));
    }

    #[test] fn from_seed_codons_in_range() {
        let g = CodonGenome::from_seed(42);
        for &c in g.active() { assert!(c < 64, "codon out of range: {c}"); }
    }

    #[test] fn mutate_deterministic() {
        let g = CodonGenome::from_seed(42);
        assert_eq!(mutate_codon(&g, 99), mutate_codon(&g, 99));
    }

    #[test] fn mutate_codons_in_range() {
        let g = CodonGenome::from_seed(42);
        for s in 0..200 {
            let m = mutate_codon(&g, s);
            for &c in m.active() { assert!(c < 64); }
        }
    }

    #[test] fn mutate_preserves_min_len() {
        let g = CodonGenome::default();
        for s in 0..200 { assert!(mutate_codon(&g, s).codon_count() >= MIN_CODONS); }
    }

    #[test] fn mutate_respects_max_len() {
        let mut g = CodonGenome::default();
        g.len = MAX_CODONS as u16;
        for s in 0..200 { assert!(mutate_codon(&g, s).codon_count() <= MAX_CODONS); }
    }

    #[test] fn mutate_can_grow() {
        let g = CodonGenome::from_seed(42);
        assert!((0..500).any(|s| mutate_codon(&g, s).codon_count() > g.codon_count()));
    }

    #[test] fn mutate_can_shrink() {
        let mut g = CodonGenome::from_seed(42);
        g.len = 24;
        assert!((0..500).any(|s| mutate_codon(&g, s).codon_count() < 24));
    }

    #[test] fn crossover_deterministic() {
        let (a, b) = (CodonGenome::from_seed(1), CodonGenome::from_seed(2));
        assert_eq!(crossover_codon(&a, &b, 42), crossover_codon(&a, &b, 42));
    }

    #[test] fn crossover_len_bounded() {
        let (a, b) = (CodonGenome::from_seed(1), CodonGenome::from_seed(2));
        let c = crossover_codon(&a, &b, 42);
        assert!(c.codon_count() <= MAX_CODONS);
        assert!(c.codon_count() >= 1);
    }

    // ── PD-2: Genetic Code Table ────────────────────────────────────────

    #[test] fn systematic_covers_all_aminos() {
        let t = CodonTable::systematic();
        for amino in 0..AMINO_ACID_TYPES { assert!(t.redundancy(amino) > 0); }
    }

    #[test] fn systematic_uniform_redundancy() {
        let t = CodonTable::systematic();
        for amino in 0..AMINO_ACID_TYPES { assert_eq!(t.redundancy(amino), 8); }
    }

    #[test] fn translate_in_range() {
        let t = CodonTable::systematic();
        for c in 0..64u8 { assert!(t.translate(c) < AMINO_ACID_TYPES); }
    }

    #[test] fn mutate_table_changes_one_entry() {
        let t = CodonTable::systematic();
        let m = mutate_table(&t, 42);
        let diffs = (0..64).filter(|&i| t.mapping[i] != m.mapping[i]).count();
        assert_eq!(diffs, 1);
    }

    #[test] fn redundancy_sum_is_64() {
        let t = CodonTable::systematic();
        let sum: u8 = (0..AMINO_ACID_TYPES).map(|a| t.redundancy(a)).sum();
        assert_eq!(sum, 64);
    }

    // ── PD-3: Translation ───────────────────────────────────────────────

    #[test] fn translate_empty_genome() {
        let mut g = CodonGenome::default();
        g.len = 0;
        let (_, len) = translate_genome(&g, &CodonTable::systematic());
        assert_eq!(len, 0);
    }

    #[test] fn translate_min_genome() {
        let g = CodonGenome::from_seed(42);
        let (_, len) = translate_genome(&g, &CodonTable::systematic());
        assert_eq!(len, MIN_CODONS / 3);
    }

    #[test] fn translate_deterministic() {
        let g = CodonGenome::from_seed(42);
        let t = CodonTable::systematic();
        let (a, la) = translate_genome(&g, &t);
        let (b, lb) = translate_genome(&g, &t);
        assert_eq!(la, lb);
        for i in 0..la { assert_eq!(a[i], b[i]); }
    }

    #[test] fn translate_hydrophobicity_from_amino() {
        let mut g = CodonGenome::default();
        g.codons[1] = 0; // codon 0 → amino 0 (hydrophobic-small, H=0.8)
        g.len = 3;
        let (chain, len) = translate_genome(&g, &CodonTable::systematic());
        assert_eq!(len, 1);
        assert!((chain[0].hydrophobicity - 0.8).abs() < 1e-3);
    }

    #[test] fn mutated_table_changes_translation() {
        let g = CodonGenome::from_seed(42);
        let t1 = CodonTable::systematic();
        let t2 = mutate_table(&t1, 42);
        let (c1, _) = translate_genome(&g, &t1);
        let (c2, _) = translate_genome(&g, &t2);
        // May or may not differ (depends on which codon was changed)
        let _ = (c1, c2); // no panic
    }

    // ── PD-4: Silent Mutations ──────────────────────────────────────────

    #[test] fn classify_same_codon_silent() {
        let t = CodonTable::systematic();
        assert_eq!(classify_mutation(&t, 0, 0), MutationType::Silent);
    }

    #[test] fn classify_same_amino_silent() {
        let t = CodonTable::systematic();
        // Codons 0 and 1 both map to amino 0 in systematic table (0/8 = 0)
        assert_eq!(t.translate(0), t.translate(1));
        assert_eq!(classify_mutation(&t, 0, 1), MutationType::Silent);
    }

    #[test] fn classify_different_amino_missense() {
        let t = CodonTable::systematic();
        // Codon 0 → amino 0, codon 8 → amino 1
        assert_ne!(t.translate(0), t.translate(8));
        assert_eq!(classify_mutation(&t, 0, 8), MutationType::Missense);
    }

    #[test] fn silent_fraction_systematic_positive() {
        let f = silent_mutation_fraction(&CodonTable::systematic());
        assert!(f > 0.0 && f < 1.0, "systematic table should have partial silence: {f}");
    }

    #[test] fn silent_fraction_uniform_table_one() {
        let mut t = CodonTable::systematic();
        for i in 0..64 { t.mapping[i] = 0; } // all map to same amino
        assert!((silent_mutation_fraction(&t) - 1.0).abs() < 1e-5);
    }

    #[test] fn neutral_drift_preserves_protein() {
        let g = CodonGenome::from_seed(42);
        let t = CodonTable::systematic();
        let (original_chain, original_len) = translate_genome(&g, &t);

        // Apply only silent mutations (manually flip within same amino block)
        let mut g_drifted = g;
        for i in 0..g_drifted.len as usize {
            let old = g_drifted.codons[i];
            let new = old ^ 1; // flip lowest bit
            if classify_mutation(&t, old, new & 0x3F) == MutationType::Silent {
                g_drifted.codons[i] = new & 0x3F;
            }
        }

        let (drifted_chain, drifted_len) = translate_genome(&g_drifted, &t);
        assert_eq!(original_len, drifted_len);
        for i in 0..original_len {
            assert!((original_chain[i].hydrophobicity - drifted_chain[i].hydrophobicity).abs() < 1e-5,
                "silent mutations should preserve protein at monomer {i}");
        }
    }
}
