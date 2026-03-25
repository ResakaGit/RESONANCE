// ── Capa 4: Growth Budget (TL3) ──
/// Referencia de energía de enlace para estimar eficiencia genética.
pub const BOND_ENERGY_REFERENCE: f32 = 5000.0;

/// Bonus relativo por electronegatividad en eficiencia genética.
pub const GENETIC_ELECTRO_BONUS: f32 = 0.3;

/// Umbral mínimo de biomasa para considerar crecimiento activo.
pub const GROWTH_BUDGET_MIN_THRESHOLD: f32 = 0.01;

/// Presupuesto máximo de entidades evaluadas por frame en `growth_budget_system`.
pub const MAX_GROWTH_BUDGET_PER_FRAME: u32 = 64;

/// Tolerancia de escritura para evitar `Changed<T>` espurio en updates de growth.
pub const GROWTH_WRITE_EPS: f32 = 1e-4;

/// Biomasa de referencia para modular energía visual por salud de crecimiento.
pub const MAX_VISUAL_BIOMASS: f32 = 1.0;

