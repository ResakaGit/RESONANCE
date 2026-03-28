//! Batch simulator constants. Re-exports blueprint constants + batch-specific tuning.
//!
//! Every tuning value lives here. Zero magic numbers in systems (Coding Rule 10).

pub use crate::blueprint::constants::QE_MIN_EXISTENCE;

/// Maximum entity slots per world. Power of 2, fits in `u64` bitmask.
pub const MAX_ENTITIES: usize = 64;

/// Nutrient/irradiance grid cell count (GRID_SIDE × GRID_SIDE).
pub const GRID_CELLS: usize = 256;

/// Grid side length in cells.
pub const GRID_SIDE: usize = 16;

// ── Collision ───────────────────────────────────────────────────────────────

/// Fraction of available energy exchanged per collision pair per tick.
pub const COLLISION_EXCHANGE_FRACTION: f32 = 0.01;

// ── Photosynthesis (Axiom 8: resonance with solar frequency) ────────────────

/// Efficiency: fraction of irradiance × area × resonance → qe.
pub const PHOTOSYNTHESIS_EFFICIENCY: f32 = 0.4;

/// Solar emission frequency — entities resonating absorb more light.
pub const SOLAR_FREQUENCY: f32 = 400.0;

/// Minimum solar resonance to photosynthesize (below = too far from sun freq).
pub const SOLAR_RESONANCE_MIN: f32 = 0.1;

/// Fraction of photosynthetic gain deposited as soil nutrients.
pub const NUTRIENT_DEPOSIT_FRACTION: f32 = 0.3;

// ── Nutrient uptake ─────────────────────────────────────────────────────────

/// Nutrient extraction rate per unit radius per tick.
pub const NUTRIENT_UPTAKE_RATE: f32 = 0.5;

/// Maximum speed² for foraging — must be slow to graze (Axiom 6: emergence).
pub const FORAGE_MAX_SPEED_SQ: f32 = 1.0;

// ── Predation (Axiom 6: energy dominance, not tags) ─────────────────────────

/// Predation capture range.
pub const PREDATION_RANGE: f32 = 3.0;

/// Assimilation efficiency: fraction of drained qe predator receives.
pub const CARNIVORE_ASSIMILATION: f32 = 0.6;

/// Fraction of prey qe drained per successful predation.
pub const PREDATION_DRAIN_FRACTION: f32 = 0.15;

/// Dominance ratio: attacker needs qe > target × this to drain (Axiom 6).
pub const PREDATION_DOMINANCE_RATIO: f32 = 0.7;

// ── Behavior (Axiom 6: from composition, not tags) ──────────────────────────

/// Threat detection: threat.qe > self.qe × this.
pub const THREAT_QE_RATIO: f32 = 1.5;

/// Food detection: target.qe < self.qe × this.
pub const FOOD_QE_RATIO: f32 = 0.8;

/// Minimum mobility_bias to exhibit hunting behavior.
pub const HUNT_MOBILITY_THRESHOLD: f32 = 0.2;

/// Minimum mobility_bias to exhibit any behavior (movement intent).
pub const BEHAVIOR_MOBILITY_MIN: f32 = 0.01;

// ── Social / cooperation / culture ──────────────────────────────────────────

/// Social pack cohesion scan radius.
pub const PACK_SCAN_RADIUS: f32 = 8.0;

/// Pack cohesion spring strength.
pub const PACK_COHESION_STRENGTH: f32 = 0.5;

/// Culture imitation scan radius.
pub const CULTURE_SCAN_RADIUS: f32 = 10.0;

/// Culture expression blend rate per tick (scaled by affinity).
pub const CULTURE_BLEND_RATE: f32 = 0.01;

/// Cooperation scan radius.
pub const COOPERATION_SCAN_RADIUS: f32 = 8.0;

// ── Containment / tension ───────────────────────────────────────────────────

/// Containment overlap drag coefficient.
pub const CONTAINMENT_DRAG_COEFF: f32 = 0.1;

/// Tension field influence radius = entity.radius × this.
pub const TENSION_RADIUS_MULTIPLIER: f32 = 3.0;

/// Tension field inverse-square denominator scaling.
pub const TENSION_FORCE_SCALE: f32 = 100.0;

/// Tension field gravity/magnetic range.
pub const TENSION_FIELD_RANGE: f32 = 6.0;

// ── Irradiance grid ─────────────────────────────────────────────────────────

/// Base solar irradiance per grid cell per tick.
pub const SOLAR_FLUX_BASE: f32 = 2.0;

/// Spatial variation frequency for irradiance heterogeneity.
pub const IRRADIANCE_VARIATION_FREQ: f32 = 0.1;

/// Spatial variation amplitude.
pub const IRRADIANCE_VARIATION_AMP: f32 = 0.3;

/// Minimum irradiance variation floor.
pub const IRRADIANCE_VARIATION_MIN: f32 = 0.5;

// ── Lifecycle ───────────────────────────────────────────────────────────────

/// Minimum qe for reproduction eligibility.
pub const REPRODUCTION_THRESHOLD: f32 = 50.0;

/// Fraction of parent qe transferred to child at birth.
pub const REPRODUCTION_TRANSFER_FRACTION: f32 = 0.3;

/// Mutation sigma for genome biases during reproduction.
pub const DEFAULT_MUTATION_SIGMA: f32 = 0.05;

/// Population cap above which abiogenesis is suppressed.
pub const ABIOGENESIS_POP_CAP: u8 = 48;

/// Minimum irradiance grid energy sum for abiogenesis.
pub const ABIOGENESIS_ENERGY_THRESHOLD: f32 = 1000.0;

/// Initial qe of spontaneously generated cells.
pub const ABIOGENESIS_INITIAL_QE: f32 = 10.0;

/// Initial radius of spontaneously generated cells.
pub const ABIOGENESIS_INITIAL_RADIUS: f32 = 0.3;

/// Fraction of dying entity's qe returned to nutrient grid.
pub const DEATH_NUTRIENT_RETURN: f32 = 0.5;

// ── Internal energy field ───────────────────────────────────────────────────

/// Diffusion conductivity between adjacent internal nodes per tick.
pub const INTERNAL_DIFFUSION_CONDUCTIVITY: f32 = 0.05;

/// Frequency entrainment coupling between adjacent nodes.
pub const INTERNAL_FREQ_COUPLING: f32 = 0.02;

/// Minimum per-node radius ratio (prevents zero-thickness).
pub const FIELD_RADIUS_MIN_RATIO: f32 = 0.3;

/// Maximum per-node radius ratio (prevents excessive bulging).
pub const FIELD_RADIUS_MAX_RATIO: f32 = 2.5;

/// Numeric guard epsilon for division/normalization.
pub const GUARD_EPSILON: f32 = 0.01;

/// Convergence epsilon for radial field diffusion (AS-2).
pub const CONVERGENCE_EPSILON: f32 = 0.001;

/// Maximum interaction range for isolation detection (AS-1).
pub const ISOLATION_RANGE_SQ: f32 = 64.0; // 8.0²

/// Satiation level above which entity stops hunting.
pub const SATIATION_WELL_FED: f32 = 0.7;

/// Minimum dissipation floor (prevents zero-dissipation immortality).
pub const DISSIPATION_FLOOR: f32 = 0.001;

/// Minimum oscillatory affinity for cultural imitation.
pub const CULTURE_AFFINITY_MIN: f32 = 0.3;

/// Frequency field initialization spread per node (Hz offset from center).
pub const FREQ_FIELD_SPREAD: f32 = 5.0;

// ── Emergent morphology (radial field) ──────────────────────────────────────

/// Peak detection threshold: node must exceed mean × this factor.
pub const PEAK_THRESHOLD_FACTOR: f32 = 1.8;

/// Minimum peak qe to generate an appendage mesh.
pub const APPENDAGE_QE_MIN: f32 = 0.5;

/// Solar resonance Gaussian bandwidth (Hz). Controls photosynthetic frequency selectivity.
pub const SOLAR_BANDWIDTH: f32 = 200.0;

/// Abiogenesis frequency range min (Hz).
pub const ABIOGENESIS_FREQ_MIN: f32 = 200.0;

/// Abiogenesis frequency range max (Hz).
pub const ABIOGENESIS_FREQ_MAX: f32 = 800.0;

/// Default resilience for abiogenesis-spawned cells.
pub const ABIOGENESIS_DEFAULT_RESILIENCE: f32 = 0.5;

/// Default dissipation for abiogenesis-spawned cells.
pub const ABIOGENESIS_DEFAULT_DISSIPATION: f32 = 0.01;

// ── Environmental pressures (Axiom 4 + 7) ──────────────────────────────────

/// Gravitational acceleration toward y=0 (Axiom 7: distance attenuation from ground).
pub const GRAVITY_ACCELERATION: f32 = 0.5;

/// Seasonal irradiance oscillation rate (radians per tick).
pub const SEASON_RATE: f32 = 0.001;

/// Seasonal irradiance amplitude (0 = no seasons, 0.5 = ±50% flux).
pub const SEASON_AMPLITUDE: f32 = 0.4;

/// Ticks between asteroid impacts (Axiom 4: catastrophic dissipation).
pub const ASTEROID_INTERVAL: u64 = 5000;

/// Asteroid impact radius squared (Axiom 7: localized).
pub const ASTEROID_RADIUS_SQ: f32 = 25.0;

/// Fraction of qe surviving an asteroid impact (0.1 = 90% loss).
pub const ASTEROID_SURVIVAL_FRACTION: f32 = 0.1;
