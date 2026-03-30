//! Proto-DNA constants — derived from the 4 fundamental constants.

use super::super::equations::derived_thresholds::DISSIPATION_SOLID;

/// Maximum codons per genome (3 nucleotides each). Derived: MAX_GENES × 3 = 96.
pub const MAX_CODONS: usize = 96;
/// Minimum codons (4 amino acids × 3 nucleotides = 12).
pub const MIN_CODONS: usize = 12;
/// Number of distinct amino acid types (2^3 = 8, simplified from biological 20).
pub const AMINO_ACID_TYPES: u8 = 8;
/// Total possible codons (2^6 = 64, 3 nucleotides × 2 bits each).
pub const TOTAL_CODONS: u8 = 64;
/// Mutation rate per nucleotide per reproduction. Derived: DISSIPATION_SOLID × 30 = 0.15.
pub const CODON_MUTATION_RATE: f32 = DISSIPATION_SOLID * 30.0;
/// Translation cost per codon (qe). Derived: DISSIPATION_SOLID × 2 = 0.01.
pub const TRANSLATION_COST_PER_CODON: f32 = DISSIPATION_SOLID * 2.0;
/// Codon duplication rate. Derived: DISSIPATION_SOLID × 8 = 0.04 (4%).
pub const CODON_DUPLICATION_RATE: f32 = DISSIPATION_SOLID * 8.0;
/// Codon deletion rate. Derived: DISSIPATION_SOLID × 5 = 0.025 (2.5%).
pub const CODON_DELETION_RATE: f32 = DISSIPATION_SOLID * 5.0;
