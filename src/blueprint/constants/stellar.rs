//! Constants for stellar-scale simulation.
//! 1 world unit = 1 AU (Astronomical Unit) in stellar mode.

/// Default velocity cap for stellar simulation (orbital velocities are large).
/// Earth orbital velocity ≈ 30 km/s; in game units this scales with time_step.
pub const STELLAR_VELOCITY_CAP: f32 = 500.0;

/// Gravity gain for stellar bodies. Calibrated so that:
/// star(qe=1e6) + planet(qe=1e3) at 1 AU → stable orbit at v ≈ 30 units/s.
/// F = G * m1 * m2 / r², with m = qe, G = gravity_gain.
pub const STELLAR_GRAVITY_GAIN: f32 = 0.001;

/// Magnetic gain for stellar fields (oscillatory interference at stellar scale).
pub const STELLAR_MAGNETIC_GAIN: f32 = 0.0001;

/// TensionField radius for a star (covers entire system).
pub const STAR_FIELD_RADIUS: f32 = 200.0; // AU

/// TensionField radius for a planet (local gravity well).
pub const PLANET_FIELD_RADIUS: f32 = 0.5; // AU

/// Star default energy (very high — gravitational center).
pub const STAR_DEFAULT_QE: f32 = 1_000_000.0;

/// Star spatial radius (visual size in AU).
pub const STAR_DEFAULT_RADIUS: f32 = 0.05; // ~7 million km / 150 million km/AU

/// Star emission rate (qe/s radiated into field).
pub const STAR_EMISSION_RATE: f32 = 10_000.0;

/// Star default frequency (Lux band — highest energy).
pub const STAR_FREQUENCY_HZ: f32 = 1000.0;

/// Planet default energy.
pub const PLANET_DEFAULT_QE: f32 = 1_000.0;

/// Planet spatial radius.
pub const PLANET_DEFAULT_RADIUS: f32 = 0.01; // ~1.5 million km

/// Planet default bond energy (solid, high structural integrity).
pub const PLANET_DEFAULT_BOND: f32 = 5000.0;

/// Planet thermal conductivity (moderate — allows surface processes).
pub const PLANET_DEFAULT_CONDUCTIVITY: f32 = 1.5;
