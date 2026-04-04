//! Puente de calibración (TT-7).
//! Calibration bridge (TT-7).
//!
//! Convierte resultados de reconciliación en pesos para normalizadores.
//! Stateless: (record, weights, history, config) → weights.
//! Flujo: Ancla → DiffReport → Puente → NormalizerWeights → Telescopio.

use crate::blueprint::equations::temporal_telescope::{NormalizerWeights, RegimeMetrics};

use super::diff::DiffClass;
use super::ReconciliationRecord;

/// Dimensiones de los normalizadores.
/// Normalizer dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizerDimension {
    Hurst,
    Inertia,
    Fisher,
    Horizon,
    EventDensity,
    Entropy,
}

/// Config del puente de calibración.
/// Calibration bridge config.
#[derive(Clone, Copy, Debug)]
pub struct CalibrationConfig {
    pub learning_rate: f32,
    pub min_history_for_adjust: u16,
    pub weight_floor: f32,
    pub weight_ceiling: f32,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        use crate::blueprint::constants::temporal_telescope as c;
        Self {
            learning_rate: c::CALIBRATION_LEARNING_RATE,
            min_history_for_adjust: c::CALIBRATION_MIN_HISTORY,
            weight_floor: c::CALIBRATION_WEIGHT_FLOOR,
            weight_ceiling: c::CALIBRATION_WEIGHT_CEILING,
        }
    }
}

/// Identifica qué normalizador falló.
/// Identifies which normalizer failed.
///
/// Heurística: la métrica con mayor magnitud absoluta es la más "cargada"
/// al momento del fork — si la proyección falló, esa métrica debió haber
/// ajustado K pero no lo hizo suficientemente.
pub fn identify_weak_normalizer(metrics: &RegimeMetrics) -> NormalizerDimension {
    let candidates = [
        (metrics.hurst.abs(), NormalizerDimension::Hurst),
        (metrics.autocorrelation.abs(), NormalizerDimension::Inertia),
        (metrics.fisher, NormalizerDimension::Fisher),
        (metrics.lambda_max.abs(), NormalizerDimension::Horizon),
        (metrics.event_rate, NormalizerDimension::EventDensity),
        (metrics.entropy_accel.abs(), NormalizerDimension::Entropy),
    ];

    candidates.iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal))
        .map(|&(_, dim)| dim)
        .unwrap_or(NormalizerDimension::Hurst)
}

/// Calibra los pesos de los normalizadores.
/// Calibrates normalizer weights.
///
/// PERFECT → mantener. LOCAL → ajustar normalizador débil. SYSTEMIC → reducir todos.
pub fn calibrate(
    record: &ReconciliationRecord,
    current_weights: &NormalizerWeights,
    history: &[ReconciliationRecord],
    config: &CalibrationConfig,
) -> NormalizerWeights {
    // No ajustar si historial insuficiente.
    if (history.len() as u16) < config.min_history_for_adjust {
        return *current_weights;
    }

    // Guard: non-finite metrics → no ajustar (datos corruptos).
    let m = &record.metrics_at_fork;
    if !m.hurst.is_finite() || !m.fisher.is_finite() || !m.autocorrelation.is_finite()
        || !m.lambda_max.is_finite() || !m.event_rate.is_finite()
    {
        return *current_weights;
    }

    let mut w = *current_weights;
    let lr = config.learning_rate;

    match record.diff_class {
        DiffClass::Perfect => {
            // Telescopio acertó — relajar suavemente hacia neutral (1.0).
            let damp = crate::blueprint::constants::temporal_telescope::CALIBRATION_PERFECT_DAMPENING;
            w.hurst_weight += (1.0 - w.hurst_weight) * lr * damp;
            w.inertia_weight += (1.0 - w.inertia_weight) * lr * damp;
            w.fisher_sensitivity += (1.0 - w.fisher_sensitivity) * lr * damp;
        }
        DiffClass::Local => {
            // Identificar normalizador débil y subirle el peso.
            let weak = identify_weak_normalizer(&record.metrics_at_fork);
            match weak {
                NormalizerDimension::Hurst => w.hurst_weight += lr,
                NormalizerDimension::Inertia => w.inertia_weight += lr,
                NormalizerDimension::Fisher => w.fisher_sensitivity += lr,
                NormalizerDimension::Horizon | NormalizerDimension::EventDensity => {
                    // Reducir max_k proporcionalmente.
                    w.max_k = (w.max_k as f32 * (1.0 - lr)) as u32;
                }
                NormalizerDimension::Entropy => {
                    let ent_s = crate::blueprint::constants::temporal_telescope::CALIBRATION_ENTROPY_SENSITIVITY;
                    w.fisher_sensitivity += lr * ent_s;
                    w.max_k = (w.max_k as f32 * (1.0 - lr * ent_s)) as u32;
                }
            }
        }
        DiffClass::Systemic => {
            // Todos los pesos bajan conservadoramente.
            w.hurst_weight *= 1.0 - lr;
            w.inertia_weight *= 1.0 - lr;
            w.fisher_sensitivity *= 1.0 - lr;
            w.max_k = (w.max_k as f32 * (1.0 - lr)) as u32;
        }
    }

    // Clamp pesos a [floor, ceiling].
    w.hurst_weight = w.hurst_weight.clamp(config.weight_floor, config.weight_ceiling);
    w.inertia_weight = w.inertia_weight.clamp(config.weight_floor, config.weight_ceiling);
    w.fisher_sensitivity = w.fisher_sensitivity.clamp(config.weight_floor, config.weight_ceiling);
    w.max_k = w.max_k.max(crate::blueprint::constants::temporal_telescope::TELESCOPE_K_MIN);

    w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(class: DiffClass) -> ReconciliationRecord {
        ReconciliationRecord {
            tick: 100,
            k_used: 16,
            metrics_at_fork: RegimeMetrics::default(),
            diff_class: class,
            mean_qe_error: 0.0,
            affected_fraction: 0.0,
        }
    }

    fn make_history(n: usize) -> Vec<ReconciliationRecord> {
        (0..n).map(|i| ReconciliationRecord {
            tick: i as u64,
            k_used: 16,
            metrics_at_fork: RegimeMetrics::default(),
            diff_class: DiffClass::Perfect,
            mean_qe_error: 0.0,
            affected_fraction: 0.0,
        }).collect()
    }

    #[test]
    fn calibrate_perfect_minimal_change() {
        let config = CalibrationConfig::default();
        let w = NormalizerWeights::default();
        let history = make_history(10);
        let record = make_record(DiffClass::Perfect);
        let new_w = calibrate(&record, &w, &history, &config);
        assert!((new_w.hurst_weight - w.hurst_weight).abs() < 0.02);
        assert!((new_w.inertia_weight - w.inertia_weight).abs() < 0.02);
    }

    #[test]
    fn calibrate_systemic_reduces_all() {
        let config = CalibrationConfig::default();
        let w = NormalizerWeights { hurst_weight: 2.0, inertia_weight: 2.0, fisher_sensitivity: 2.0, max_k: 100 };
        let history = make_history(10);
        let record = make_record(DiffClass::Systemic);
        let new_w = calibrate(&record, &w, &history, &config);
        assert!(new_w.hurst_weight < w.hurst_weight);
        assert!(new_w.inertia_weight < w.inertia_weight);
        assert!(new_w.fisher_sensitivity < w.fisher_sensitivity);
    }

    #[test]
    fn calibrate_empty_history_no_change() {
        let config = CalibrationConfig::default();
        let w = NormalizerWeights::default();
        let record = make_record(DiffClass::Systemic);
        let new_w = calibrate(&record, &w, &[], &config);
        assert_eq!(new_w.hurst_weight, w.hurst_weight);
    }

    #[test]
    fn calibrate_respects_floor() {
        let config = CalibrationConfig::default();
        let w = NormalizerWeights { hurst_weight: 0.11, inertia_weight: 0.11, fisher_sensitivity: 0.11, max_k: 100 };
        let history = make_history(10);
        let record = make_record(DiffClass::Systemic);
        let new_w = calibrate(&record, &w, &history, &config);
        assert!(new_w.hurst_weight >= config.weight_floor);
        assert!(new_w.inertia_weight >= config.weight_floor);
        assert!(new_w.fisher_sensitivity >= config.weight_floor);
    }

    #[test]
    fn calibrate_respects_ceiling() {
        let config = CalibrationConfig::default();
        let w = NormalizerWeights { hurst_weight: 4.9, inertia_weight: 4.9, fisher_sensitivity: 4.9, max_k: 100 };
        let history = make_history(10);
        let mut record = make_record(DiffClass::Local);
        record.metrics_at_fork.hurst = 10.0; // force Hurst as weak
        let new_w = calibrate(&record, &w, &history, &config);
        assert!(new_w.hurst_weight <= config.weight_ceiling);
    }

    #[test]
    fn calibrate_local_adjusts_weak_normalizer() {
        let config = CalibrationConfig::default();
        let w = NormalizerWeights::default();
        let history = make_history(10);
        let mut record = make_record(DiffClass::Local);
        record.metrics_at_fork.event_rate = 100.0; // EventDensity should be weak
        let new_w = calibrate(&record, &w, &history, &config);
        // EventDensity → max_k reduces
        assert!(new_w.max_k < w.max_k);
    }

    #[test]
    fn identify_weak_highest_metric_wins() {
        let m = RegimeMetrics { fisher: 100.0, ..Default::default() };
        assert_eq!(identify_weak_normalizer(&m), NormalizerDimension::Fisher);
    }

    #[test]
    fn identify_weak_default_metrics() {
        let m = RegimeMetrics::default();
        // All zeros — arbitrary winner is fine, just don't panic
        let _ = identify_weak_normalizer(&m);
    }

    #[test]
    fn convergence_100_calibrations_weights_bounded() {
        let config = CalibrationConfig::default();
        let mut w = NormalizerWeights::default();
        let history = make_history(10);
        for i in 0..100 {
            let class = if i % 3 == 0 { DiffClass::Systemic } else if i % 3 == 1 { DiffClass::Local } else { DiffClass::Perfect };
            let record = make_record(class);
            w = calibrate(&record, &w, &history, &config);
        }
        assert!(w.hurst_weight >= config.weight_floor && w.hurst_weight <= config.weight_ceiling);
        assert!(w.inertia_weight >= config.weight_floor && w.inertia_weight <= config.weight_ceiling);
    }
}
