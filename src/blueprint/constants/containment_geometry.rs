// ── Containment (geometría host–huésped) ──
/// Fracción del radio del huésped: por encima = Immersed, por debajo = Surface.
pub const IMMERSION_DEPTH_THRESHOLD_RATIO: f32 = 0.5;

/// Rango de radiación: host influye hasta ~2 radios sin contacto.
pub const RADIATED_HOST_RANGE_MULTIPLIER: f32 = 2.0;
