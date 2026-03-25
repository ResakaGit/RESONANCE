use crate::blueprint::constants::*;

// ═══════════════════════════════════════════════
// Color cuantizado (Sprint 14 / docs/design/QUANTIZED_COLOR_ENGINE.md) — CPU alineado a WGSL para entradas finitas; NaN no es contrato en GPU.
// ═══════════════════════════════════════════════

/// Índice en paleta discretizada (O(1), determinista; misma fórmula que `quantized_color.wgsl`).
///
/// Contrato: `enorm` no finito se trata como en `clamp` WGSL (`+∞` → 1, `NaN`/`-∞` → 0).
/// **GPU (EPI4):** `assets/shaders/cell_field_snapshot.wgsl` replica la lógica; NaN/inf vía
/// comprobación IEEE en bits (`bitcast<u32>` + exponente 0xFF), y finitos enormes vía `|x| > 1e38`.
#[inline]
pub fn quantized_palette_index(enorm: f32, rho: f32, n_max: u32) -> u32 {
    if n_max <= 1 {
        return 0;
    }
    let n_max_f = n_max as f32;
    let enorm = if enorm.is_nan() || enorm == f32::NEG_INFINITY {
        0.0
    } else if !enorm.is_finite() {
        1.0
    } else {
        enorm.clamp(0.0, 1.0)
    };
    let rho = if rho.is_finite() {
        rho.clamp(QUANTIZED_COLOR_RHO_CLAMP_FLOOR, 1.0)
    } else {
        1.0
    };
    let s = (n_max_f * rho).ceil().max(1.0);
    let eq = (enorm * s).floor() / s;
    let idx_f = (eq * (n_max_f - 1.0)).floor();
    let idx = if idx_f.is_finite() { idx_f as u32 } else { 0 };
    idx.min(n_max - 1)
}

/// Factor ρ ∈ \[ρ_min, 1\] desde distancia planar y bandas LOD Near/Mid/Far (Sprint 13).
#[inline]
pub fn precision_rho_from_lod_distance(
    distance: f32,
    lod_near_max: f32,
    lod_mid_max: f32,
    rho_min: f32,
) -> f32 {
    let rho_min = if rho_min.is_finite() {
        rho_min.clamp(QUANTIZED_COLOR_RHO_CLAMP_FLOOR, 1.0)
    } else {
        QUANTIZED_COLOR_RHO_MIN
    };
    if !distance.is_finite() {
        return rho_min;
    }
    let d = distance.max(0.0);
    let near = lod_near_max.max(0.0);
    let mid = lod_mid_max.max(near);
    if d <= near {
        return 1.0;
    }
    if d >= mid {
        return rho_min;
    }
    let span = (mid - near).max(QUANTIZED_LOD_RHO_SPAN_EPS);
    let t = ((d - near) / span).clamp(0.0, 1.0);
    1.0 + (rho_min - 1.0) * t
}
