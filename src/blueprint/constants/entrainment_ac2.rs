//! AC-2: Kuramoto Entrainment — tuning constants.
//! Physical derivation: L2 OscillatorySignature + L12 Homeostasis + AC-4 freq purity.

/// Base Kuramoto coupling strength (dimensionless, [0, 1]).
/// Calibrated so siblings entrain within ~10 ticks at contact distance.
pub const KURAMOTO_BASE_COUPLING: f32 = 0.15;

/// Frequency gap below which two oscillators are considered phase-locked (Hz).
/// Smaller than the narrowest band gap (Terra/Stone: 75 Hz) to avoid spurious lock.
pub const KURAMOTO_LOCK_THRESHOLD_HZ: f32 = 1.0;

/// Spatial radius within which an entity scans for Kuramoto neighbours (world units).
/// Matches the default sensory perception distance (PERCEPTION_RADIUS_DEFAULT).
pub const ENTRAINMENT_SCAN_RADIUS: f32 = 12.0;

/// Coherence decay lambda for entrainment coupling (world units).
/// Reuses `FREQ_COHERENCE_DECAY_LAMBDA` from signal_propagation — same physical law.
/// Separate constant so entrainment can be tuned independently if needed.
pub const ENTRAINMENT_COHERENCE_LAMBDA: f32 = 12.0;
