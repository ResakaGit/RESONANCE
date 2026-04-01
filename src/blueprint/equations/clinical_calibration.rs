//! Calibración clínica — mapeo de unidades abstractas a unidades biológicas.
//! Clinical calibration — mapping abstract units to biological units.
//!
//! Converts Resonance simulation output (qe, Hz, generations) to clinical
//! units (nM, days, cell count) using published data from cited references.
//!
//! Three calibration constants, one calibration profile per tumor type.
//! All source data from open-access papers already in references.bib.
//!
//! Pure functions. No ECS. No side effects.

use crate::blueprint::equations::derived_thresholds::{COHERENCE_BANDWIDTH, DISSIPATION_SOLID};

// ─── Calibration Profile ────────────────────────────────────────────────────

/// Perfil de calibración para un tipo tumoral específico.
/// Calibration profile for a specific tumor type.
///
/// Three numbers bridge abstract simulation to clinical reality:
/// - time_scale: days per generation (tumor doubling time)
/// - concentration_scale: nM per unit concentration (drug IC50)
/// - population_scale: real cells per simulated entity
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationProfile {
    /// Nombre del perfil.
    /// Profile name.
    pub name: &'static str,
    /// Días por generación. Source: tumor doubling time.
    /// Days per generation. Source: tumor doubling time.
    pub days_per_generation: f32,
    /// nM por unidad de concentración (= IC50 del drug de referencia).
    /// nM per concentration unit (= IC50 of reference drug).
    pub nm_per_concentration: f32,
    /// Células reales por entidad simulada.
    /// Real cells per simulated entity.
    pub cells_per_entity: f32,
    /// Tasa de mutación por gen por división. Source: published.
    /// Mutation rate per gene per division. Source: published.
    pub mutation_rate: f32,
}

// ─── Published Calibration Profiles ─────────────────────────────────────────

/// CML (Leucemia mieloide crónica). Source: Bozic et al. 2013 (eLife).
/// CML (Chronic Myeloid Leukemia). Source: Bozic et al. 2013 (eLife).
///
/// - Doubling time: 4 days (Bozic 2013, Table 1)
/// - IC50 imatinib: 260 nM (Druker et al. 2001, NEJM)
/// - Tumor at detection: ~10^9 cells (Bozic 2013)
/// - Mutation rate: ~10^-9 per gene per division (Bozic 2013)
pub const CML_IMATINIB: CalibrationProfile = CalibrationProfile {
    name: "CML (imatinib)",
    days_per_generation: 4.0,
    nm_per_concentration: 260.0,
    cells_per_entity: 1e9 / 128.0, // ~7.8M cells per entity
    mutation_rate: 1e-9,
};

/// Cáncer de próstata metastásico. Source: Gatenby et al. 2009 (Cancer Research).
/// Metastatic prostate cancer. Source: Gatenby et al. 2009 (Cancer Research).
///
/// - PSA doubling time: 30 days (Gatenby 2009, adaptive therapy trial)
/// - IC50 abiraterone: 5.1 nM (Li et al. 2015)
/// - Tumor cells at metastasis: ~10^10
/// - Mutation rate: ~5×10^-9 (prostate, higher genomic instability)
pub const PROSTATE_ABIRATERONE: CalibrationProfile = CalibrationProfile {
    name: "Prostate (abiraterone)",
    days_per_generation: 30.0,
    nm_per_concentration: 5.1,
    cells_per_entity: 1e10 / 128.0, // ~78M cells per entity
    mutation_rate: 5e-9,
};

/// NSCLC (carcinoma de pulmón). Erlotinib.
/// NSCLC (non-small cell lung cancer). Erlotinib.
///
/// - Doubling time: 7 days (typical NSCLC)
/// - IC50 erlotinib: 20 nM (EGFR mutant)
/// - Tumor at diagnosis: ~10^9
/// - Mutation rate: ~2×10^-9
pub const NSCLC_ERLOTINIB: CalibrationProfile = CalibrationProfile {
    name: "NSCLC (erlotinib)",
    days_per_generation: 7.0,
    nm_per_concentration: 20.0,
    cells_per_entity: 1e9 / 128.0,
    mutation_rate: 2e-9,
};

// ─── Conversion Functions (pure) ────────────────────────────────────────────

/// Convierte generación simulada a días clínicos.
/// Convert simulated generation to clinical days.
#[inline]
pub fn generation_to_days(generation: u32, profile: &CalibrationProfile) -> f32 {
    generation as f32 * profile.days_per_generation
}

/// Convierte concentración normalizada [0,1] a nM.
/// Convert normalized concentration [0,1] to nM.
#[inline]
pub fn concentration_to_nm(concentration: f32, profile: &CalibrationProfile) -> f32 {
    concentration * profile.nm_per_concentration
}

/// Convierte nM a concentración normalizada.
/// Convert nM to normalized concentration.
#[inline]
pub fn nm_to_concentration(nm: f32, profile: &CalibrationProfile) -> f32 {
    nm / profile.nm_per_concentration
}

/// Convierte alive_count simulado a número real de células.
/// Convert simulated alive_count to real cell number.
#[inline]
pub fn entities_to_cells(entity_count: f32, profile: &CalibrationProfile) -> f64 {
    entity_count as f64 * profile.cells_per_entity as f64
}

/// Convierte frecuencia simulada a mutation burden estimado.
/// Convert simulated frequency to estimated mutation burden.
///
/// Mapping: frequency_spread × mutation_rate × genes / bandwidth.
/// Entities with frequency far from tumor center have accumulated
/// more mutations (higher mutation burden = different frequency).
#[inline]
pub fn frequency_to_mutation_burden(
    freq_delta: f32,
    profile: &CalibrationProfile,
    n_genes: u32,
) -> f64 {
    let normalized = (freq_delta / COHERENCE_BANDWIDTH).abs();
    normalized as f64 * profile.mutation_rate as f64 * n_genes as f64
}

/// Genera protocolo clínico desde decisión del controlador.
/// Generate clinical protocol from controller decision.
///
/// Translates abstract (frequency, concentration) pairs to
/// (drug_name_hint, dose_nM, timing_days).
pub fn calibrate_protocol(
    decision_drugs: &[(f32, f32)],
    generation: u32,
    profile: &CalibrationProfile,
) -> Vec<CalibratedDrug> {
    decision_drugs.iter().map(|&(freq, conc)| {
        CalibratedDrug {
            target_frequency: freq,
            dose_nm: concentration_to_nm(conc, profile),
            start_day: generation_to_days(generation, profile),
            concentration_normalized: conc,
        }
    }).collect()
}

/// Fármaco calibrado con unidades clínicas.
/// Calibrated drug with clinical units.
#[derive(Clone, Debug, PartialEq)]
pub struct CalibratedDrug {
    pub target_frequency: f32,
    pub dose_nm: f32,
    pub start_day: f32,
    pub concentration_normalized: f32,
}

/// Calibra un snapshot completo a unidades clínicas.
/// Calibrate a full snapshot to clinical units.
pub fn calibrate_snapshot(
    alive_mean: f32,
    efficiency: f32,
    generation: u32,
    profile: &CalibrationProfile,
) -> CalibratedSnapshot {
    CalibratedSnapshot {
        day: generation_to_days(generation, profile),
        estimated_cells: entities_to_cells(alive_mean, profile),
        metabolic_efficiency: efficiency,
        doubling_time_days: if efficiency > DISSIPATION_SOLID {
            profile.days_per_generation / efficiency
        } else {
            f32::INFINITY // No growth
        },
    }
}

/// Snapshot calibrado con unidades clínicas.
/// Calibrated snapshot with clinical units.
#[derive(Clone, Debug)]
pub struct CalibratedSnapshot {
    pub day: f32,
    pub estimated_cells: f64,
    pub metabolic_efficiency: f32,
    pub doubling_time_days: f32,
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Profile constants match published data ──────────────────────────

    #[test]
    fn cml_doubling_time_is_4_days() {
        // Bozic 2013, Table 1: CML doubling time ≈ 4 days
        assert!((CML_IMATINIB.days_per_generation - 4.0).abs() < 1e-5);
    }

    #[test]
    fn cml_imatinib_ic50_is_260nm() {
        // Druker et al. 2001: imatinib IC50 ≈ 260 nM
        assert!((CML_IMATINIB.nm_per_concentration - 260.0).abs() < 1e-3);
    }

    #[test]
    fn prostate_doubling_time_is_30_days() {
        // Gatenby 2009: PSA doubling time ≈ 30 days
        assert!((PROSTATE_ABIRATERONE.days_per_generation - 30.0).abs() < 1e-5);
    }

    // ── Conversion correctness ──────────────────────────────────────────

    #[test]
    fn generation_10_cml_is_40_days() {
        let days = generation_to_days(10, &CML_IMATINIB);
        assert!((days - 40.0).abs() < 1e-5);
    }

    #[test]
    fn concentration_half_is_half_ic50() {
        // conc=0.5 → 130 nM for imatinib (half IC50)
        let nm = concentration_to_nm(0.5, &CML_IMATINIB);
        assert!((nm - 130.0).abs() < 1e-3);
    }

    #[test]
    fn roundtrip_nm_concentration() {
        let nm = 104.0; // Our protocol: 0.40 × 260
        let conc = nm_to_concentration(nm, &CML_IMATINIB);
        let back = concentration_to_nm(conc, &CML_IMATINIB);
        assert!((back - nm).abs() < 1e-3);
    }

    #[test]
    fn entities_128_is_billion_cells_cml() {
        // 128 entities × 7.8M cells/entity ≈ 10^9
        let cells = entities_to_cells(128.0, &CML_IMATINIB);
        assert!(cells > 9e8 && cells < 1.1e9, "cells={cells}");
    }

    // ── Protocol calibration ────────────────────────────────────────────

    #[test]
    fn protocol_399hz_040_cml() {
        // Our adaptive result: 399 Hz @ 0.40, gen 3
        let drugs = calibrate_protocol(&[(399.0, 0.40)], 3, &CML_IMATINIB);
        assert_eq!(drugs.len(), 1);
        // 0.40 × 260 nM = 104 nM
        assert!((drugs[0].dose_nm - 104.0).abs() < 1e-2, "dose={}", drugs[0].dose_nm);
        // gen 3 × 4 days = day 12
        assert!((drugs[0].start_day - 12.0).abs() < 1e-5, "day={}", drugs[0].start_day);
    }

    #[test]
    fn protocol_combo_cml() {
        // Bozic combo: drug A 400Hz@0.8 + drug B 300Hz@0.8
        let drugs = calibrate_protocol(&[(400.0, 0.8), (300.0, 0.8)], 5, &CML_IMATINIB);
        assert_eq!(drugs.len(), 2);
        // Each: 0.8 × 260 = 208 nM
        assert!((drugs[0].dose_nm - 208.0).abs() < 1e-2);
        assert!((drugs[1].dose_nm - 208.0).abs() < 1e-2);
        // gen 5 × 4 = day 20
        assert!((drugs[0].start_day - 20.0).abs() < 1e-5);
    }

    // ── Snapshot calibration ────────────────────────────────────────────

    #[test]
    fn snapshot_cml_gen10() {
        let snap = calibrate_snapshot(128.0, 0.536, 10, &CML_IMATINIB);
        // Day 40
        assert!((snap.day - 40.0).abs() < 1e-5);
        // ~10^9 cells
        assert!(snap.estimated_cells > 9e8);
        // Doubling time extended: 4 / 0.536 ≈ 7.5 days (slower growth)
        assert!(snap.doubling_time_days > 7.0 && snap.doubling_time_days < 8.0,
            "dt={}", snap.doubling_time_days);
    }

    #[test]
    fn snapshot_prostate_gen10() {
        let snap = calibrate_snapshot(128.0, 0.536, 10, &PROSTATE_ABIRATERONE);
        // Day 300
        assert!((snap.day - 300.0).abs() < 1e-3);
        // Doubling time: 30 / 0.536 ≈ 56 days
        assert!(snap.doubling_time_days > 50.0 && snap.doubling_time_days < 60.0,
            "dt={}", snap.doubling_time_days);
    }

    #[test]
    fn zero_efficiency_infinite_doubling() {
        let snap = calibrate_snapshot(128.0, 0.0, 10, &CML_IMATINIB);
        assert!(snap.doubling_time_days.is_infinite());
    }

    // ── Mutation burden ─────────────────────────────────────────────────

    #[test]
    fn same_frequency_zero_burden() {
        let b = frequency_to_mutation_burden(0.0, &CML_IMATINIB, 20000);
        assert_eq!(b, 0.0);
    }

    #[test]
    fn distant_frequency_higher_burden() {
        let near = frequency_to_mutation_burden(10.0, &CML_IMATINIB, 20000);
        let far = frequency_to_mutation_burden(100.0, &CML_IMATINIB, 20000);
        assert!(far > near, "far={far}, near={near}");
    }
}
