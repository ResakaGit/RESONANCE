//! Constantes del track Energy Competition (EC-1E).
//! Pool model, extraction thresholds, conservation tolerances.

// --- Energy Competition: Pool ---
pub const DISSIPATION_RATE_MIN: f32 = 0.001;
pub const DISSIPATION_RATE_MAX: f32 = 0.5;
pub const DISSIPATION_RATE_DEFAULT: f32 = 0.01;
pub const POOL_CAPACITY_MIN: f32 = 1.0;

// --- Energy Competition: Extraction ---
pub const EXTRACTION_EPSILON: f32 = 1e-6;
pub const REGULATED_AGGRESSIVE_MULT: f32 = 1.5;
pub const REGULATED_THROTTLE_MULT: f32 = 0.3;
pub const REGULATED_THRESHOLD_LOW_DEFAULT: f32 = 0.3;
pub const REGULATED_THRESHOLD_HIGH_DEFAULT: f32 = 0.7;
pub const AGGRESSION_FACTOR_DEFAULT: f32 = 0.5;
pub const DAMAGE_RATE_DEFAULT: f32 = 0.1;

// --- Energy Competition: Conservation ---
pub const POOL_CONSERVATION_EPSILON: f32 = 1e-3;

// --- Energy Competition: Composition ---
/// Número máximo de modificadores en un ExtractionProfile (DOD: max 4).
pub const MAX_EXTRACTION_MODIFIERS: usize = 4;
