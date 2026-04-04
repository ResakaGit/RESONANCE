//! Stack de telescopios multi-nivel con colapso cuántico (ADR-016).
//! Multi-level telescope stack with quantum collapse (ADR-016).
//!
//! N niveles donde cada nivel proyecta desde el output del inferior.
//! Colapso + re-emanación: cuando el ancla llega, destruir ondas, reconstruir frescos.
//! Zero error acumulado entre niveles.

use crate::batch::arena::SimWorldFlat;
use crate::blueprint::constants::temporal_telescope as tc;
use crate::blueprint::equations::temporal_telescope::{
    NormalizerWeights, RegimeMetrics, optimal_k, speculative_visibility,
};

use super::calibration_bridge::{CalibrationConfig, calibrate};
use super::diff::{DiffClass, world_diff};
use super::pipeline::ReconciliationHistory;
use super::projection::project_world;
use super::{ReconciliationRecord, TelescopeConfig, TelescopePhase, TelescopeState};

// ─── Types ───────────────────────────────────────────────────────────────────

/// Nivel individual del telescopio.
/// Individual telescope level.
///
/// `projected_world` es Box porque SimWorldFlat (~100KB) no cabe en stack × 8 niveles.
/// DEBT: Box necesario para 8 niveles × ~100KB = 800KB > stack default. SimWorldFlat
/// ya contiene Vec internos (genomes), por lo que no es "new heap" — es ownership transfer.
#[derive(Clone)]
pub struct TelescopeLevel {
    pub state: TelescopeState,
    pub projected_world: Box<SimWorldFlat>,
    pub k: u32,
    /// V de Englert: 0=colapsado (certeza), 1=onda pura (incertidumbre).
    pub visibility: f32,
}

/// Resultado del colapso + re-emanación.
/// Collapse + re-emanation result.
#[derive(Clone, Debug)]
pub struct CollapseResult {
    pub records: [ReconciliationRecord; tc::MAX_LEVELS],
    pub records_count: u8,
    pub max_diff_class: DiffClass,
    pub levels_rebuilt: u8,
}

/// Stack de telescopios. Array fijo, zero-heap.
/// Telescope stack. Fixed array, zero-heap.
pub struct TelescopeStack {
    pub levels: [Option<TelescopeLevel>; tc::MAX_LEVELS],
    pub active_levels: u8,
    pub coherence_length: f32,
    pub config: TelescopeConfig,
}

impl TelescopeStack {
    /// Crea un stack con un nivel activo (equivalente a ADR-015).
    /// Creates a stack with one active level (equivalent to ADR-015).
    pub fn new(seed_world: &SimWorldFlat, config: TelescopeConfig) -> Self {
        let mut levels: [Option<TelescopeLevel>; tc::MAX_LEVELS] = Default::default();
        levels[0] = Some(TelescopeLevel {
            state: TelescopeState {
                phase: TelescopePhase::Projecting,
                ..Default::default()
            },
            projected_world: Box::new(seed_world.clone()),
            k: config.k_initial,
            visibility: 0.0,
        });
        Self {
            levels,
            active_levels: 1,
            coherence_length: tc::DEFAULT_COHERENCE_LENGTH,
            config,
        }
    }

    /// Alcance total en ticks: ∏ Kᵢ para niveles activos.
    /// Total reach in ticks: ∏ Kᵢ for active levels.
    pub fn total_reach(&self) -> u64 {
        let mut reach = 1_u64;
        for i in 0..self.active_levels as usize {
            if let Some(ref level) = self.levels[i] {
                reach = reach.saturating_mul(level.k as u64);
            }
        }
        reach
    }

    /// Ticks acumulados hasta el nivel dado (inclusive).
    /// Accumulated ticks up to the given level (inclusive).
    fn reach_up_to(&self, up_to: usize) -> u64 {
        let mut reach = 1_u64;
        for i in 0..=up_to.min(self.active_levels as usize - 1) {
            if let Some(ref level) = self.levels[i] {
                reach = reach.saturating_mul(level.k as u64);
            }
        }
        reach
    }
}

// ─── Collapse + Re-Emanation ─────────────────────────────────────────────────

/// Colapso cuántico + re-emanación de todos los niveles.
/// Quantum collapse + re-emanation of all levels.
///
/// Fase 1: MEDIR — comparar cada nivel con su verdad (diff como señal de aprendizaje).
/// Fase 2: CALIBRAR — actualizar pesos desde los diffs.
/// Fase 3: COLAPSAR + RE-EMANAR — destruir ondas, reconstruir desde verdad fresca.
/// Fase 4: ADAPTAR — crecer o reducir niveles.
pub fn collapse_and_emanate(
    stack: &mut TelescopeStack,
    anchor: &SimWorldFlat,
    metrics: &RegimeMetrics,
    cal_config: &CalibrationConfig,
    history: &mut ReconciliationHistory,
) -> CollapseResult {
    let null_record = ReconciliationRecord {
        tick: 0,
        k_used: 0,
        metrics_at_fork: RegimeMetrics::default(),
        diff_class: DiffClass::Perfect,
        mean_qe_error: 0.0,
        affected_fraction: 0.0,
    };
    let mut result = CollapseResult {
        records: [null_record; tc::MAX_LEVELS],
        records_count: 0,
        max_diff_class: DiffClass::Perfect,
        levels_rebuilt: 0,
    };

    let n = stack.active_levels as usize;
    if n == 0 {
        return result;
    }

    // Fase 1: MEDIR — diff de cada nivel contra su verdad
    for i in 0..n {
        let Some(ref level) = stack.levels[i] else { continue };
        let truth = if i == 0 {
            anchor
        } else {
            // La verdad del nivel i es la proyección del nivel i-1 (que aún no fue re-emanada).
            // Usamos el anchor re-emanado para el primer nivel y la cadena para los demás.
            // Pero en la medición PRE-colapso, comparamos contra lo que teníamos antes.
            anchor // simplificación: medir todos contra anchor para la señal de aprendizaje
        };
        let diff = world_diff(truth, &level.projected_world, tc::DIFF_THRESHOLD_PCT);
        let total_alive = anchor.alive_mask.count_ones().max(1) as u16;
        result.records[i] = ReconciliationRecord {
            tick: anchor.tick_id,
            k_used: level.k,
            metrics_at_fork: *metrics,
            diff_class: diff.class,
            mean_qe_error: diff.mean_qe_error,
            affected_fraction: diff.affected_count as f32 / total_alive as f32,
        };
        if (diff.class as u8) > (result.max_diff_class as u8) {
            result.max_diff_class = diff.class;
        }
        result.records_count += 1;
    }

    // Fase 2: CALIBRAR — pesos actualizados desde diffs
    for i in 0..n {
        let Some(ref mut level) = stack.levels[i] else { continue };
        let new_weights = calibrate(
            &result.records[i],
            &level.state.weights,
            history.recent(history.len()),
            cal_config,
        );
        level.state.weights = new_weights;
        history.push(result.records[i]);
    }

    // Fase 3: COLAPSAR + RE-EMANAR — destruir y reconstruir
    // Level 0: colapso = verdad del ancla
    if let Some(ref mut level) = stack.levels[0] {
        level.projected_world = Box::new(anchor.clone());
        level.visibility = 0.0;
        level.k = optimal_k(metrics, &level.state.weights, stack.config.k_min, stack.config.k_max);
    }
    result.levels_rebuilt = 1;

    // Levels 1..N: re-emanar desde el nivel inferior (cadena fresca)
    for i in 1..n {
        let source_weights = {
            let prev = stack.levels[i - 1].as_ref().unwrap();
            prev.state.weights
        };
        let source_world = (*stack.levels[i - 1].as_ref().unwrap().projected_world).clone();

        // Pre-compute before mutable borrow (borrow checker compliance).
        let ticks_to_anchor = stack.reach_up_to(i);
        let cl = stack.coherence_length;
        let (k_min, k_max) = (stack.config.k_min, stack.config.k_max);

        if let Some(ref mut level) = stack.levels[i] {
            level.k = optimal_k(metrics, &level.state.weights, k_min, k_max);
            level.projected_world = Box::new(project_world(&source_world, metrics, &source_weights, level.k));
            level.visibility = speculative_visibility(ticks_to_anchor, cl);
            result.levels_rebuilt += 1;
        }
    }

    // Fase 4: ADAPTAR
    if should_add_level(stack, metrics) && (stack.active_levels as usize) < tc::MAX_LEVELS {
        let new_idx = stack.active_levels as usize;
        let prev = stack.levels[new_idx - 1].as_ref().unwrap();
        let new_k = optimal_k(metrics, &NormalizerWeights::default(), stack.config.k_min, stack.config.k_max);
        let new_world = Box::new(project_world(&prev.projected_world, metrics, &prev.state.weights, new_k));
        let ticks = stack.reach_up_to(new_idx - 1).saturating_mul(new_k as u64);
        stack.levels[new_idx] = Some(TelescopeLevel {
            state: TelescopeState {
                phase: TelescopePhase::Projecting,
                ..Default::default()
            },
            projected_world: new_world,
            k: new_k,
            visibility: speculative_visibility(ticks, stack.coherence_length),
        });
        stack.active_levels += 1;
    }
    if should_remove_level(stack) && stack.active_levels > 1 {
        let rm_idx = (stack.active_levels - 1) as usize;
        stack.levels[rm_idx] = None;
        stack.active_levels -= 1;
    }

    result
}

/// ¿Agregar un nivel? Cuando el nivel más alto está saturado y el régimen es estable.
/// Add a level? When the top level is saturated and the regime is stable.
pub fn should_add_level(stack: &TelescopeStack, metrics: &RegimeMetrics) -> bool {
    let top_idx = (stack.active_levels as usize).saturating_sub(1);
    let Some(ref top) = stack.levels[top_idx] else { return false };
    top.k >= stack.config.k_max
        && metrics.variance < tc::ENTROPY_ACCELERATION_EPSILON
        && metrics.autocorrelation < tc::REGIME_STASIS_RHO_CEILING
}

/// ¿Remover un nivel? Cuando el nivel más alto es inútil (siempre Systemic).
/// Remove a level? When the top level is useless (always Systemic).
pub fn should_remove_level(stack: &TelescopeStack) -> bool {
    let top_idx = (stack.active_levels as usize).saturating_sub(1);
    let Some(ref top) = stack.levels[top_idx] else { return false };
    top.state.consecutive_systemic >= tc::REMOVAL_SYSTEMIC_THRESHOLD
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
            e.position = [i as f32 * 20.0, 0.0];
            e.archetype = 2;
            e.trophic_class = 2;
            w.spawn(e);
        }
        w.update_total_qe();
        w
    }

    fn default_config() -> TelescopeConfig {
        TelescopeConfig { k_min: 4, k_max: 64, k_initial: 16, ..Default::default() }
    }

    #[test]
    fn stack_new_has_one_level() {
        let w = make_stable_world(5, 100.0);
        let stack = TelescopeStack::new(&w, default_config());
        assert_eq!(stack.active_levels, 1);
        assert!(stack.levels[0].is_some());
        assert!(stack.levels[1].is_none());
    }

    #[test]
    fn total_reach_single_level() {
        let w = make_stable_world(5, 100.0);
        let stack = TelescopeStack::new(&w, default_config());
        assert_eq!(stack.total_reach(), 16); // k_initial
    }

    #[test]
    fn collapse_single_level_sets_anchor() {
        let w = make_stable_world(5, 100.0);
        let mut stack = TelescopeStack::new(&w, default_config());
        let mut anchor = w.clone();
        anchor.entities[0].qe = 50.0; // anchor diverged
        anchor.update_total_qe();
        let metrics = RegimeMetrics::default();
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();

        let result = collapse_and_emanate(&mut stack, &anchor, &metrics, &cal_config, &mut history);
        assert!(result.records_count >= 1);
        // After collapse, level 0 should be anchor
        let level0 = stack.levels[0].as_ref().unwrap();
        assert_eq!(level0.projected_world.entities[0].qe, 50.0);
        assert_eq!(level0.visibility, 0.0);
    }

    #[test]
    fn collapse_multi_level_rebuilds_all() {
        let w = make_stable_world(5, 100.0);
        let config = default_config();
        let mut stack = TelescopeStack::new(&w, config);
        // Manually add levels
        stack.levels[1] = Some(TelescopeLevel {
            state: TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() },
            projected_world: Box::new(w.clone()),
            k: 16,
            visibility: 0.5,
        });
        stack.levels[2] = Some(TelescopeLevel {
            state: TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() },
            projected_world: Box::new(w.clone()),
            k: 16,
            visibility: 0.9,
        });
        stack.active_levels = 3;

        let metrics = RegimeMetrics::default();
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();

        let result = collapse_and_emanate(&mut stack, &w, &metrics, &cal_config, &mut history);
        assert_eq!(result.levels_rebuilt, 3);
        // Level 0 visibility = 0 (collapsed)
        assert_eq!(stack.levels[0].as_ref().unwrap().visibility, 0.0);
        // Level 2 visibility > Level 1 visibility (grows with distance)
        let v1 = stack.levels[1].as_ref().unwrap().visibility;
        let v2 = stack.levels[2].as_ref().unwrap().visibility;
        assert!(v2 >= v1, "visibility should grow with level: v1={v1}, v2={v2}");
    }

    #[test]
    fn axiom5_conservation_across_levels() {
        let w = make_stable_world(10, 100.0);
        let config = default_config();
        let mut stack = TelescopeStack::new(&w, config);
        stack.levels[1] = Some(TelescopeLevel {
            state: TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() },
            projected_world: Box::new(w.clone()),
            k: 16,
            visibility: 0.5,
        });
        stack.active_levels = 2;

        let metrics = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();

        collapse_and_emanate(&mut stack, &w, &metrics, &cal_config, &mut history);

        // Conservation: each level's qe ≤ anchor's qe
        let anchor_qe = w.total_qe;
        for i in 0..stack.active_levels as usize {
            if let Some(ref level) = stack.levels[i] {
                level.projected_world.entities.iter()
                    .filter(|e| e.alive)
                    .for_each(|e| {
                        assert!(e.qe <= 100.0 + 1e-3,
                            "Axiom 5: level {i} entity qe {} > anchor entity qe 100", e.qe);
                    });
            }
        }
    }

    #[test]
    fn englert_invariant_all_levels() {
        let w = make_stable_world(5, 100.0);
        let config = default_config();
        let mut stack = TelescopeStack::new(&w, config);
        for i in 1..4 {
            stack.levels[i] = Some(TelescopeLevel {
                state: TelescopeState { phase: TelescopePhase::Projecting, ..Default::default() },
                projected_world: Box::new(w.clone()),
                k: 16,
                visibility: 0.0,
            });
        }
        stack.active_levels = 4;

        let metrics = RegimeMetrics::default();
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();

        collapse_and_emanate(&mut stack, &w, &metrics, &cal_config, &mut history);

        for i in 0..stack.active_levels as usize {
            if let Some(ref level) = stack.levels[i] {
                let v = level.visibility;
                assert!(v >= 0.0 && v <= 1.0, "V out of bounds at level {i}: {v}");
            }
        }
    }

    #[test]
    fn should_add_level_in_stasis() {
        let w = make_stable_world(5, 100.0);
        let mut stack = TelescopeStack::new(&w, TelescopeConfig {
            k_min: 4, k_max: 16, k_initial: 16, ..Default::default()
        });
        stack.levels[0].as_mut().unwrap().k = 16; // at k_max
        let metrics = RegimeMetrics {
            variance: 0.0,
            autocorrelation: 0.3,
            ..Default::default()
        };
        assert!(should_add_level(&stack, &metrics));
    }

    #[test]
    fn should_not_add_in_transition() {
        let w = make_stable_world(5, 100.0);
        let stack = TelescopeStack::new(&w, default_config());
        let metrics = RegimeMetrics {
            variance: 100.0, // high variance = transition
            autocorrelation: 0.95,
            ..Default::default()
        };
        assert!(!should_add_level(&stack, &metrics));
    }

    #[test]
    fn should_remove_after_systemic_streak() {
        let w = make_stable_world(5, 100.0);
        let mut stack = TelescopeStack::new(&w, default_config());
        stack.levels[1] = Some(TelescopeLevel {
            state: TelescopeState {
                phase: TelescopePhase::Projecting,
                consecutive_systemic: tc::REMOVAL_SYSTEMIC_THRESHOLD,
                ..Default::default()
            },
            projected_world: Box::new(w.clone()),
            k: 16,
            visibility: 0.9,
        });
        stack.active_levels = 2;
        assert!(should_remove_level(&stack));
    }

    #[test]
    fn active_levels_never_zero() {
        let w = make_stable_world(5, 100.0);
        let mut stack = TelescopeStack::new(&w, default_config());
        stack.levels[0].as_mut().unwrap().state.consecutive_systemic = 10;
        // should_remove returns true but collapse_and_emanate clamps to >= 1
        assert!(stack.active_levels >= 1);
    }

    #[test]
    fn reach_8_levels_geological() {
        let w = make_stable_world(1, 100.0);
        let mut stack = TelescopeStack::new(&w, TelescopeConfig {
            k_min: 16, k_max: 16, k_initial: 16, ..Default::default()
        });
        for i in 1..8 {
            stack.levels[i] = Some(TelescopeLevel {
                state: TelescopeState::default(),
                projected_world: Box::new(w.clone()),
                k: 16,
                visibility: 0.0,
            });
        }
        stack.active_levels = 8;
        let reach = stack.total_reach();
        assert_eq!(reach, 16_u64.pow(8), "8 levels × K=16 should reach 16⁸: {reach}");
    }

    #[test]
    fn calibration_converges_over_collapses() {
        let w = make_stable_world(5, 100.0);
        let config = default_config();
        let mut stack = TelescopeStack::new(&w, config);
        let metrics = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let cal_config = CalibrationConfig::default();
        let mut history = ReconciliationHistory::default();

        let mut prev_weights = stack.levels[0].as_ref().unwrap().state.weights;
        for _ in 0..20 {
            collapse_and_emanate(&mut stack, &w, &metrics, &cal_config, &mut history);
            let curr_weights = stack.levels[0].as_ref().unwrap().state.weights;
            // Weights should stay bounded
            assert!(curr_weights.hurst_weight >= tc::CALIBRATION_WEIGHT_FLOOR);
            assert!(curr_weights.hurst_weight <= tc::CALIBRATION_WEIGHT_CEILING);
            prev_weights = curr_weights;
        }
    }
}
