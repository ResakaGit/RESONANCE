/// Maximum frequency band span for resonance calculations (Hz).
/// Covers Umbra (10 Hz) to Lux (1100 Hz) — full elemental range.
pub const FREQ_BAND_MAX_HZ: f32 = 1100.0;

/// Default extraction capacity used when no profile overrides it.
pub const BASE_EXTRACTION_CAPACITY: f32 = 10.0;

/// Cumulative threat magnitude above which an agent switches to Flee.
pub const FLEE_THREAT_THRESHOLD: f32 = 30.0;

/// Minimum own qe required to initiate a Hunt / FocusFire decision.
pub const HUNT_MINIMUM_OWN_QE: f32 = 50.0;
