// ── MG-3 — Paso temporal del DAG metabólico: umbrales y mapeo T_env ──

/// Umbral de cambio para guard detection en flujos y entropía (qe).
pub const METABOLIC_STEP_EPSILON: f32 = 1e-4;

/// Flujo mínimo por arista: por debajo se colapsa a 0 (evita ruido numérico).
pub const METABOLIC_MIN_FLOW: f32 = 0.01;

/// Energía mínima para operar un nodo del DAG (qe). Debajo = inanición, flujos → 0.
pub const METABOLIC_STARVATION_THRESHOLD: f32 = 1.0;

/// Temperatura ambiental base (modelo equivalente). Bioma neutro (plain, viscosity=1).
pub const AMBIENT_BASE_TEMPERATURE: f32 = 280.0;

/// Sensibilidad térmica a la viscosidad del terreno: T_env = T_base + (viscosity - 1) * scale.
/// Biomas densos (agua, pantano) → T_env más alta; biomas enrarecidos (ley line) → T_env más baja.
pub const AMBIENT_TEMP_VISCOSITY_SCALE: f32 = 20.0;
