//! AC-5: Cooperation Emergence — tuning constants.
//! Physical derivation: Axiom 3 (competition) game theory + AC-1 interference cost.

/// Group extraction bonus for entities in an alliance (dimensionless multiplier total).
/// A group of N shares this bonus equally: each member gains `GROUP_BONUS / N` extra.
pub const COOPERATION_GROUP_BONUS: f32 = 3.0;

/// Maximum scan radius for cooperation partner search (world units).
/// Matches trophic capture radius — entities within predation range may also cooperate.
pub const COOPERATION_SCAN_RADIUS: f32 = 8.0;

/// Minimum extraction rate for a solo entity to be considered a viable cooperation candidate.
/// Prevents degenerate alliances with near-dead entities.
pub const COOPERATION_MIN_VIABLE_RATE: f32 = 1.0;

/// Defection temptation threshold: if solo rate exceeds group rate by more than this,
/// the entity defects (sends AllianceDefectEvent).
pub const COOPERATION_DEFECT_THRESHOLD: f32 = 5.0;

/// Fraction of an entity's intake rate used as the interference cost baseline.
/// interference_cost = rate * SCALING * (1 - interference_factor)
pub const COOPERATION_INTERFERENCE_RATE_SCALING: f32 = 0.1;
