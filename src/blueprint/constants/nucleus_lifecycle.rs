// ── Nucleus lifecycle constants ───────────────────────────────────────────────

/// Default fuel reservoir for a nucleus (qe). Determines total energy a nucleus can emit.
pub const NUCLEUS_DEFAULT_RESERVOIR_QE: f32 = 50_000.0;

/// Minimum reservoir below which a nucleus stops emitting.
pub const NUCLEUS_EMISSION_CUTOFF_QE: f32 = 1.0;

/// Fraction of emission that comes from the reservoir (vs infinite).
/// 1.0 = fully finite. 0.0 = legacy infinite emission.
pub const NUCLEUS_DEPLETION_FACTOR: f32 = 1.0;

// ── Radiation pressure constants ─────────────────────────────────────────────

/// qe threshold above which a cell exerts outward pressure on neighbors.
pub const RADIATION_PRESSURE_THRESHOLD_QE: f32 = 100.0;

/// Fraction of excess qe transferred per tick to each neighbor.
pub const RADIATION_PRESSURE_TRANSFER_RATE: f32 = 0.05;

// ── Nucleus recycling constants ──────────────────────────────────────────────

/// Nutrient accumulation threshold to spawn a new nucleus from dead matter.
pub const NUCLEUS_RECYCLING_NUTRIENT_THRESHOLD: f32 = 0.8;

/// Energy of the recycled nucleus (qe/s emission rate).
pub const NUCLEUS_RECYCLING_EMISSION_RATE: f32 = 80.0;

/// Reservoir of the recycled nucleus (smaller than primordial).
pub const NUCLEUS_RECYCLING_RESERVOIR_QE: f32 = 10_000.0;

/// Propagation radius of the recycled nucleus.
pub const NUCLEUS_RECYCLING_RADIUS: f32 = 12.0;

/// Scan budget: max cells checked per tick for recycling candidates.
pub const NUCLEUS_RECYCLING_SCAN_BUDGET: usize = 16;
