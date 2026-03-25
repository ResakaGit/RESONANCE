#[inline]
fn phenology_unit_scalar(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Fase fenológica agregada en [0, 1]. Pesos no negativos; si la suma es ~0 → 0.
/// NaN/Inf en `growth_t`, `qe_t`, `purity_t` se tratan como 0.
#[inline]
pub fn phenology_phase(
    growth_t: f32,
    qe_t: f32,
    purity_t: f32,
    w_growth: f32,
    w_qe: f32,
    w_purity: f32,
) -> f32 {
    let g = phenology_unit_scalar(growth_t);
    let q = phenology_unit_scalar(qe_t);
    let p = phenology_unit_scalar(purity_t);
    let wg = phenology_unit_scalar(w_growth);
    let wq = phenology_unit_scalar(w_qe);
    let wp = phenology_unit_scalar(w_purity);
    let sum = wg + wq + wp;
    if sum <= crate::blueprint::constants::PHENOLOGY_WEIGHT_SUM_EPSILON {
        return 0.0;
    }
    (g * wg + q * wq + p * wp) / sum
}

/// `true` si conviene refrescar color (cambio de fase > epsilon). `prev` no finito → true.
/// `next` no finito → false (no forzar escritura con fase inválida).
#[inline]
pub fn phenology_refresh_needed(prev: f32, next: f32, epsilon: f32) -> bool {
    let eps = if epsilon.is_finite() && epsilon >= 0.0 {
        epsilon
    } else {
        0.0
    };
    if !prev.is_finite() {
        return true;
    }
    if !next.is_finite() {
        return false;
    }
    (next - prev).abs() > eps
}
