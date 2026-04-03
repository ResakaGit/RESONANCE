// ── Matemática numérica (equations) ──
/// Umbral de distancia casi nula: evita singularidades en geometría de círculos y muelles.
pub const DISTANCE_EPSILON: f32 = 1e-6;

/// Piso para divisiones y falloff: evita infinitos sin cambiar el régimen físico lejos del cero.
pub const DIVISION_GUARD_EPSILON: f32 = 1e-4;

/// Velocidad mínima para aplicar arrastre (evita ruido numérico en v ≈ 0).
pub const DRAG_SPEED_EPSILON: f32 = 0.001;
