// ── D5: Sensory & Perception ──

/// Base sensitivity for frequency-based detection (entities with SENSE).
pub const SENSORY_BASE_SENSITIVITY: f32 = 3.0;
/// Default noise floor for frequency detection range.
pub const SENSORY_NOISE_FLOOR: f32 = 1.0;
/// Maximum scan range clamp (avoid unbounded spatial queries).
pub const SENSORY_MAX_SCAN_RANGE: f32 = 60.0;
/// Reference qe for normalizing threat levels.
pub const SENSORY_REFERENCE_QE: f32 = 500.0;
/// Scale factor: speed contribution to threat assessment.
pub const SENSORY_SPEED_THREAT_SCALE: f32 = 0.5;
/// Multiplier for confirmed predator in threat assessment.
pub const SENSORY_PREDATOR_FACTOR: f32 = 2.0;
/// Multiplier for non-predator in threat assessment.
pub const SENSORY_NON_PREDATOR_FACTOR: f32 = 1.0;
/// Ticks before threat memory decays completely.
pub const SENSORY_MEMORY_DECAY_TICKS: u32 = 120;
/// Threat level threshold for emitting ThreatDetectedEvent.
pub const SENSORY_PANIC_THRESHOLD: f32 = 0.8;
/// Maximum entities to scan per frame (throttle budget).
pub const SENSORY_SCAN_BUDGET: usize = 128;
