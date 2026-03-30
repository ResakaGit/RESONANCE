//! Particle charge constants — ALL derived from the 4 fundamental constants.
//!
//! Axiom 1: charge IS energy polarity.
//! Axiom 7: Coulomb force ∝ 1/r² (distance attenuation).
//! Axiom 4: Lennard-Jones equilibrium = minimum dissipation configuration.

use super::super::equations::derived_thresholds::{DENSITY_SCALE, DISSIPATION_SOLID};

/// Coulomb force normalization. Derived: 1/DENSITY_SCALE.
/// Scales electromagnetic force to grid units. Same role as G in gravity.
pub const COULOMB_SCALE: f32 = 1.0 / DENSITY_SCALE;

/// Lennard-Jones sigma: equilibrium distance between particles.
/// Derived: 1/DENSITY_SCALE. Particle "size" in grid units.
pub const LJ_SIGMA: f32 = 1.0 / DENSITY_SCALE;

/// Lennard-Jones epsilon: depth of potential well (binding strength).
/// Derived: DISSIPATION_SOLID × 100. Deeper well = stronger bond.
pub const LJ_EPSILON: f32 = DISSIPATION_SOLID * 100.0;

/// Bond energy threshold: pair is stable when |bond_energy| exceeds this.
/// Derived: COULOMB_SCALE × 0.5 = 0.025.
/// Pair with unit charges at distance 1.0 has V = -0.05 → bound (|V| > threshold).
/// Pair at distance 3.0 has V = -0.017 → not bound.
pub const BOND_ENERGY_THRESHOLD: f32 = COULOMB_SCALE * 0.5;

/// Softening parameter: prevents force singularity at r→0.
/// Derived: LJ_SIGMA × 0.1. Small fraction of particle size.
pub const FORCE_SOFTENING: f32 = LJ_SIGMA * 0.1;

/// Maximum force magnitude (prevents numerical instability).
/// Derived: COULOMB_SCALE / (FORCE_SOFTENING²). Force at minimum distance.
pub const MAX_FORCE: f32 = COULOMB_SCALE / (FORCE_SOFTENING * FORCE_SOFTENING);
