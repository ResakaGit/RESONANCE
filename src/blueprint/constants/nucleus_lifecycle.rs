// ── Nucleus lifecycle constants ───────────────────────────────────────────────
// Pressure constants derived from axioms via derived_thresholds.rs.
// Reservoir/recycling constants are gameplay tuning (not axiomatically derivable).

use crate::blueprint::equations::derived_thresholds as dt;

/// Default fuel reservoir for a nucleus (qe). Gameplay tuning.
pub const NUCLEUS_DEFAULT_RESERVOIR_QE: f32 = 15_000.0;

/// Minimum reservoir below which a nucleus stops emitting.
pub const NUCLEUS_EMISSION_CUTOFF_QE: f32 = 1.0;

/// 1.0 = fully finite. 0.0 = legacy infinite emission.
pub const NUCLEUS_DEPLETION_FACTOR: f32 = 1.0;

// ── Radiation pressure — derived from dissipation rates (Axiom 4) ────────────

/// Pressure activates at gas density transition (derived).
pub fn radiation_pressure_threshold_qe() -> f32 { dt::radiation_pressure_threshold() }

/// Transfer rate = gas dissipation rate (derived).
pub fn radiation_pressure_transfer_rate() -> f32 { dt::radiation_pressure_transfer_rate() }

// ── Nucleus recycling — gameplay tuning (not axiomatically derivable) ─────────

/// Nutrient density threshold to spawn a recycled nucleus.
pub const NUCLEUS_RECYCLING_NUTRIENT_THRESHOLD: f32 = 0.5;

/// Recycled nucleus emission rate (qe/s).
pub const NUCLEUS_RECYCLING_EMISSION_RATE: f32 = 80.0;

/// Recycled nucleus fuel reservoir (smaller than primordial).
pub const NUCLEUS_RECYCLING_RESERVOIR_QE: f32 = 10_000.0;

/// Recycled nucleus propagation radius.
pub const NUCLEUS_RECYCLING_RADIUS: f32 = 12.0;

/// Max cells checked per tick for recycling candidates (structural).
pub const NUCLEUS_RECYCLING_SCAN_BUDGET: usize = 16;
