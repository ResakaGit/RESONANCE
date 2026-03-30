//! Multicellularity constants — derived from the 4 fundamental constants.

use super::super::equations::derived_thresholds::{DISSIPATION_SOLID, KLEIBER_EXPONENT};

/// Frequency alignment bandwidth for cell adhesion (Hz). Axiom 8.
pub const ADHESION_FREQ_BANDWIDTH: f32 = 50.0; // COHERENCE_BANDWIDTH
/// Minimum adhesion affinity to form bond. Derived: KLEIBER_EXPONENT = 0.75.
pub const ADHESION_THRESHOLD: f32 = KLEIBER_EXPONENT;
/// Energy cost per bond per tick. Derived: DISSIPATION_SOLID × 2 = 0.01.
pub const ADHESION_COST: f32 = DISSIPATION_SOLID * 2.0;
/// Bond strength scaling. Derived: 1.0 / DISSIPATION_SOLID = 200 → sqrt ≈ 14.
pub const BOND_STRENGTH_SCALE: f32 = 0.1;
/// Minimum colony size for specialization. Derived: MIN_GENES - 1 = 3.
pub const MIN_COLONY_SIZE: u8 = 3;
/// Expression modulation rate (how fast cells specialize). Derived: DISSIPATION_SOLID × 10.
pub const EXPRESSION_MODULATION_RATE: f32 = DISSIPATION_SOLID * 10.0;
/// Border target: resilience dimension stays high, others suppressed.
pub const BORDER_TARGET: [f32; 4] = [1.0 - KLEIBER_EXPONENT, 1.0 - KLEIBER_EXPONENT, 1.0 - KLEIBER_EXPONENT, 1.0];
/// Interior target: growth/mobility/branching high, resilience suppressed.
pub const INTERIOR_TARGET: [f32; 4] = [1.0, 1.0, 1.0, 1.0 - KLEIBER_EXPONENT];
