// ── Capa 5: motor (layers/engine + defaults) ──
/// Buffer máximo por defecto de un motor sin blueprint (tuning de “caster estándar”).
pub const ENGINE_DEFAULT_MAX_BUFFER: f32 = 1000.0;

/// Válvula de entrada por defecto (qe/s que puede acumular el buffer).
pub const ENGINE_DEFAULT_INPUT_VALVE: f32 = 10.0;

/// Válvula de salida por defecto (qe/s máximo para habilidades).
pub const ENGINE_DEFAULT_OUTPUT_VALVE: f32 = 50.0;

/// Divisor de rango de frecuencias en eficiencia de forja (anchura típica del espectro útil).
pub const ENGINE_EFFICIENCY_FREQ_DIVISOR: f32 = 1100.0;

/// Penalización por desajuste de frecuencia (0.7 = hasta 70 % de pérdida si está lejos).
pub const ENGINE_EFFICIENCY_FALLOFF: f32 = 0.7;

/// Bonus fijo si el elemento objetivo está dominado (recompensa por afinidad declarada).
pub const ENGINE_MASTERY_BONUS: f32 = 0.1;

