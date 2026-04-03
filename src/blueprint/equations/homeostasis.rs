//! D4: Homeostasis & thermoregulation — pure math.

use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::DIVISION_GUARD_EPSILON;

/// Cost of thermoregulation: Q = k * SA * |t_core - t_env| / insulation.
/// SA = mass^(2/3) (allometric surface area approximation).
#[inline]
pub fn thermoregulation_cost(
    t_core: f32,
    t_env: f32,
    mass: f32,
    conductivity: f32,
    insulation: f32,
) -> f32 {
    if !t_core.is_finite() || !t_env.is_finite() {
        return 0.0;
    }
    let m = finite_non_negative(mass);
    let k = finite_non_negative(conductivity);
    let ins = finite_non_negative(insulation).max(DIVISION_GUARD_EPSILON);
    let delta_t = (t_core - t_env).abs();
    let surface_area = m.powf(2.0 / 3.0);
    k * surface_area * delta_t / ins
}

/// Ectotherm body temperature: converges toward t_env each tick.
/// t_new = t_current + (t_env - t_current) * rate.
#[inline]
pub fn ectotherm_temperature(t_current: f32, t_env: f32, convergence_rate: f32) -> f32 {
    if !t_current.is_finite() || !t_env.is_finite() {
        return 0.0;
    }
    let rate = finite_unit(convergence_rate);
    t_current + (t_env - t_current) * rate
}

/// Endotherm body temperature: maintains t_target when qe suffices.
/// Full maintenance when qe >= gap/insulation; partial interpolation otherwise.
#[inline]
pub fn endotherm_temperature(t_target: f32, t_env: f32, insulation: f32, qe_available: f32) -> f32 {
    if !t_target.is_finite() || !t_env.is_finite() {
        return 0.0;
    }
    let qe = finite_non_negative(qe_available);
    if qe <= 0.0 {
        return t_env;
    }
    let ins = finite_non_negative(insulation).max(DIVISION_GUARD_EPSILON);
    let gap = (t_target - t_env).abs();
    let required_qe = gap / ins;
    if required_qe <= 0.0 {
        return t_target;
    }
    let fraction = (qe / required_qe).clamp(0.0, 1.0);
    t_env + (t_target - t_env) * fraction
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── thermoregulation_cost ──

    #[test]
    fn thermoreg_cost_zero_when_same_temperature() {
        let cost = thermoregulation_cost(310.0, 310.0, 100.0, 0.5, 1.0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn thermoreg_endotherm_costs_more_in_cold() {
        let cost_mild = thermoregulation_cost(310.0, 300.0, 100.0, 0.5, 1.0);
        let cost_cold = thermoregulation_cost(310.0, 200.0, 100.0, 0.5, 1.0);
        assert!(cost_cold > cost_mild);
        assert!(cost_mild > 0.0);
    }

    #[test]
    fn thermoreg_cost_zero_mass_gives_zero() {
        let cost = thermoregulation_cost(310.0, 200.0, 0.0, 0.5, 1.0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn thermoreg_cost_zero_conductivity_gives_zero() {
        let cost = thermoregulation_cost(310.0, 200.0, 100.0, 0.0, 1.0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn thermoreg_cost_high_insulation_reduces_cost() {
        let cost_low_ins = thermoregulation_cost(310.0, 200.0, 100.0, 0.5, 1.0);
        let cost_high_ins = thermoregulation_cost(310.0, 200.0, 100.0, 0.5, 5.0);
        assert!(cost_high_ins < cost_low_ins);
        assert!(cost_high_ins > 0.0);
    }

    #[test]
    fn thermoreg_cost_nan_inputs_return_zero() {
        assert_eq!(thermoregulation_cost(f32::NAN, 310.0, 100.0, 0.5, 1.0), 0.0);
        assert_eq!(thermoregulation_cost(310.0, f32::NAN, 100.0, 0.5, 1.0), 0.0);
    }

    #[test]
    fn thermoreg_cost_infinite_inputs_return_zero() {
        assert_eq!(
            thermoregulation_cost(f32::INFINITY, 310.0, 100.0, 0.5, 1.0),
            0.0
        );
    }

    #[test]
    fn thermoreg_cost_negative_mass_treated_as_zero() {
        let cost = thermoregulation_cost(310.0, 200.0, -50.0, 0.5, 1.0);
        assert_eq!(cost, 0.0);
    }

    // ── ectotherm_temperature ──

    #[test]
    fn thermoreg_ectotherm_converges_to_ambient() {
        let t_new = ectotherm_temperature(310.0, 280.0, 0.5);
        assert!((t_new - 295.0).abs() < 1e-5);
    }

    #[test]
    fn ectotherm_full_convergence_at_rate_one() {
        let t_new = ectotherm_temperature(310.0, 280.0, 1.0);
        assert!((t_new - 280.0).abs() < 1e-5);
    }

    #[test]
    fn ectotherm_no_change_at_rate_zero() {
        let t_new = ectotherm_temperature(310.0, 280.0, 0.0);
        assert!((t_new - 310.0).abs() < 1e-5);
    }

    #[test]
    fn ectotherm_already_at_ambient_stays() {
        let t_new = ectotherm_temperature(280.0, 280.0, 0.5);
        assert!((t_new - 280.0).abs() < 1e-5);
    }

    #[test]
    fn ectotherm_nan_returns_zero() {
        assert_eq!(ectotherm_temperature(f32::NAN, 280.0, 0.5), 0.0);
        assert_eq!(ectotherm_temperature(310.0, f32::NAN, 0.5), 0.0);
    }

    #[test]
    fn ectotherm_rate_clamped_to_unit() {
        let t_clamped = ectotherm_temperature(310.0, 280.0, 2.0);
        let t_one = ectotherm_temperature(310.0, 280.0, 1.0);
        assert!((t_clamped - t_one).abs() < 1e-5);
    }

    // ── endotherm_temperature ──

    #[test]
    fn endotherm_maintains_target_with_excess_qe() {
        let t = endotherm_temperature(310.0, 280.0, 1.0, 1000.0);
        assert!((t - 310.0).abs() < 1e-5);
    }

    #[test]
    fn endotherm_drifts_to_env_without_qe() {
        let t = endotherm_temperature(310.0, 280.0, 1.0, 0.0);
        assert!((t - 280.0).abs() < 1e-5);
    }

    #[test]
    fn endotherm_partial_maintenance_with_limited_qe() {
        // gap=30, insulation=1.0, required=30, qe=15 → fraction=0.5
        // result = 280 + 30*0.5 = 295
        let t = endotherm_temperature(310.0, 280.0, 1.0, 15.0);
        assert!((t - 295.0).abs() < 1e-5);
    }

    #[test]
    fn endotherm_high_insulation_needs_less_qe() {
        // gap=30, insulation=3.0, required=10, qe=10 → fraction=1.0
        let t = endotherm_temperature(310.0, 280.0, 3.0, 10.0);
        assert!((t - 310.0).abs() < 1e-5);
    }

    #[test]
    fn endotherm_nan_returns_zero() {
        assert_eq!(endotherm_temperature(f32::NAN, 280.0, 1.0, 100.0), 0.0);
        assert_eq!(endotherm_temperature(310.0, f32::NAN, 1.0, 100.0), 0.0);
    }

    #[test]
    fn endotherm_same_temp_returns_target() {
        let t = endotherm_temperature(310.0, 310.0, 1.0, 0.0);
        // gap=0, required=0 → returns t_target
        assert!((t - 310.0).abs() < 1e-5);
    }

    #[test]
    fn endotherm_negative_qe_treated_as_zero() {
        let t = endotherm_temperature(310.0, 280.0, 1.0, -10.0);
        assert!((t - 280.0).abs() < 1e-5);
    }
}
