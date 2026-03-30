//! Protein Folding — PF-1 through PF-5.
//!
//! Coarse-grained lattice protein model. Gene sequences fold into 2D structures
//! by energy minimization. Active sites emerge from contact geometry.
//! Function follows form. Evolution acts on sequence → fold → function → fitness.
//!
//! Axiom 1: monomers ARE energy (hydrophobicity = energy affinity).
//! Axiom 4: fold minimizes energy (dissipation drives compaction).
//! Axiom 7: only adjacent lattice cells interact (distance attenuation).
//! Axiom 8: H-H contact strength modulated by frequency alignment.
//!
//! Based on: Dill (1985) HP model, simplified for emergent evolution.

use super::derived_thresholds::{DISSIPATION_SOLID, KLEIBER_EXPONENT};
use super::determinism;
use super::variable_genome::{VariableGenome, MIN_GENES, MAX_GENES};
use crate::layers::OrganRole;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Hydrophobicity threshold: gene > this = H (hydrophobic), else P (polar).
/// Derived: 1.0 - KLEIBER_EXPONENT = 0.25. Same as CAPABILITY_BIAS_THRESHOLD.
const HYDROPHOBIC_THRESHOLD: f32 = 1.0 - KLEIBER_EXPONENT;

/// Energy per H-H contact. Derived: DISSIPATION_SOLID × 200 = 1.0 qe.
/// Negative = attractive (minimized by fold). Axiom 4.
const HH_CONTACT_ENERGY: f32 = -(DISSIPATION_SOLID * 200.0);

/// Frequency alignment bandwidth for contact strength. Axiom 8.
/// Same as COHERENCE_BANDWIDTH (4th fundamental constant).
const FOLD_BANDWIDTH_HZ: f32 = 50.0;

/// Frequency modulation range (Hz). Gene value offsets base frequency by ±this.
/// Derived: FOLD_BANDWIDTH_HZ × 4 = 200. Spans 4 bandwidths for full diversity.
const FREQ_MODULATION_RANGE: f32 = FOLD_BANDWIDTH_HZ * 4.0;

/// Base frequencies per dimension (same as bridge::frequency_for_archetype).
/// Derived from COHERENCE_BANDWIDTH spacing: bands separated by ≥ 2×BW.
const DIM_BASE_FREQ: [f32; 4] = [400.0, 600.0, 300.0, 800.0];

/// Maximum chain length for folding (= MAX_GENES).
pub const MAX_CHAIN: usize = MAX_GENES;

/// Lattice grid size (chain folds within this). Must fit longest chain.
const LATTICE_SIZE: usize = 2 * MAX_CHAIN + 1; // 65×65, centered at (32,32)
const LATTICE_CENTER: i8 = MAX_CHAIN as i8;

/// Minimum contacts for an active site cluster.
/// Derived: MIN_GENES - 1 = 3. Need 3+ non-bonded contacts to form a pocket.
const ACTIVE_SITE_MIN_CONTACTS: u8 = (MIN_GENES - 1) as u8;

/// 4 cardinal directions on the lattice.
const DIRS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

// ─── PF-1: Polymer Chain ────────────────────────────────────────────────────

/// One monomer in the chain. Two properties derived from gene value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Monomer {
    /// Hydrophobicity ∈ [0,1]. > HYDROPHOBIC_THRESHOLD = H type.
    pub hydrophobicity: f32,
    /// Local charge: gradient between this monomer and its neighbors.
    pub charge: f32,
}

impl Monomer {
    #[inline]
    pub fn is_hydrophobic(&self) -> bool {
        self.hydrophobicity > HYDROPHOBIC_THRESHOLD
    }
}

/// Extract polymer chain from VariableGenome. Order = gene order (chain connectivity).
///
/// `hydrophobicity = gene_value`.
/// `charge = (gene[i] - gene[i-1]) / 2` — local gradient. Axiom 7: neighbor influence.
pub fn genome_to_polymer(genome: &VariableGenome) -> ([Monomer; MAX_CHAIN], usize) {
    let n = genome.gene_count();
    let mut chain = [Monomer::default(); MAX_CHAIN];
    for i in 0..n {
        let h = genome.genes[i].clamp(0.0, 1.0);
        let prev = if i > 0 { genome.genes[i - 1].clamp(0.0, 1.0) } else { h };
        chain[i] = Monomer {
            hydrophobicity: h,
            charge: (h - prev) * 0.5,
        };
    }
    (chain, n)
}

// ─── PF-2: Lattice Fold ────────────────────────────────────────────────────

/// 2D lattice position for each monomer.
pub type FoldState = [(i8, i8); MAX_CHAIN];

/// Fold energy: Σ H-H contacts between non-consecutive monomers on adjacent cells.
///
/// `E = Σ_{|i-j|>1, adjacent} hydro_i × hydro_j × HH_CONTACT_ENERGY × freq_align`.
/// Lower = more stable. Axiom 4: minimum energy principle.
pub fn fold_energy(
    chain: &[Monomer],
    positions: &FoldState,
    len: usize,
    frequencies: &[f32],
) -> f32 {
    let mut energy = 0.0f32;
    for i in 0..len {
        for j in (i + 2)..len {
            if !lattice_adjacent(positions[i], positions[j]) { continue; }
            if !chain[i].is_hydrophobic() || !chain[j].is_hydrophobic() { continue; }
            let freq_align = frequency_alignment(
                frequencies.get(i).copied().unwrap_or(0.0),
                frequencies.get(j).copied().unwrap_or(0.0),
            );
            energy += chain[i].hydrophobicity * chain[j].hydrophobicity
                * HH_CONTACT_ENERGY * freq_align;
        }
    }
    energy
}

/// Greedy fold: place monomers one by one, choosing direction that minimizes energy.
///
/// O(N × 4) per monomer. Not globally optimal, but deterministic and fast.
/// Axiom 6: fold emerges from energy minimization, not template.
pub fn fold_greedy(
    chain: &[Monomer],
    len: usize,
    frequencies: &[f32],
    seed: u64,
) -> FoldState {
    let mut pos = [(0i8, 0i8); MAX_CHAIN];
    let mut occupied = [[false; LATTICE_SIZE]; LATTICE_SIZE];
    let n = len.min(MAX_CHAIN);
    if n == 0 { return pos; }

    // First monomer at center
    pos[0] = (LATTICE_CENTER, LATTICE_CENTER);
    occupied[LATTICE_CENTER as usize][LATTICE_CENTER as usize] = true;

    let mut rng = seed;
    for i in 1..n {
        let prev = pos[i - 1];
        let mut best_dir = 0usize;
        let mut best_energy = f32::MAX;
        let mut best_count = 0u8;

        for (d, &(dx, dy)) in DIRS.iter().enumerate() {
            let nx = prev.0 + dx;
            let ny = prev.1 + dy;
            if !in_lattice(nx, ny) || occupied[nx as usize][ny as usize] { continue; }

            // Temporarily place and evaluate
            pos[i] = (nx, ny);
            let e = partial_energy(chain, &pos, i, frequencies);
            let contacts = count_hh_contacts(chain, &pos, i);

            if e < best_energy || (e == best_energy && contacts > best_count) {
                best_energy = e;
                best_dir = d;
                best_count = contacts;
            }
        }

        // Apply best direction (or random if all occupied)
        let (dx, dy) = DIRS[best_dir];
        let chosen = (prev.0 + dx, prev.1 + dy);
        if in_lattice(chosen.0, chosen.1) && !occupied[chosen.0 as usize][chosen.1 as usize] {
            pos[i] = chosen;
            occupied[chosen.0 as usize][chosen.1 as usize] = true;
        } else {
            // Fallback: find any free adjacent cell
            rng = determinism::next_u64(rng);
            let start = (determinism::unit_f32(rng) * 4.0) as usize;
            let mut placed = false;
            for k in 0..4 {
                let (fdx, fdy) = DIRS[(start + k) % 4];
                let fx = prev.0 + fdx;
                let fy = prev.1 + fdy;
                if in_lattice(fx, fy) && !occupied[fx as usize][fy as usize] {
                    pos[i] = (fx, fy);
                    occupied[fx as usize][fy as usize] = true;
                    placed = true;
                    break;
                }
            }
            if !placed {
                // Chain is stuck (all neighbors occupied). Truncate.
                return pos;
            }
        }
    }
    pos
}

// ─── PF-3: Contact Map ─────────────────────────────────────────────────────

/// Contact map: which non-consecutive monomers are adjacent in folded structure.
pub fn contact_map(positions: &FoldState, len: usize) -> [[bool; MAX_CHAIN]; MAX_CHAIN] {
    let mut map = [[false; MAX_CHAIN]; MAX_CHAIN];
    let n = len.min(MAX_CHAIN);
    for i in 0..n {
        for j in (i + 2)..n {
            if lattice_adjacent(positions[i], positions[j]) {
                map[i][j] = true;
                map[j][i] = true;
            }
        }
    }
    map
}

/// Contact density: number of non-chain contacts per monomer.
pub fn contact_density(contacts: &[[bool; MAX_CHAIN]; MAX_CHAIN], len: usize) -> [u8; MAX_CHAIN] {
    let mut density = [0u8; MAX_CHAIN];
    let n = len.min(MAX_CHAIN);
    for i in 0..n {
        density[i] = (0..n).filter(|&j| contacts[i][j]).count() as u8;
    }
    density
}

// ─── PF-4: Function from Shape ──────────────────────────────────────────────

/// Inferred protein function from folded geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProteinFunction {
    /// Which metabolic node type this protein can catalyze.
    pub catalytic_target: OrganRole,
    /// How much it reduces activation energy of the target. Axiom 4.
    pub efficiency_boost: f32,
    /// Frequency specificity of the active site. Axiom 8.
    pub specificity: f32,
    /// Number of monomers in the active site.
    pub active_site_size: u8,
}

/// Infer function from folded contact geometry.
///
/// Active site = cluster of H monomers with high contact density.
/// Function type = OrganRole inferred from mean frequency of active site.
/// Efficiency = density × hydrophobicity mean of cluster. Axiom 4: better fold = more catalysis.
pub fn infer_protein_function(
    chain: &[Monomer],
    density: &[u8; MAX_CHAIN],
    frequencies: &[f32],
    len: usize,
) -> Option<ProteinFunction> {
    let n = len.min(MAX_CHAIN);
    if n < MIN_GENES { return None; }

    // Find active site: H monomers with density >= threshold
    let mut site_indices = [0usize; MAX_CHAIN];
    let mut site_count = 0usize;
    for i in 0..n {
        if chain[i].is_hydrophobic() && density[i] >= ACTIVE_SITE_MIN_CONTACTS {
            site_indices[site_count] = i;
            site_count += 1;
        }
    }

    if site_count == 0 { return None; }

    // Mean frequency of active site → determines catalytic target
    let mean_freq: f32 = site_indices[..site_count].iter()
        .map(|&i| frequencies.get(i).copied().unwrap_or(0.0))
        .sum::<f32>() / site_count as f32;

    // Map frequency to OrganRole via dimension (same as MGN-1 mapping)
    let target = infer_target_from_frequency(mean_freq);

    // Efficiency: mean hydrophobicity × mean density, scaled by Kleiber
    let mean_hydro: f32 = site_indices[..site_count].iter()
        .map(|&i| chain[i].hydrophobicity)
        .sum::<f32>() / site_count as f32;
    let mean_density: f32 = site_indices[..site_count].iter()
        .map(|&i| density[i] as f32)
        .sum::<f32>() / site_count as f32;
    let efficiency = (mean_hydro * mean_density * DISSIPATION_SOLID * 10.0)
        .powf(KLEIBER_EXPONENT)
        .min(0.5); // Cap: a single protein can't boost more than 50%

    // Specificity: how tight is the frequency alignment within active site
    let freq_variance: f32 = site_indices[..site_count].iter()
        .map(|&i| {
            let d = frequencies.get(i).copied().unwrap_or(0.0) - mean_freq;
            d * d
        })
        .sum::<f32>() / site_count.max(1) as f32;
    let specificity = (-freq_variance / (2.0 * FOLD_BANDWIDTH_HZ * FOLD_BANDWIDTH_HZ)).exp();

    Some(ProteinFunction {
        catalytic_target: target,
        efficiency_boost: efficiency,
        specificity,
        active_site_size: site_count as u8,
    })
}

/// Map mean frequency to OrganRole target.
fn infer_target_from_frequency(freq: f32) -> OrganRole {
    // Frequency bands (same as bridge::frequency_for_archetype):
    // 0-150: Stem, 150-250: Thorn, 250-350: Core/Shell,
    // 350-500: Root/Leaf, 500-700: Fin/Limb, 700+: Sensory/Fruit
    match freq as u32 {
        0..=149     => OrganRole::Stem,
        150..=249   => OrganRole::Thorn,
        250..=349   => OrganRole::Core,
        350..=499   => OrganRole::Root,
        500..=699   => OrganRole::Fin,
        _           => OrganRole::Sensory,
    }
}

// ─── PF-5: Full pipeline + Cache ────────────────────────────────────────────

/// Complete protein phenotype: sequence → fold → contacts → function.
#[derive(Clone, Debug)]
pub struct ProteinPhenotype {
    pub chain_length: u8,
    pub fold_energy: f32,
    pub h_count: u8,
    pub p_count: u8,
    pub total_contacts: u16,
    pub function: Option<ProteinFunction>,
}

/// Full pipeline. Cache-friendly: one call per genome per change.
///
/// Deterministic: same genome + seed → identical phenotype.
pub fn compute_protein_phenotype(
    genome: &VariableGenome,
    seed: u64,
) -> ProteinPhenotype {
    let (chain, len) = genome_to_polymer(genome);

    // Derive frequencies from gene positions (same scheme as MetabolicGenome)
    let mut frequencies = [0.0f32; MAX_CHAIN];
    for i in 0..len {
        // Base freq from dimension, modulated by gene value
        frequencies[i] = DIM_BASE_FREQ[i % 4] + (genome.genes[i] - 0.5) * FREQ_MODULATION_RANGE;
    }

    let fold = fold_greedy(&chain, len, &frequencies, seed);
    let energy = fold_energy(&chain, &fold, len, &frequencies);
    let contacts = contact_map(&fold, len);
    let density = contact_density(&contacts, len);
    let function = infer_protein_function(&chain, &density, &frequencies, len);

    let h_count = chain[..len].iter().filter(|m| m.is_hydrophobic()).count() as u8;

    let total_contacts: u16 = density[..len].iter().map(|&d| d as u16).sum::<u16>() / 2;

    ProteinPhenotype {
        chain_length: len as u8,
        fold_energy: energy,
        h_count,
        p_count: len as u8 - h_count,
        total_contacts,
        function,
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

#[inline]
fn lattice_adjacent(a: (i8, i8), b: (i8, i8)) -> bool {
    let dx = (a.0 - b.0).abs();
    let dy = (a.1 - b.1).abs();
    (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
}

#[inline]
fn in_lattice(x: i8, y: i8) -> bool {
    x >= 0 && y >= 0 && (x as usize) < LATTICE_SIZE && (y as usize) < LATTICE_SIZE
}

/// Axiom 8: frequency alignment. Delegates to centralized implementation.
fn frequency_alignment(f_a: f32, f_b: f32) -> f32 {
    super::determinism::gaussian_frequency_alignment(f_a, f_b, FOLD_BANDWIDTH_HZ)
}

/// H-H contact contribution between two monomers. Extracted to avoid duplication.
fn hh_contact_contribution(chain: &[Monomer], i: usize, j: usize, freq: &[f32]) -> f32 {
    let fa = frequency_alignment(
        freq.get(i).copied().unwrap_or(0.0),
        freq.get(j).copied().unwrap_or(0.0),
    );
    chain[i].hydrophobicity * chain[j].hydrophobicity * HH_CONTACT_ENERGY * fa
}

/// Energy contribution of placing monomer `idx` given current positions.
/// Checks all previous monomers except direct chain neighbor (i-1).
fn partial_energy(chain: &[Monomer], pos: &FoldState, idx: usize, freq: &[f32]) -> f32 {
    if idx < 2 || !chain[idx].is_hydrophobic() { return 0.0; }
    (0..idx.saturating_sub(1))
        .filter(|&j| chain[j].is_hydrophobic() && lattice_adjacent(pos[idx], pos[j]))
        .map(|j| hh_contact_contribution(chain, idx, j, freq))
        .sum()
}

/// Count H-H contacts for monomer at `idx` (non-consecutive neighbors only).
fn count_hh_contacts(chain: &[Monomer], pos: &FoldState, idx: usize) -> u8 {
    if idx < 2 || !chain[idx].is_hydrophobic() { return 0; }
    (0..idx.saturating_sub(1))
        .filter(|&j| chain[j].is_hydrophobic() && lattice_adjacent(pos[idx], pos[j]))
        .count() as u8
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PF-1: Polymer Chain ─────────────────────────────────────────────────

    #[test]
    fn polymer_four_genes_four_monomers() {
        let g = VariableGenome::from_biases(0.8, 0.1, 0.9, 0.3);
        let (chain, len) = genome_to_polymer(&g);
        assert_eq!(len, 4);
        assert!(chain[0].is_hydrophobic()); // 0.8 > 0.25
        assert!(!chain[1].is_hydrophobic()); // 0.1 < 0.25
    }

    #[test]
    fn polymer_charge_is_gradient() {
        let g = VariableGenome::from_biases(0.2, 0.8, 0.2, 0.8);
        let (chain, _) = genome_to_polymer(&g);
        assert!(chain[1].charge > 0.0, "0.2→0.8 = positive gradient");
        assert!(chain[2].charge < 0.0, "0.8→0.2 = negative gradient");
    }

    #[test]
    fn polymer_first_monomer_zero_charge() {
        let g = VariableGenome::default();
        let (chain, _) = genome_to_polymer(&g);
        assert_eq!(chain[0].charge, 0.0, "first monomer has no previous → zero charge");
    }

    #[test]
    fn polymer_length_matches_genome() {
        let mut g = VariableGenome::default();
        g.len = 10;
        let (_, len) = genome_to_polymer(&g);
        assert_eq!(len, 10);
    }

    // ── PF-2: Lattice Fold ──────────────────────────────────────────────────

    #[test]
    fn fold_single_monomer_at_center() {
        let g = VariableGenome::from_biases(0.5, 0.0, 0.0, 0.0);
        let mut g1 = g;
        g1.len = 1;
        let (chain, len) = genome_to_polymer(&g1);
        let fold = fold_greedy(&chain, len, &[400.0], 42);
        assert_eq!(fold[0], (LATTICE_CENTER, LATTICE_CENTER));
    }

    #[test]
    fn fold_two_monomers_adjacent() {
        let mut g = VariableGenome::default();
        g.len = 2;
        let (chain, len) = genome_to_polymer(&g);
        let fold = fold_greedy(&chain, len, &[400.0, 400.0], 42);
        assert!(lattice_adjacent(fold[0], fold[1]), "consecutive monomers must be adjacent");
    }

    #[test]
    fn fold_chain_connectivity_maintained() {
        let mut g = VariableGenome::from_biases(0.8, 0.9, 0.1, 0.7);
        for i in 4..12 { g.genes[i] = 0.5; }
        g.len = 12;
        let (chain, len) = genome_to_polymer(&g);
        let fold = fold_greedy(&chain, len, &[400.0; 12], 42);
        for i in 1..len {
            assert!(lattice_adjacent(fold[i - 1], fold[i]),
                "chain break at {}: {:?} → {:?}", i, fold[i-1], fold[i]);
        }
    }

    #[test]
    fn fold_no_overlaps() {
        let mut g = VariableGenome::default();
        for i in 0..16 { g.genes[i] = (i as f32 / 16.0); }
        g.len = 16;
        let (chain, len) = genome_to_polymer(&g);
        let fold = fold_greedy(&chain, len, &[400.0; 16], 42);
        for i in 0..len {
            for j in (i+1)..len {
                assert_ne!(fold[i], fold[j], "overlap at {i} and {j}");
            }
        }
    }

    #[test]
    fn fold_deterministic() {
        let g = VariableGenome::from_biases(0.8, 0.3, 0.9, 0.1);
        let (chain, len) = genome_to_polymer(&g);
        let freq = [400.0; 4];
        let a = fold_greedy(&chain, len, &freq, 42);
        let b = fold_greedy(&chain, len, &freq, 42);
        assert_eq!(a[..len], b[..len]);
    }

    #[test]
    fn fold_energy_all_polar_zero() {
        let g = VariableGenome::from_biases(0.1, 0.1, 0.1, 0.1); // all P
        let (chain, len) = genome_to_polymer(&g);
        let fold = fold_greedy(&chain, len, &[400.0; 4], 42);
        let e = fold_energy(&chain, &fold, len, &[400.0; 4]);
        assert_eq!(e, 0.0, "all polar → no H-H contacts → zero energy");
    }

    #[test]
    fn fold_energy_negative_for_hh_contacts() {
        // Create a chain with H monomers that will contact each other
        let mut g = VariableGenome::default();
        for i in 0..8 { g.genes[i] = 0.9; } // all H
        g.len = 8;
        let (chain, len) = genome_to_polymer(&g);
        let freq = [400.0; 8];
        let fold = fold_greedy(&chain, len, &freq, 42);
        let e = fold_energy(&chain, &fold, len, &freq);
        assert!(e <= 0.0, "H-H contacts should give negative (attractive) energy: {e}");
    }

    // ── PF-3: Contact Map ───────────────────────────────────────────────────

    #[test]
    fn contact_map_empty_for_two() {
        let g = VariableGenome::from_biases(0.5, 0.5, 0.0, 0.0);
        let mut g2 = g;
        g2.len = 2;
        let (_, len) = genome_to_polymer(&g2);
        let fold = [(5i8, 5i8), (5, 6), (0,0), (0,0), (0,0), (0,0), (0,0), (0,0),
            (0,0),(0,0),(0,0),(0,0),(0,0),(0,0),(0,0),(0,0),
            (0,0),(0,0),(0,0),(0,0),(0,0),(0,0),(0,0),(0,0),
            (0,0),(0,0),(0,0),(0,0),(0,0),(0,0),(0,0),(0,0)];
        let map = contact_map(&fold, len);
        assert!(!map[0][1], "consecutive monomers are NOT contacts (bonded)");
    }

    #[test]
    fn contact_map_detects_nonadjacent_contact() {
        // Fold: U-shape. positions: (0,0), (1,0), (1,1), (0,1) → 0 and 3 are adjacent
        let mut fold = [(0i8, 0i8); MAX_CHAIN];
        fold[0] = (5, 5); fold[1] = (6, 5); fold[2] = (6, 6); fold[3] = (5, 6);
        let map = contact_map(&fold, 4);
        assert!(map[0][3], "monomers 0 and 3 should be in contact (U-shape)");
        assert!(map[3][0], "symmetric");
    }

    #[test]
    fn contact_density_correct() {
        let mut fold = [(0i8, 0i8); MAX_CHAIN];
        fold[0] = (5, 5); fold[1] = (6, 5); fold[2] = (6, 6); fold[3] = (5, 6);
        let map = contact_map(&fold, 4);
        let dens = contact_density(&map, 4);
        assert_eq!(dens[0], 1, "monomer 0 contacts monomer 3");
        assert_eq!(dens[3], 1, "monomer 3 contacts monomer 0");
    }

    // ── PF-4: Function from Shape ───────────────────────────────────────────

    #[test]
    fn function_none_for_short_chain() {
        let g = VariableGenome::from_biases(0.9, 0.9, 0.9, 0.9);
        let mut g3 = g;
        g3.len = 3;
        let density = [0u8; MAX_CHAIN];
        let func = infer_protein_function(&[Monomer::default(); MAX_CHAIN], &density, &[400.0; MAX_CHAIN], 3);
        assert!(func.is_none(), "too short for active site");
    }

    #[test]
    fn function_none_for_all_polar() {
        let g = VariableGenome::from_biases(0.1, 0.1, 0.1, 0.1);
        let (chain, len) = genome_to_polymer(&g);
        let density = [5u8; MAX_CHAIN]; // high density but all P
        let func = infer_protein_function(&chain, &density, &[400.0; MAX_CHAIN], len);
        assert!(func.is_none(), "all polar → no H residues → no active site");
    }

    #[test]
    fn function_some_for_hydrophobic_cluster() {
        let mut chain = [Monomer::default(); MAX_CHAIN];
        for i in 0..8 { chain[i] = Monomer { hydrophobicity: 0.9, charge: 0.0 }; }
        let mut density = [0u8; MAX_CHAIN];
        for i in 0..8 { density[i] = 4; } // high density
        let freq = [400.0f32; MAX_CHAIN];
        let func = infer_protein_function(&chain, &density, &freq, 8);
        assert!(func.is_some(), "H cluster with high density → active site");
        let f = func.unwrap();
        assert!(f.efficiency_boost > 0.0);
        assert!(f.active_site_size > 0);
    }

    #[test]
    fn function_efficiency_bounded() {
        let mut chain = [Monomer { hydrophobicity: 1.0, charge: 0.0 }; MAX_CHAIN];
        let mut density = [10u8; MAX_CHAIN];
        let func = infer_protein_function(&chain, &density, &[400.0; MAX_CHAIN], 32);
        if let Some(f) = func {
            assert!(f.efficiency_boost <= 0.5, "capped at 50%: {}", f.efficiency_boost);
        }
    }

    #[test]
    fn function_specificity_high_for_uniform_freq() {
        let mut chain = [Monomer { hydrophobicity: 0.9, charge: 0.0 }; MAX_CHAIN];
        let mut density = [4u8; MAX_CHAIN];
        let func_uniform = infer_protein_function(&chain, &density, &[400.0; MAX_CHAIN], 8);
        let mut mixed_freq = [0.0f32; MAX_CHAIN];
        for i in 0..8 { mixed_freq[i] = 100.0 + i as f32 * 200.0; }
        let func_mixed = infer_protein_function(&chain, &density, &mixed_freq, 8);
        if let (Some(u), Some(m)) = (func_uniform, func_mixed) {
            assert!(u.specificity > m.specificity,
                "uniform freq → higher specificity: {} > {}", u.specificity, m.specificity);
        }
    }

    // ── PF-5: Full Pipeline ─────────────────────────────────────────────────

    #[test]
    fn phenotype_deterministic() {
        let g = VariableGenome::from_biases(0.8, 0.3, 0.9, 0.6);
        let a = compute_protein_phenotype(&g, 42);
        let b = compute_protein_phenotype(&g, 42);
        assert_eq!(a.fold_energy.to_bits(), b.fold_energy.to_bits());
        assert_eq!(a.h_count, b.h_count);
        assert_eq!(a.total_contacts, b.total_contacts);
    }

    #[test]
    fn phenotype_h_p_count_sum_to_length() {
        let g = VariableGenome::from_biases(0.8, 0.1, 0.9, 0.05);
        let p = compute_protein_phenotype(&g, 42);
        assert_eq!(p.h_count + p.p_count, p.chain_length);
    }

    #[test]
    fn phenotype_longer_chain_more_contacts() {
        let g4 = VariableGenome::from_biases(0.8, 0.8, 0.8, 0.8);
        let mut g12 = g4;
        for i in 4..12 { g12.genes[i] = 0.8; }
        g12.len = 12;
        let p4 = compute_protein_phenotype(&g4, 42);
        let p12 = compute_protein_phenotype(&g12, 42);
        assert!(p12.total_contacts >= p4.total_contacts,
            "longer chain should have ≥ contacts: {} vs {}", p12.total_contacts, p4.total_contacts);
    }

    #[test]
    fn phenotype_mutation_changes_fold() {
        use super::super::variable_genome::mutate_variable;
        let mut g = VariableGenome::from_biases(0.8, 0.3, 0.9, 0.6);
        for i in 4..12 { g.genes[i] = 0.7; }
        g.len = 12; // longer chain = more sensitive to mutation
        // Try multiple seeds — at least one should produce a different fold
        let p_original = compute_protein_phenotype(&g, 42);
        let mut found_diff = false;
        for seed in 0..50u64 {
            let m = mutate_variable(&g, seed);
            let p_mutated = compute_protein_phenotype(&m, 42);
            if p_original.fold_energy != p_mutated.fold_energy
                || p_original.total_contacts != p_mutated.total_contacts
                || p_original.h_count != p_mutated.h_count {
                found_diff = true;
                break;
            }
        }
        assert!(found_diff, "at least one mutation should change protein phenotype");
    }

    #[test]
    fn phenotype_fold_energy_conservation() {
        let mut g = VariableGenome::default();
        for i in 0..MAX_GENES { g.genes[i] = 0.9; }
        g.len = MAX_GENES as u8;
        let p = compute_protein_phenotype(&g, 42);
        assert!(p.fold_energy <= 0.0, "fold energy should be ≤ 0 (attractive): {}", p.fold_energy);
    }

    // ── Helpers ─────────────────────────────────────────────────────────────

    #[test]
    fn lattice_adjacent_orthogonal() {
        assert!(lattice_adjacent((5, 5), (6, 5)));
        assert!(lattice_adjacent((5, 5), (5, 6)));
        assert!(!lattice_adjacent((5, 5), (6, 6))); // diagonal
        assert!(!lattice_adjacent((5, 5), (7, 5))); // distance 2
    }

    #[test]
    fn frequency_alignment_same_is_one() {
        assert!((frequency_alignment(400.0, 400.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn frequency_alignment_decreases_with_distance() {
        let near = frequency_alignment(400.0, 420.0);
        let far = frequency_alignment(400.0, 600.0);
        assert!(near > far);
    }
}
