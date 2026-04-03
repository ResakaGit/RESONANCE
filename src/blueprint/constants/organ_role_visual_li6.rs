// ── LI6 — OrganRole visual modulation tables (12 roles, stateless lookup O(1)) ──
use super::gf1_branch_role_tint::{
    GF1_BRANCH_ROLE_BLEND_LEAF, GF1_BRANCH_ROLE_BLEND_STEM, GF1_BRANCH_ROLE_BLEND_THORN,
};

/// Perfil visual completo por rol de órgano.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrganRoleVisualProfile {
    pub accent_lin: [f32; 3],
    pub blend: f32,
    pub scale: f32,
    pub opacity: f32,
}

/// Tabla canónica LI6: una entrada por `OrganRole` (Stem..Fin).
pub const ORGAN_ROLE_VISUAL_PROFILES: [OrganRoleVisualProfile; 12] = [
    OrganRoleVisualProfile {
        accent_lin: [0.52, 0.38, 0.24], // Stem
        blend: GF1_BRANCH_ROLE_BLEND_STEM,
        scale: 1.0,
        opacity: 1.0,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.38, 0.28, 0.18], // Root
        blend: 0.15,
        scale: 0.6,
        opacity: 1.0,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.45, 0.40, 0.35], // Core
        blend: 0.05,
        scale: 1.2,
        opacity: 1.0,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.18, 0.82, 0.22], // Leaf
        blend: GF1_BRANCH_ROLE_BLEND_LEAF,
        scale: 1.0,
        opacity: 0.85,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.92, 0.35, 0.45], // Petal
        blend: 0.80,
        scale: 1.4,
        opacity: 0.95,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.15, 0.15, 0.15], // Sensory
        blend: 0.50,
        scale: 0.15,
        opacity: 1.0,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.45, 0.38, 0.32], // Thorn
        blend: GF1_BRANCH_ROLE_BLEND_THORN,
        scale: 0.2,
        opacity: 1.0,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.60, 0.55, 0.48], // Shell
        blend: 0.18,
        scale: 0.8,
        opacity: 0.95,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.85, 0.65, 0.20], // Fruit
        blend: 0.35,
        scale: 0.5,
        opacity: 0.90,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.70, 0.85, 0.40], // Bud
        blend: 0.25,
        scale: 0.25,
        opacity: 0.70,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.50, 0.42, 0.35], // Limb
        blend: 0.12,
        scale: 0.7,
        opacity: 1.0,
    },
    OrganRoleVisualProfile {
        accent_lin: [0.40, 0.55, 0.70], // Fin
        blend: 0.20,
        scale: 0.35,
        opacity: 0.80,
    },
];

/// Acento de color lineal RGB por `OrganRole` (Stem..Fin).
pub const ORGAN_ROLE_ACCENT_LIN: [[f32; 3]; 12] = [
    [0.52, 0.38, 0.24], // Stem
    [0.38, 0.28, 0.18], // Root
    [0.45, 0.40, 0.35], // Core
    [0.18, 0.82, 0.22], // Leaf
    [0.92, 0.35, 0.45], // Petal
    [0.15, 0.15, 0.15], // Sensory
    [0.45, 0.38, 0.32], // Thorn
    [0.60, 0.55, 0.48], // Shell
    [0.85, 0.65, 0.20], // Fruit
    [0.70, 0.85, 0.40], // Bud
    [0.50, 0.42, 0.35], // Limb
    [0.40, 0.55, 0.70], // Fin
];

/// Peso de mezcla hacia el acento por `OrganRole` en [0,1].
pub const ORGAN_ROLE_BLEND: [f32; 12] = [
    GF1_BRANCH_ROLE_BLEND_STEM,  // Stem
    0.15,                        // Root
    0.05,                        // Core
    GF1_BRANCH_ROLE_BLEND_LEAF,  // Leaf
    0.80,                        // Petal
    0.50,                        // Sensory
    GF1_BRANCH_ROLE_BLEND_THORN, // Thorn
    0.18,                        // Shell
    0.35,                        // Fruit
    0.25,                        // Bud
    0.12,                        // Limb
    0.20,                        // Fin
];

/// Escala relativa del radio base por `OrganRole` (> 0).
pub const ORGAN_ROLE_SCALE: [f32; 12] = [
    1.0,  // Stem
    0.6,  // Root
    1.2,  // Core
    0.8,  // Leaf
    1.4,  // Petal
    0.15, // Sensory
    0.2,  // Thorn
    0.8,  // Shell
    0.5,  // Fruit
    0.25, // Bud
    0.7,  // Limb
    0.35, // Fin
];

/// Opacidad base por `OrganRole` en [0,1].
pub const ORGAN_ROLE_OPACITY: [f32; 12] = [
    1.0,  // Stem
    1.0,  // Root
    1.0,  // Core
    0.85, // Leaf
    0.95, // Petal
    1.0,  // Sensory
    1.0,  // Thorn
    0.95, // Shell
    0.90, // Fruit
    0.70, // Bud
    1.0,  // Limb
    0.80, // Fin
];
