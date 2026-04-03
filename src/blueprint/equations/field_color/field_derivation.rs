//! Derivación de color de campo: Hz dominante + pureza → RGB, mezcla compuesta.

use crate::blueprint::almanac::{AlchemicalAlmanac, ElementDef};
use crate::blueprint::constants::*;

use super::spectrum::{linear_rgb_from_hz_with_identity, sanitize_eac4_hz_identity_weight_pub};
use super::{
    field_linear_rgb_sanitize_finite, field_visual_clamp01_or_non_finite,
    field_visual_sanitize_unit, linear_rgb_lerp_preclamped, linear_rgba_lerp_preclamped,
    neutral_field_visual_linear_rgb,
};

#[inline]
fn element_def_color_to_linear_rgb(def: &ElementDef) -> [f32; 3] {
    let nch = FIELD_VISUAL_NEUTRAL_GRAY_CHANNEL;
    let r = field_visual_sanitize_unit(def.color.0, nch);
    let g = field_visual_sanitize_unit(def.color.1, nch);
    let b = field_visual_sanitize_unit(def.color.2, nch);
    [
        super::srgb_to_linear(r),
        super::srgb_to_linear(g),
        super::srgb_to_linear(b),
    ]
}

/// Hz dominante + pureza → RGB lineal. Misma semántica que `worldgen::visual_derivation::derive_color`.
#[inline]
pub fn field_linear_rgb_from_hz_purity(
    frequency_hz: f32,
    purity: f32,
    almanac: &AlchemicalAlmanac,
) -> [f32; 3] {
    let purity = field_visual_sanitize_unit(purity, 0.0);
    let neutral = neutral_field_visual_linear_rgb();
    let base = if frequency_hz.is_finite() {
        let freq = frequency_hz.max(0.0);
        match almanac.find_stable_band(freq) {
            None => neutral,
            Some(def) => {
                let ron_lin = element_def_color_to_linear_rgb(def);
                let w = sanitize_eac4_hz_identity_weight_pub(def.hz_identity_weight);
                let blended = if w
                    >= FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY - FIELD_EAC4_IDENTITY_FULL_RON_EPS
                {
                    ron_lin
                } else if let Some((f_min, f_max)) = almanac.game_frequency_hz_bounds() {
                    linear_rgb_from_hz_with_identity(
                        freq,
                        FIELD_EAC4_HYBRID_SPECTRUM_PURITY,
                        ron_lin,
                        w,
                        f_min,
                        f_max,
                    )
                } else {
                    ron_lin
                };
                field_linear_rgb_sanitize_finite(blended)
            }
        }
    } else {
        neutral
    };
    let out = linear_rgb_lerp_preclamped(neutral, base, purity);
    field_linear_rgb_sanitize_finite(out)
}

/// Mezcla compuesta en RGB lineal + alpha. Misma semántica que `compound_color_blend` en `visual_derivation`.
#[inline]
pub fn compound_field_linear_rgba(
    primary: [f32; 4],
    secondary: [f32; 4],
    interference: f32,
    purity: f32,
) -> [f32; 4] {
    let purity = field_visual_sanitize_unit(purity, 0.0);
    let interference = interference.clamp(
        FIELD_COMPOUND_INTERFERENCE_CLAMP_MIN,
        FIELD_COMPOUND_INTERFERENCE_CLAMP_MAX,
    );
    let neutral = neutral_field_visual_linear_rgb();
    let neutral_rgba = [
        neutral[0],
        neutral[1],
        neutral[2],
        FIELD_VISUAL_OPAQUE_ALPHA,
    ];

    let out = if interference >= 0.0 {
        let secondary_weight = (1.0 - purity)
            * (1.0 - FIELD_COMPOUND_BLEND_CONSTRUCTIVE_INTERFERENCE_WEIGHT * interference);
        let tw = field_visual_sanitize_unit(secondary_weight, 0.0);
        linear_rgba_lerp_preclamped(primary, secondary, tw)
    } else {
        let mixed =
            linear_rgba_lerp_preclamped(primary, secondary, FIELD_COMPOUND_BLEND_DESTRUCTIVE_BASE);
        let destructive_strength =
            field_visual_sanitize_unit((-interference) * (1.0 - purity), 0.0);
        linear_rgba_lerp_preclamped(mixed, neutral_rgba, destructive_strength)
    };
    let rgb = field_linear_rgb_sanitize_finite([out[0], out[1], out[2]]);
    let a = if out[3].is_finite() {
        field_visual_clamp01_or_non_finite(out[3], FIELD_VISUAL_OPAQUE_ALPHA)
    } else {
        FIELD_VISUAL_OPAQUE_ALPHA
    };
    [rgb[0], rgb[1], rgb[2], a]
}
