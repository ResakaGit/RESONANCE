//! Color de campo energético: derivación visual, espectro Hz, modulación por rol.

mod spectrum;
mod field_derivation;
mod role_modulation;

pub use spectrum::{
    game_frequency_to_hue, linear_rgb_from_game_hz_spectrum, linear_rgb_from_hz_with_identity,
};
pub use field_derivation::{compound_field_linear_rgba, field_linear_rgb_from_hz_purity};
pub use role_modulation::{
    BranchRole, branch_child_role_from_branch_index, branch_role_modulated_linear_rgb,
    organ_role_modulated_rgb, organ_role_opacity, organ_role_scale,
};

use bevy::color::Color;
use crate::blueprint::constants::*;

// ═══════════════════════════════════════════════
// Primitivas compartidas — EPI2 / EAC3
// ═══════════════════════════════════════════════

#[inline]
pub(super) fn field_visual_clamp01_or_non_finite(value: f32, non_finite: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 1.0) } else { non_finite }
}

/// Factor de mezcla en [0, 1] para interpolación de tintes (NaN/Inf → 0).
#[inline]
pub fn field_visual_mix_unit(t: f32) -> f32 {
    field_visual_clamp01_or_non_finite(t, 0.0)
}

/// Interpolación RGB lineal con `t` ya en [0, 1] (sin volver a clamp).
#[inline]
pub(crate) fn linear_rgb_lerp_preclamped(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Interpolación por canal en RGB **lineal** (sin gamma). `t` se normaliza con [`field_visual_mix_unit`].
#[inline]
pub fn linear_rgb_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    linear_rgb_lerp_preclamped(a, b, field_visual_mix_unit(t))
}

#[inline]
pub(super) fn field_visual_sanitize_unit(value: f32, fallback: f32) -> f32 {
    field_visual_clamp01_or_non_finite(value, fallback)
}

#[inline]
pub(super) fn linear_rgba_lerp_preclamped(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// RGB lineal del gris neutro de campo.
#[inline]
pub fn neutral_field_visual_linear_rgb() -> [f32; 3] {
    let c = FIELD_VISUAL_NEUTRAL_GRAY_CHANNEL;
    let lin = Color::srgb(c, c, c).to_linear();
    [lin.red, lin.green, lin.blue]
}

/// Canales RGB lineal: si alguno no es finito → gris neutro.
#[inline]
pub fn field_linear_rgb_sanitize_finite(rgb: [f32; 3]) -> [f32; 3] {
    if rgb.iter().all(|x| x.is_finite()) { rgb } else { neutral_field_visual_linear_rgb() }
}
