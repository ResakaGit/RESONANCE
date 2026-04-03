//! Color de campo energético: derivación visual, espectro Hz, modulación por rol.

mod field_derivation;
mod role_modulation;
mod spectrum;

pub use field_derivation::{compound_field_linear_rgba, field_linear_rgb_from_hz_purity};
pub use role_modulation::{
    BranchRole, branch_child_role_from_branch_index, branch_role_modulated_linear_rgb,
    organ_role_modulated_rgb, organ_role_opacity, organ_role_scale,
};
pub use spectrum::{
    game_frequency_to_hue, linear_rgb_from_game_hz_spectrum, linear_rgb_from_hz_with_identity,
};

use crate::blueprint::constants::*;

// ═══════════════════════════════════════════════
// Primitivas compartidas — EPI2 / EAC3
// ═══════════════════════════════════════════════

#[inline]
pub(super) fn field_visual_clamp01_or_non_finite(value: f32, non_finite: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        non_finite
    }
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

/// Conversión sRGB → lineal (IEC 61966-2-1). Bevy-free.
/// sRGB to linear conversion (IEC 61966-2-1). Bevy-free.
#[inline]
pub(super) fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// RGB lineal del gris neutro de campo.
#[inline]
pub fn neutral_field_visual_linear_rgb() -> [f32; 3] {
    let lin = srgb_to_linear(FIELD_VISUAL_NEUTRAL_GRAY_CHANNEL);
    [lin, lin, lin]
}

/// Canales RGB lineal: si alguno no es finito → gris neutro.
#[inline]
pub fn field_linear_rgb_sanitize_finite(rgb: [f32; 3]) -> [f32; 3] {
    if rgb.iter().all(|x| x.is_finite()) {
        rgb
    } else {
        neutral_field_visual_linear_rgb()
    }
}

/// Color por vértice de flujo en RGBA lineal (vertex color stateless para GF1 / primitivas de órgano).
///
/// Fórmula: `gain = (0.75 + 0.25 * s_along) * (0.7 + 0.3 * qe_norm)`; borde oscuro proporcional al azimut.
/// Inputs: `qe_norm` ∈ [0,1], `tint_rgb` lineal, `s_along` ∈ [0,1] (fracción longitudinal),
/// `azimuth_along_ring` ∈ [0,1] (fracción angular del anillo).
#[inline]
pub fn vertex_flow_color(
    qe_norm: f32,
    tint_rgb: [f32; 3],
    s_along: f32,
    azimuth_along_ring: f32,
) -> [f32; 4] {
    let q = qe_norm.clamp(0.0, 1.0);
    let edge = (1.0 - azimuth_along_ring.clamp(0.0, 1.0)) * 0.15;
    let g = (0.75 + 0.25 * s_along.clamp(0.0, 1.0)) * (0.7 + 0.3 * q);
    [
        (tint_rgb[0] * g - edge).clamp(0.0, 1.0),
        (tint_rgb[1] * g - edge).clamp(0.0, 1.0),
        (tint_rgb[2] * g - edge).clamp(0.0, 1.0),
        1.0,
    ]
}

/// Tinte de pétalo: `ring_t` ∈ [0,1] (pétalos externos más claros), `u` ∈ [0,1] (base→punta).
///
/// Aplica sombreado radial y longitudinal antes de delegar a [`vertex_flow_color`].
#[inline]
pub fn petal_shaded_flow_color(
    tint_rgb: [f32; 3],
    ring_t: f32,
    u: f32,
    qe_norm: f32,
    v: f32,
) -> [f32; 4] {
    let ring_shade = 0.70 + 0.30 * ring_t.clamp(0.0, 1.0);
    let along_shade = 0.75 + 0.25 * u.clamp(0.0, 1.0);
    let shade = ring_shade * along_shade;
    let tinted = [
        (tint_rgb[0] * shade).clamp(0.0, 1.0),
        (tint_rgb[1] * shade).clamp(0.0, 1.0),
        (tint_rgb[2] * shade).clamp(0.0, 1.0),
    ];
    vertex_flow_color(qe_norm, tinted, u, v)
}
