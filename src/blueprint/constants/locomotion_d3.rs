/// Kinetic energy cost factor: qe per unit of mass × speed².
pub const LOCOMOTION_KINETIC_FACTOR: f32 = 0.002;

/// Slope cost scale: uphill ×2.5 at 45° (slope=1.0 → 1 + 1.0 × 1.5 = 2.5).
pub const SLOPE_COST_SCALE: f32 = 1.5;

/// Speed below this threshold incurs no locomotion cost.
pub const LOCOMOTION_MIN_SPEED_THRESHOLD: f32 = 0.1;

/// Base stamina recovery rate (qe/s at full buffer).
pub const STAMINA_BASE_RECOVERY: f32 = 0.5;

/// Sprint costs this multiple of normal locomotion energy.
pub const SPRINT_COST_MULTIPLIER: f32 = 3.0;

/// Buffer fraction below which exhaustion forces rest.
pub const EXHAUSTION_BUFFER_FRACTION: f32 = 0.05;

/// Ticks of forced idle when exhausted.
pub const EXHAUSTION_REST_TICKS: u32 = 8;
