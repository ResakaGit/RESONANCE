/// Re-evaluation interval in simulation ticks.
pub const BEHAVIOR_DECISION_INTERVAL: u32 = 4;

/// Buffer fraction below which an entity is considered hungry.
pub const HUNGER_THRESHOLD_FRACTION: f32 = 0.3;

/// Buffer fraction above which an entity is considered satiated.
pub const SATIATED_THRESHOLD_FRACTION: f32 = 0.7;

/// Flee utility threshold that triggers panic override.
pub const PANIC_THRESHOLD: f32 = 0.8;

/// Maximum simulation ticks a hunt is sustained before abandoning.
pub const MAX_CHASE_TICKS: u32 = 120;

/// Resilience reduces flee urgency by `resilience × scale`.
pub const FLEE_RESILIENCE_SCALE: f32 = 0.5;

/// Normalization reference for prey qe value in hunt utility.
pub const HUNT_QE_REFERENCE: f32 = 500.0;

/// Maximum detection range for hunting.
pub const HUNT_MAX_RANGE: f32 = 15.0;

/// Maximum detection range for foraging.
pub const FORAGE_MAX_RANGE: f32 = 20.0;

/// Movement intent multiplier during hunting (sprint).
pub const BEHAVIOR_SPRINT_FACTOR: f32 = 1.5;

/// Movement intent multiplier during fleeing (panic).
pub const BEHAVIOR_PANIC_FACTOR: f32 = 1.8;

/// Minimum biomass (qe) required to consider reproduction.
pub const REPRODUCE_BIOMASS_THRESHOLD: f32 = 800.0;

/// Number of utility actions evaluated per decision cycle.
pub const BEHAVIOR_ACTION_COUNT: usize = 5;

/// Idle utility baseline when satiated and safe.
pub const IDLE_SATIATED_SCORE: f32 = 0.15;

/// Idle utility baseline when not satiated.
pub const IDLE_DEFAULT_SCORE: f32 = 0.05;

/// Relative power divisor for threat level normalization.
pub const THREAT_LEVEL_DIVISOR: f32 = 2.0;

/// Maximum relative power ratio before clamping.
pub const THREAT_POWER_CLAMP: f32 = 5.0;

/// Default mobility bias when no InferenceProfile is present.
pub const DEFAULT_MOBILITY_BIAS: f32 = 0.5;
