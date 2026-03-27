//! Surrogate reliability thresholds (R8).

/// Maximum acceptable relative error for fitness surrogate vs exact.
pub const SURROGATE_FITNESS_EPSILON: f32 = 0.05; // 5% relative error

/// Maximum acceptable absolute error for energy surrogate.
pub const SURROGATE_ENERGY_EPSILON: f32 = 10.0; // 10 qe absolute error

/// Minimum acceptable cache hit rate.
pub const SURROGATE_MIN_HIT_RATE: f32 = 0.70; // 70% hit rate minimum

/// Epsilon for top-K convergence check.
pub const SURROGATE_TOP_K_EPSILON: f32 = 0.01; // 1% for top-K matching
