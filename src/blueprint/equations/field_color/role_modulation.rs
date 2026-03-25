//! Modulación de color por rol (BranchRole GF1, OrganRole LI6).

use crate::blueprint::constants::*;
use crate::layers::OrganRole;

use super::{
    field_visual_mix_unit, linear_rgb_lerp_preclamped,
};

/// Rol de rama GF1 (dato de inferencia; la modulación es tabla + puras, no `match` por especie).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BranchRole {
    #[default]
    Stem  = 0,
    Leaf  = 1,
    Thorn = 2,
}

/// Acentos y pesos alineados a [`BranchRole`] discriminant (Stem=0, Leaf=1, Thorn=2).
const GF1_ROLE_ACCENT_LIN: [[f32; 3]; 3] = [
    GF1_BRANCH_ROLE_ACCENT_STEM_LIN,
    GF1_BRANCH_ROLE_ACCENT_LEAF_LIN,
    GF1_BRANCH_ROLE_ACCENT_THORN_LIN,
];
const GF1_ROLE_BLEND: [f32; 3] = [
    GF1_BRANCH_ROLE_BLEND_STEM,
    GF1_BRANCH_ROLE_BLEND_LEAF,
    GF1_BRANCH_ROLE_BLEND_THORN,
];

const _: () = assert!(GF1_ROLE_ACCENT_LIN.len() == GF1_ROLE_BLEND.len());
const _: () = assert!(GF1_ROLE_ACCENT_LIN.len() == 3);
const _: () = assert!(ORGAN_ROLE_VISUAL_PROFILES.len() == OrganRole::COUNT);
const _: () = assert!(ORGAN_ROLE_ACCENT_LIN.len() == OrganRole::COUNT);
const _: () = assert!(ORGAN_ROLE_BLEND.len() == OrganRole::COUNT);
const _: () = assert!(ORGAN_ROLE_SCALE.len() == OrganRole::COUNT);
const _: () = assert!(ORGAN_ROLE_OPACITY.len() == OrganRole::COUNT);

#[inline]
fn sanitize_linear_rgb_non_finite_to_zero(c: f32) -> f32 {
    if c.is_finite() { c } else { 0.0 }
}

#[inline]
fn sanitize_linear_rgb3_non_finite_to_zero(rgb: [f32; 3]) -> [f32; 3] {
    [
        sanitize_linear_rgb_non_finite_to_zero(rgb[0]),
        sanitize_linear_rgb_non_finite_to_zero(rgb[1]),
        sanitize_linear_rgb_non_finite_to_zero(rgb[2]),
    ]
}

#[inline]
fn role_modulated_linear_rgb_from_tables(
    field_rgb: [f32; 3],
    role_index: usize,
    accent_table: &[[f32; 3]],
    blend_table: &[f32],
) -> [f32; 3] {
    debug_assert!(role_index < accent_table.len());
    debug_assert!(accent_table.len() == blend_table.len());
    let field_rgb = sanitize_linear_rgb3_non_finite_to_zero(field_rgb);
    let w = field_visual_mix_unit(blend_table[role_index]);
    linear_rgb_lerp_preclamped(field_rgb, accent_table[role_index], w)
}

#[inline]
fn organ_role_visual_profile(role: OrganRole) -> OrganRoleVisualProfile {
    ORGAN_ROLE_VISUAL_PROFILES[role as usize]
}

/// Mezcla RGB lineal del muestreo de campo hacia acento de rol (EPI3).
/// Canales no finitos del campo → 0; finitos se preservan.
#[inline]
pub fn branch_role_modulated_linear_rgb(field_rgb: [f32; 3], role: BranchRole) -> [f32; 3] {
    let i = role as usize;
    role_modulated_linear_rgb_from_tables(field_rgb, i, &GF1_ROLE_ACCENT_LIN, &GF1_ROLE_BLEND)
}

/// Mezcla RGB lineal del muestreo de campo hacia acento de `OrganRole` (LI6).
/// Canales no finitos del campo → 0; finitos se preservan.
#[inline]
pub fn organ_role_modulated_rgb(field_rgb: [f32; 3], role: OrganRole) -> [f32; 3] {
    let profile = organ_role_visual_profile(role);
    let field_rgb = sanitize_linear_rgb3_non_finite_to_zero(field_rgb);
    let w = field_visual_mix_unit(profile.blend);
    linear_rgb_lerp_preclamped(field_rgb, profile.accent_lin, w)
}

/// Escala de radio para primitiva de órgano según rol.
#[inline]
pub fn organ_role_scale(role: OrganRole, base_radius: f32) -> f32 {
    let scaled = base_radius * organ_role_visual_profile(role).scale;
    if scaled.is_finite() { scaled.max(0.001) } else { 0.001 }
}

/// Opacidad base de órgano en [0,1] para la capa visual.
#[inline]
pub fn organ_role_opacity(role: OrganRole) -> f32 {
    field_visual_mix_unit(organ_role_visual_profile(role).opacity)
}

/// Rol de hijo en ramificación recursiva: ciclo determinista por índice + profundidad (sin RNG).
#[inline]
pub fn branch_child_role_from_branch_index(child_index: usize, depth: u32) -> BranchRole {
    const CYCLE: [BranchRole; 3] = [BranchRole::Leaf, BranchRole::Thorn, BranchRole::Stem];
    let i = (child_index as u32).wrapping_add(depth) % 3;
    CYCLE[i as usize]
}
