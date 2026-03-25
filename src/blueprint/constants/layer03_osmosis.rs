// ── Capa 3: Ósmosis (TL1) ──
/// Permeabilidad base de membrana para transferencia osmótica entre celdas.
pub const OSMOTIC_BASE_PERMEABILITY: f32 = 0.02;

/// Escala de permeabilidad por diferencial de electronegatividad.
pub const OSMOTIC_ELECTRO_SCALE: f32 = 0.5;

/// Tope de transferencia osmótica por par de celdas y tick (estabilidad numérica).
pub const OSMOTIC_MAX_TRANSFER_PER_TICK: f32 = 15.0;

/// Presupuesto máximo de celdas procesadas por frame para ósmosis.
pub const MAX_OSMOSIS_PER_FRAME: u32 = 128;

