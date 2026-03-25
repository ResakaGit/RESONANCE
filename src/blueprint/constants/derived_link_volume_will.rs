// ── Capas derivadas / enlace / volumen / voluntad ──
/// Multiplicador de disipación si no hay coherencia (sin capa 4 = sin freno entrópico extra).
pub const DERIVED_DEFAULT_DISSIPATION_MULTIPLIER: f32 = 1.0;

pub const LINK_NEUTRAL_MULTIPLIER: f32 = 1.0;

pub const DEFAULT_SPHERE_RADIUS: f32 = 1.0;

pub const VOLUME_MIN_RADIUS: f32 = 0.01;

/// Factor 4/3 en volumen esférico (geometría, no tuning de gameplay).
pub const SPHERE_VOLUME_NUMERATOR: f32 = 4.0;

pub const SPHERE_VOLUME_DENOMINATOR: f32 = 3.0;

/// Intención de movimiento considerada “cero” (evita jitter).
pub const WILL_MOVEMENT_INTENT_SQ_EPSILON: f32 = 0.001;

