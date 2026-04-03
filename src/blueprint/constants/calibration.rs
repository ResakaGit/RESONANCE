//! Calibration dataset R4 — empirically-grounded reference ranges for simulation parameters.
//! Units: qe (quantum energy), ticks, m (radius).
//! Version-tagged for golden snapshot tracing.

/// Reference range for BaseEnergy in viable organisms (qe).
pub const BASE_ENERGY_VIABLE_MIN: f32 = 50.0;
pub const BASE_ENERGY_VIABLE_MAX: f32 = 5000.0;

/// Intake rate: fraction of available energy absorbed per tick.
pub const INTAKE_RATE_MIN: f32 = 0.01; // 1% per tick minimum
pub const INTAKE_RATE_MAX: f32 = 0.25; // 25% per tick maximum
pub const INTAKE_RATE_NOMINAL: f32 = 0.08; // 8% empirically plausible

/// Maintenance cost: fraction of BaseEnergy consumed per tick.
pub const MAINTENANCE_RATE_MIN: f32 = 0.001;
pub const MAINTENANCE_RATE_MAX: f32 = 0.05;
pub const MAINTENANCE_RATE_NOMINAL: f32 = 0.01;

/// Growth rate: fraction of surplus energy converted to structural gain per tick.
pub const GROWTH_RATE_MIN: f32 = 0.001;
pub const GROWTH_RATE_MAX: f32 = 0.10;
pub const GROWTH_RATE_NOMINAL: f32 = 0.02;

/// Decay rate when energy-starved: fraction of BaseEnergy lost per tick.
pub const DECAY_RATE_MIN: f32 = 0.005;
pub const DECAY_RATE_MAX: f32 = 0.15;
pub const DECAY_RATE_NOMINAL: f32 = 0.02;

/// Calibration version for golden snapshot traceability.
pub const CALIBRATION_VERSION: &str = "v1.0.0-2026-03";

/// Unit mapping: external biological rates (% per hour) → internal fraction per tick.
/// Assumes 60 ticks/s game time.
pub const EXTERNAL_RATE_TO_INTERNAL: f32 = 1.0 / (60.0 * 3600.0);

/// Temperature external (Kelvin) → internal bond energy scale factor.
/// 100 K → 10 bond energy units.
pub const KELVIN_TO_BOND_SCALE: f32 = 0.1;
