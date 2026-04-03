//! Variable-length genome equations — VG-1 through VG-6.
//!
//! Pure stateless math for variable-length genomes with duplication/deletion,
//! expression mapping, epigenetic gating, and serialization.
//!
//! Axiom 4: genome maintenance cost ∝ length^KLEIBER — dissipation scales with complexity.
//! Axiom 6: genome length itself emerges, not hardcoded.

use super::derived_thresholds::{DISSIPATION_SOLID, KLEIBER_EXPONENT};
use super::determinism;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Maximum genes a genome can hold (fixed-size backing array, no heap).
pub const MAX_GENES: usize = 32;
/// Minimum genes (the 4 core biases are always present).
pub const MIN_GENES: usize = 4;
/// Base gene count for cost normalization (cost=1× at 4 genes).
pub const BASE_GENE_COUNT: usize = 4;

/// Probability of gene duplication per mutation event.
/// Derived: DISSIPATION_SOLID × 10 = 0.05 (5%).
pub const DUPLICATION_RATE: f32 = DISSIPATION_SOLID * 10.0;
/// Probability of gene deletion per mutation event.
/// Derived: DISSIPATION_SOLID × 6 = 0.03 (3%). Deletion < duplication → net genome growth.
pub const DELETION_RATE: f32 = DISSIPATION_SOLID * 6.0;

/// Minimum effective bias for a capability to be unlocked (VG-4).
/// Derived: 1.0 - KLEIBER_EXPONENT = 0.25. Organism must invest >25% of a
/// bias dimension for the capability to activate. Axiom 4: function has a cost.
pub const CAPABILITY_BIAS_THRESHOLD: f32 = 1.0 - KLEIBER_EXPONENT; // 0.25

/// Gene count thresholds for capability unlock (VG-4).
/// Derived: MIN_GENES × tier multiplier. Tier 1=1.5×, Tier 2=2×, Tier 3=2.5×, Tier 4=3×.
const SENSE_GENE_THRESHOLD: usize = MIN_GENES + MIN_GENES / 2; // 6
const REPRODUCE_GENE_THRESHOLD: usize = MIN_GENES * 2; // 8
const ARMOR_GENE_THRESHOLD: usize = MIN_GENES * 2 + MIN_GENES / 2; // 10
const PHOTOSYNTH_GENE_THRESHOLD: usize = MIN_GENES * 3; // 12

/// Duplicate mutation damping: copy is mutated at sigma × this factor.
/// Derived: 1.0 - DISSIPATION_SOLID / DISSIPATION_SOLID = ... No. This is a biological
/// constant (gene duplication fidelity). Justified: duplicates diverge slowly.
/// Equivalent to HEBBIAN_BASELINE (0.5) — half the parent's mutation rate.
const DUPLICATE_SIGMA_FACTOR: f32 = 0.5;

// ─── CapabilitySet bit constants ────────────────────────────────────────────
// Mirror of layers/inference.rs::CapabilitySet. Duplicated because equations/
// cannot import layers/ (Bevy dependency boundary). Values must match exactly.
// If CapabilitySet flags change, update these. Verified by test: `caps_match_layer_values`.

const CAP_GROW: u8 = 1 << 0;
const CAP_MOVE: u8 = 1 << 1;
const CAP_BRANCH: u8 = 1 << 2;
const CAP_ROOT: u8 = 1 << 3;
const CAP_SENSE: u8 = 1 << 4;
const CAP_ARMOR: u8 = 1 << 5;
const CAP_REPRODUCE: u8 = 1 << 6;
const CAP_PHOTOSYNTH: u8 = 1 << 7;

// ─── VG-1: VariableGenome struct ────────────────────────────────────────────

/// Variable-length genome. Fixed-size backing array (no heap, Copy-safe).
///
/// `genes[0..3]` = core biases (growth, mobility, branching, resilience).
/// `genes[4..len]` = additional modulator genes that emerged by duplication.
/// All genes ∈ [0, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct VariableGenome {
    pub genes: [f32; MAX_GENES],
    pub len: u8,
    pub sigma: f32,
}

impl Default for VariableGenome {
    fn default() -> Self {
        let mut genes = [0.0; MAX_GENES];
        genes[0] = 0.5;
        genes[1] = 0.5;
        genes[2] = 0.5;
        genes[3] = 0.5;
        Self {
            genes,
            len: 4,
            sigma: 0.15,
        }
    }
}

impl VariableGenome {
    /// Create from core 4 biases (backward compatible with GenomeBlob).
    pub fn from_biases(growth: f32, mobility: f32, branching: f32, resilience: f32) -> Self {
        let mut g = Self::default();
        g.genes[0] = growth.clamp(0.0, 1.0);
        g.genes[1] = mobility.clamp(0.0, 1.0);
        g.genes[2] = branching.clamp(0.0, 1.0);
        g.genes[3] = resilience.clamp(0.0, 1.0);
        g
    }

    /// Active gene slice (read-only).
    #[inline]
    pub fn active(&self) -> &[f32] {
        &self.genes[..self.len as usize]
    }

    /// Core 4 biases (always present).
    #[inline]
    pub fn core_biases(&self) -> [f32; 4] {
        [self.genes[0], self.genes[1], self.genes[2], self.genes[3]]
    }

    #[inline]
    pub fn growth(&self) -> f32 {
        self.genes[0]
    }
    #[inline]
    pub fn mobility(&self) -> f32 {
        self.genes[1]
    }
    #[inline]
    pub fn branching(&self) -> f32 {
        self.genes[2]
    }
    #[inline]
    pub fn resilience(&self) -> f32 {
        self.genes[3]
    }
    #[inline]
    pub fn gene_count(&self) -> usize {
        self.len as usize
    }

    /// Euclidean distance using the shorter genome's length.
    pub fn distance(&self, other: &Self) -> f32 {
        let n = (self.len as usize).min(other.len as usize);
        (0..n)
            .map(|i| {
                let d = self.genes[i] - other.genes[i];
                d * d
            })
            .sum::<f32>()
            .sqrt()
    }

    /// Deterministic hash of active genes.
    pub fn hash(&self) -> u64 {
        determinism::hash_f32_slice(self.active())
    }
}

// ─── VG-2: Genome maintenance cost ─────────────────────────────────────────

/// Metabolic cost: `base × (n/4)^KLEIBER`. Axiom 4.
pub fn genome_maintenance_cost(n_genes: usize, base_dissipation: f32) -> f32 {
    let ratio = n_genes as f32 / BASE_GENE_COUNT as f32;
    base_dissipation * ratio.powf(KLEIBER_EXPONENT)
}

/// Effective bias with diminishing modulation from extra genes.
///
/// `gene[4+]` modulates `core[i%4]` with weight `1/(1+distance)`.
pub fn effective_bias(genome: &VariableGenome, core_index: usize) -> f32 {
    let base = genome.genes[core_index.min(3)];
    if genome.len <= 4 {
        return base;
    }

    let (modulation, weight_sum) = (MIN_GENES..genome.len as usize)
        .filter(|&i| i % MIN_GENES == core_index)
        .fold((0.0f32, 0.0f32), |(mod_acc, w_acc), i| {
            let w = 1.0 / (1.0 + ((i - MIN_GENES) / MIN_GENES + 1) as f32);
            (mod_acc + (genome.genes[i] - 0.5) * w, w_acc + w)
        });

    if weight_sum > 0.0 {
        (base + modulation / (1.0 + weight_sum)).clamp(0.0, 1.0)
    } else {
        base
    }
}

/// All 4 effective biases (cache-friendly batch call).
pub fn effective_biases(genome: &VariableGenome) -> [f32; 4] {
    std::array::from_fn(|i| effective_bias(genome, i))
}

// ─── Cached effective biases ────────────────────────────────────────────────

/// Pre-computed effective biases + capabilities. Avoids redundant computation
/// when multiple systems need the same genome's phenotype in one tick.
#[derive(Clone, Copy, Debug)]
pub struct GenomePhenotype {
    pub biases: [f32; 4],
    pub capabilities: u8,
    pub maintenance_cost: f32,
    pub gene_count: u8,
}

/// Compute full phenotype from genome + expression mask. One call per entity per tick.
pub fn compute_phenotype(
    genome: &VariableGenome,
    expression_mask: &[f32; 4],
    base_dissipation: f32,
) -> GenomePhenotype {
    let biases = gated_effective_biases(genome, expression_mask);
    GenomePhenotype {
        biases,
        capabilities: capabilities_from_biases(&biases, genome.gene_count()),
        maintenance_cost: gated_maintenance_cost(genome, expression_mask, base_dissipation),
        gene_count: genome.len,
    }
}

// ─── VG-3: Mutation (decomposed into pure sub-functions) ────────────────────

/// Mutate a variable-length genome. Schwefel self-adaptive + structural mutations.
///
/// Deterministic: same genome + same seed → identical output.
pub fn mutate_variable(genome: &VariableGenome, rng_state: u64) -> VariableGenome {
    let mut g = *genome;
    let mut s = rng_state;

    s = mutate_sigma(&mut g, s);
    s = mutate_point_genes(&mut g, s);
    s = try_duplicate_gene(&mut g, s);
    let _ = try_delete_gene(&mut g, s);

    g
}

/// Sigma self-adaptation (Schwefel 1981). Returns next RNG state.
fn mutate_sigma(g: &mut VariableGenome, s: u64) -> u64 {
    let tau = super::batch_fitness::SELF_ADAPTIVE_TAU;
    let s = determinism::next_u64(s);
    g.sigma = (g.sigma * (tau * determinism::gaussian_f32(s, 1.0)).exp()).clamp(0.001, 0.3);
    s
}

/// Point mutations on all active genes. Returns next RNG state.
fn mutate_point_genes(g: &mut VariableGenome, mut s: u64) -> u64 {
    for i in 0..g.len as usize {
        s = determinism::next_u64(s);
        g.genes[i] = (g.genes[i] + determinism::gaussian_f32(s, g.sigma)).clamp(0.0, 1.0);
    }
    s
}

/// Gene duplication (DUPLICATION_RATE chance). Returns next RNG state.
fn try_duplicate_gene(g: &mut VariableGenome, mut s: u64) -> u64 {
    s = determinism::next_u64(s);
    if determinism::unit_f32(s) < DUPLICATION_RATE && (g.len as usize) < MAX_GENES {
        s = determinism::next_u64(s);
        let src = ((determinism::unit_f32(s) * g.len as f32) as usize).min(g.len as usize - 1);
        let dst = g.len as usize;
        g.genes[dst] = g.genes[src];
        g.len += 1;
        s = determinism::next_u64(s);
        g.genes[dst] = (g.genes[dst]
            + determinism::gaussian_f32(s, g.sigma * DUPLICATE_SIGMA_FACTOR))
        .clamp(0.0, 1.0);
    }
    s
}

/// Gene deletion (DELETION_RATE chance, never below MIN_GENES). Returns next RNG state.
fn try_delete_gene(g: &mut VariableGenome, mut s: u64) -> u64 {
    s = determinism::next_u64(s);
    if determinism::unit_f32(s) < DELETION_RATE && g.len as usize > MIN_GENES {
        s = determinism::next_u64(s);
        let del = (MIN_GENES
            + (determinism::unit_f32(s) * (g.len as usize - MIN_GENES) as f32) as usize)
            .min(g.len as usize - 1);
        for j in del..(g.len as usize - 1) {
            g.genes[j] = g.genes[j + 1];
        }
        g.len -= 1;
        g.genes[g.len as usize] = 0.0;
    }
    s
}

/// Crossover two variable-length genomes.
///
/// Length = max(parents). Sigma = mean. Genes: uniform 50/50 where both have them.
pub fn crossover_variable(
    a: &VariableGenome,
    b: &VariableGenome,
    rng_state: u64,
) -> VariableGenome {
    let max_len = (a.len as usize).max(b.len as usize);
    let mut child = VariableGenome {
        len: max_len as u8,
        sigma: (a.sigma + b.sigma) * 0.5,
        ..Default::default()
    };

    let mut s = rng_state;
    for i in 0..max_len {
        s = determinism::next_u64(s);
        child.genes[i] = match (i < a.len as usize, i < b.len as usize) {
            (true, true) => {
                if determinism::unit_f32(s) < 0.5 {
                    a.genes[i]
                } else {
                    b.genes[i]
                }
            }
            (true, false) => a.genes[i],
            (false, true) => b.genes[i],
            (false, false) => 0.5,
        };
    }
    child
}

// ─── VG-4: Expression mapping ───────────────────────────────────────────────

/// Map genome → 4 phenotype biases via effective_biases.
pub fn genome_to_profile(genome: &VariableGenome) -> [f32; 4] {
    effective_biases(genome)
}

/// Derive capability flags from genome length + effective biases.
pub fn capabilities_from_genome(genome: &VariableGenome) -> u8 {
    capabilities_from_biases(&effective_biases(genome), genome.gene_count())
}

/// Internal: capabilities from pre-computed biases + gene count. Avoids redundant effective_biases call.
fn capabilities_from_biases(eb: &[f32; 4], n: usize) -> u8 {
    let mut flags = CAP_GROW | CAP_MOVE | CAP_BRANCH | CAP_ROOT;
    if n >= SENSE_GENE_THRESHOLD && eb[1] > CAPABILITY_BIAS_THRESHOLD {
        flags |= CAP_SENSE;
    }
    if n >= REPRODUCE_GENE_THRESHOLD && eb[0] > CAPABILITY_BIAS_THRESHOLD {
        flags |= CAP_REPRODUCE;
    }
    if n >= ARMOR_GENE_THRESHOLD && eb[3] > CAPABILITY_BIAS_THRESHOLD {
        flags |= CAP_ARMOR;
    }
    if n >= PHOTOSYNTH_GENE_THRESHOLD && eb[2] > CAPABILITY_BIAS_THRESHOLD {
        flags |= CAP_PHOTOSYNTH;
    }
    flags
}

// ─── VG-5: Epigenetic gating ────────────────────────────────────────────────

/// Effective bias gated by epigenetic expression mask. `raw × mask[i%4]`.
pub fn gated_effective_bias(
    genome: &VariableGenome,
    expression_mask: &[f32; 4],
    core_index: usize,
) -> f32 {
    effective_bias(genome, core_index) * expression_mask[core_index.min(3)].clamp(0.0, 1.0)
}

/// All 4 gated biases.
pub fn gated_effective_biases(genome: &VariableGenome, expression_mask: &[f32; 4]) -> [f32; 4] {
    std::array::from_fn(|i| gated_effective_bias(genome, expression_mask, i))
}

/// Maintenance cost discounted by expression level. Silenced genes cost less.
pub fn gated_maintenance_cost(
    genome: &VariableGenome,
    expression_mask: &[f32; 4],
    base_dissipation: f32,
) -> f32 {
    let n = genome.gene_count();
    if n == 0 {
        return 0.0;
    }
    let expression_avg: f32 = (0..n)
        .map(|i| expression_mask[i % MIN_GENES].clamp(0.0, 1.0))
        .sum::<f32>()
        / n as f32;
    genome_maintenance_cost(n, base_dissipation) * expression_avg
}

// ─── VG-6: Bridge / Serialization ───────────────────────────────────────────

/// Convert GenomeBlob fields → VariableGenome (backward compatible).
pub fn from_genome_blob(
    growth: f32,
    mobility: f32,
    branching: f32,
    resilience: f32,
    sigma: f32,
) -> VariableGenome {
    let mut g = VariableGenome::from_biases(growth, mobility, branching, resilience);
    g.sigma = sigma;
    g
}

/// Extract effective biases + sigma for GenomeBlob conversion.
pub fn to_genome_blob_biases(genome: &VariableGenome) -> ([f32; 4], f32) {
    (effective_biases(genome), genome.sigma)
}

/// Serialize: `[len:1][sigma:4 LE][genes:4×len LE]`. Size = 5 + 4×len.
pub fn serialize_variable_genome(genome: &VariableGenome) -> Vec<u8> {
    let n = genome.len as usize;
    let mut buf = Vec::with_capacity(1 + 4 + n * 4);
    buf.push(genome.len);
    buf.extend_from_slice(&genome.sigma.to_le_bytes());
    genome
        .active()
        .iter()
        .for_each(|g| buf.extend_from_slice(&g.to_le_bytes()));
    buf
}

/// Deserialize. Returns `None` if malformed.
pub fn deserialize_variable_genome(data: &[u8]) -> Option<VariableGenome> {
    let &len_byte = data.first()?;
    let len = len_byte as usize;
    if len < MIN_GENES || len > MAX_GENES {
        return None;
    }
    if data.len() < 1 + 4 + len * 4 {
        return None;
    }

    let sigma = f32::from_le_bytes([data[1], data[2], data[3], data[4]]);
    if !sigma.is_finite() {
        return None;
    }

    let mut genes = [0.0f32; MAX_GENES];
    for i in 0..len {
        let off = 5 + i * 4;
        let v = f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        if !v.is_finite() {
            return None;
        }
        genes[i] = v;
    }
    Some(VariableGenome {
        genes,
        len: len_byte,
        sigma,
    })
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── VG-1: Struct contracts ──────────────────────────────────────────────

    #[test]
    fn default_has_four_genes() {
        assert_eq!(VariableGenome::default().gene_count(), 4);
    }
    #[test]
    fn default_active_len() {
        assert_eq!(VariableGenome::default().active().len(), 4);
    }

    #[test]
    fn from_biases_sets_core() {
        let g = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        assert!((g.growth() - 0.1).abs() < 1e-5);
        assert!((g.mobility() - 0.2).abs() < 1e-5);
        assert!((g.branching() - 0.3).abs() < 1e-5);
        assert!((g.resilience() - 0.4).abs() < 1e-5);
    }

    #[test]
    fn from_biases_clamps() {
        let g = VariableGenome::from_biases(2.0, -1.0, 0.5, 0.5);
        assert_eq!(g.growth(), 1.0);
        assert_eq!(g.mobility(), 0.0);
    }

    #[test]
    fn core_biases_compatible() {
        let g = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        assert_eq!(g.core_biases(), [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn distance_self_zero() {
        assert_eq!(
            VariableGenome::default().distance(&VariableGenome::default()),
            0.0
        );
    }

    #[test]
    fn distance_symmetric() {
        let a = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        let b = VariableGenome::from_biases(0.5, 0.6, 0.7, 0.8);
        assert!((a.distance(&b) - b.distance(&a)).abs() < 1e-5);
    }

    #[test]
    fn distance_different_lengths_uses_shorter() {
        let a = VariableGenome::from_biases(0.0, 0.0, 0.0, 0.0);
        let mut b = VariableGenome::from_biases(0.0, 0.0, 0.0, 0.0);
        b.genes[4] = 1.0;
        b.len = 5;
        assert_eq!(a.distance(&b), 0.0);
    }

    #[test]
    fn hash_deterministic() {
        let g = VariableGenome::default();
        assert_eq!(g.hash(), g.hash());
    }

    #[test]
    fn hash_differs() {
        let a = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        let b = VariableGenome::from_biases(0.5, 0.6, 0.7, 0.8);
        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn inactive_genes_are_zero() {
        let g = VariableGenome::default();
        for i in 4..MAX_GENES {
            assert_eq!(g.genes[i], 0.0, "inactive gene[{i}] not zero");
        }
    }

    // ── VG-2: Cost contracts ────────────────────────────────────────────────

    #[test]
    fn cost_base_equals_dissipation() {
        assert!((genome_maintenance_cost(4, 0.005) - 0.005).abs() < 1e-6);
    }
    #[test]
    fn cost_zero_genes_zero() {
        assert_eq!(genome_maintenance_cost(0, 0.005), 0.0);
    }

    #[test]
    fn cost_monotonically_increases() {
        let costs: Vec<f32> = [4, 8, 16, 32]
            .iter()
            .map(|&n| genome_maintenance_cost(n, 0.005))
            .collect();
        for w in costs.windows(2) {
            assert!(w[1] > w[0]);
        }
    }

    #[test]
    fn cost_kleiber_scaling() {
        let ratio = genome_maintenance_cost(8, 1.0) / genome_maintenance_cost(4, 1.0);
        assert!((ratio - 2.0f32.powf(KLEIBER_EXPONENT)).abs() < 1e-3);
    }

    #[test]
    fn effective_bias_identity_four_genes() {
        let g = VariableGenome::from_biases(0.3, 0.7, 0.1, 0.9);
        for i in 0..4 {
            assert_eq!(effective_bias(&g, i), g.genes[i]);
        }
    }

    #[test]
    fn effective_bias_modulated() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        g.genes[4] = 1.0;
        g.len = 5;
        assert!(effective_bias(&g, 0) > 0.5);
    }

    #[test]
    fn effective_bias_bounded() {
        let mut g = VariableGenome::default();
        g.genes[0] = 1.0;
        for i in 4..MAX_GENES {
            g.genes[i] = 1.0;
        }
        g.len = MAX_GENES as u8;
        let eff = effective_bias(&g, 0);
        assert!(eff >= 0.0 && eff <= 1.0, "out of [0,1]: {eff}");
    }

    #[test]
    fn effective_biases_len_four() {
        assert_eq!(effective_biases(&VariableGenome::default()).len(), 4);
    }

    // ── VG-3: Mutation contracts ────────────────────────────────────────────

    #[test]
    fn mutate_min_length() {
        let g = VariableGenome::default();
        (0..200).for_each(|s| assert!(mutate_variable(&g, s).gene_count() >= MIN_GENES));
    }

    #[test]
    fn mutate_max_length() {
        let mut g = VariableGenome::default();
        g.len = MAX_GENES as u8;
        for i in 0..MAX_GENES {
            g.genes[i] = 0.5;
        }
        (0..200).for_each(|s| assert!(mutate_variable(&g, s).gene_count() <= MAX_GENES));
    }

    #[test]
    fn mutate_deterministic() {
        let g = VariableGenome::from_biases(0.3, 0.6, 0.2, 0.8);
        assert_eq!(mutate_variable(&g, 42), mutate_variable(&g, 42));
    }

    #[test]
    fn mutate_produces_variation() {
        let g = VariableGenome::default();
        assert_ne!(g.genes[0], mutate_variable(&g, 42).genes[0]);
    }

    #[test]
    fn mutate_genes_in_unit() {
        let g = VariableGenome::from_biases(0.01, 0.99, 0.5, 0.5);
        for s in 0..500 {
            for &v in mutate_variable(&g, s).active() {
                assert!(v >= 0.0 && v <= 1.0, "out of bounds: {v}");
            }
        }
    }

    #[test]
    fn mutate_inactive_genes_zero_after_deletion() {
        let mut g = VariableGenome::default();
        g.len = 8;
        for i in 4..8 {
            g.genes[i] = 0.5;
        }
        for s in 0..500 {
            let m = mutate_variable(&g, s);
            for i in m.gene_count()..MAX_GENES {
                assert_eq!(
                    m.genes[i], 0.0,
                    "inactive gene[{i}] not zero after mutation seed {s}"
                );
            }
        }
    }

    #[test]
    fn mutate_can_grow() {
        assert!((0..1000).any(|s| mutate_variable(&VariableGenome::default(), s).gene_count() > 4));
    }

    #[test]
    fn mutate_can_shrink() {
        let mut g = VariableGenome::default();
        g.len = 8;
        for i in 4..8 {
            g.genes[i] = 0.5;
        }
        assert!((0..1000).any(|s| mutate_variable(&g, s).gene_count() < 8));
    }

    #[test]
    fn mutate_sigma_adapts() {
        let g = VariableGenome::default();
        let m = mutate_variable(&g, 42);
        assert_ne!(g.sigma, m.sigma);
        assert!(m.sigma >= 0.001 && m.sigma <= 0.3);
    }

    // ── Crossover contracts ─────────────────────────────────────────────────

    #[test]
    fn crossover_deterministic() {
        let (a, b) = (
            VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4),
            VariableGenome::from_biases(0.9, 0.8, 0.7, 0.6),
        );
        assert_eq!(
            crossover_variable(&a, &b, 42),
            crossover_variable(&a, &b, 42)
        );
    }

    #[test]
    fn crossover_same_len() {
        let a = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        assert_eq!(crossover_variable(&a, &a, 42).gene_count(), 4);
    }

    #[test]
    fn crossover_max_len() {
        let a = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        let mut b = VariableGenome::from_biases(0.9, 0.8, 0.7, 0.6);
        b.genes[4] = 0.5;
        b.len = 5;
        assert_eq!(crossover_variable(&a, &b, 42).gene_count(), 5);
    }

    #[test]
    fn crossover_genes_from_parents() {
        let a = VariableGenome::from_biases(0.0, 0.0, 0.0, 0.0);
        let b = VariableGenome::from_biases(1.0, 1.0, 1.0, 1.0);
        for &v in crossover_variable(&a, &b, 42).active() {
            assert!(v == 0.0 || v == 1.0);
        }
    }

    #[test]
    fn crossover_sigma_average() {
        let (mut a, mut b) = (VariableGenome::default(), VariableGenome::default());
        a.sigma = 0.1;
        b.sigma = 0.3;
        assert!((crossover_variable(&a, &b, 42).sigma - 0.2).abs() < 1e-5);
    }

    // ── VG-4: Expression mapping ────────────────────────────────────────────

    #[test]
    fn profile_identity() {
        let p = genome_to_profile(&VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4));
        assert!((p[0] - 0.1).abs() < 1e-5);
    }

    #[test]
    fn profile_modulated() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        g.genes[4] = 1.0;
        g.genes[5] = 0.0;
        g.len = 6;
        let p = genome_to_profile(&g);
        assert!(p[0] > 0.5);
        assert!(p[1] < 0.5);
    }

    #[test]
    fn caps_baseline() {
        let caps = capabilities_from_genome(&VariableGenome::default());
        assert_eq!(caps & 0x0F, 0x0F);
        assert_eq!(caps & 0xF0, 0x00);
    }

    #[test]
    fn caps_unlock_with_length() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..12 {
            g.genes[i] = 0.5;
        }
        g.len = 12;
        let caps = capabilities_from_genome(&g);
        assert!(caps & CAP_SENSE != 0);
        assert!(caps & CAP_REPRODUCE != 0);
        assert!(caps & CAP_ARMOR != 0);
    }

    #[test]
    fn caps_gated_by_bias() {
        let mut g = VariableGenome::from_biases(0.1, 0.1, 0.1, 0.1);
        for i in 4..12 {
            g.genes[i] = 0.1;
        }
        g.len = 12;
        assert_eq!(capabilities_from_genome(&g) & CAP_SENSE, 0);
    }

    // ── VG-5: Epigenetic gating ─────────────────────────────────────────────

    #[test]
    fn gated_full_mask_equals_raw() {
        let g = VariableGenome::from_biases(0.7, 0.3, 0.5, 0.9);
        for i in 0..4 {
            assert!((gated_effective_bias(&g, &[1.0; 4], i) - effective_bias(&g, i)).abs() < 1e-6);
        }
    }

    #[test]
    fn gated_zero_mask_is_zero() {
        let g = VariableGenome::from_biases(0.7, 0.3, 0.5, 0.9);
        for i in 0..4 {
            assert_eq!(gated_effective_bias(&g, &[0.0; 4], i), 0.0);
        }
    }

    #[test]
    fn gated_half_mask_halves() {
        let g = VariableGenome::from_biases(0.8, 0.8, 0.8, 0.8);
        for i in 0..4 {
            assert!(
                (gated_effective_bias(&g, &[0.5; 4], i) - effective_bias(&g, i) * 0.5).abs() < 1e-5
            );
        }
    }

    #[test]
    fn gated_biases_correct() {
        let gb = gated_effective_biases(
            &VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5),
            &[0.8, 0.2, 1.0, 0.0],
        );
        assert!(gb[0] > gb[1]);
        assert_eq!(gb[3], 0.0);
    }

    #[test]
    fn gated_cost_full_equals_raw() {
        let g = VariableGenome::default();
        let raw = genome_maintenance_cost(g.gene_count(), DISSIPATION_SOLID);
        assert!((gated_maintenance_cost(&g, &[1.0; 4], DISSIPATION_SOLID) - raw).abs() < 1e-6);
    }

    #[test]
    fn gated_cost_silenced_cheaper() {
        let mut g = VariableGenome::default();
        g.len = 8;
        for i in 4..8 {
            g.genes[i] = 0.5;
        }
        let full = gated_maintenance_cost(&g, &[1.0; 4], DISSIPATION_SOLID);
        assert!(gated_maintenance_cost(&g, &[0.5; 4], DISSIPATION_SOLID) < full);
    }

    #[test]
    fn gated_cost_zero_mask_zero() {
        assert_eq!(
            gated_maintenance_cost(&VariableGenome::default(), &[0.0; 4], DISSIPATION_SOLID),
            0.0
        );
    }

    // ── VG-6: Bridge / Serialization ────────────────────────────────────────

    #[test]
    fn blob_round_trip() {
        let vg = from_genome_blob(0.3, 0.6, 0.2, 0.8, 0.15);
        assert_eq!(vg.gene_count(), 4);
        assert!((vg.growth() - 0.3).abs() < 1e-5);
    }

    #[test]
    fn blob_biases_identity() {
        let (b, s) = to_genome_blob_biases(&from_genome_blob(0.1, 0.2, 0.3, 0.4, 0.1));
        assert!((b[0] - 0.1).abs() < 1e-5);
        assert!((s - 0.1).abs() < 1e-5);
    }

    #[test]
    fn serialize_round_trip_4() {
        let vg = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        assert_eq!(
            deserialize_variable_genome(&serialize_variable_genome(&vg)).unwrap(),
            vg
        );
    }

    #[test]
    fn serialize_round_trip_variable() {
        let mut vg = VariableGenome::from_biases(0.1, 0.2, 0.3, 0.4);
        vg.genes[4] = 0.55;
        vg.genes[5] = 0.66;
        vg.len = 6;
        vg.sigma = 0.123;
        let decoded = deserialize_variable_genome(&serialize_variable_genome(&vg)).unwrap();
        assert_eq!(decoded, vg);
    }

    #[test]
    fn serialize_size_proportional() {
        assert_eq!(
            serialize_variable_genome(&VariableGenome::default()).len(),
            1 + 4 + 4 * 4
        );
        let mut g8 = VariableGenome::default();
        g8.len = 8;
        assert_eq!(serialize_variable_genome(&g8).len(), 1 + 4 + 4 * 8);
    }

    #[test]
    fn deserialize_rejects_empty() {
        assert!(deserialize_variable_genome(&[]).is_none());
    }
    #[test]
    fn deserialize_rejects_short() {
        assert!(deserialize_variable_genome(&[4]).is_none());
    }
    #[test]
    fn deserialize_rejects_under_min() {
        let mut d = vec![2u8, 0, 0, 0, 0];
        d.extend_from_slice(&[0u8; 8]);
        assert!(deserialize_variable_genome(&d).is_none());
    }
    #[test]
    fn deserialize_rejects_over_max() {
        assert!(deserialize_variable_genome(&[33u8]).is_none());
    }

    #[test]
    fn deserialize_rejects_nan_sigma() {
        let mut vg = VariableGenome::default();
        vg.sigma = f32::NAN;
        let bytes = {
            let n = vg.len as usize;
            let mut buf = vec![vg.len];
            buf.extend_from_slice(&vg.sigma.to_le_bytes());
            for i in 0..n {
                buf.extend_from_slice(&vg.genes[i].to_le_bytes());
            }
            buf
        };
        assert!(
            deserialize_variable_genome(&bytes).is_none(),
            "NaN sigma rejected"
        );
    }

    #[test]
    fn deserialize_rejects_inf_gene() {
        let mut vg = VariableGenome::default();
        vg.genes[0] = f32::INFINITY;
        let bytes = {
            let n = vg.len as usize;
            let mut buf = vec![vg.len];
            buf.extend_from_slice(&vg.sigma.to_le_bytes());
            for i in 0..n {
                buf.extend_from_slice(&vg.genes[i].to_le_bytes());
            }
            buf
        };
        assert!(
            deserialize_variable_genome(&bytes).is_none(),
            "Inf gene rejected"
        );
    }

    #[test]
    fn serialize_bit_exact() {
        let vg = VariableGenome::from_biases(
            std::f32::consts::PI / 7.0,
            std::f32::consts::E / 3.0,
            0.123_456_78,
            0.987_654_3,
        );
        let d = deserialize_variable_genome(&serialize_variable_genome(&vg)).unwrap();
        for i in 0..4 {
            assert_eq!(vg.genes[i].to_bits(), d.genes[i].to_bits());
        }
    }

    // ── Cache / Phenotype ───────────────────────────────────────────────────

    #[test]
    fn phenotype_cache_consistent() {
        let g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        let mask = [1.0; 4];
        let p = compute_phenotype(&g, &mask, DISSIPATION_SOLID);
        assert_eq!(p.biases, gated_effective_biases(&g, &mask));
        assert_eq!(p.capabilities, capabilities_from_genome(&g));
        assert!(
            (p.maintenance_cost - gated_maintenance_cost(&g, &mask, DISSIPATION_SOLID)).abs()
                < 1e-6
        );
    }

    #[test]
    fn phenotype_gene_count_matches() {
        let mut g = VariableGenome::default();
        g.len = 10;
        let p = compute_phenotype(&g, &[1.0; 4], DISSIPATION_SOLID);
        assert_eq!(p.gene_count, 10);
    }

    // ── Full pipeline integration ───────────────────────────────────────────

    #[test]
    fn pipeline_mutate_gate_profile() {
        let m = mutate_variable(&VariableGenome::default(), 42);
        let biases = gated_effective_biases(&m, &[1.0, 0.5, 1.0, 0.0]);
        assert_eq!(biases[3], 0.0);
        assert!(biases[0] > 0.0);
        assert!(gated_maintenance_cost(&m, &[1.0, 0.5, 1.0, 0.0], DISSIPATION_SOLID) > 0.0);
    }

    #[test]
    fn pipeline_serialize_mutate() {
        let m = mutate_variable(&VariableGenome::from_biases(0.3, 0.7, 0.2, 0.8), 123);
        assert_eq!(
            deserialize_variable_genome(&serialize_variable_genome(&m)).unwrap(),
            m
        );
    }

    #[test]
    fn pipeline_longer_genome_higher_cost() {
        let c4 = genome_maintenance_cost(4, DISSIPATION_SOLID);
        let c16 = genome_maintenance_cost(16, DISSIPATION_SOLID);
        assert!(c16 > c4);
    }
}
