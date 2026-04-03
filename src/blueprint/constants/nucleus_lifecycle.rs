// ── Nucleus lifecycle constants ───────────────────────────────────────────────
// Pressure constants derived from axioms via derived_thresholds.rs.
// Recycling constants now also axiom-derived (conservation-respecting).

use crate::blueprint::equations::derived_thresholds as dt;

/// Minimum reservoir below which a nucleus stops emitting.
pub const NUCLEUS_EMISSION_CUTOFF_QE: f32 = 1.0;

/// 1.0 = fully finite. 0.0 = legacy infinite emission.
pub const NUCLEUS_DEPLETION_FACTOR: f32 = 1.0;

// ── Radiation pressure — derived from dissipation rates (Axiom 4) ────────────

/// Pressure activates at gas density transition (derived).
pub fn radiation_pressure_threshold_qe() -> f32 {
    dt::radiation_pressure_threshold()
}

/// Transfer rate = gas dissipation rate (derived).
pub fn radiation_pressure_transfer_rate() -> f32 {
    dt::radiation_pressure_transfer_rate()
}

// ── Nucleus recycling — axiom-derived (conservation-respecting) ──────────────

/// Nutrient density threshold: sum of conversion losses (derived).
pub fn nucleus_recycling_nutrient_threshold() -> f32 {
    dt::recycling_nutrient_threshold()
}

/// Max cells checked per tick for recycling candidates (structural budget).
pub const NUCLEUS_RECYCLING_SCAN_BUDGET: usize = 16;
