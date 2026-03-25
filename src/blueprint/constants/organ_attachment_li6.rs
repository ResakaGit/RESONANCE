// ── LI6 — Organ attachment/orientation tuning (centralized, no magic numbers in runtime) ──
/// Proyección apical: inicio de fracción de spine.
pub const ORGAN_ZONE_APICAL_OFFSET: f32 = 0.8;
/// Proyección apical/basal: amplitud de distribución.
pub const ORGAN_ZONE_APICAL_BASAL_SPAN: f32 = 0.18;
/// Proyección basal: inicio de fracción de spine.
pub const ORGAN_ZONE_BASAL_OFFSET: f32 = 0.02;
/// Proyección full: inicio de fracción de spine.
pub const ORGAN_ZONE_FULL_OFFSET: f32 = 0.01;
/// Proyección full: amplitud de distribución.
pub const ORGAN_ZONE_FULL_SPAN: f32 = 0.98;
/// Corte para considerar normal y tangente casi paralelas.
pub const ORGAN_ORIENTATION_PARALLEL_DOT_CUTOFF: f32 = 0.95;

