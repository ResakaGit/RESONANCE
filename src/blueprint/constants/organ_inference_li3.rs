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

