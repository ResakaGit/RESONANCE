// ── Capa 4: Transiciones de Fase ──
/// Constante de Boltzmann normalizada (convierte densidad → temperatura equivalente).
/// Game Boltzmann constant: normalized to 1.0 so equivalent_temperature = density.
/// Semantic placeholder: in real physics k_B = 1.38×10⁻²³ J/K, but simulation
/// operates in abstract qe units where density IS temperature (natural units).
pub const GAME_BOLTZMANN: f32 = 1.0;

/// Umbral: T < SOLID_TRANSITION * eb → Sólido
pub const SOLID_TRANSITION: f32 = 0.3;

/// Umbral: T < LIQUID_TRANSITION * eb → Líquido
pub const LIQUID_TRANSITION: f32 = 1.0;

/// Umbral: T < GAS_TRANSITION * eb → Gas, else → Plasma
pub const GAS_TRANSITION: f32 = 3.0;
