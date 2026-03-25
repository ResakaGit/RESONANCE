//! Mecánica constructal MG-1: costo de forma, transporte vascular, arrastre inferido.

use crate::blueprint::constants::morphogenesis as mg;
use crate::blueprint::constants::{DIVISION_GUARD_EPSILON, DRAG_SPEED_EPSILON};

use super::{san_nonneg, san_velocity_mag};

/// Costo constructal de forma: C = ½ ρ v² C_D A_proj + C_vasc.
/// Con v ≈ 0 solo queda el costo vascular interno.
#[inline]
pub fn shape_cost(
    medium_density: f32,
    velocity: f32,
    drag_coeff: f32,
    projected_area: f32,
    vascular_cost: f32,
) -> f32 {
    let rho = san_nonneg(medium_density);
    let v   = san_velocity_mag(velocity);
    let cd  = san_nonneg(drag_coeff);
    let a   = san_nonneg(projected_area);
    let cv  = san_nonneg(vascular_cost);
    let drag = if v <= DRAG_SPEED_EPSILON {
        0.0
    } else {
        0.5 * rho * v * v * cd * a
    };
    drag + cv
}

/// Costo de transporte viscoso (Hagen-Poiseuille simplificado): C_t = μ L³ / max(r⁴, ε).
#[inline]
pub fn vascular_transport_cost(viscosity: f32, length: f32, radius: f32) -> f32 {
    let mu = san_nonneg(viscosity);
    let l  = san_nonneg(length);
    let r  = san_nonneg(radius);
    let r4 = r.powi(4).max(DIVISION_GUARD_EPSILON);
    let out = mu * l.powi(3) / r4;
    if out.is_finite() { out } else { 0.0 }
}

/// Coeficiente de arrastre inferido (cuerpo de revolución tipo Myring):
/// fineness = L / D, C_D = C_base / (1 + k * fineness²), acotado a [DRAG_COEFF_MIN, DRAG_COEFF_BASE].
#[inline]
pub fn inferred_drag_coefficient(length: f32, max_diameter: f32) -> f32 {
    let l = san_nonneg(length);
    let d = san_nonneg(max_diameter);
    let fineness = l / d.max(DIVISION_GUARD_EPSILON);
    let denom = 1.0 + mg::DRAG_FINENESS_SCALE * fineness * fineness;
    let cd = mg::DRAG_COEFF_BASE / denom;
    cd.clamp(mg::DRAG_COEFF_MIN, mg::DRAG_COEFF_BASE)
}

/// Descenso acotado del fineness_ratio para minimizar shape_cost (MG-4).
/// Gradiente numérico finite-difference, `max_iter` pasos, damping ∈ (0,1].
/// Retorna fineness clamped a [FINENESS_MIN, FINENESS_MAX].
pub fn bounded_fineness_descent(
    current_fineness: f32,
    medium_density: f32,
    velocity: f32,
    projected_area: f32,
    vascular_cost: f32,
    damping: f32,
    max_iter: u32,
) -> f32 {
    let mut f = san_nonneg(current_fineness).clamp(mg::FINENESS_MIN, mg::FINENESS_MAX);
    let rho = san_nonneg(medium_density);
    let v   = san_velocity_mag(velocity);
    let a   = san_nonneg(projected_area);
    let cv  = san_nonneg(vascular_cost);
    let damp = san_nonneg(damping).clamp(DIVISION_GUARD_EPSILON, 1.0);

    for _ in 0..max_iter {
        let f_lo = (f - mg::SHAPE_FD_DELTA).max(mg::FINENESS_MIN);
        let f_hi = (f + mg::SHAPE_FD_DELTA).min(mg::FINENESS_MAX);
        let span = f_hi - f_lo;
        if span < DIVISION_GUARD_EPSILON {
            break;
        }
        let cd_lo = inferred_drag_coefficient(f_lo, 1.0);
        let cd_hi = inferred_drag_coefficient(f_hi, 1.0);
        let c_lo = shape_cost(rho, v, cd_lo, a, cv);
        let c_hi = shape_cost(rho, v, cd_hi, a, cv);
        let grad = (c_hi - c_lo) / span;
        let step = (damp * grad).clamp(-mg::SHAPE_FD_DELTA, mg::SHAPE_FD_DELTA);
        f -= step;
        f = f.clamp(mg::FINENESS_MIN, mg::FINENESS_MAX);
    }
    f
}
