// ── Simulation defaults: tuning values shared across simulation/ systems ──
use crate::math_types::Vec2;

/// Default thermal load for structural link stress (no thermal stress present).
pub const STRUCTURAL_DEFAULT_THERMAL_LOAD: f32 = 0.0;

/// Softening epsilon for tension field acceleration (prevents singularity at d→0).
pub const TENSION_FIELD_SOFTENING_EPS: f32 = 0.1;

/// Passive entropy drain rate for quantum-suspended entities (J/s).
pub const ATTENTION_ENTROPY_DRAIN_RATE: f32 = 0.5;

/// Default grid dimensions (width/height) for energy, fog, terrain, and nutrient grids.
pub const DEFAULT_GRID_DIMS: u32 = 64;

/// Default grid origin for energy, fog, terrain, and nutrient grids.
pub const DEFAULT_GRID_ORIGIN: Vec2 = Vec2::new(-64.0, -64.0);

/// Spawn offset past caster radius for projectiles (world units).
pub const PROJECTILE_SPAWN_OFFSET: f32 = 0.05;

/// Epsilon for direction normalization (`length_squared` threshold).
pub const DIRECTION_NORMALIZE_EPS: f32 = 1e-6;

/// Epsilon for growth intent confidence/stability field writes.
pub const GROWTH_INTENT_FIELD_EPS: f32 = 1e-4;

/// Default `competition_t` for evolution surrogate baseline scenario.
pub const EVOLUTION_BASELINE_COMPETITION: f32 = 0.5;
