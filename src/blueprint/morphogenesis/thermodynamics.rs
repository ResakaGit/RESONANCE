//! Termodinámica MG-1: Carnot, entropía, exergía, capacidad calorífica.

use crate::blueprint::constants::DIVISION_GUARD_EPSILON;

use super::{san_efficiency_01, san_nonneg};

/// Eficiencia máxima de Carnot: η_max = 1 - T_env / T_core.
/// **Dominio:** temperaturas equivalentes ≥ 0 (modelo tipo Kelvin); si `t_core ≤ 0`, `t_env < 0` o `t_core ≤ t_env` → 0.
/// Rango de salida: [0, 1) (techo estricto por estabilidad numérica).
#[inline]
pub fn carnot_efficiency(t_core: f32, t_env: f32) -> f32 {
    if !t_core.is_finite() || !t_env.is_finite() {
        return 0.0;
    }
    if t_core <= 0.0 || t_env < 0.0 {
        return 0.0;
    }
    if t_core <= t_env {
        return 0.0;
    }
    let tc = t_core.max(DIVISION_GUARD_EPSILON);
    let eta = 1.0 - t_env / tc;
    eta.clamp(0.0, 1.0 - f32::EPSILON)
}

/// Producción entrópica: S_gen = Q_diss / T_core (qe/K equivalente en el modelo).
/// Q_diss < 0 se trata como 0 (la entropía generada no es negativa).
/// `t_core` no finito o ≤ 0 → 0 (sin producción definida).
#[inline]
pub fn entropy_production(q_diss: f32, t_core: f32) -> f32 {
    if !t_core.is_finite() || t_core <= 0.0 {
        return 0.0;
    }
    let q = san_nonneg(q_diss);
    let tc = t_core.max(DIVISION_GUARD_EPSILON);
    q / tc
}

/// Exergía útil tras conversión: Ex = max(0, J_in * η - E_a).
#[inline]
pub fn exergy_balance(j_in: f32, efficiency: f32, activation_energy: f32) -> f32 {
    let j = san_nonneg(j_in);
    let eta = san_efficiency_01(efficiency);
    let ea = san_nonneg(activation_energy);
    (j * eta - ea).max(0.0)
}

/// Capacidad calorífica efectiva: C_v = qe * k_C (permite dT = dQ / C_v).
#[inline]
pub fn heat_capacity(qe: f32, specific_heat_factor: f32) -> f32 {
    san_nonneg(qe) * san_nonneg(specific_heat_factor)
}
