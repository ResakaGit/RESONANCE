// ── Cultura como observable energético (CE track) ──────────────────────────
//
// Cultura(G) = f(coherencia, síntesis, resiliencia, longevidad)
// Todos los umbrales derivan de física existente — no son números arbitrarios.

// ── Ancho de banda ───────────────────────────────────────────────────────────

/// Rango frecuencial completo del universo Resonance: Umbra (~10 Hz) → Lux (~1100 Hz).
/// Denominador para normalizar interferencia entre bandas.
pub const CULTURE_FREQ_BANDWIDTH_HZ: f32 = 1100.0;

// ── Umbrales de emergencia cultural ─────────────────────────────────────────
// Derivados de la física de L2 + L12 + Catalysis.
// No son tuning arbitrario — cada uno tiene una fuente física.

/// Coherencia mínima para cultura emergente (C_min).
/// Fuente: radio de entrainment de L12 Homeostasis.
/// Por debajo de este valor la varianza de frecuencias supera la capacidad de
/// sincronización del grupo — no puede sostenerse coherencia bajo perturbación.
pub const CULTURE_COHERENCE_MIN: f32 = 0.65;

/// Tasa de síntesis catalítica interna mínima (S_min).
/// Fuente: condición de refuerzo neto > disipación del grupo.
/// Por debajo de este valor el grupo se disuelve más rápido de lo que se construye.
pub const CULTURE_SYNTHESIS_MIN: f32 = 0.55;

/// Resiliencia mínima del patrón de frecuencia (R_min).
/// Fracción de coherencia que el grupo recupera tras perturbación externa.
pub const CULTURE_RESILIENCE_MIN: f32 = 0.50;

/// Tamaño mínimo del grupo para percolación cultural (N_min).
/// Fuente: umbral de percolación en red 2D — por debajo de este tamaño
/// el patrón no puede propagarse al campo circundante.
pub const CULTURE_GROUP_MIN_SIZE: usize = 3;

/// Conectividad espacial mínima para percolación (p_c en red 2D ≈ 0.59).
/// Fuente: teoría de percolación en redes aleatorias 2D.
pub const CULTURE_PERCOLATION_CONNECTIVITY: f32 = 0.59;

// ── Transición de fase cultural ──────────────────────────────────────────────
// Análogo directo a layer04 MatterState pero en espacio de frecuencias.

/// Coherencia máxima para fase "gas" — frecuencias dispersas, sin identidad grupal.
pub const CULTURE_PHASE_GAS_MAX: f32 = 0.25;

/// Coherencia mínima para fase "sólido" — identidad grupal estable y resistente.
pub const CULTURE_PHASE_SOLID_MIN: f32 = 0.70;

// ── Conflicto ────────────────────────────────────────────────────────────────

/// Potencial de interferencia inter-grupo por debajo del cual emerge conflicto activo.
/// cos(Δfreq) < -0.25 → los grupos se dañan mutuamente en cada catalysis.
pub const CULTURE_CONFLICT_THRESHOLD: f32 = -0.25;

// ── Longevidad ───────────────────────────────────────────────────────────────

/// Horizonte de normalización de longevidad: tick_age máximo esperado para [0,1].
/// Calibrado empíricamente desde CSV de simulación (SF-4).
pub const CULTURE_MAX_EXPECTED_AGE_TICKS: f32 = 10_000.0;

/// Intervalo de ticks entre observaciones culturales (throttle del sistema).
/// 30 ticks ≈ 0.5 s a 60 TPS — mismo intervalo que atmosphere_inference.
/// El O(n²) de síntesis hace inviable correr cada tick.
pub const CULTURE_OBSERVATION_INTERVAL_TICKS: u64 = 30;

// ── AC-3: Frequency × Culture constants ─────────────────────────────────────

/// Maximum coherence bonus multiplier for group imitation (AC-3).
/// A perfectly coherent group boosts imitation by this factor above 1.0.
pub const CULTURE_COHERENCE_IMITATION_BONUS_CAP: f32 = 0.4;
