//! Inferencia de proteoma desde observables de un organismo. Matemática pura.
//! Proteome inference from organism observables. Pure math.
//!
//! CT-3 / ADR-036 §D4 (S3→S4). Genera especificaciones de proteínas derivadas de
//! qe, freq y edad del organismo. Kleiber scaling + Axiom 8 + Pool Invariant.

use crate::blueprint::equations::derived_thresholds::{
    COHERENCE_BANDWIDTH, DISSIPATION_SOLID, KLEIBER_EXPONENT,
};
use crate::blueprint::equations::determinism;
use crate::blueprint::equations::go_model::amino_acid_frequency;

/// Espec. de una proteína inferida: tamaño, secuencia de tipos AA, energía asignada.
/// Inferred protein spec: size, AA-type sequence, energy budget.
#[derive(Clone, Debug, PartialEq)]
pub struct ProteinSpec {
    pub n_residues: usize,
    pub sequence: Vec<u8>,
    pub qe_budget: f64,
}

/// Rango de tamaños válidos para proteínas inferidas.
/// Valid size range for inferred proteins.
pub const N_RESIDUES_MIN: usize = 20;
pub const N_RESIDUES_MAX: usize = 100;

/// Número de proteínas inferido por Kleiber: N = clamp((qe^0.75 / SCALE), 1, 20).
/// Protein count via Kleiber scaling.
///
/// `SCALE = 30` calibrado para que qe=1000 → ~6 proteínas (spec CT-3 §1).
pub fn kleiber_protein_count(organism_qe: f64) -> usize {
    const KLEIBER_PROTEIN_SCALE: f64 = 30.0;
    if organism_qe <= 0.0 { return 0; }
    let raw = organism_qe.powf(KLEIBER_EXPONENT as f64) / KLEIBER_PROTEIN_SCALE;
    (raw.round() as usize).clamp(1, 20)
}

/// Tamaño de una proteína en residuos: 20 + 5·log10(qe), clamp [20,100].
/// Protein size in residues.
pub fn residue_count(protein_qe: f64) -> usize {
    if protein_qe <= 1.0 { return N_RESIDUES_MIN; }
    let raw = N_RESIDUES_MIN as f64 + 5.0 * protein_qe.log10();
    (raw.round() as usize).clamp(N_RESIDUES_MIN, N_RESIDUES_MAX)
}

/// Tipo de aminoácido (0..=19) cuya frecuencia nominal es más cercana a `target_freq`.
/// Amino acid type (0..=19) whose nominal frequency is closest to `target_freq`.
pub fn aa_type_from_target_freq(target_freq: f64) -> u8 {
    let mut best = 0u8;
    let mut best_diff = f64::MAX;
    for aa in 0..20u8 {
        let f = amino_acid_frequency(aa);
        let diff = (f - target_freq).abs();
        if diff < best_diff { best_diff = diff; best = aa; }
    }
    best
}

/// Secuencia de N residuos cuyo perfil de frecuencias resuena con `organism_freq`.
/// Cada posición i: `target_i ~ N(organism_freq, bandwidth)`, AA más cercano gana.
/// AA sequence whose frequency profile resonates with `organism_freq` (Axiom 8).
pub fn sequence_from_organism_freq(
    organism_freq: f64,
    n_residues: usize,
    bandwidth: f64,
    seed: u64,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(n_residues);
    let mut rng = seed.wrapping_add(0x6A09E667F3BCC908);
    for _ in 0..n_residues {
        rng = determinism::next_u64(rng);
        let jitter = determinism::gaussian_f32(rng, bandwidth as f32) as f64;
        out.push(aa_type_from_target_freq(organism_freq + jitter));
    }
    out
}

/// Inferir proteoma completo respetando Pool Invariant (Ax 2) + Dissipation (Ax 4).
/// Infers full proteome honoring Pool Invariant + Dissipation.
///
/// `sum(protein.qe_budget) = organism_qe × (1 - DISSIPATION_SOLID) < organism_qe`.
/// Edad del organismo modula `seed` (proteome evoluciona con el tiempo).
pub fn infer_proteome(
    organism_qe: f64,
    organism_freq: f64,
    organism_age: u64,
    seed: u64,
) -> Vec<ProteinSpec> {
    let n = kleiber_protein_count(organism_qe);
    if n == 0 { return Vec::new(); }

    let available = organism_qe * (1.0 - DISSIPATION_SOLID as f64);
    let qe_each = available / n as f64;
    let bw = COHERENCE_BANDWIDTH as f64;
    let age_mix = organism_age.wrapping_mul(0x9E3779B97F4A7C15);
    let base_seed = seed ^ age_mix;

    (0..n)
        .map(|i| {
            let sub_seed = determinism::next_u64(base_seed.wrapping_add(i as u64));
            let n_residues = residue_count(qe_each);
            let sequence = sequence_from_organism_freq(organism_freq, n_residues, bw, sub_seed);
            ProteinSpec { n_residues, sequence, qe_budget: qe_each }
        })
        .collect()
}

/// Score de salud del proteoma: media ponderada de `(Q × coherence)` por qe_budget.
/// Proteome health score: qe-weighted mean of `(Q × coherence)`.
///
/// Usado en zoom-out S4→S3: afecta viabilidad del organismo.
pub fn aggregate_health_score(q_values: &[f64], coherence: &[f64], budgets: &[f64]) -> f64 {
    debug_assert_eq!(q_values.len(), coherence.len());
    debug_assert_eq!(q_values.len(), budgets.len());
    let total: f64 = budgets.iter().sum();
    if total <= 0.0 || q_values.is_empty() { return 0.0; }
    q_values
        .iter()
        .zip(coherence)
        .zip(budgets)
        .map(|((q, c), b)| q * c * b)
        .sum::<f64>()
        / total
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kleiber_zero_qe_no_proteins() {
        assert_eq!(kleiber_protein_count(0.0), 0);
    }

    #[test]
    fn kleiber_spec_range_for_qe_1000() {
        let n = kleiber_protein_count(1000.0);
        assert!((3..=8).contains(&n), "qe=1000 → expected 3..=8 proteins, got {n}");
    }

    #[test]
    fn kleiber_monotone_with_qe() {
        let small = kleiber_protein_count(100.0);
        let large = kleiber_protein_count(10_000.0);
        assert!(large >= small);
    }

    #[test]
    fn residue_count_within_bounds() {
        for qe in [1.0, 50.0, 500.0, 10_000.0, 1e9] {
            let n = residue_count(qe);
            assert!((N_RESIDUES_MIN..=N_RESIDUES_MAX).contains(&n));
        }
    }

    #[test]
    fn aa_type_matches_nominal_frequency() {
        // amino_acid_frequency(0) = 100.0 (ALA). Target 100.0 must map to 0.
        assert_eq!(aa_type_from_target_freq(100.0), 0);
        // 185.0 = VAL (19)
        assert_eq!(aa_type_from_target_freq(185.0), 19);
        // 90.0 = PRO (14)
        assert_eq!(aa_type_from_target_freq(90.0), 14);
    }

    #[test]
    fn sequence_deterministic_with_seed() {
        let a = sequence_from_organism_freq(120.0, 30, 50.0, 42);
        let b = sequence_from_organism_freq(120.0, 30, 50.0, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn sequence_differs_with_seed() {
        let a = sequence_from_organism_freq(120.0, 30, 50.0, 1);
        let b = sequence_from_organism_freq(120.0, 30, 50.0, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn sequence_aa_types_valid() {
        let s = sequence_from_organism_freq(120.0, 50, 50.0, 7);
        for aa in &s { assert!(*aa < 20, "invalid AA type {aa}"); }
    }

    #[test]
    fn proteome_respects_pool_invariant() {
        let organism_qe = 1000.0;
        let proteome = infer_proteome(organism_qe, 120.0, 100, 42);
        let sum: f64 = proteome.iter().map(|p| p.qe_budget).sum();
        assert!(sum < organism_qe, "sum {sum} >= organism {organism_qe}");
        // Dissipation accounts for at most DISSIPATION_SOLID
        let min_expected = organism_qe * (1.0 - DISSIPATION_SOLID as f64 - 1e-9);
        assert!(sum >= min_expected, "sum {sum} < min_expected {min_expected}");
    }

    #[test]
    fn proteome_deterministic_with_seed() {
        let a = infer_proteome(500.0, 110.0, 50, 42);
        let b = infer_proteome(500.0, 110.0, 50, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn proteome_age_changes_result() {
        let a = infer_proteome(500.0, 110.0, 10, 42);
        let b = infer_proteome(500.0, 110.0, 1000, 42);
        assert_ne!(a, b, "same organism at different ages should evolve sequences");
    }

    #[test]
    fn proteome_zero_qe_is_empty() {
        assert!(infer_proteome(0.0, 100.0, 0, 1).is_empty());
    }

    #[test]
    fn health_score_bounded_by_inputs() {
        // Q and coherence in [0,1]; health score in [0,1] for normalized budgets.
        let q = vec![0.5, 0.8, 0.3];
        let c = vec![0.6, 0.9, 0.4];
        let b = vec![100.0, 200.0, 50.0];
        let score = aggregate_health_score(&q, &c, &b);
        assert!(score >= 0.0 && score <= 1.0, "score {score} out of [0,1]");
    }

    #[test]
    fn health_score_empty_is_zero() {
        assert_eq!(aggregate_health_score(&[], &[], &[]), 0.0);
    }
}
