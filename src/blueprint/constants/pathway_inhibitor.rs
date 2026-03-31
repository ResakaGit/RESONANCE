//! Constantes derivadas para inhibición de pathway (PI).
//! Derived constants for pathway inhibition (PI).
//!
//! Every constant derives from the 4 fundamentals in derived_thresholds.rs.
//! No hardcoded values. No magic numbers.

use crate::blueprint::equations::derived_thresholds::{
    COHERENCE_BANDWIDTH, DISSIPATION_SOLID, KLEIBER_EXPONENT,
};

/// Ancho de banda de binding fármaco-proteína. Axioma 8.
/// Drug-protein binding bandwidth. Axiom 8.
///
/// Same window as all Axiom 8 interactions. Reuse, not reinvent.
pub const INHIBITION_BANDWIDTH: f32 = COHERENCE_BANDWIDTH;

/// Ki por defecto (concentración para 50% ocupación a afinidad perfecta).
/// Default Ki (concentration for 50% occupancy at perfect affinity).
///
/// Derived: `DISSIPATION_SOLID × 200 = 1.0`. The metabolic amplification factor
/// (same inverse as basal_drain_rate). Ki=1 means EC50 at unit concentration.
pub const DEFAULT_KI: f32 = DISSIPATION_SOLID * 200.0;

/// Multiplicador de energía de activación aparente (inhibición competitiva).
/// Apparent activation energy multiplier (competitive inhibition).
///
/// Derived: `1/KLEIBER_EXPONENT = 1.333`. Larger organisms experience
/// proportionally larger metabolic barrier increase from competitive inhibitors.
pub const COMPETITIVE_EA_MULTIPLIER: f32 = 1.0 / KLEIBER_EXPONENT;

/// Umbral de off-target: ocupación debajo de esto es despreciable.
/// Off-target threshold: occupancy below this is negligible.
///
/// Derived: `DISSIPATION_SOLID × 40 = 0.2`. Same as NODE_EXPRESSION_THRESHOLD
/// in metabolic_genome.rs (threshold for "something real is happening").
pub const OFF_TARGET_THRESHOLD: f32 = DISSIPATION_SOLID * 40.0;

/// Costo de disipación por mantener el binding. Axioma 4.
/// Dissipation cost per binding maintenance. Axiom 4.
///
/// Derived: `DISSIPATION_SOLID × 4 = 0.02`. Same order as CATALYSIS_COST_FRACTION.
/// Drug binding and catalysis are thermodynamically symmetric.
pub const INHIBITION_DISSIPATION_COST: f32 = DISSIPATION_SOLID * 4.0;

/// Fracción máxima de inhibición (nunca llega a cero). Axioma 4.
/// Maximum inhibition fraction (never reaches zero). Axiom 4.
///
/// Derived: `1.0 - DISSIPATION_SOLID = 0.995`. Even perfect inhibition
/// leaves residual efficiency — Second Law guarantees minimum dissipation.
pub const MAX_INHIBITION_FRACTION: f32 = 1.0 - DISSIPATION_SOLID;

/// Eficiencia residual mínima bajo inhibición. Axioma 4.
/// Minimum residual efficiency under inhibition. Axiom 4.
///
/// Derived: `DISSIPATION_SOLID = 0.005`. Floor for inhibited pathway.
/// No process can be completely stopped — energy always leaks.
pub const MIN_RESIDUAL_EFFICIENCY: f32 = DISSIPATION_SOLID;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_derived_from_fundamentals() {
        assert!((DEFAULT_KI - 1.0).abs() < 1e-5);
        assert!((COMPETITIVE_EA_MULTIPLIER - 1.333_333_3).abs() < 1e-4);
        assert!((OFF_TARGET_THRESHOLD - 0.2).abs() < 1e-5);
        assert!((INHIBITION_DISSIPATION_COST - 0.02).abs() < 1e-5);
        assert!((MAX_INHIBITION_FRACTION - 0.995).abs() < 1e-5);
        assert!((MIN_RESIDUAL_EFFICIENCY - 0.005).abs() < 1e-5);
    }

    #[test]
    fn bandwidth_reuses_coherence() {
        assert!((INHIBITION_BANDWIDTH - COHERENCE_BANDWIDTH).abs() < 1e-5);
    }

    #[test]
    fn max_inhibition_plus_residual_equals_one() {
        assert!((MAX_INHIBITION_FRACTION + MIN_RESIDUAL_EFFICIENCY - 1.0).abs() < 1e-5);
    }
}
