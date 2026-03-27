// ── LI3: escalas de inferencia de órganos ──
/// Escala base de hojas inferidas por biomasa y sesgo de crecimiento.
pub const LEAF_COUNT_SCALE: f32 = 2.0;
/// Escala base de pétalos inferidos por biomasa y sesgo de ramificación.
pub const PETAL_COUNT_SCALE: f32 = 8.0;
/// Escala base de espinas inferidas por biomasa y resiliencia.
pub const THORN_COUNT_SCALE: f32 = 1.5;
/// Escala base de raíces inferidas por biomasa y sesgo anti-crecimiento.
pub const ROOT_COUNT_SCALE: f32 = 1.0;
/// Escala base de extremidades inferidas por biomasa y movilidad.
pub const LIMB_COUNT_SCALE: f32 = 1.5;
/// Biomasa mínima para manifestar fruto en fase reproductiva.
pub const FRUIT_BIOMASS_THRESHOLD: f32 = 3.0;
/// Techo por tipo de órgano para una entidad en un tick.
pub const MAX_ORGAN_INSTANCE_COUNT: u8 = 16;
/// Factor de atenuación global de órganos en fase de deterioro.
pub const DECLINING_ORGAN_FALLOFF: f32 = 0.5;
/// Divisor de biomasa para normalizar growth_progress a [0, 1] en organ_manifest_inputs.
pub const ORGAN_MANIFEST_BIOMASS_NORM_DIVISOR: f32 = 3.0;

// ── Constructal body plan inference ─────────────────────────────────────────
/// Maximum limb count evaluated in `optimal_appendage_count` search.
pub const MAX_CONSTRUCTAL_LIMBS: u8 = 8;
/// Internal vascular viscosity proxy (Hagen-Poiseuille μ parameter).
pub const CONSTRUCTAL_VISCOSITY: f32 = 0.1;
/// Default limb length as fraction of entity radius.
pub const CONSTRUCTAL_LIMB_LENGTH_RATIO: f32 = 0.8;
/// Default limb radius as fraction of entity radius (per-limb, divided by √N).
pub const CONSTRUCTAL_LIMB_RADIUS_RATIO: f32 = 0.15;

