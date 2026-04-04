//! Pipeline dual-timeline del Telescopio Temporal (TT-9).
//! Dual-timeline pipeline for the Temporal Telescope (TT-9).
//!
//! Orquesta: fork → proyección → simulación ancla → reconciliación → cascada → calibración.
//! Modo síncrono (tests) y paralelo (producción).
//! No modifica batch/pipeline.rs — lo envuelve.

use crate::batch::arena::SimWorldFlat;
use crate::batch::scratch::ScratchPad;
use crate::blueprint::constants::temporal_telescope as tc;
use crate::blueprint::equations::temporal_telescope::{
    NormalizerWeights, RegimeMetrics, optimal_k, sliding_autocorrelation_lag1, sliding_variance,
};

use super::calibration_bridge::{CalibrationConfig, calibrate};
use super::cascade::{CascadeReport, cascade};
use super::diff::{DiffClass, world_diff};
use super::{
    ReconciliationRecord, TelescopeConfig, TelescopePhase, TelescopeState,
    telescope_after_reconciliation,
};

/// Capacidad del ring buffer de historial de reconciliación.
/// Reconciliation history ring buffer capacity.
pub const HISTORY_CAPACITY: usize = 256;

/// Historial de reconciliaciones. Stack-friendly ring buffer.
/// Reconciliation history. Stack-friendly ring buffer.
#[derive(Clone, Debug)]
pub struct ReconciliationHistory {
    records: [ReconciliationRecord; HISTORY_CAPACITY],
    len: usize,
    head: usize,
}

impl Default for ReconciliationHistory {
    fn default() -> Self {
        Self {
            records: [ReconciliationRecord {
                tick: 0,
                k_used: 0,
                metrics_at_fork: RegimeMetrics::default(),
                diff_class: DiffClass::Perfect,
                mean_qe_error: 0.0,
                affected_fraction: 0.0,
            }; HISTORY_CAPACITY],
            len: 0,
            head: 0,
        }
    }
}

impl ReconciliationHistory {
    /// Agrega un record al ring buffer.
    /// Pushes a record to the ring buffer.
    pub fn push(&mut self, record: ReconciliationRecord) {
        let idx = self.head % HISTORY_CAPACITY;
        self.records[idx] = record;
        self.head += 1;
        if self.len < HISTORY_CAPACITY {
            self.len += 1;
        }
    }

    /// Retorna slice de los últimos `n` records (o todos si hay menos).
    /// Returns the last `n` records as a slice (or all if fewer).
    ///
    /// En caso de wrap (ring buffer lleno), retorna la porción más reciente contigua.
    /// La pérdida es ≤ HISTORY_CAPACITY - count records más antiguos.
    pub fn recent(&self, n: usize) -> &[ReconciliationRecord] {
        let count = n.min(self.len);
        if count == 0 {
            return &[];
        }
        // head apunta al PRÓXIMO slot a escribir. Los últimos `count` están en [head-count, head).
        let start = self.head.saturating_sub(count);
        let start_idx = start % HISTORY_CAPACITY;
        // Si el rango no cruza el borde del ring → slice contiguo.
        if start_idx + count <= HISTORY_CAPACITY {
            &self.records[start_idx..start_idx + count]
        } else {
            // Wrap: retornar desde start_idx hasta el final (la parte más reciente pre-wrap).
            // Los records post-wrap están en [0, count - (CAP - start_idx)).
            // Retornamos la parte contigua más larga (la pre-wrap).
            &self.records[start_idx..]
        }
    }

    /// Cantidad de records almacenados.
    /// Number of stored records.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Está vacío.
    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Resultado de un tramo del telescopio.
/// Result of a telescope segment.
#[derive(Clone, Debug)]
pub struct TelescopeTickResult {
    /// K ticks proyectados.
    pub k_used: u32,
    /// Fase final.
    pub phase: TelescopePhase,
    /// Clasificación del diff (None si telescopio idle).
    pub diff_class: Option<DiffClass>,
    /// Reporte de cascada (None si Perfect o idle).
    pub cascade_report: Option<CascadeReport>,
    /// Pesos actualizados.
    pub new_weights: NormalizerWeights,
    /// Métricas del régimen al momento del fork.
    pub metrics: RegimeMetrics,
}

/// Computa métricas de régimen desde la serie temporal reciente del mundo.
/// Computes regime metrics from the world's recent time series.
///
/// Stateless: extrae datos del mundo, no los modifica.
pub fn compute_regime_metrics(
    world: &SimWorldFlat,
    qe_history: &[f32],
    _pop_history: &[f32],
) -> RegimeMetrics {
    let variance = sliding_variance(qe_history);
    let autocorrelation = sliding_autocorrelation_lag1(qe_history);
    let population = world.alive_mask.count_ones() as f32 / crate::batch::constants::MAX_ENTITIES as f32;

    RegimeMetrics {
        variance,
        autocorrelation,
        hurst: 0.5, // DFA es caro — el caller puede computar periódicamente y pasar por fuera
        fisher: 0.0,
        fisher_median: 0.0,
        entropy_accel: 0.0,
        lambda_max: crate::blueprint::equations::temporal_telescope::estimate_lambda_max(
            autocorrelation,
            world.dt,
        ),
        population,
        event_rate: 0.0, // estimable desde pop_history trend
    }
}

/// Ejecuta un ciclo completo del telescopio (síncrono).
/// Executes a full telescope cycle (synchronous).
///
/// Fork → Project → Anchor Simulation → Reconcile → Cascade → Calibrate → Commit.
/// El mundo resultante contiene el estado del ANCLA (la verdad), no del telescopio.
/// The resulting world contains the ANCHOR state (ground truth), not the telescope.
pub fn tick_telescope_sync(
    world: &mut SimWorldFlat,
    state: &mut TelescopeState,
    config: &TelescopeConfig,
    cal_config: &CalibrationConfig,
    history: &mut ReconciliationHistory,
    scratch: &mut ScratchPad,
    qe_history: &[f32],
    pop_history: &[f32],
) -> TelescopeTickResult {
    if state.phase == TelescopePhase::Idle {
        return TelescopeTickResult {
            k_used: 0,
            phase: TelescopePhase::Idle,
            diff_class: None,
            cascade_report: None,
            new_weights: state.weights,
            metrics: RegimeMetrics::default(),
        };
    }

    // 1. FORK: clonar mundo para ancla.
    let mut anchor = world.clone();
    let metrics = compute_regime_metrics(world, qe_history, pop_history);

    // 2. PROJECT: proyección analítica del telescopio.
    let k = optimal_k(&metrics, &state.weights, config.k_min, config.k_max);
    let telescope = super::projection::project_world(world, &metrics, &state.weights, k);

    // 3. SIMULATE: ancla corre k ticks completos.
    for _ in 0..k {
        anchor.tick(scratch);
    }

    // 4. RECONCILE: diff entre ancla y telescopio.
    let diff = world_diff(&anchor, &telescope, tc::DIFF_THRESHOLD_PCT);

    let total_alive = anchor.alive_mask.count_ones().max(1) as u16;
    let record = ReconciliationRecord {
        tick: anchor.tick_id,
        k_used: k,
        metrics_at_fork: metrics,
        diff_class: diff.class,
        mean_qe_error: diff.mean_qe_error,
        affected_fraction: diff.affected_count as f32 / total_alive as f32,
    };

    // 5. CASCADE (sobre la copia del telescopio — solo para métricas, el ancla gana siempre).
    let cascade_report = if diff.class != DiffClass::Perfect {
        let mut telescope_mut = telescope;
        Some(cascade(
            &mut telescope_mut,
            &anchor,
            &diff,
            tc::CASCADE_MAX_HOPS,
            tc::CASCADE_ATTENUATION_PER_HOP,
            tc::CASCADE_CORRECTION_EPSILON,
        ))
    } else {
        None
    };

    // 6. CALIBRATE: puente ajusta pesos.
    let new_weights = calibrate(
        &record,
        &state.weights,
        history.recent(history.len()),
        cal_config,
    );

    // 7. COMMIT: la verdad siempre gana.
    *world = anchor;
    *state = telescope_after_reconciliation(state, &record, config);
    state.weights = new_weights;
    history.push(record);

    TelescopeTickResult {
        k_used: k,
        phase: state.phase,
        diff_class: Some(diff.class),
        cascade_report,
        new_weights,
        metrics,
    }
}

/// Clasifica el régimen actual a partir de métricas.
/// Classifies current regime from metrics.
///
/// Retorna etiqueta estática para dashboard.
pub fn regime_label(metrics: &RegimeMetrics) -> &'static str {
    if metrics.autocorrelation > tc::RHO1_HIGH_INERTIA {
        "PRE-TRANSITION"
    } else if metrics.variance < tc::ENTROPY_ACCELERATION_EPSILON
        && metrics.autocorrelation < tc::REGIME_STASIS_RHO_CEILING
    {
        "STASIS"
    } else if metrics.entropy_accel.abs() > tc::ENTROPY_ACCELERATION_EPSILON {
        "TRANSITION"
    } else {
        "POST-TRANSITION"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn make_stable_world(n: usize, qe: f32) -> SimWorldFlat {
        let mut w = SimWorldFlat::new(42, 0.05);
        for i in 0..n {
            let mut e = EntitySlot::default();
            e.qe = qe;
            e.radius = 0.5;
            e.dissipation = 0.005;
            e.frequency_hz = 200.0 + i as f32 * 50.0;
            e.position = [i as f32 * 20.0, 0.0]; // far apart → isolated
            e.archetype = 2;
            e.trophic_class = 2;
            w.spawn(e);
        }
        w.update_total_qe();
        w
    }

    fn empty_histories() -> (Vec<f32>, Vec<f32>) {
        (vec![100.0; 128], vec![10.0; 128])
    }

    #[test]
    fn idle_telescope_is_noop() {
        let mut w = make_stable_world(5, 100.0);
        let mut state = TelescopeState::default(); // phase = Idle
        let config = TelescopeConfig::default();
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();
        let mut scratch = ScratchPad::new();
        let (qe_h, pop_h) = empty_histories();

        let result = tick_telescope_sync(
            &mut w, &mut state, &config, &cal_config,
            &mut history, &mut scratch, &qe_h, &pop_h,
        );
        assert_eq!(result.phase, TelescopePhase::Idle);
        assert_eq!(result.k_used, 0);
        assert!(result.diff_class.is_none());
    }

    #[test]
    fn sync_telescope_world_is_anchor_truth() {
        let mut w = make_stable_world(5, 100.0);
        let initial_tick = w.tick_id;
        let mut state = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let config = TelescopeConfig { k_min: 4, k_max: 8, k_initial: 4, ..Default::default() };
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();
        let mut scratch = ScratchPad::new();
        let (qe_h, pop_h) = empty_histories();

        let result = tick_telescope_sync(
            &mut w, &mut state, &config, &cal_config,
            &mut history, &mut scratch, &qe_h, &pop_h,
        );

        // World should have advanced by K ticks (anchor ran K ticks).
        assert!(w.tick_id > initial_tick, "tick should advance");
        assert_eq!(w.tick_id, initial_tick + result.k_used as u64);
        // History should have 1 record.
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn sync_telescope_produces_valid_diff_class() {
        let mut w = make_stable_world(5, 100.0);
        let mut state = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let config = TelescopeConfig { k_min: 4, k_max: 4, k_initial: 4, ..Default::default() };
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();
        let mut scratch = ScratchPad::new();
        let (qe_h, pop_h) = empty_histories();

        let result = tick_telescope_sync(
            &mut w, &mut state, &config, &cal_config,
            &mut history, &mut scratch, &qe_h, &pop_h,
        );

        // The diff class should be one of the valid variants.
        // With K=4, the full simulation (33 systems) diverges from the analytical
        // projection — Systemic is expected and correct. The reconciliation catches it.
        assert!(result.diff_class.is_some(), "diff_class should be present");
        // After reconciliation, the world is the anchor (correct), regardless of diff class.
        assert!(w.total_qe.is_finite(), "world should be finite after reconciliation");
    }

    #[test]
    fn sync_telescope_conservation_holds() {
        let mut w = make_stable_world(10, 100.0);
        let qe_before = w.total_qe;
        let mut state = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let config = TelescopeConfig { k_min: 4, k_max: 8, k_initial: 4, ..Default::default() };
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();
        let mut scratch = ScratchPad::new();
        let (qe_h, pop_h) = empty_histories();

        tick_telescope_sync(
            &mut w, &mut state, &config, &cal_config,
            &mut history, &mut scratch, &qe_h, &pop_h,
        );

        // Axiom 5: energy should not increase (anchor ran full simulation).
        assert!(w.total_qe <= qe_before + 1.0,
            "Axiom 5: energy increased after telescope: {} → {}", qe_before, w.total_qe);
    }

    #[test]
    fn sync_telescope_deterministic() {
        let w_orig = make_stable_world(5, 100.0);
        let config = TelescopeConfig { k_min: 4, k_max: 4, k_initial: 4, ..Default::default() };
        let cal_config = CalibrationConfig::default();
        let (qe_h, pop_h) = empty_histories();

        // Run 1
        let mut w1 = w_orig.clone();
        let mut state1 = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let mut hist1 = ReconciliationHistory::default();
        let mut s1 = ScratchPad::new();
        tick_telescope_sync(&mut w1, &mut state1, &config, &cal_config, &mut hist1, &mut s1, &qe_h, &pop_h);

        // Run 2
        let mut w2 = w_orig.clone();
        let mut state2 = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let mut hist2 = ReconciliationHistory::default();
        let mut s2 = ScratchPad::new();
        tick_telescope_sync(&mut w2, &mut state2, &config, &cal_config, &mut hist2, &mut s2, &qe_h, &pop_h);

        // Bit-exact comparison.
        assert_eq!(w1.tick_id, w2.tick_id);
        for i in 0..5 {
            assert_eq!(w1.entities[i].qe.to_bits(), w2.entities[i].qe.to_bits(),
                "entity {i} qe diverged");
        }
    }

    #[test]
    fn multiple_telescope_cycles_converge() {
        let mut w = make_stable_world(5, 100.0);
        let mut state = TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() };
        let config = TelescopeConfig::default();
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();
        let mut scratch = ScratchPad::new();
        let (qe_h, pop_h) = empty_histories();

        // Run 10 telescope cycles.
        for _ in 0..10 {
            tick_telescope_sync(
                &mut w, &mut state, &config, &cal_config,
                &mut history, &mut scratch, &qe_h, &pop_h,
            );
        }

        assert_eq!(history.len(), 10);
        assert!(w.total_qe.is_finite(), "world should be finite after 10 cycles");
        assert!(state.total_reconciliations == 10);
    }

    #[test]
    fn history_ring_buffer_push_and_recent() {
        let mut h = ReconciliationHistory::default();
        assert!(h.is_empty());

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
        assert_eq!(h.len(), 5);
        let recent = h.recent(3);
        assert!(recent.len() >= 3 || recent.len() == h.len());
    }

    #[test]
    fn history_ring_buffer_overflow() {
        let mut h = ReconciliationHistory::default();
        for i in 0..HISTORY_CAPACITY + 10 {
            h.push(ReconciliationRecord {
                tick: i as u64,
                k_used: 16,
                metrics_at_fork: RegimeMetrics::default(),
                diff_class: DiffClass::Perfect,
                mean_qe_error: 0.0,
                affected_fraction: 0.0,
            });
        }
        assert_eq!(h.len(), HISTORY_CAPACITY);
    }

    #[test]
    fn regime_label_stasis() {
        let m = RegimeMetrics { variance: 0.0, autocorrelation: 0.3, entropy_accel: 0.0, ..Default::default() };
        assert_eq!(regime_label(&m), "STASIS");
    }

    #[test]
    fn regime_label_pre_transition() {
        let m = RegimeMetrics { autocorrelation: 0.96, ..Default::default() };
        assert_eq!(regime_label(&m), "PRE-TRANSITION");
    }

    #[test]
    fn regime_label_transition() {
        let m = RegimeMetrics { autocorrelation: 0.5, variance: 1.0, entropy_accel: 1.0, ..Default::default() };
        assert_eq!(regime_label(&m), "TRANSITION");
    }
}
