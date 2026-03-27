// ── Senescence constants ──────────────────────────────────────────────────────
// Tuning values for programmed mortality. Separated by entity origin.

/// Gompertz hazard coefficient for materialized terrain tiles.
/// Low: terrain decays slowly (replaced by field propagation).
pub const SENESCENCE_COEFF_MATERIALIZED: f32 = 0.0001;

/// Maximum age for materialized terrain tiles (ticks).
/// Terrain regenerates from field energy; this caps stale tiles.
pub const SENESCENCE_MAX_AGE_MATERIALIZED: u64 = 8_000;

/// Gompertz hazard coefficient for abiogenesis flora.
/// Medium: plants have moderate lifespan, faster turnover than terrain.
pub const SENESCENCE_COEFF_FLORA: f32 = 0.0002;

/// Maximum age for abiogenesis flora (ticks).
pub const SENESCENCE_MAX_AGE_FLORA: u64 = 5_000;

/// Gompertz hazard coefficient for abiogenesis fauna.
/// Higher: animals live shorter, faster generational turnover.
pub const SENESCENCE_COEFF_FAUNA: f32 = 0.0005;

/// Maximum age for abiogenesis fauna (ticks).
pub const SENESCENCE_MAX_AGE_FAUNA: u64 = 3_000;

/// Reproduction strategy: 0 = Iteroparous (repeat breeder), 1 = Semelparous (one-shot).
pub const SENESCENCE_DEFAULT_STRATEGY: u8 = 0;
