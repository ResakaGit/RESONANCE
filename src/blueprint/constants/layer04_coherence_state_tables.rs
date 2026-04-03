// ── Capa 4: coherencia por defecto y tablas por estado ──
/// Energía de enlace por defecto (materia “típica” estable en simulación).
pub const DEFAULT_BOND_ENERGY: f32 = 5000.0;

/// Conductividad térmica inicial por defecto [0,1].
pub const DEFAULT_THERMAL_CONDUCTIVITY: f32 = 0.2;

/// Límite de velocidad para líquidos (sólido = 0 vía capa motor).
pub const VELOCITY_LIMIT_LIQUID: f32 = 5.0;

/// Multiplicador de disipación L4 (coherence-internal, NOT axiom dissipation rates).
/// These scale bond_energy degradation in MatterCoherence, distinct from the
/// fundamental DISSIPATION_SOLID/LIQUID/GAS/PLASMA in derived_thresholds.rs.
pub const DISSIPATION_MULT_SOLID: f32 = 0.2;

/// Multiplicador de disipación en líquido.
pub const DISSIPATION_MULT_LIQUID: f32 = 0.5;

/// Multiplicador de disipación en gas.
pub const DISSIPATION_MULT_GAS: f32 = 1.5;

/// Multiplicador de disipación en plasma.
pub const DISSIPATION_MULT_PLASMA: f32 = 3.0;
