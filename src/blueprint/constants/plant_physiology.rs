//! Constantes del track PLANT_PHYSIOLOGY — todas derivadas de 4 fundamentales.
//! Plant physiology track constants — all derived from 4 fundamentals.

use crate::blueprint::equations::derived_thresholds::*;

/// Minimum organ energy before considered dead.
/// `DISSIPATION_SOLID × DENSITY_SCALE = 0.1 qe`
pub const ORGAN_DEATH_THRESHOLD: f32 = DISSIPATION_SOLID * DENSITY_SCALE;

/// Volatile emission efficiency. `1.0 - DISSIPATION_GAS = 0.92`
pub const VOLATILE_EFFICIENCY: f32 = 1.0 - DISSIPATION_GAS;

/// Volatile decay rate per tick. `DISSIPATION_GAS = 0.08`
pub const VOLATILE_DECAY_RATE: f32 = DISSIPATION_GAS;

/// Inner/outer nutrient gradient ratio for tissue curvature.
/// `DISSIPATION_LIQUID / DISSIPATION_SOLID = 4.0`
pub const CURVATURE_NUTRIENT_RATIO: f32 = DISSIPATION_LIQUID / DISSIPATION_SOLID;

/// Phototropism sensitivity — spine tilt per unit irradiance gradient.
/// `1.0 / DENSITY_SCALE = 0.05`
pub const PHOTOTROPISM_SENSITIVITY: f32 = 1.0 / DENSITY_SCALE;

/// Nutrient concentration threshold for subterranean topology switch.
/// `DISSIPATION_LIQUID × DENSITY_SCALE = 0.4`
pub const CONCENTRATION_THRESHOLD: f32 = DISSIPATION_LIQUID * DENSITY_SCALE;

/// Minimum frequency alignment for cross-transfer compatibility.
/// `1.0 - DISSIPATION_PLASMA = 0.75` — but clamped to 0.5 for practical use.
pub const TRANSFER_THRESHOLD: f32 = 0.5;

/// EnergyTag lifetime in ticks. `1.0 / DISSIPATION_LIQUID ≈ 50`
pub const TAG_LIFETIME_TICKS: u32 = (1.0 / DISSIPATION_LIQUID) as u32;

/// Phenology bloom irradiance threshold.
/// `DISSIPATION_LIQUID × DENSITY_SCALE = 0.4`
pub const PHENOLOGY_BLOOM_THRESHOLD: f32 = DISSIPATION_LIQUID * DENSITY_SCALE;

/// Maximum curvature scale applied to GF1 ring radius asymmetry.
pub const CURVATURE_SCALE: f32 = 0.3;
