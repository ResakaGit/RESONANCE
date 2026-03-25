// ── Capa 4: Transiciones de Fase ──
/// Constante de Boltzmann del juego (convierte densidad a temperatura equivalente).
pub const GAME_BOLTZMANN: f32 = 1.0;

/// Umbral: T < SOLID_TRANSITION * eb → Sólido
pub const SOLID_TRANSITION: f32 = 0.3;

/// Umbral: T < LIQUID_TRANSITION * eb → Líquido
pub const LIQUID_TRANSITION: f32 = 1.0;

/// Umbral: T < GAS_TRANSITION * eb → Gas, else → Plasma
pub const GAS_TRANSITION: f32 = 3.0;

