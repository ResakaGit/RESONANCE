//! Telescopio Temporal — ejecución especulativa dual-timeline (ADR-015).
//! Temporal Telescope — dual-timeline speculative execution (ADR-015).
//!
//! Tres componentes stateless:
//! - Telescopio: proyecta el futuro con solvers analíticos + normalizadores.
//! - Ancla: simulación completa tick-a-tick (usa batch pipeline existente).
//! - Puente de calibración: convierte diffs en pesos de normalización.
//!
//! Flujo unidireccional: Ancla → Puente → Telescopio (nunca al revés).

pub mod activation;
pub mod calibration_bridge;
pub mod cascade;
pub mod diff;
pub mod pipeline;
pub mod projection;
pub mod stack;

use crate::blueprint::equations::temporal_telescope::{NormalizerWeights, RegimeMetrics};

pub use diff::{DiffClass, DiffReport, EntityDiff};

// ─── TT-5: Telescope State ──────────────────────────────────────────────────

/// Fase actual del telescopio.
/// Current telescope phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelescopePhase {
    /// Telescopio tiene estado especulativo, ancla en background.
    Projecting,
    /// Ancla alcanzó, comparando.
    Reconciling,
    /// Aplicando cascada de correcciones.
    Correcting,
    /// Sin proyección activa (K=0 o deshabilitado).
    Idle,
}

/// Estado completo del telescopio. Transiciones producen nuevo estado (inmutable semánticamente).
/// Complete telescope state. Transitions produce new state (semantically immutable).
#[derive(Clone, Debug)]
pub struct TelescopeState {
    pub phase: TelescopePhase,
    pub current_k: u32,
    pub fork_tick: u64,
    pub consecutive_perfect: u16,
    pub consecutive_systemic: u16,
    pub total_reconciliations: u32,
    pub last_metrics: RegimeMetrics,
    pub weights: NormalizerWeights,
}

impl Default for TelescopeState {
    fn default() -> Self {
        use crate::blueprint::constants::temporal_telescope::TELESCOPE_K_INITIAL;
        Self {
            phase: TelescopePhase::Idle,
            current_k: TELESCOPE_K_INITIAL,
            fork_tick: 0,
            consecutive_perfect: 0,
            consecutive_systemic: 0,
            total_reconciliations: 0,
            last_metrics: RegimeMetrics::default(),
            weights: NormalizerWeights::default(),
        }
    }
}

/// Registro de una reconciliación (dato de entrenamiento para el puente).
/// Reconciliation record (training datum for the bridge).
#[derive(Clone, Copy, Debug)]
pub struct ReconciliationRecord {
    pub tick: u64,
    pub k_used: u32,
    pub metrics_at_fork: RegimeMetrics,
    pub diff_class: DiffClass,
    pub mean_qe_error: f32,
    pub affected_fraction: f32,
}

/// Config inmutable del telescopio.
/// Immutable telescope config.
#[derive(Clone, Copy, Debug)]
pub struct TelescopeConfig {
    pub k_min: u32,
    pub k_max: u32,
    pub k_initial: u32,
    pub grow_factor: f32,
    pub shrink_factor: f32,
    pub perfect_streak_to_grow: u16,
}

impl Default for TelescopeConfig {
    fn default() -> Self {
        use crate::blueprint::constants::temporal_telescope as c;
        Self {
            k_min: c::TELESCOPE_K_MIN,
            k_max: c::TELESCOPE_K_MAX,
            k_initial: c::TELESCOPE_K_INITIAL,
            grow_factor: c::TELESCOPE_K_GROW_FACTOR,
            shrink_factor: c::TELESCOPE_K_SHRINK_FACTOR,
            perfect_streak_to_grow: c::TELESCOPE_PERFECT_STREAK_TO_GROW,
        }
    }
}

/// Transición de estado post-reconciliación. Pura: (state, record, config) → state.
/// Post-reconciliation state transition. Pure: (state, record, config) → state.
pub fn telescope_after_reconciliation(
    state: &TelescopeState,
    record: &ReconciliationRecord,
    config: &TelescopeConfig,
) -> TelescopeState {
    if state.phase == TelescopePhase::Idle {
        return state.clone();
    }
    let mut next = state.clone();
    next.total_reconciliations = state.total_reconciliations.saturating_add(1);

    match record.diff_class {
        DiffClass::Perfect => {
            next.consecutive_perfect = state.consecutive_perfect.saturating_add(1);
            next.consecutive_systemic = 0;
        }
        DiffClass::Local => {
            next.consecutive_perfect = 0;
            next.consecutive_systemic = 0;
        }
        DiffClass::Systemic => {
            next.consecutive_perfect = 0;
            next.consecutive_systemic = state.consecutive_systemic.saturating_add(1);
        }
    }

    next.current_k = next_k(&next, config);
    next.phase = TelescopePhase::Projecting;
    next
}

/// Calcula K para el próximo fork. Pura: (state, config) → u32.
/// Computes K for the next fork. Pure: (state, config) → u32.
pub fn next_k(state: &TelescopeState, config: &TelescopeConfig) -> u32 {
    let mut k = state.current_k;

    if state.consecutive_perfect >= config.perfect_streak_to_grow {
        k = (k as f32 * config.grow_factor) as u32;
    }
    if state.consecutive_systemic > 0 {
        k = (k as f32 * config.shrink_factor) as u32;
    }

    k.clamp(config.k_min, config.k_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(k: u32, perfect: u16, systemic: u16) -> TelescopeState {
        TelescopeState {
            phase: TelescopePhase::Projecting,
            current_k: k,
            consecutive_perfect: perfect,
            consecutive_systemic: systemic,
            ..Default::default()
        }
    }

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

    #[test]
    fn next_k_grows_after_perfect_streak() {
        let config = TelescopeConfig::default();
        let state = make_state(16, config.perfect_streak_to_grow, 0);
        let k = next_k(&state, &config);
        assert_eq!(k, (16.0 * config.grow_factor) as u32);
    }

    #[test]
    fn next_k_shrinks_after_systemic() {
        let config = TelescopeConfig::default();
        let state = make_state(64, 0, 1);
        let k = next_k(&state, &config);
        assert_eq!(k, (64.0 * config.shrink_factor) as u32);
    }

    #[test]
    fn next_k_never_below_min() {
        let config = TelescopeConfig::default();
        let state = make_state(config.k_min, 0, 5);
        let k = next_k(&state, &config);
        assert!(k >= config.k_min);
    }

    #[test]
    fn next_k_never_above_max() {
        let config = TelescopeConfig::default();
        let state = make_state(config.k_max, 100, 0);
        let k = next_k(&state, &config);
        assert!(k <= config.k_max);
    }

    #[test]
    fn reconciliation_perfect_increments_streak() {
        let config = TelescopeConfig::default();
        let state = make_state(16, 2, 0);
        let record = make_record(DiffClass::Perfect);
        let next = telescope_after_reconciliation(&state, &record, &config);
        assert_eq!(next.consecutive_perfect, 3);
        assert_eq!(next.consecutive_systemic, 0);
        assert_eq!(next.total_reconciliations, 1);
    }

    #[test]
    fn reconciliation_local_resets_streaks() {
        let config = TelescopeConfig::default();
        let state = make_state(16, 5, 0);
        let record = make_record(DiffClass::Local);
        let next = telescope_after_reconciliation(&state, &record, &config);
        assert_eq!(next.consecutive_perfect, 0);
        assert_eq!(next.consecutive_systemic, 0);
    }

    #[test]
    fn reconciliation_systemic_increments_systemic_streak() {
        let config = TelescopeConfig::default();
        let state = make_state(64, 3, 1);
        let record = make_record(DiffClass::Systemic);
        let next = telescope_after_reconciliation(&state, &record, &config);
        assert_eq!(next.consecutive_perfect, 0);
        assert_eq!(next.consecutive_systemic, 2);
    }

    #[test]
    fn idle_state_stays_idle() {
        let config = TelescopeConfig::default();
        let state = TelescopeState { phase: TelescopePhase::Idle, ..Default::default() };
        let record = make_record(DiffClass::Perfect);
        let next = telescope_after_reconciliation(&state, &record, &config);
        assert_eq!(next.phase, TelescopePhase::Idle);
    }

    #[test]
    fn default_config_consistent() {
        let c = TelescopeConfig::default();
        assert!(c.k_min < c.k_initial);
        assert!(c.k_initial < c.k_max);
        assert!(c.grow_factor > 1.0);
        assert!(c.shrink_factor > 0.0 && c.shrink_factor < 1.0);
    }
}
