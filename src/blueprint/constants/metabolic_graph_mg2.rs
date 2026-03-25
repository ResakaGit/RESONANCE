// ── MG-2 — DAG metabólico: factores por `OrganRole` (12 roles, orden Stem..Fin) ──
/// Factor de eficiencia relativa al límite de Carnot (η_nodal = η_Carnot × factor).
pub const ROLE_EFFICIENCY_FACTOR: [f32; 12] = [
    0.8,  // Stem
    0.9,  // Root
    0.7,  // Core
    0.95, // Leaf
    0.6,  // Petal
    0.5,  // Sensory
    0.3,  // Thorn
    0.4,  // Shell
    0.7,  // Fruit
    0.6,  // Bud
    0.75, // Limb
    0.8,  // Fin
];
/// E_a mínima por rol (qe) para arrancar el nodo.
pub const ROLE_ACTIVATION_ENERGY: [f32; 12] = [
    5.0, // Stem
    3.0, // Root
    8.0, // Core
    2.0, // Leaf
    1.0, // Petal
    4.0, // Sensory
    0.5, // Thorn
    1.0, // Shell
    6.0, // Fruit
    2.0, // Bud
    7.0, // Limb
    5.0, // Fin
];
/// Escala base (qe/s) para `ExergyEdge::max_capacity` en inferencia desde `scale_factor`.
pub const METABOLIC_EDGE_CAPACITY_BASE: f32 = 50.0;

