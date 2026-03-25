// ── MG-6 — Writer Monad + EntropyLedger: constantes ──

/// Cota inferior de calor por nodo (Landauer). Defer post-v1.
pub const LANDAUER_MIN_HEAT: f32 = 0.001;

/// Tolerancia de conservacion en debug asserts de organ_transform / cadena.
pub const CHAIN_CONSERVATION_EPSILON: f32 = 1e-3;
