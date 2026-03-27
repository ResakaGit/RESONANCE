// ── Senescence constants — derived from dissipation rates (Axiom 4) ──────────
// All values computed by `blueprint/equations/derived_thresholds.rs`.
// Re-exported here for backward compat with spawn paths that import from constants/.

use crate::blueprint::equations::derived_thresholds as dt;

pub fn senescence_coeff_materialized() -> f32 { dt::senescence_coeff_materialized() }
pub fn senescence_max_age_materialized() -> u64 { dt::max_age_materialized() }
pub fn senescence_coeff_flora() -> f32 { dt::senescence_coeff_flora() }
pub fn senescence_max_age_flora() -> u64 { dt::max_age_flora() }
pub fn senescence_coeff_fauna() -> f32 { dt::senescence_coeff_fauna() }
pub fn senescence_max_age_fauna() -> u64 { dt::max_age_fauna() }

/// Default reproduction strategy: 0 = Iteroparous.
/// Derived: optimal_reproduction_strategy() selects per-entity; default = iteroparous.
pub const SENESCENCE_DEFAULT_STRATEGY: u8 = 0;
