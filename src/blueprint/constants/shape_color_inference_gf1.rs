// ── Shape & Color Inference (GF1 + Sprint 14 bridge) ──
/// Base length of inferred GF1 shapes (world units).
pub const SHAPE_INF_BASE_LENGTH: f32 = 0.65;
/// Scale factor: length grows with normalized energy.
pub const SHAPE_INF_QE_LENGTH_SCALE: f32 = 1.2;
/// Maximum spine segments per inferred shape (perf ceiling).
pub const SHAPE_INF_MAX_SEGMENTS: u32 = 8;
/// Tube radius as fraction of cell size.
pub const SHAPE_INF_RADIUS_FACTOR: f32 = 0.08;
/// How much `bond_energy` stiffens the shape (higher = straighter).
pub const SHAPE_INF_BOND_RESISTANCE_SCALE: f32 = 0.0003;
/// Fallback resistance when `bond_energy` is not available.
pub const SHAPE_INF_DEFAULT_RESISTANCE: f32 = 0.55;
/// Blend weight of horizontal gradient vs vertical growth direction.
pub const SHAPE_INF_GRADIENT_BLEND: f32 = 0.4;
/// Presupuesto por frame en **unidades de coste** (~ muestreos EPI3 por nodo de spine × ramas estimadas).
pub const SHAPE_INF_MAX_PER_FRAME: u32 = 384;
/// LOD detail assigned to inferred shapes (moderate fidelity).
pub const SHAPE_INF_DETAIL: f32 = 0.55;
