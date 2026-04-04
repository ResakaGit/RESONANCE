//! Activación y métricas del Telescopio (TT-10).
//! Telescope activation and metrics (TT-10).
//!
//! Funciones puras que conectan sistemas dormidos al Telescopio.
//! No modifica los sistemas dormidos — los consume como input.

use crate::blueprint::equations::temporal_telescope::{
    RegimeMetrics, fisher_information, shannon_entropy,
    sliding_autocorrelation_lag1, sliding_variance,
};

use super::{TelescopePhase, TelescopeState};
use super::diff::DiffClass;
use super::pipeline::ReconciliationHistory;

// ─── GeologicalLOD Wiring ────────────────────────────────────────────────────

/// Niveles de compresión de LOD: [1, 10, 100, 1000].
/// LOD compression levels: [1, 10, 100, 1000].
const LOD_LEVELS: [u32; 4] = [1, 10, 100, 1000];

/// Selecciona nivel de LOD según K del telescopio.
/// Selects LOD level from telescope K.
///
/// Busca el bucket más cercano (sin exceder K).
/// Finds nearest bucket (without exceeding K).
#[inline]
pub fn lod_level_from_k(telescope_k: u32) -> u32 {
    LOD_LEVELS.iter()
        .copied()
        .filter(|&level| level <= telescope_k)
        .last()
        .unwrap_or(LOD_LEVELS[0])
}

// ─── MultiscaleSignalGrid → RegimeMetrics ────────────────────────────────────

/// Computa RegimeMetrics desde señales regionales + historial de qe.
/// Computes RegimeMetrics from regional signals + qe history.
///
/// `regional_current`/`regional_previous`: señales de MultiscaleSignalGrid (64 regiones).
/// `qe_history`: últimos N valores de total_qe.
pub fn regime_metrics_from_multiscale(
    regional_current: &[f32],
    regional_previous: &[f32],
    qe_history: &[f32],
    dt: f32,
    population_fraction: f32,
    event_rate: f32,
) -> RegimeMetrics {
    let fisher = if regional_current.len() == regional_previous.len() && !regional_current.is_empty() {
        fisher_information(regional_current, regional_previous, dt)
    } else {
        0.0
    };

    let _entropy = shannon_entropy(regional_current);
    let variance = sliding_variance(qe_history);
    let autocorrelation = sliding_autocorrelation_lag1(qe_history);
    let lambda_max = crate::blueprint::equations::temporal_telescope::estimate_lambda_max(
        autocorrelation, dt,
    );

    RegimeMetrics {
        variance,
        autocorrelation,
        hurst: 0.5, // DFA costoso — computar externamente.
        fisher,
        fisher_median: 0.0, // se computa externamente con historial de Fisher; 0 = no disponible.
        entropy_accel: 0.0, // computar externamente con 3 valores de entropía.
        lambda_max,
        population: population_fraction,
        event_rate,
    }
}

// ─── Dashboard Summary ───────────────────────────────────────────────────────

/// Resumen del telescopio para dashboard. Datos puros, sin Bevy.
/// Telescope summary for dashboard. Pure data, no Bevy.
#[derive(Clone, Copy, Debug, Default)]
pub struct TelescopeSummary {
    pub phase: u8,                // 0=Idle, 1=Projecting, 2=Reconciling, 3=Correcting
    pub current_k: u32,
    pub projection_accuracy: f32,
    pub correction_frequency: f32,
    pub hurst: f32,
    pub autocorrelation: f32,
    pub fisher: f32,
    pub lambda_max: f32,
    pub regime_label_id: u8,     // 0=STASIS, 1=PRE-TRANS, 2=TRANSITION, 3=POST-TRANS
}

/// Fase del telescopio como u8 (para serialización sin String).
/// Telescope phase as u8 (for serialization without String).
#[inline]
fn phase_to_u8(phase: TelescopePhase) -> u8 {
    match phase {
        TelescopePhase::Idle => 0,
        TelescopePhase::Projecting => 1,
        TelescopePhase::Reconciling => 2,
        TelescopePhase::Correcting => 3,
    }
}

/// Régimen como u8.
/// Regime as u8.
#[inline]
fn regime_to_u8(label: &str) -> u8 {
    match label {
        "STASIS" => 0,
        "PRE-TRANSITION" => 1,
        "TRANSITION" => 2,
        _ => 3, // POST-TRANSITION
    }
}

/// Computa precisión media de las últimas N reconciliaciones.
/// Computes mean accuracy of the last N reconciliations.
pub fn projection_accuracy(history: &ReconciliationHistory, window: usize) -> f32 {
    let recent = history.recent(window);
    if recent.is_empty() {
        return 1.0; // sin datos → asumir perfecto.
    }
    let perfect_count = recent.iter()
        .filter(|r| r.diff_class == DiffClass::Perfect)
        .count();
    perfect_count as f32 / recent.len() as f32
}

/// Computa frecuencia de corrección (LOCAL + SYSTEMIC / total).
/// Computes correction frequency (LOCAL + SYSTEMIC / total).
pub fn correction_frequency(history: &ReconciliationHistory) -> f32 {
    if history.is_empty() {
        return 0.0;
    }
    let recent = history.recent(history.len());
    let corrections = recent.iter()
        .filter(|r| r.diff_class != DiffClass::Perfect)
        .count();
    corrections as f32 / recent.len() as f32
}

/// Genera TelescopeSummary desde estado actual + historial.
/// Generates TelescopeSummary from current state + history.
pub fn telescope_summary(
    state: &TelescopeState,
    history: &ReconciliationHistory,
    metrics: &RegimeMetrics,
) -> TelescopeSummary {
    let label = super::pipeline::regime_label(metrics);
    TelescopeSummary {
        phase: phase_to_u8(state.phase),
        current_k: state.current_k,
        projection_accuracy: projection_accuracy(history, crate::blueprint::constants::temporal_telescope::ACCURACY_WINDOW),
        correction_frequency: correction_frequency(history),
        hurst: metrics.hurst,
        autocorrelation: metrics.autocorrelation,
        fisher: metrics.fisher,
        lambda_max: metrics.lambda_max,
        regime_label_id: regime_to_u8(label),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::telescope::ReconciliationRecord;
    use crate::blueprint::equations::temporal_telescope::RegimeMetrics;

    // ── GeologicalLOD ────────────────────────────────────────────

    #[test]
    fn lod_level_k1_returns_1() {
        assert_eq!(lod_level_from_k(1), 1);
    }

    #[test]
    fn lod_level_k64_returns_10() {
        assert_eq!(lod_level_from_k(64), 10);
    }

    #[test]
    fn lod_level_k100_returns_100() {
        assert_eq!(lod_level_from_k(100), 100);
    }

    #[test]
    fn lod_level_k1024_returns_1000() {
        assert_eq!(lod_level_from_k(1024), 1000);
    }

    #[test]
    fn lod_level_k0_returns_minimum() {
        // K=0 debería retornar el menor bucket que no exceda (ninguno excede 0 excepto 1>0).
        // Con la lógica actual, best=1 porque 1 > 0 es false. Verify.
        let level = lod_level_from_k(0);
        // 1 > 0, so LOD_LEVELS[0]=1 is NOT <= 0. best stays at initial LOD_LEVELS[0]=1.
        // Actually, the loop: for &level in &LOD_LEVELS { if level <= 0 → false }
        // So best = LOD_LEVELS[0] = 1 (initial value, never overwritten).
        assert_eq!(level, 1);
    }

    // ── MultiscaleSignalGrid → RegimeMetrics ─────────────────────

    #[test]
    fn regime_metrics_from_identical_signals() {
        let signals = vec![1.0_f32; 64];
        let m = regime_metrics_from_multiscale(&signals, &signals, &[100.0; 128], 0.05, 0.5, 0.01);
        assert!(m.fisher < 1e-3, "identical signals should have near-zero Fisher: {}", m.fisher);
        assert_eq!(m.population, 0.5);
    }

    #[test]
    fn regime_metrics_from_changed_signals() {
        let prev = vec![1.0_f32; 64];
        let curr: Vec<f32> = (0..64).map(|i| 1.0 + i as f32 * 0.1).collect();
        let m = regime_metrics_from_multiscale(&curr, &prev, &[100.0; 128], 0.05, 0.5, 0.01);
        assert!(m.fisher > 0.0, "changed signals should have positive Fisher: {}", m.fisher);
    }

    #[test]
    fn regime_metrics_empty_signals() {
        let m = regime_metrics_from_multiscale(&[], &[], &[100.0; 128], 0.05, 0.5, 0.01);
        assert_eq!(m.fisher, 0.0);
    }

    // ── Dashboard Summary ────────────────────────────────────────

    #[test]
    fn projection_accuracy_no_history() {
        let h = ReconciliationHistory::default();
        assert_eq!(projection_accuracy(&h, 10), 1.0);
    }

    #[test]
    fn projection_accuracy_all_perfect() {
        let mut h = ReconciliationHistory::default();
        for i in 0..5 {
            h.push(ReconciliationRecord {
                tick: i,
                k_used: 16,
                metrics_at_fork: RegimeMetrics::default(),
                diff_class: DiffClass::Perfect,
                mean_qe_error: 0.0,
                affected_fraction: 0.0,
            });
        }
        assert_eq!(projection_accuracy(&h, 10), 1.0);
    }

    #[test]
    fn projection_accuracy_half_systemic() {
        let mut h = ReconciliationHistory::default();
        for i in 0..10 {
            let class = if i % 2 == 0 { DiffClass::Perfect } else { DiffClass::Systemic };
            h.push(ReconciliationRecord {
                tick: i,
                k_used: 16,
                metrics_at_fork: RegimeMetrics::default(),
                diff_class: class,
                mean_qe_error: 0.0,
                affected_fraction: 0.0,
            });
        }
        let acc = projection_accuracy(&h, 10);
        assert!((acc - 0.5).abs() < 0.1, "expected ~50% accuracy: {acc}");
    }

    #[test]
    fn correction_frequency_no_history() {
        let h = ReconciliationHistory::default();
        assert_eq!(correction_frequency(&h), 0.0);
    }

    #[test]
    fn correction_frequency_all_perfect() {
        let mut h = ReconciliationHistory::default();
        for i in 0..5 {
            h.push(ReconciliationRecord {
                tick: i,
                k_used: 16,
                metrics_at_fork: RegimeMetrics::default(),
                diff_class: DiffClass::Perfect,
                mean_qe_error: 0.0,
                affected_fraction: 0.0,
            });
        }
        assert_eq!(correction_frequency(&h), 0.0);
    }

    #[test]
    fn telescope_summary_stasis() {
        let state = TelescopeState::default();
        let h = ReconciliationHistory::default();
        let m = RegimeMetrics { variance: 0.0, autocorrelation: 0.3, ..Default::default() };
        let s = telescope_summary(&state, &h, &m);
        assert_eq!(s.regime_label_id, 0); // STASIS
        assert_eq!(s.projection_accuracy, 1.0);
    }

    #[test]
    fn telescope_summary_pre_transition() {
        let state = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let h = ReconciliationHistory::default();
        let m = RegimeMetrics { autocorrelation: 0.96, ..Default::default() };
        let s = telescope_summary(&state, &h, &m);
        assert_eq!(s.regime_label_id, 1); // PRE-TRANSITION
        assert_eq!(s.phase, 1); // Projecting
    }

    #[test]
    fn telescope_summary_idle() {
        let state = TelescopeState::default(); // Idle
        let h = ReconciliationHistory::default();
        let m = RegimeMetrics::default();
        let s = telescope_summary(&state, &h, &m);
        assert_eq!(s.phase, 0); // Idle
    }
}
