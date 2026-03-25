// ── Simulation: actuador y colisiones (usa ecuaciones blueprint) ──
/// Buffer máximo asumido si la entidad no tiene motor (fallback para will_force).
pub const ACTUATOR_FALLBACK_BUFFER_MAX: f32 = 100.0;

/// Tope de velocidad para actuadores cuando la materia impondría < 1 (héroe debe poder moverse).
pub const ACTUATOR_VELOCITY_LIMIT: f32 = 10.0;

/// Umbral |v|² para trazas de depuración de héroe.
pub const ACTUATOR_VELOCITY_SQ_TRACE_EPSILON: f32 = 0.001;

/// Límite de materia por debajo del cual se sube a ACTUATOR_VELOCITY_LIMIT.
pub const ACTUATOR_MATTER_LOW_VELOCITY_CAP: f32 = 1.0;

/// Promedio aritmético de conductividades en transferencia por colisión.
pub const COLLISION_CONDUCTIVITY_BLEND: f32 = 0.5;

