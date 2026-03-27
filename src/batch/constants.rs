//! Batch simulator constants. Re-exports blueprint constants + batch-specific tuning.

pub use crate::blueprint::constants::QE_MIN_EXISTENCE;

/// Maximum entity slots per world. Power of 2, fits in `u64` bitmask.
pub const MAX_ENTITIES: usize = 64;

/// Nutrient/irradiance grid cell count (GRID_SIDE × GRID_SIDE).
pub const GRID_CELLS: usize = 256;

/// Grid side length in cells.
pub const GRID_SIDE: usize = 16;

/// Fraction of available energy exchanged per collision pair per tick.
pub const COLLISION_EXCHANGE_FRACTION: f32 = 0.01;

/// Photosynthesis efficiency: fraction of irradiance × area converted to qe.
/// High enough that sessile producers are a viable strategy (Axiom 6: emergence).
pub const PHOTOSYNTHESIS_EFFICIENCY: f32 = 0.4;

/// Nutrient extraction rate per unit radius per tick.
pub const NUTRIENT_UPTAKE_RATE: f32 = 0.5;

/// Predation capture range (squared for fast comparison).
pub const PREDATION_RANGE: f32 = 3.0;

/// Assimilation efficiency: fraction of drained qe predator receives.
pub const CARNIVORE_ASSIMILATION: f32 = 0.6;

/// Fraction of prey qe drained per successful predation.
/// Low enough that hunting is not always worth the locomotion cost (Axiom 4: dissipation).
pub const PREDATION_DRAIN_FRACTION: f32 = 0.15;

/// Social pack cohesion scan radius.
pub const PACK_SCAN_RADIUS: f32 = 8.0;

/// Pack cohesion spring strength.
pub const PACK_COHESION_STRENGTH: f32 = 0.5;

/// Culture imitation scan radius.
pub const CULTURE_SCAN_RADIUS: f32 = 10.0;

/// Cooperation scan radius.
pub const COOPERATION_SCAN_RADIUS: f32 = 8.0;

/// Containment overlap drag coefficient.
pub const CONTAINMENT_DRAG_COEFF: f32 = 0.1;

/// Tension field gravity/magnetic range.
pub const TENSION_FIELD_RANGE: f32 = 6.0;

// ── BS-3: Lifecycle ─────────────────────────────────────────────────────────

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
