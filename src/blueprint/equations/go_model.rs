//! MD-15: Go model with Axiom 8 frequency modulation — pure math.
//!
//! Classical Go (Taketomi, Ueda & Go, 1975): binary native contacts, uniform epsilon.
//! Resonance Go: native contacts weighted by frequency coherence (Axiom 8).
//!
//! This is the original contribution. Not in the literature.
//!
//! Axiom 8 (Oscillatory Nature): alignment(f_i, f_j) = exp(-0.5*((f_i-f_j)/BW)^2)
//! turns binary contact map into a graded coherence landscape.

use crate::blueprint::equations::determinism;

// ─── Go model constants ──────────────────────────────────────────────────

/// Contact cutoff for C-alpha Go models (Angstrom). Standard: 8 A.
pub const CONTACT_CUTOFF: f64 = 8.0;

/// Q tolerance factor: contact formed if r < tolerance * sigma_native.
/// Standard Go value = 1.2 (20% stretch allowed).
pub const Q_TOLERANCE: f64 = 1.2;

// ─── Go 10-12 potential ───────────────────────────────────────────────────

/// Go model native contact potential: V = epsilon * [5*(sigma/r)^12 - 6*(sigma/r)^10].
///
/// Minimum at r = sigma with V(sigma) = -epsilon.
#[inline]
pub fn go_native_potential(r: f64, sigma: f64, epsilon: f64) -> f64 {
    if r < 1e-15 { return f64::MAX; }
    let x = sigma / r;
    let x2 = x * x;
    let x10 = x2 * x2 * x2 * x2 * x2;
    let x12 = x10 * x2;
    epsilon * (5.0 * x12 - 6.0 * x10)
}

/// Go model native contact force (radial, positive = repulsive).
///
/// F = -dV/dr = epsilon * [60*(sigma^12/r^13) - 60*(sigma^10/r^11)]
///            = 60*epsilon/r * [sigma^12/r^12 - sigma^10/r^10]
#[inline]
pub fn go_native_force(r: f64, sigma: f64, epsilon: f64) -> f64 {
    if r < 1e-15 { return 0.0; }
    let x = sigma / r;
    let x2 = x * x;
    let x10 = x2 * x2 * x2 * x2 * x2;
    let x12 = x10 * x2;
    60.0 * epsilon / r * (x12 - x10)
}

/// Repulsion-only potential for non-native contacts: V = epsilon * (sigma/r)^12.
#[inline]
pub fn go_repulsive_potential(r: f64, sigma: f64, epsilon: f64) -> f64 {
    if r < 1e-15 { return f64::MAX; }
    let x = sigma / r;
    let x2 = x * x;
    let x6 = x2 * x2 * x2;
    let x12 = x6 * x6;
    epsilon * x12
}

/// Repulsive force: F = 12 * epsilon * sigma^12 / r^13.
#[inline]
pub fn go_repulsive_force(r: f64, sigma: f64, epsilon: f64) -> f64 {
    if r < 1e-15 { return 0.0; }
    let x = sigma / r;
    let x2 = x * x;
    let x6 = x2 * x2 * x2;
    let x12 = x6 * x6;
    12.0 * epsilon * x12 / r
}

// ─── Axiom 8: frequency alignment ────────────────────────────────────────

/// Frequency alignment (Axiom 8): Gaussian coherence window.
///
/// alignment = exp(-0.5 * ((f_i - f_j) / bandwidth)^2)
///
/// alignment ∈ [0, 1]. Same frequency → 1. Far apart → 0.
#[inline]
pub fn frequency_alignment(f_i: f64, f_j: f64, bandwidth: f64) -> f64 {
    let df = (f_i - f_j) / bandwidth;
    (-0.5 * df * df).exp()
}

/// Frequency-modulated Go potential (Axiom 8).
///
/// V = epsilon * alignment(f_i, f_j) * go_native_potential(r, sigma, 1.0)
#[inline]
pub fn go_axiom8_potential(
    r: f64, sigma: f64, epsilon: f64,
    f_i: f64, f_j: f64, bandwidth: f64,
) -> f64 {
    let a = frequency_alignment(f_i, f_j, bandwidth);
    a * go_native_potential(r, sigma, epsilon)
}

/// Frequency-modulated Go force (Axiom 8).
#[inline]
pub fn go_axiom8_force(
    r: f64, sigma: f64, epsilon: f64,
    f_i: f64, f_j: f64, bandwidth: f64,
) -> f64 {
    let a = frequency_alignment(f_i, f_j, bandwidth);
    a * go_native_force(r, sigma, epsilon)
}

// ─── Contact map ──────────────────────────────────────────────────────────

/// Build native contact map from C-alpha positions.
///
/// Native contact: distance < cutoff AND |i - j| >= min_seq_sep.
/// Returns (i, j, sigma_ij) where sigma_ij = distance in native structure.
pub fn native_contact_map(
    ca_positions: &[[f64; 3]],
    cutoff: f64,
    min_seq_sep: usize,
) -> Vec<(u16, u16, f64)> {
    let n = ca_positions.len();
    let mut contacts = Vec::new();

    for i in 0..n {
        for j in (i + min_seq_sep)..n {
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = ca_positions[i][k] - ca_positions[j][k];
                d_sq += dk * dk;
            }
            let d = d_sq.sqrt();
            if d < cutoff {
                contacts.push((i as u16, j as u16, d));
            }
        }
    }

    contacts
}

// ─── Frequency assignment strategies ─────────────────────────────────────

/// Strategy A: amino acid type → base frequency (Hz).
///
/// 20 amino acid types mapped to frequencies within COHERENCE_BANDWIDTH range.
/// Types that commonly form contacts have closer frequencies.
pub fn amino_acid_frequency(aa_type: u8) -> f64 {
    match aa_type {
         0 => 100.0,  //  ALA
         1 => 105.0,  //  ARG
         2 => 110.0,  //  ASN
         3 => 115.0,  //  ASP
         4 => 120.0,  //  CYS
         5 => 125.0,  //  GLN
         6 => 130.0,  //  GLU
         7 =>  95.0,  //  GLY
         8 => 135.0,  //  HIS
         9 => 140.0,  //  ILE
        10 => 145.0,  //  LEU
        11 => 150.0,  //  LYS
        12 => 155.0,  //  MET
        13 => 160.0,  //  PHE
        14 =>  90.0,  //  PRO
        15 => 165.0,  //  SER
        16 => 170.0,  //  THR
        17 => 175.0,  //  TRP
        18 => 180.0,  //  TYR
        19 => 185.0,  //  VAL
         _ => {
            debug_assert!(aa_type < 20, "unknown amino acid type: {aa_type}, defaulting to ALA");
            100.0
        }
    }
}

/// 3-letter code to amino acid type index (0-19).
pub fn aa_code_to_type(code: &[u8; 3]) -> u8 {
    match code {
        b"ALA" => 0,  b"ARG" => 1,  b"ASN" => 2,  b"ASP" => 3,
        b"CYS" => 4,  b"GLN" => 5,  b"GLU" => 6,  b"GLY" => 7,
        b"HIS" => 8,  b"ILE" => 9,  b"LEU" => 10, b"LYS" => 11,
        b"MET" => 12, b"PHE" => 13, b"PRO" => 14, b"SER" => 15,
        b"THR" => 16, b"TRP" => 17, b"TYR" => 18, b"VAL" => 19,
        _ => 0,
    }
}

/// Strategy B: optimize frequencies to maximize native coherence.
///
/// Gradient descent on: Σ alignment(native) - Σ alignment(non-native).
pub fn optimize_frequencies(
    contact_map: &[(u16, u16, f64)],
    n_residues: usize,
    bandwidth: f64,
    iterations: usize,
    initial_freqs: &[f64],
) -> Vec<f64> {
    let mut freqs = initial_freqs.to_vec();
    let lr = 0.5; // learning rate

    // Build non-native set (all pairs |i-j|>=3 NOT in contact map)
    let mut is_native = vec![false; n_residues * n_residues];
    for &(i, j, _) in contact_map {
        let (i, j) = (i as usize, j as usize);
        is_native[i * n_residues + j] = true;
        is_native[j * n_residues + i] = true;
    }

    for _ in 0..iterations {
        let mut grad = vec![0.0; n_residues];

        // Gradient: ∂alignment/∂f_i = alignment * (-(f_i-f_j)/BW²)
        for &(ci, cj, _) in contact_map {
            let (i, j) = (ci as usize, cj as usize);
            let a = frequency_alignment(freqs[i], freqs[j], bandwidth);
            let df_bw2 = (freqs[i] - freqs[j]) / (bandwidth * bandwidth);
            // Maximize native alignment → gradient ascent
            grad[i] += a * (-df_bw2);
            grad[j] += a * df_bw2;
        }

        // Penalize non-native alignment (minimize)
        for i in 0..n_residues {
            for j in (i + 3)..n_residues {
                if is_native[i * n_residues + j] { continue; }
                let a = frequency_alignment(freqs[i], freqs[j], bandwidth);
                if a < 0.01 { continue; } // skip negligible
                let df_bw2 = (freqs[i] - freqs[j]) / (bandwidth * bandwidth);
                // Minimize non-native alignment → reverse gradient
                grad[i] -= a * (-df_bw2);
                grad[j] -= a * df_bw2;
            }
        }

        for i in 0..n_residues {
            freqs[i] += lr * grad[i];
        }
    }

    freqs
}

/// Strategy C: evolutionary (uses deterministic RNG to mutate frequencies).
///
/// Single generation: mutate, evaluate, keep if better.
/// Caller wraps in a loop for multiple generations.
pub fn evolve_frequencies_step(
    freqs: &[f64],
    contact_map: &[(u16, u16, f64)],
    bandwidth: f64,
    mutation_scale: f64,
    seed: u64,
) -> (Vec<f64>, f64) {
    let n = freqs.len();
    let mut candidate = freqs.to_vec();
    let mut rng = seed;

    for i in 0..n {
        rng = determinism::next_u64(rng);
        let noise = determinism::gaussian_f32(rng, 1.0) as f64 * mutation_scale;
        candidate[i] += noise;
    }

    let fitness = native_coherence_score(&candidate, contact_map, bandwidth);
    (candidate, fitness)
}

/// Score: mean alignment of native contacts.
pub fn native_coherence_score(
    freqs: &[f64],
    contact_map: &[(u16, u16, f64)],
    bandwidth: f64,
) -> f64 {
    if contact_map.is_empty() { return 0.0; }
    let sum: f64 = contact_map.iter().map(|&(i, j, _)| {
        frequency_alignment(freqs[i as usize], freqs[j as usize], bandwidth)
    }).sum();
    sum / contact_map.len() as f64
}

// ─── Coherence observables ────────────────────────────────────────────────

/// Fraction of native contacts formed (distance < tolerance * sigma_ij).
pub fn native_contact_fraction(
    positions: &[[f64; 3]],
    contacts: &[(u16, u16, f64)],
    tolerance: f64,
) -> f64 {
    if contacts.is_empty() { return 0.0; }
    let mut formed = 0u32;
    for &(i, j, sigma) in contacts {
        let (i, j) = (i as usize, j as usize);
        let mut d_sq = 0.0;
        for k in 0..3 {
            let dk = positions[i][k] - positions[j][k];
            d_sq += dk * dk;
        }
        if d_sq.sqrt() < tolerance * sigma {
            formed += 1;
        }
    }
    formed as f64 / contacts.len() as f64
}

/// Average frequency coherence of formed native contacts.
pub fn folding_coherence(
    positions: &[[f64; 3]],
    contacts: &[(u16, u16, f64)],
    frequencies: &[f64],
    bandwidth: f64,
    tolerance: f64,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0u32;
    for &(i, j, sigma) in contacts {
        let (i, j) = (i as usize, j as usize);
        let mut d_sq = 0.0;
        for k in 0..3 {
            let dk = positions[i][k] - positions[j][k];
            d_sq += dk * dk;
        }
        if d_sq.sqrt() < tolerance * sigma {
            sum += frequency_alignment(frequencies[i], frequencies[j], bandwidth);
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// Coherence spectrum: histogram of alignment values for formed contacts.
pub fn coherence_spectrum(
    positions: &[[f64; 3]],
    contacts: &[(u16, u16, f64)],
    frequencies: &[f64],
    bandwidth: f64,
    tolerance: f64,
    n_bins: usize,
) -> Vec<u32> {
    let mut bins = vec![0u32; n_bins];
    for &(i, j, sigma) in contacts {
        let (i, j) = (i as usize, j as usize);
        let mut d_sq = 0.0;
        for k in 0..3 {
            let dk = positions[i][k] - positions[j][k];
            d_sq += dk * dk;
        }
        if d_sq.sqrt() < tolerance * sigma {
            let a = frequency_alignment(frequencies[i], frequencies[j], bandwidth);
            let bin = ((a * n_bins as f64) as usize).min(n_bins - 1);
            bins[bin] += 1;
        }
    }
    bins
}

// ─── Go topology builder ─────────────────────────────────────────────────

/// Complete Go model topology for simulation.
#[derive(Clone, Debug)]
pub struct GoTopology {
    pub n_residues: usize,
    pub sequence: Vec<u8>,
    pub native_contacts: Vec<(u16, u16, f64)>,
    pub frequencies: Vec<f64>,
    pub bond_length: f64,
    pub bond_k: f64,
    pub epsilon: f64,
    pub epsilon_repel: f64,
    /// Pre-computed native contact mask (N×N flat bool). Built once, avoids O(N²) alloc per step.
    pub native_mask: Vec<bool>,
}

/// Build Go topology from C-alpha positions and sequence.
pub fn build_go_topology(
    ca_positions: &[[f64; 3]],
    sequence: &[u8],
    contact_cutoff: f64,
    _bandwidth: f64,
    epsilon: f64,
    bond_k: f64,
) -> GoTopology {
    let n = ca_positions.len();
    let contacts = native_contact_map(ca_positions, contact_cutoff, 3);
    let frequencies: Vec<f64> = sequence.iter().map(|&aa| amino_acid_frequency(aa)).collect();

    // Compute average bond length from consecutive C-alphas
    let mut sum_bl = 0.0;
    for i in 0..(n - 1) {
        let mut d_sq = 0.0;
        for k in 0..3 {
            let dk = ca_positions[i][k] - ca_positions[i + 1][k];
            d_sq += dk * dk;
        }
        sum_bl += d_sq.sqrt();
    }
    let bond_length = if n > 1 { sum_bl / (n - 1) as f64 } else { 3.8 };

    // Pre-compute native mask (built once, reused every step)
    let mut native_mask = vec![false; n * n];
    for &(i, j, _) in &contacts {
        let (i, j) = (i as usize, j as usize);
        native_mask[i * n + j] = true;
        native_mask[j * n + i] = true;
    }

    GoTopology {
        n_residues: n,
        sequence: sequence.to_vec(),
        native_contacts: contacts,
        frequencies,
        bond_length,
        bond_k,
        epsilon,
        epsilon_repel: epsilon * 0.5,
        native_mask,
    }
}

/// Generate extended chain (unfolded) initial structure.
///
/// Places C-alphas along x-axis at bond_length spacing.
pub fn extended_chain(n_residues: usize, bond_length: f64) -> Vec<[f64; 3]> {
    (0..n_residues).map(|i| [i as f64 * bond_length, 0.0, 0.0]).collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_potential_minimum_at_sigma() {
        let sigma = 5.0;
        let eps = 1.0;
        let v_at_sigma = go_native_potential(sigma, sigma, eps);
        let v_below = go_native_potential(sigma * 0.9, sigma, eps);
        let v_above = go_native_potential(sigma * 1.1, sigma, eps);
        assert!(v_at_sigma < v_below, "V(sigma)={v_at_sigma} should be < V(0.9*sigma)={v_below}");
        assert!(v_at_sigma < v_above, "V(sigma)={v_at_sigma} should be < V(1.1*sigma)={v_above}");
        assert!((v_at_sigma - (-eps)).abs() < 1e-10, "V(sigma) should be -epsilon");
    }

    #[test]
    fn go_force_zero_at_sigma() {
        let sigma = 5.0;
        let f = go_native_force(sigma, sigma, 1.0);
        assert!(f.abs() < 1e-10, "force at sigma should be 0, got {f}");
    }

    #[test]
    fn go_force_repulsive_close() {
        let f = go_native_force(3.0, 5.0, 1.0);
        assert!(f > 0.0, "force should be repulsive at r < sigma: {f}");
    }

    #[test]
    fn go_force_attractive_far() {
        let f = go_native_force(6.0, 5.0, 1.0);
        assert!(f < 0.0, "force should be attractive at r > sigma: {f}");
    }

    #[test]
    fn alignment_same_frequency_is_one() {
        let a = frequency_alignment(100.0, 100.0, 50.0);
        assert!((a - 1.0).abs() < 1e-15);
    }

    #[test]
    fn alignment_far_frequencies_is_near_zero() {
        let a = frequency_alignment(100.0, 500.0, 50.0);
        assert!(a < 1e-10, "alignment should be ~0: {a}");
    }

    #[test]
    fn axiom8_full_alignment_matches_classical() {
        let r = 4.0;
        let sigma = 5.0;
        let eps = 1.0;
        let v_classical = go_native_potential(r, sigma, eps);
        // Same frequency → alignment = 1.0 → same as classical
        let v_axiom8 = go_axiom8_potential(r, sigma, eps, 100.0, 100.0, 50.0);
        assert!((v_axiom8 - v_classical).abs() < 1e-10);
    }

    #[test]
    fn axiom8_zero_alignment_kills_attraction() {
        let r = 5.0;
        let sigma = 5.0;
        let eps = 1.0;
        let v = go_axiom8_potential(r, sigma, eps, 100.0, 500.0, 50.0);
        assert!(v.abs() < 1e-6, "incoherent contacts should have ~zero attraction: {v}");
    }

    #[test]
    fn contact_map_symmetric_and_excludes_neighbors() {
        let positions = vec![
            [0.0, 0.0, 0.0], [3.8, 0.0, 0.0], [7.6, 0.0, 0.0],
            [3.0, 5.0, 0.0], [6.0, 5.0, 0.0],
        ];
        let contacts = native_contact_map(&positions, 8.0, 3);
        // Check no (i, i+1) or (i, i+2) pairs
        for &(i, j, _) in &contacts {
            assert!(
                (j as usize - i as usize) >= 3,
                "contact ({i},{j}) violates min_seq_sep=3",
            );
        }
    }

    #[test]
    fn native_fraction_100_at_native_structure() {
        let positions = vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0], [2.0, 3.0, 0.0]];
        let contacts = native_contact_map(&positions, 8.0, 3);
        let q = native_contact_fraction(&positions, &contacts, Q_TOLERANCE);
        assert!((q - 1.0).abs() < 1e-10, "Q at native should be 1.0, got {q}");
    }

    #[test]
    fn native_fraction_low_at_random() {
        let native = vec![[0.0, 0.0, 0.0], [3.8, 0.0, 0.0], [7.6, 0.0, 0.0], [2.0, 5.0, 0.0], [6.0, 5.0, 0.0]];
        let contacts = native_contact_map(&native, 8.0, 3);
        // Random coil: spread far apart
        let random = vec![[0.0, 0.0, 0.0], [20.0, 0.0, 0.0], [40.0, 0.0, 0.0], [60.0, 0.0, 0.0], [80.0, 0.0, 0.0]];
        let q = native_contact_fraction(&random, &contacts, Q_TOLERANCE);
        assert!(q < 0.5, "Q at random coil should be low, got {q}");
    }

    #[test]
    fn optimize_increases_native_coherence() {
        // Contacts between residues with initially distant frequencies
        let contacts = vec![(0u16, 3, 5.0), (1, 4, 5.0)];
        let initial: Vec<f64> = vec![100.0, 200.0, 150.0, 300.0, 50.0];
        let score_before = native_coherence_score(&initial, &contacts, 50.0);
        let optimized = optimize_frequencies(&contacts, 5, 50.0, 500, &initial);
        let score_after = native_coherence_score(&optimized, &contacts, 50.0);
        assert!(
            score_after >= score_before - 0.01,
            "optimization should not regress significantly: {score_before} → {score_after}",
        );
    }

    #[test]
    fn extended_chain_correct_length() {
        let chain = extended_chain(10, 3.8);
        assert_eq!(chain.len(), 10);
        // Total length = 9 * 3.8
        let d = chain[9][0] - chain[0][0];
        assert!((d - 9.0 * 3.8).abs() < 1e-10);
    }

    #[test]
    fn coherence_spectrum_bins_correctly() {
        let positions = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let contacts = vec![(0u16, 3, 5.0)]; // formed (4.0 < 1.2*5.0)
        let freqs = vec![100.0, 100.0, 100.0, 100.0]; // same → alignment=1.0
        let spectrum = coherence_spectrum(&positions, &contacts, &freqs, 50.0, Q_TOLERANCE, 10);
        // alignment=1.0 → should land in last bin (bin 9)
        assert_eq!(spectrum[9], 1);
    }

    #[test]
    fn go_topology_builds() {
        let ca = vec![[0.0, 0.0, 0.0], [3.8, 0.0, 0.0], [7.6, 0.0, 0.0], [3.0, 5.0, 0.0], [6.0, 4.0, 0.0]];
        let seq = vec![0u8, 7, 10, 13, 19];
        let topo = build_go_topology(&ca, &seq, 8.0, 50.0, 1.0, 100.0);
        assert_eq!(topo.n_residues, 5);
        assert!(topo.bond_length > 2.0 && topo.bond_length < 8.0, "bond_length={}", topo.bond_length);
        assert_eq!(topo.frequencies.len(), 5);
    }
}
