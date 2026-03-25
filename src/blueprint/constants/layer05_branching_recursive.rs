// ── Capa 5: Branching recursivo (TL5) ──
/// Profundidad máxima de ramificación recursiva por entidad.
pub const BRANCH_MAX_DEPTH: u32 = 3;
/// Decaimiento de radio por nivel de profundidad.
pub const BRANCH_RADIUS_DECAY: f32 = 0.6;
/// Decaimiento de empuje energético por nivel de profundidad.
pub const BRANCH_ENERGY_DECAY: f32 = 0.7;
/// Decaimiento de qe visual por nivel de profundidad.
pub const BRANCH_QE_DECAY: f32 = 0.85;
/// Decaimiento de detalle (LOD natural) por nivel de profundidad.
pub const BRANCH_DETAIL_DECAY: f32 = 0.8;
/// Apertura angular de ramas laterales respecto al eje del padre (radianes).
pub const BRANCH_ANGLE_SPREAD: f32 = 0.6;
/// Techo absoluto de nodos rama por entidad para evitar explosión combinatoria.
pub const MAX_TOTAL_BRANCHES: u32 = 32;
/// Umbral mínimo de biomasa para habilitar branching sobre GF1.
pub const BRANCH_MIN_BIOMASS: f32 = 0.1;
/// Decaimiento del budget hacia hijos por nivel de recursión.
pub const BRANCH_CHILD_BUDGET_DECAY: f32 = 0.5;
/// Presupuesto máximo de regeneraciones de malla por growth por frame (misma escala que EPI3 / `shape_mesh_cost`).
pub const MAX_GROWTH_MORPH_PER_FRAME: u32 = 256;
/// Epsilon geométrico para direcciones degeneradas en branching.
pub const BRANCH_DIR_EPSILON: f32 = 1e-12;

