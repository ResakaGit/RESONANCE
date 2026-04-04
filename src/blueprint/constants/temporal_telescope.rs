//! Constantes derivadas para el Telescopio Temporal (ADR-015).
//! Derived constants for the Temporal Telescope (ADR-015).
//!
//! Every constant derives from the 4 fundamentals in derived_thresholds.rs.

use super::super::equations::derived_thresholds::DISSIPATION_SOLID;

// ─── Window & Timing ─────────────────────────────────────────────────────────

/// Ventana deslizante para estadísticas (ticks). Potencia de 2 para bitwise modulo.
/// Sliding window for statistics (ticks). Power of 2 for bitwise modulo.
pub const TELESCOPE_WINDOW_SIZE: usize = 128;

/// K inicial (ticks a proyectar).
/// Initial K (ticks to project).
pub const TELESCOPE_K_INITIAL: u32 = 16;

/// K mínimo (floor). Nunca proyectar menos de 4 ticks.
/// Minimum K (floor). Never project fewer than 4 ticks.
pub const TELESCOPE_K_MIN: u32 = 4;

/// K máximo (ceiling).
/// Maximum K (ceiling).
pub const TELESCOPE_K_MAX: u32 = 1024;

/// Factor de crecimiento tras reconciliaciones perfectas.
/// Growth factor after perfect reconciliations.
pub const TELESCOPE_K_GROW_FACTOR: f32 = 1.5;

/// Factor de reducción tras reconciliación sistémica.
/// Shrink factor after systemic reconciliation.
pub const TELESCOPE_K_SHRINK_FACTOR: f32 = 0.5;

/// Reconciliaciones perfectas consecutivas necesarias para crecer K.
/// Consecutive perfect reconciliations needed to grow K.
pub const TELESCOPE_PERFECT_STREAK_TO_GROW: u16 = 4;

// ─── Detection Thresholds ────────────────────────────────────────────────────

/// Umbral de autocorrelación para "alta inercia". Derivado: exp(-DISSIPATION_SOLID × 10).
/// Autocorrelation threshold for "high inertia". Derived: exp(-DISSIPATION_SOLID × 10).
pub const RHO1_HIGH_INERTIA: f32 = {
    // exp(-0.005 × 10) = exp(-0.05) ≈ 0.951 — precomputed for const context.
    0.951
};

/// Multiplicador de Fisher para spike distribucional. 3σ sobre mediana.
/// Fisher spike multiplier. 3σ above median.
pub const FISHER_SPIKE_MULTIPLIER: f32 = 3.0;

/// Epsilon para aceleración de entropía. Derivado: DISSIPATION_SOLID².
/// Entropy acceleration epsilon. Derived: DISSIPATION_SOLID².
pub const ENTROPY_ACCELERATION_EPSILON: f32 = DISSIPATION_SOLID * DISSIPATION_SOLID;

// ─── Diff Thresholds ─────────────────────────────────────────────────────────

/// Umbral de diff por entidad (fracción). 2% = 0.02.
/// Per-entity diff threshold (fraction). 2% = 0.02.
pub const DIFF_THRESHOLD_PCT: f32 = 0.02;

/// Fracción de entidades afectadas para clasificar como SYSTEMIC. 10% = 0.10.
/// Fraction of affected entities for SYSTEMIC classification.
pub const DIFF_SYSTEMIC_FRACTION: f32 = 0.10;

// ─── Cascade ─────────────────────────────────────────────────────────────────

/// Hops máximos de cascada. Axioma 7: atenuación por distancia limita la propagación.
/// Maximum cascade hops. Axiom 7: distance attenuation limits propagation.
pub const CASCADE_MAX_HOPS: u8 = 3;

/// Atenuación por hop. Calibración (no derivado de fundamentales).
/// Attenuation per hop. Calibration (not derived from fundamentals).
///
/// Valor 0.1 elegido para que cascada muera en ≤3 hops (0.1³ = 0.001 < DISSIPATION_SOLID).
/// Físicamente: Axioma 7 (atenuación por distancia) justifica decaimiento exponencial;
/// la tasa exacta es calibración del grid, no física fundamental.
pub const CASCADE_ATTENUATION_PER_HOP: f32 = 0.1;

/// Epsilon de corrección: debajo de esto, no propagar. Derivado: DISSIPATION_SOLID.
/// Correction epsilon: below this, don't propagate. Derived: DISSIPATION_SOLID.
pub const CASCADE_CORRECTION_EPSILON: f32 = DISSIPATION_SOLID;

// ─── DFA ─────────────────────────────────────────────────────────────────────

/// Espaciado logarítmico para escalas DFA (fracción del rango log).
/// Logarithmic spacing for DFA scales (fraction of log range).
pub const DFA_LOG_SPACING: f32 = 0.25;

// ─── Regime Classification ───────────────────────────────────────────────────

/// Autocorrelación máxima para clasificar como STASIS.
/// Maximum autocorrelation to classify as STASIS.
pub const REGIME_STASIS_RHO_CEILING: f32 = 0.8;

// ─── Projection ──────────────────────────────────────────────────────────────

/// Radio alométrico máximo (factor sobre growth_bias). Calibración.
/// Maximum allometric radius (factor over growth_bias). Calibration.
pub const PROJECTION_MAX_ALLOMETRIC_RADIUS: f32 = 5.0;

/// Tasa de decaimiento de nutrientes por tick (estimación geológica conservadora).
/// Nutrient decay rate per tick (conservative geological estimate).
///
/// Derivado: DISSIPATION_SOLID / 5 ≈ 0.001. Turnover geológico << biológico.
pub const PROJECTION_NUTRIENT_DECAY_RATE: f32 = DISSIPATION_SOLID / 5.0;

/// Densidad de eventos alta (A-series: cada evento es un branch point).
/// High event density (A-series: each event is a branch point).
pub const EVENT_DENSITY_HIGH: f32 = 5.0;

/// Densidad de eventos media (borderline: un evento puede o no ocurrir).
/// Medium event density (borderline: one event may or may not happen).
pub const EVENT_DENSITY_MEDIUM: f32 = 1.0;

/// Hurst mínimo para considerar persistencia fuerte (seguro extrapolar lejos).
/// Minimum Hurst for strong persistence (safe to extrapolate far).
pub const HURST_SAFE_PERSISTENCE: f32 = 0.7;

// ─── Calibration Bridge ──────────────────────────────────────────────────────

/// Tasa de aprendizaje del puente de calibración.
/// Calibration bridge learning rate.
pub const CALIBRATION_LEARNING_RATE: f32 = 0.1;

/// Mínimo de records antes de ajustar pesos.
/// Minimum records before adjusting weights.
pub const CALIBRATION_MIN_HISTORY: u16 = 8;

/// Peso mínimo de un normalizador (nunca ignorar una métrica).
/// Minimum normalizer weight (never ignore a metric).
pub const CALIBRATION_WEIGHT_FLOOR: f32 = 0.1;

/// Peso máximo de un normalizador (nunca sobreponderar).
/// Maximum normalizer weight (never over-weight).
pub const CALIBRATION_WEIGHT_CEILING: f32 = 5.0;

/// Factor de amortiguación para reconciliaciones perfectas (10% del learning rate).
/// Dampening factor for perfect reconciliations (10% of learning rate).
pub const CALIBRATION_PERFECT_DAMPENING: f32 = 0.1;

/// Factor de sensibilidad entrópica (cuánto afecta entropía al Fisher weight).
/// Entropy sensitivity factor (how much entropy affects Fisher weight).
pub const CALIBRATION_ENTROPY_SENSITIVITY: f32 = 0.5;

// ─── Projection — internal scaling ───────────────────────────────────────────

/// Factor de escala alométrico para growth_bias → tasa de crecimiento.
/// Allometric scale factor for growth_bias → growth rate.
pub const PROJECTION_ALLOMETRIC_SCALE: f32 = 0.01;

/// Floor estacional para irradiancia (previene división por ~0).
/// Seasonal floor for irradiance (prevents near-zero division).
pub const PROJECTION_IRRADIANCE_SEASON_FLOOR: f32 = 0.1;

/// qe de referencia mínimo para cálculo de error relativo en diff.
/// Minimum reference qe for relative error calculation in diff.
pub const DIFF_QE_MIN_REFERENCE: f32 = 1.0;

// ─── Multi-Telescope (ADR-016) ───────────────────────────────────────────────

/// Niveles máximos del stack. 16⁸ ≈ 4.3×10⁹ ticks alcanzables. Zero-heap.
/// Maximum stack levels. 16⁸ ≈ 4.3×10⁹ reachable ticks. Zero-heap.
pub const MAX_LEVELS: usize = 8;

/// Longitud de coherencia base (ticks). Calibración.
/// Base coherence length (ticks). Calibration.
///
/// Modula qué tan rápido decae la visibilidad especulativa con la distancia al ancla.
/// Calibración (no derivado). Valor base 100 ticks para stack responsivo en grillas estándar.
/// El dynamic_coherence_length() lo escala por estabilidad del régimen (H, ρ₁, λ_max).
pub const DEFAULT_COHERENCE_LENGTH: f32 = 100.0;

/// Umbral de reconciliaciones Systemic consecutivas para remover un nivel.
/// Consecutive Systemic reconciliations threshold for level removal.
pub const REMOVAL_SYSTEMIC_THRESHOLD: u16 = 3;

/// Escalas DFA máximas (puntos log-space para regresión).
/// Maximum DFA scales (log-space points for regression).
pub const DFA_MAX_SCALES: usize = 16;

/// Tamaño máximo de serie integrada para DFA (stack-allocated).
/// Maximum integrated series size for DFA (stack-allocated).
pub const DFA_MAX_SERIES: usize = 1024;

/// Divisor de K para densidad alta de eventos (McTaggart A-series).
/// K divisor for high event density (McTaggart A-series).
pub const OPTIMAL_K_HIGH_EVENT_DIVISOR: u32 = 4;

/// Divisor de K para densidad media de eventos.
/// K divisor for medium event density.
pub const OPTIMAL_K_MEDIUM_EVENT_DIVISOR: u32 = 2;

/// Factor de reducción de K por spike de Fisher.
/// K reduction factor for Fisher spike.
pub const OPTIMAL_K_FISHER_DIVISOR: u32 = 2;

/// Factor de reducción de K por aceleración de entropía (3/4).
/// K reduction factor for entropy acceleration (3/4).
pub const OPTIMAL_K_ENTROPY_NUMERATOR: u32 = 3;
pub const OPTIMAL_K_ENTROPY_DENOMINATOR: u32 = 4;

/// Ventana de precisión para dashboard (últimas N reconciliaciones).
/// Accuracy window for dashboard (last N reconciliations).
pub const ACCURACY_WINDOW: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rho1_derived_from_dissipation() {
        let expected = (-DISSIPATION_SOLID * 10.0_f32).exp();
        assert!((RHO1_HIGH_INERTIA - expected).abs() < 0.001);
    }

    #[test]
    fn entropy_epsilon_derived_from_dissipation() {
        assert!((ENTROPY_ACCELERATION_EPSILON - 0.000025).abs() < 1e-7);
    }

    #[test]
    fn k_bounds_consistent() {
        assert!(TELESCOPE_K_MIN < TELESCOPE_K_INITIAL);
        assert!(TELESCOPE_K_INITIAL < TELESCOPE_K_MAX);
    }

    #[test]
    fn cascade_attenuation_damps() {
        let after_3_hops = CASCADE_ATTENUATION_PER_HOP.powi(CASCADE_MAX_HOPS as i32);
        assert!(after_3_hops < CASCADE_CORRECTION_EPSILON,
            "cascade should die within max_hops: {after_3_hops} >= {CASCADE_CORRECTION_EPSILON}");
    }

    #[test]
    fn calibration_floor_below_ceiling() {
        assert!(CALIBRATION_WEIGHT_FLOOR < CALIBRATION_WEIGHT_CEILING);
    }
}
