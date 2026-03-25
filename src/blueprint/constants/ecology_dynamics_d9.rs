//! D9: Ecological Dynamics — tuning constants.

/// Census run interval (ticks). ~0.5s at 60 Hz fixed.
pub const CENSUS_INTERVAL: u64 = 30;
/// Succession evaluation interval (ticks). `u64` for run condition compatibility.
pub const SUCCESSION_TICK_INTERVAL: u64 = 60;
/// Same interval as `u32` for arithmetic on `SuccessionState::time_since_disturbance`.
pub const SUCCESSION_TICK_STEP: u32 = SUCCESSION_TICK_INTERVAL as u32;
/// qe per entity slot in carrying capacity formula.
pub const CARRYING_CAPACITY_QE_FACTOR: f32 = 10.0;
/// Pressure multiplier for abiogenesis threshold modulation.
pub const ABIOGENESIS_PRESSURE_SCALE: f32 = 2.0;
/// Ticks until Pioneer → Early transition baseline.
pub const SUCCESSION_PIONEER_TICKS: u32 = 300;
/// Ticks until Early → Mid transition baseline.
pub const SUCCESSION_EARLY_TICKS: u32 = 1200;
/// Ticks until Mid → Climax transition baseline.
pub const SUCCESSION_MID_TICKS: u32 = 3600;
