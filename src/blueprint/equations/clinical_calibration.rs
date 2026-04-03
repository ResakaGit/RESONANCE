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

/// Mast cell tumor canino (caso Rosie). Sources: press reports + vet oncology literature.
/// Canine mast cell tumor (Rosie case). Sources: press reports + vet oncology literature.
///
/// DISCLAIMER: Calibrated from press reports (Japan Times, Fortune, March 2026)
/// and published veterinary oncology data. NOT from peer-reviewed trial data.
/// This profile is for SIMULATION ONLY — not veterinary advice.
///
/// - Doubling time: 21 days (intermediate-grade canine mast cell, London & Seguin 2003)
/// - IC50 toceranib (Palladia): 40 nM (KIT-mutant mast cell, London et al. 2009)
///   Used as proxy for mRNA vaccine potency (no IC50 published for Rosie's vaccine)
/// - Tumor at detection: ~10^8 cells (tennis-ball sized, ~2cm diameter solid tumor)
/// - Mutation rate: ~3×10^-9 (canine, similar to human somatic rate)
/// - KIT mutation prevalence: ~30% of canine mast cell tumors (heterogeneous)
pub const CANINE_MAST_CELL: CalibrationProfile = CalibrationProfile {
    name: "Canine mast cell (toceranib proxy)",
    days_per_generation: 21.0,
    nm_per_concentration: 40.0,
    cells_per_entity: 1e8 / 128.0, // ~781K cells per entity
    mutation_rate: 3e-9,
};

/// Predicción del caso Rosie: partial response esperado.
/// Rosie case prediction: expected partial response.
///
/// Models the observed outcome: ~75% main tumor reduction + resistant subpopulation.
/// - Responsive fraction: ~70% cells at target frequency (KIT+)
/// - Resistant fraction: ~30% cells at different frequency (KIT-)
/// - Treatment: single-target vaccine (equivalent to mono pathway inhibitor)
/// - Expected: partial response + resistance in KIT- subpopulation
#[derive(Clone, Copy, Debug)]
pub struct RosieCasePrediction {
    /// Fracción responsiva (KIT+, ~70% en mast cell con mutación KIT).
    /// Responsive fraction (KIT+, ~70% in KIT-mutant mast cell).
    pub responsive_fraction: f32,
    /// Fracción resistente (KIT-, ~30%).
    /// Resistant fraction (KIT-, ~30%).
    pub resistant_fraction: f32,
    /// Días hasta respuesta parcial observada.
    /// Days to observed partial response.
    pub days_to_partial_response: f32,
    /// Reducción observada del tumor principal.
    /// Observed main tumor reduction.
    pub observed_reduction: f32,
}

/// Datos publicados del caso Rosie. Fuente: press reports marzo 2026.
/// Published Rosie case data. Source: press reports March 2026.
pub const ROSIE_OBSERVED: RosieCasePrediction = RosieCasePrediction {
    responsive_fraction: 0.70,      // ~70% KIT+ (London & Seguin 2003)
    resistant_fraction: 0.30,       // ~30% KIT- (heterogeneous)
    days_to_partial_response: 42.0, // ~6 weeks (reported)
    observed_reduction: 0.75,       // 75% tumor reduction (reported)
};

/// Estima generaciones para respuesta parcial dado un perfil.
/// Estimate generations to partial response given a profile.
#[inline]
pub fn days_to_generations(days: f32, profile: &CalibrationProfile) -> u32 {
    (days / profile.days_per_generation).ceil() as u32
}

/// Predice fracción responsiva/resistente como entity counts.
/// Predict responsive/resistant fraction as entity counts.
#[inline]
pub fn fraction_to_entity_counts(total_entities: u8, responsive_fraction: f32) -> (u8, u8) {
    let responsive = (total_entities as f32 * responsive_fraction).round() as u8;
    let resistant = total_entities.saturating_sub(responsive);
    (responsive, resistant)
}

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
    decision_drugs
        .iter()
        .map(|&(freq, conc)| CalibratedDrug {
            target_frequency: freq,
            dose_nm: concentration_to_nm(conc, profile),
            start_day: generation_to_days(generation, profile),
            concentration_normalized: conc,
        })
        .collect()
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
        assert!(
            (drugs[0].dose_nm - 104.0).abs() < 1e-2,
            "dose={}",
            drugs[0].dose_nm
        );
        // gen 3 × 4 days = day 12
        assert!(
            (drugs[0].start_day - 12.0).abs() < 1e-5,
            "day={}",
            drugs[0].start_day
        );
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
        assert!(
            snap.doubling_time_days > 7.0 && snap.doubling_time_days < 8.0,
            "dt={}",
            snap.doubling_time_days
        );
    }

    #[test]
    fn snapshot_prostate_gen10() {
        let snap = calibrate_snapshot(128.0, 0.536, 10, &PROSTATE_ABIRATERONE);
        // Day 300
        assert!((snap.day - 300.0).abs() < 1e-3);
        // Doubling time: 30 / 0.536 ≈ 56 days
        assert!(
            snap.doubling_time_days > 50.0 && snap.doubling_time_days < 60.0,
            "dt={}",
            snap.doubling_time_days
        );
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

    // ── Canine mast cell (Rosie case) ───────────────────────────────────

    #[test]
    fn mast_cell_doubling_21_days() {
        // London & Seguin 2003: intermediate grade mast cell doubling ~21 days
        assert!((CANINE_MAST_CELL.days_per_generation - 21.0).abs() < 1e-5);
    }

    #[test]
    fn mast_cell_toceranib_ic50_40nm() {
        // London et al. 2009: toceranib IC50 ~40 nM for KIT-mutant mast cell
        assert!((CANINE_MAST_CELL.nm_per_concentration - 40.0).abs() < 1e-3);
    }

    #[test]
    fn rosie_6_weeks_is_2_generations() {
        // 42 days / 21 days per gen = 2 generations
        let gens = days_to_generations(ROSIE_OBSERVED.days_to_partial_response, &CANINE_MAST_CELL);
        assert_eq!(
            gens, 2,
            "6 weeks should be 2 generations at 21-day doubling"
        );
    }

    #[test]
    fn rosie_responsive_resistant_split() {
        // 70% responsive (KIT+), 30% resistant (KIT-)
        let (resp, resist) = fraction_to_entity_counts(45, ROSIE_OBSERVED.responsive_fraction);
        assert_eq!(resp, 32, "70% of 45 = ~32 responsive");
        assert_eq!(resist, 13, "30% of 45 = ~13 resistant");
    }

    #[test]
    fn rosie_calibrated_protocol() {
        // Adaptive result "399 Hz @ 0.40" calibrated to mast cell
        let drugs = calibrate_protocol(&[(399.0, 0.40)], 1, &CANINE_MAST_CELL);
        // 0.40 × 40 nM = 16 nM
        assert!(
            (drugs[0].dose_nm - 16.0).abs() < 1e-2,
            "dose={}",
            drugs[0].dose_nm
        );
        // gen 1 × 21 days = day 21
        assert!((drugs[0].start_day - 21.0).abs() < 1e-5);
    }

    #[test]
    fn rosie_tumor_cell_count() {
        // Tennis-ball sized tumor ~10^8 cells
        let cells = entities_to_cells(128.0, &CANINE_MAST_CELL);
        assert!(cells > 9e7 && cells < 1.1e8, "cells={cells}");
    }

    #[test]
    fn rosie_snapshot_at_response() {
        // At partial response (gen 2 = day 42): efficiency should model 75% reduction
        // If drug reduces efficiency to 0.25, tumor metabolic output is 25% of baseline
        let snap = calibrate_snapshot(128.0, 0.25, 2, &CANINE_MAST_CELL);
        assert!((snap.day - 42.0).abs() < 1e-3, "day={}", snap.day);
        // Doubling time: 21 / 0.25 = 84 days (tumor barely growing)
        assert!(
            snap.doubling_time_days > 80.0,
            "dt={}",
            snap.doubling_time_days
        );
    }
}
