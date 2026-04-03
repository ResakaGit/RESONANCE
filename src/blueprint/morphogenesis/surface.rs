//! Física de superficie MG-1: disipación, albedo inferido, rugosidad.

use std::f32::consts::PI;

use crate::blueprint::constants::DIVISION_GUARD_EPSILON;
use crate::blueprint::constants::morphogenesis as mg;

use super::{san_convection_or_default, san_emissivity, san_finite_or_zero, san_nonneg};

/// Potencia disipable por radiación y convección en superficie:
/// Q_dissipable = ε σ (T_core⁴ - T_env⁴) A_surf + h (T_core - T_env) A_surf.
/// Radiación usa T acotadas a [0, RAD_T_CEILING] para evitar powi(4) overflow en f32.
#[inline]
pub fn surface_dissipation_power(
    emissivity: f32,
    t_core: f32,
    t_env: f32,
    surf_area: f32,
    convection_coeff: f32,
) -> f32 {
    // T^4 overflow en f32 a ~136 000; techo generoso para el modelo.
    const RAD_T_CEILING: f32 = 100_000.0;
    let eps = san_emissivity(emissivity);
    let tc = san_finite_or_zero(t_core);
    let te = san_finite_or_zero(t_env);
    let a = san_nonneg(surf_area);
    let h = san_convection_or_default(convection_coeff);
    let tc_rad = tc.clamp(0.0, RAD_T_CEILING);
    let te_rad = te.clamp(0.0, RAD_T_CEILING);
    let rad = eps * mg::STEFAN_BOLTZMANN * (tc_rad.powi(4) - te_rad.powi(4)) * a;
    let conv = h * (tc - te) * a;
    let out = rad + conv;
    if out.is_finite() { out } else { 0.0 }
}

/// Albedo inferido por balance en superficie:
/// Q_met + (1-α) I A_proj = Q_dissipable → α = 1 - (Q_dissipable - Q_met) / max(I A_proj, ε).
/// Si I A_proj ≈ 0 → `ALBEDO_FALLBACK`. Resultado ∈ [ALBEDO_MIN, ALBEDO_MAX].
/// **Dominio:** `Q_metabolic ≥ 0` (calor generado); valores negativos se tratan como 0.
#[inline]
pub fn inferred_albedo(
    q_metabolic: f32,
    solar_irradiance: f32,
    proj_area: f32,
    emissivity: f32,
    t_core: f32,
    t_env: f32,
    surf_area: f32,
    convection_coeff: f32,
) -> f32 {
    let q_met = san_nonneg(q_metabolic);
    let i = san_nonneg(solar_irradiance);
    let ap = san_nonneg(proj_area);
    let flux = i * ap;
    if flux < mg::ALBEDO_IRRADIANCE_FLUX_EPS {
        return mg::ALBEDO_FALLBACK;
    }
    let q_diss = surface_dissipation_power(emissivity, t_core, t_env, surf_area, convection_coeff);
    let alpha = 1.0 - (q_diss - q_met) / flux;
    alpha.clamp(mg::ALBEDO_MIN, mg::ALBEDO_MAX)
}

/// Irradiancia solar efectiva para cálculo de albedo: `photon_density * absorbed_fraction`.
/// Retorna 0.0 si algún input es negativo o no finito.
#[inline]
pub fn irradiance_effective_for_albedo(photon_density: f32, absorbed_fraction: f32) -> f32 {
    san_nonneg(photon_density) * san_nonneg(absorbed_fraction)
}

/// Modula luminosidad base por albedo: `luminosity_base * (BASE_W + ALBEDO_W * α)`.
/// Preserva matiz elemental (Hz); solo cambia brillo.
/// α=0.05 → factor ≈ 0.335 (oscuro), α=0.95 → factor ≈ 0.965 (claro).
#[inline]
pub fn albedo_luminosity_blend(luminosity_base: f32, albedo: f32) -> f32 {
    let a = san_nonneg(albedo).clamp(mg::ALBEDO_MIN, mg::ALBEDO_MAX);
    luminosity_base * (mg::ALBEDO_LUMINOSITY_BASE_WEIGHT + mg::ALBEDO_LUMINOSITY_ALBEDO_WEIGHT * a)
}

/// Rugosidad de superficie relativa a esfera equivalente:
/// A_needed = Q_total / max(h ΔT, ε), A_sphere = 4π (3V/(4π))^(2/3),
/// rugosity = (A_needed / A_sphere).clamp(RUGOSITY_MIN, RUGOSITY_MAX).
#[inline]
pub fn inferred_surface_rugosity(
    q_total: f32,
    volume: f32,
    t_core: f32,
    t_env: f32,
    convection_coeff: f32,
) -> f32 {
    let q = san_nonneg(q_total);
    let v = if volume.is_finite() {
        volume.max(DIVISION_GUARD_EPSILON)
    } else {
        DIVISION_GUARD_EPSILON
    };
    let tc = san_finite_or_zero(t_core);
    let te = san_finite_or_zero(t_env);
    let h = san_convection_or_default(convection_coeff);
    // Ley MG-1D: divisor h * (T_core - T_env); si ΔT ≤ 0 → piso ε (superficie extra máxima).
    let h_dt = (h * (tc - te)).max(DIVISION_GUARD_EPSILON);
    let a_needed = q / h_dt;
    let r = (3.0 * v / (4.0 * PI)).cbrt().max(DIVISION_GUARD_EPSILON);
    let a_sphere = 4.0 * PI * r * r;
    let rug = a_needed / a_sphere.max(DIVISION_GUARD_EPSILON);
    rug.clamp(mg::RUGOSITY_MIN, mg::RUGOSITY_MAX)
}

/// Traduce rugosity a multiplicador de detail para `GeometryInfluence`.
/// Piecewise-linear C⁰ continua: `[1.0,1.5)→1.0`, `[1.5,2.5)→1.0..1.5`, `[2.5,4.0]→1.5..~2.0`.
#[inline]
pub fn rugosity_to_detail_multiplier(rugosity: f32) -> f32 {
    let r = rugosity.clamp(mg::RUGOSITY_MIN, mg::RUGOSITY_MAX);
    if r < 1.5 {
        1.0
    } else if r < 2.5 {
        1.0 + (r - 1.5) * 0.5
    } else {
        1.5 + (r - 2.5) * 0.33
    }
}
