//! Bridge S3 → S4 — organismo a proteínas plegándose.
//! Bridge S3 → S4 — organism to folding proteins.
//!
//! CT-3 / ADR-036 §D4. Orquesta:
//! 1) Inferencia axiomática del proteoma (pure math en `proteome_inference`).
//! 2) Generación de "native fold" heurística (α-hélice) para bootstrap del Go model.
//! 3) REMD en paralelo usando la infraestructura existente (`batch::systems::remd`).

use crate::batch::systems::remd::{self, RemdConfig};
use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;
use crate::blueprint::equations::go_model::{self, CONTACT_CUTOFF};
use crate::blueprint::equations::proteome_inference::{self, ProteinSpec};

// ─── Native scaffold ────────────────────────────────────────────────────────

/// Longitud de bond C-alpha estándar (Å).
/// Standard C-alpha bond length (Å).
pub const CA_BOND_LENGTH: f64 = 3.8;

/// Genera coordenadas de una α-hélice canónica (3.6 residuos/turn, rise 1.5 Å).
/// Suficiente para bootstrap del Go model con contactos no-triviales.
/// Generates canonical α-helix coordinates. Bootstraps Go model with real contacts.
pub fn helix_positions(n_residues: usize) -> Vec<[f64; 3]> {
    const RADIUS: f64 = 2.3;
    const RISE_PER_RES: f64 = 1.5;
    const RES_PER_TURN: f64 = 3.6;
    (0..n_residues)
        .map(|i| {
            let theta = i as f64 * std::f64::consts::TAU / RES_PER_TURN;
            [RADIUS * theta.cos(), RADIUS * theta.sin(), i as f64 * RISE_PER_RES]
        })
        .collect()
}

// ─── Folding result ─────────────────────────────────────────────────────────

/// Resultado compacto del plegamiento de una proteína.
/// Compact folding result per protein.
#[derive(Clone, Debug)]
pub struct FoldingResult {
    pub n_residues: usize,
    pub best_q: f64,
    pub best_coherence: f64,
    pub min_rmsd: f64,
    pub qe_budget: f64,
}

// ─── Folding ────────────────────────────────────────────────────────────────

/// Config por defecto para fold rápido de una proteína inferida.
/// Default config for quick fold of an inferred protein.
///
/// Balanceado para validación funcional (no producción): N pocas replicas,
/// pocos swaps. Aumentar en binarios dedicados.
pub fn default_fold_config(seed: u64) -> RemdConfig {
    RemdConfig {
        n_replicas: 4,
        t_min: 0.5,
        t_max: 1.2,
        steps_per_swap: 40,
        total_swaps: 20,
        dt: 0.005,
        gamma: 0.5,
        seed,
        epsilon: 1.0,
        epsilon_repel: 0.5,
        bond_k: 100.0,
    }
}

/// Pliega una proteína inferida: construye topología Go desde α-hélice nativa,
/// corre REMD desde cadena extendida. Determinista por `spec` + `config.seed`.
/// Folds an inferred protein deterministically.
pub fn fold_protein(spec: &ProteinSpec, bandwidth: f64, config: &RemdConfig) -> FoldingResult {
    let native = helix_positions(spec.n_residues);
    let topo = go_model::build_go_topology(
        &native,
        &spec.sequence,
        CONTACT_CUTOFF,
        bandwidth,
        config.epsilon,
        config.bond_k,
    );
    let initial = go_model::extended_chain(spec.n_residues, topo.bond_length);
    let result = remd::run_remd(config, &topo, &native, &initial, bandwidth);
    FoldingResult {
        n_residues: spec.n_residues,
        best_q: result.best_q,
        best_coherence: result.best_coherence,
        min_rmsd: result.min_rmsd,
        qe_budget: spec.qe_budget,
    }
}

/// Pliega un proteoma completo de forma secuencial.
/// Folds a full proteome sequentially.
pub fn fold_proteome(proteome: &[ProteinSpec], seed: u64) -> Vec<FoldingResult> {
    let bandwidth = COHERENCE_BANDWIDTH as f64;
    proteome
        .iter()
        .enumerate()
        .map(|(i, spec)| {
            let cfg = default_fold_config(seed.wrapping_add(i as u64).wrapping_mul(0x9E37_79B1));
            fold_protein(spec, bandwidth, &cfg)
        })
        .collect()
}

/// Health score agregado de un proteoma plegado (CT-3 §E3).
/// Aggregated health score of a folded proteome.
pub fn proteome_health(results: &[FoldingResult]) -> f64 {
    let q: Vec<f64> = results.iter().map(|r| r.best_q).collect();
    let c: Vec<f64> = results.iter().map(|r| r.best_coherence).collect();
    let b: Vec<f64> = results.iter().map(|r| r.qe_budget).collect();
    proteome_inference::aggregate_health_score(&q, &c, &b)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::proteome_inference::infer_proteome;

    #[test]
    fn helix_has_expected_geometry() {
        let h = helix_positions(10);
        assert_eq!(h.len(), 10);
        // z monotone crecimiento por rise/residue.
        for i in 0..9 { assert!(h[i + 1][2] > h[i][2]); }
    }

    #[test]
    fn helix_consecutive_spacing_reasonable() {
        let h = helix_positions(20);
        for i in 0..19 {
            let dx = h[i + 1][0] - h[i][0];
            let dy = h[i + 1][1] - h[i][1];
            let dz = h[i + 1][2] - h[i][2];
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            // Rango razonable cerca del bond C-alpha típico (3.8 Å).
            assert!((3.0..=5.0).contains(&d), "bond {i}-{i}+1 = {d} fuera de rango");
        }
    }

    #[test]
    fn fold_protein_returns_non_negative_q() {
        let spec = ProteinSpec {
            n_residues: 24,
            sequence: vec![0; 24],
            qe_budget: 100.0,
        };
        let mut cfg = default_fold_config(42);
        cfg.total_swaps = 5;
        cfg.steps_per_swap = 10;
        let r = fold_protein(&spec, COHERENCE_BANDWIDTH as f64, &cfg);
        assert_eq!(r.n_residues, 24);
        assert!(r.best_q >= 0.0 && r.best_q <= 1.0);
        assert!(r.best_coherence >= 0.0);
    }

    #[test]
    fn fold_proteome_from_organism_end_to_end() {
        let proteome = infer_proteome(1000.0, 110.0, 50, 42);
        assert!(!proteome.is_empty());
        // Sanity: pool invariant at inference stage.
        let sum: f64 = proteome.iter().map(|p| p.qe_budget).sum();
        assert!(sum < 1000.0);
        // Full fold pipeline runs — use reduced config for speed.
        let bandwidth = COHERENCE_BANDWIDTH as f64;
        let results: Vec<_> = proteome
            .iter()
            .map(|spec| {
                let mut cfg = default_fold_config(7);
                cfg.total_swaps = 3;
                cfg.steps_per_swap = 8;
                fold_protein(spec, bandwidth, &cfg)
            })
            .collect();
        assert_eq!(results.len(), proteome.len());
        let health = proteome_health(&results);
        assert!(health >= 0.0, "health must be non-negative");
    }

    #[test]
    fn health_zero_for_empty_proteome() {
        assert_eq!(proteome_health(&[]), 0.0);
    }
}
