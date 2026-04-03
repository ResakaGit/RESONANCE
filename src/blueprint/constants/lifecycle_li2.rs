// ── LI2: umbrales de ciclo de vida inferido ──
/// Viabilidad mínima para salir de estado latente.
pub const LIFECYCLE_DORMANT_VIABILITY: f32 = 0.6;
/// Viabilidad por debajo de la cual el estado pasa a deterioro.
pub const LIFECYCLE_DECLINING_VIABILITY: f32 = 0.3;
/// Progreso normalizado mínimo para pasar de Dormant a Emerging.
pub const LIFECYCLE_EMERGING_GROWTH: f32 = 0.1;
/// Progreso normalizado mínimo para considerar forma madura.
pub const LIFECYCLE_MATURE_GROWTH: f32 = 0.7;
/// Biomasa mínima para habilitar fase reproductiva.
pub const LIFECYCLE_REPRODUCTIVE_BIOMASS: f32 = 1.5;
/// Viabilidad mínima para habilitar fase reproductiva.
pub const LIFECYCLE_REPRODUCTIVE_VIABILITY_MIN: f32 = 1.2;
/// Ticks mínimos antes de aceptar transición de fase (histeresis).
pub const LIFECYCLE_HYSTERESIS_TICKS: u16 = 10;
