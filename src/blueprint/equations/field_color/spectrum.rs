//! EAC4 — Hz → matiz (arco iris de juego, espectro alquímico).

use crate::blueprint::constants::*;

use super::{
    field_linear_rgb_sanitize_finite, field_visual_sanitize_unit,
    linear_rgb_lerp_preclamped, neutral_field_visual_linear_rgb,
};

#[inline]
fn sanitize_eac4_hz_identity_weight(w: f32) -> f32 {
    if w.is_finite() {
        w.clamp(0.0, 1.0)
    } else {
        FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY
    }
}

/// Matiz para el eje Hz de juego (resonancia alquímica, no λ electromagnética).
/// Mapeo lineal al span; en `f == f_max` se devuelve `1.0 - ε` para evitar salto cromático.
/// No finitos o `f_max <= f_min` → `0.0`.
#[inline]
pub fn game_frequency_to_hue(frequency_hz: f32, f_min: f32, f_max: f32) -> f32 {
    if !frequency_hz.is_finite() || !f_min.is_finite() || !f_max.is_finite() {
        return 0.0;
    }
    if f_max <= f_min {
        return 0.0;
    }
    let t = ((frequency_hz - f_min) / (f_max - f_min)).clamp(0.0, 1.0);
    if t >= 1.0 { 1.0 - f32::EPSILON } else { t }
}

/// HSV con H,S,V ∈ [0,1] → RGB lineal (vía sRGB). Sin RNG.
#[inline]
fn hsv01_to_linear_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    if !h.is_finite() || !s.is_finite() || !v.is_finite() {
        return neutral_field_visual_linear_rgb();
    }
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let h = (h.fract() + 1.0).fract();
    let region = h * 6.0;
    let i = region.floor();
    let f = region - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (rp, gp, bp) = match i as i32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    let lin = bevy::color::Color::srgb(rp, gp, bp).to_linear();
    [lin.red, lin.green, lin.blue]
}

/// Arco iris de juego: Hz normalizado al span + pureza como saturación HSV.
#[inline]
pub fn linear_rgb_from_game_hz_spectrum(
    frequency_hz: f32,
    purity: f32,
    f_min: f32,
    f_max: f32,
) -> [f32; 3] {
    if !f_min.is_finite() || !f_max.is_finite() || f_max <= f_min {
        return neutral_field_visual_linear_rgb();
    }
    let hue = game_frequency_to_hue(frequency_hz, f_min, f_max);
    let s = field_visual_sanitize_unit(purity, 0.0);
    let out = hsv01_to_linear_rgb(hue, s, FIELD_EAC4_HZ_SPECTRUM_VALUE);
    field_linear_rgb_sanitize_finite(out)
}

/// Mezcla híbrida: `w_identity = 1` → `def_linear_rgb`; `0` → espectro Hz.
#[inline]
pub fn linear_rgb_from_hz_with_identity(
    frequency_hz: f32,
    purity: f32,
    def_linear_rgb: [f32; 3],
    w_identity: f32,
    f_min: f32,
    f_max: f32,
) -> [f32; 3] {
    let hz_rgb = linear_rgb_from_game_hz_spectrum(frequency_hz, purity, f_min, f_max);
    let w = sanitize_eac4_hz_identity_weight(w_identity);
    let blended = linear_rgb_lerp_preclamped(hz_rgb, def_linear_rgb, w);
    field_linear_rgb_sanitize_finite(blended)
}

/// Peso identidad saneado (reusable desde field_derivation).
#[inline]
pub(super) fn sanitize_eac4_hz_identity_weight_pub(w: f32) -> f32 {
    sanitize_eac4_hz_identity_weight(w)
}
