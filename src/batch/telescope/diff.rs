//! Motor de diff entre Ancla y Telescopio (TT-4).
//! Diff engine between Anchor and Telescope (TT-4).
//!
//! Stateless: compara dos SimWorldFlat, retorna DiffReport.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::constants::MAX_ENTITIES;

/// Diff por entidad entre anchor y telescope.
/// Per-entity diff between anchor and telescope.
#[derive(Clone, Copy, Debug, Default)]
pub struct EntityDiff {
    pub index: usize,
    pub qe_delta: f32,
    pub pos_delta_sq: f32,
    pub freq_delta: f32,
    pub alive_mismatch: bool,
}

/// Clasificación del diff global.
/// Global diff classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffClass {
    /// diff < threshold en todas las entidades.
    Perfect,
    /// 1-10% de entidades difieren significativamente.
    Local,
    /// >10% de entidades difieren.
    Systemic,
}

/// Reporte completo del diff.
/// Complete diff report.
#[derive(Clone, Debug)]
pub struct DiffReport {
    pub class: DiffClass,
    pub entity_diffs: [EntityDiff; MAX_ENTITIES],
    pub affected_count: u16,
    pub max_qe_delta: f32,
    pub alive_mismatches: u16,
    pub mean_qe_error: f32,
}

impl Default for DiffReport {
    fn default() -> Self {
        Self {
            class: DiffClass::Perfect,
            entity_diffs: [EntityDiff::default(); MAX_ENTITIES],
            affected_count: 0,
            max_qe_delta: 0.0,
            alive_mismatches: 0,
            mean_qe_error: 0.0,
        }
    }
}

/// Compara un EntitySlot entre anchor y telescope.
/// Compares an EntitySlot between anchor and telescope.
#[inline]
pub fn entity_diff(anchor: &EntitySlot, telescope: &EntitySlot, index: usize) -> EntityDiff {
    let dx = anchor.position[0] - telescope.position[0];
    let dy = anchor.position[1] - telescope.position[1];
    EntityDiff {
        index,
        qe_delta: anchor.qe - telescope.qe,
        pos_delta_sq: dx * dx + dy * dy,
        freq_delta: (anchor.frequency_hz - telescope.frequency_hz).abs(),
        alive_mismatch: anchor.alive != telescope.alive,
    }
}

/// Clasifica según fracción de entidades afectadas.
/// Classifies by fraction of affected entities.
#[inline]
pub fn classify_diff(affected_count: u16, total_alive: u16, alive_mismatches: u16) -> DiffClass {
    if total_alive == 0 {
        return DiffClass::Perfect;
    }
    let affected_total = affected_count + alive_mismatches;
    let fraction = affected_total as f32 / total_alive as f32;
    if fraction > crate::blueprint::constants::temporal_telescope::DIFF_SYSTEMIC_FRACTION {
        DiffClass::Systemic
    } else if affected_total > 0 {
        DiffClass::Local
    } else {
        DiffClass::Perfect
    }
}

/// Compara dos SimWorldFlat. Stateless: no muta ninguno.
/// Compares two SimWorldFlat. Stateless: mutates neither.
pub fn world_diff(anchor: &SimWorldFlat, telescope: &SimWorldFlat, threshold_pct: f32) -> DiffReport {
    debug_assert!(threshold_pct > 0.0 && threshold_pct <= 1.0,
        "threshold_pct must be in (0, 1], got {threshold_pct}");
    let mut report = DiffReport::default();
    let combined_alive = anchor.alive_mask | telescope.alive_mask;
    let mut mask = combined_alive;
    let mut affected = 0_u16;
    let mut alive_mismatches = 0_u16;
    let mut max_delta = 0.0_f32;
    let mut sum_error = 0.0_f32;
    let mut alive_count = 0_u16;

    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        let a = &anchor.entities[i];
        let t = &telescope.entities[i];
        let d = entity_diff(a, t, i);
        report.entity_diffs[i] = d;

        if d.alive_mismatch {
            alive_mismatches += 1;
        }

        // Solo contar error para entidades vivas en ambos.
        let both_alive = a.alive && t.alive;
        if both_alive {
            alive_count += 1;
            let ref_qe = a.qe.max(crate::blueprint::constants::temporal_telescope::DIFF_QE_MIN_REFERENCE);
            let relative_error = d.qe_delta.abs() / ref_qe;
            sum_error += relative_error;
            if relative_error > threshold_pct {
                affected += 1;
            }
            if d.qe_delta.abs() > max_delta {
                max_delta = d.qe_delta.abs();
            }
        } else if a.alive || t.alive {
            alive_count += 1;
        }
    }

    report.affected_count = affected;
    report.alive_mismatches = alive_mismatches;
    report.max_qe_delta = max_delta;
    report.mean_qe_error = if alive_count > 0 { sum_error / alive_count as f32 } else { 0.0 };
    report.class = classify_diff(affected, alive_count, alive_mismatches);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world_with_entities(specs: &[(f32, f32, bool)]) -> SimWorldFlat {
        let mut w = SimWorldFlat::new(42, 0.05);
        for &(qe, freq, alive) in specs {
            let mut e = EntitySlot::default();
            e.qe = qe;
            e.frequency_hz = freq;
            if alive {
                w.spawn(e);
            }
        }
        w
    }

    #[test]
    fn identical_worlds_are_perfect() {
        let w = make_world_with_entities(&[(100.0, 400.0, true); 10]);
        let report = world_diff(&w, &w, 0.02);
        assert_eq!(report.class, DiffClass::Perfect);
        assert_eq!(report.affected_count, 0);
        assert_eq!(report.alive_mismatches, 0);
    }

    #[test]
    fn one_entity_different_is_local() {
        let a = make_world_with_entities(&[(100.0, 400.0, true); 10]);
        let mut t = a.clone();
        t.entities[0].qe = 50.0; // 50% diff
        let report = world_diff(&a, &t, 0.02);
        assert_eq!(report.class, DiffClass::Local);
        assert!(report.affected_count >= 1);
    }

    #[test]
    fn many_entities_different_is_systemic() {
        let a = make_world_with_entities(&[(100.0, 400.0, true); 20]);
        let mut t = a.clone();
        for i in 0..10 {
            t.entities[i].qe = 1.0; // 99% diff
        }
        let report = world_diff(&a, &t, 0.02);
        assert_eq!(report.class, DiffClass::Systemic);
    }

    #[test]
    fn alive_mismatch_detected() {
        let a = make_world_with_entities(&[(100.0, 400.0, true); 5]);
        let mut t = a.clone();
        t.kill(0);
        let report = world_diff(&a, &t, 0.02);
        assert!(report.alive_mismatches >= 1);
    }

    #[test]
    fn entity_diff_identical_is_zero() {
        let e = EntitySlot::default();
        let d = entity_diff(&e, &e, 0);
        assert_eq!(d.qe_delta, 0.0);
        assert_eq!(d.pos_delta_sq, 0.0);
        assert_eq!(d.freq_delta, 0.0);
        assert!(!d.alive_mismatch);
    }

    #[test]
    fn empty_worlds_are_perfect() {
        let w = SimWorldFlat::new(42, 0.05);
        let report = world_diff(&w, &w, 0.02);
        assert_eq!(report.class, DiffClass::Perfect);
    }

    #[test]
    fn mean_qe_error_computed() {
        let a = make_world_with_entities(&[(100.0, 400.0, true)]);
        let mut t = a.clone();
        t.entities[0].qe = 90.0; // 10% diff
        let report = world_diff(&a, &t, 0.02);
        assert!(report.mean_qe_error > 0.05, "error should be ~0.1: {}", report.mean_qe_error);
    }

    #[test]
    fn classify_zero_affected_is_perfect() {
        assert_eq!(classify_diff(0, 100, 0), DiffClass::Perfect);
    }

    #[test]
    fn classify_low_affected_is_local() {
        assert_eq!(classify_diff(5, 100, 0), DiffClass::Local);
    }

    #[test]
    fn classify_high_affected_is_systemic() {
        assert_eq!(classify_diff(15, 100, 0), DiffClass::Systemic);
    }
}
