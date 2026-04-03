use crate::blueprint::constants::{
    ALLEN_LIMB_REDUCTION_SCALE, BERGMANN_GROWTH_SCALE, MORPHO_TEMP_EPSILON, WOLFF_ADAPTATION_RATE,
    WOLFF_BOND_ENERGY_MIN, WOLFF_HOMEOSTATIC_LOAD,
};

/// E1: Bergmann radius pressure — cold environments push growth_bias up.
/// `pressure = ((t_target - t_env).max(0) / t_target) * BERGMANN_GROWTH_SCALE`
pub fn bergmann_radius_pressure(t_env: f32, t_target: f32) -> f32 {
    if !t_env.is_finite() || !t_target.is_finite() || t_target <= MORPHO_TEMP_EPSILON {
        return 0.0;
    }
    let thermal_stress = (t_target - t_env).max(0.0) / t_target;
    thermal_stress * BERGMANN_GROWTH_SCALE
}

/// E2: Allen appendage pressure — cold reduces branching (negative pressure).
/// `pressure = -((t_target - t_env).max(0) / t_target) * ALLEN_LIMB_REDUCTION_SCALE`
pub fn allen_appendage_pressure(t_env: f32, t_target: f32) -> f32 {
    if !t_env.is_finite() || !t_target.is_finite() || t_target <= MORPHO_TEMP_EPSILON {
        return 0.0;
    }
    let cold_stress = (t_target - t_env).max(0.0) / t_target;
    -cold_stress * ALLEN_LIMB_REDUCTION_SCALE
}

/// E3: Wolff use-driven bone density — load history drives bond_energy adaptation.
/// `target_bond = current_bond + (load_history - HOMEOSTATIC_LOAD) * WOLFF_ADAPTATION_RATE`
pub fn use_driven_bone_density(load_history: f32, current_bond_energy: f32) -> f32 {
    if !load_history.is_finite() || !current_bond_energy.is_finite() {
        return current_bond_energy.max(WOLFF_BOND_ENERGY_MIN);
    }
    let delta = (load_history - WOLFF_HOMEOSTATIC_LOAD) * WOLFF_ADAPTATION_RATE;
    (current_bond_energy + delta).max(WOLFF_BOND_ENERGY_MIN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        ALLEN_LIMB_REDUCTION_SCALE, BERGMANN_GROWTH_SCALE, MORPHO_TARGET_TEMPERATURE,
    };

    // --- E1: Bergmann ---

    #[test]
    fn bergmann_cold_increases_growth_bias() {
        let pressure = bergmann_radius_pressure(100.0, MORPHO_TARGET_TEMPERATURE);
        assert!(
            pressure > 0.0,
            "cold env should produce positive growth pressure"
        );
        assert!(pressure.is_finite());
    }

    #[test]
    fn bergmann_hot_no_pressure() {
        let pressure = bergmann_radius_pressure(400.0, MORPHO_TARGET_TEMPERATURE);
        assert_eq!(
            pressure, 0.0,
            "hot env should produce zero pressure (clamped)"
        );
    }

    #[test]
    fn bergmann_equal_temp_zero_pressure() {
        let pressure =
            bergmann_radius_pressure(MORPHO_TARGET_TEMPERATURE, MORPHO_TARGET_TEMPERATURE);
        assert_eq!(pressure, 0.0);
    }

    #[test]
    fn bergmann_zero_env_max_stress() {
        let pressure = bergmann_radius_pressure(0.0, MORPHO_TARGET_TEMPERATURE);
        let expected = BERGMANN_GROWTH_SCALE;
        assert!((pressure - expected).abs() < 1e-6);
    }

    #[test]
    fn bergmann_zero_target_returns_zero() {
        assert_eq!(bergmann_radius_pressure(100.0, 0.0), 0.0);
    }

    #[test]
    fn bergmann_nan_returns_zero() {
        assert_eq!(bergmann_radius_pressure(f32::NAN, 300.0), 0.0);
        assert_eq!(bergmann_radius_pressure(100.0, f32::NAN), 0.0);
    }

    #[test]
    fn bergmann_negative_env_produces_positive_pressure() {
        let pressure = bergmann_radius_pressure(-50.0, MORPHO_TARGET_TEMPERATURE);
        assert!(pressure > 0.0);
        assert!(
            pressure > BERGMANN_GROWTH_SCALE,
            "sub-zero env exceeds base scale"
        );
        assert!(pressure.is_finite());
    }

    // --- E2: Allen ---

    #[test]
    fn allen_cold_reduces_branching() {
        let pressure = allen_appendage_pressure(100.0, MORPHO_TARGET_TEMPERATURE);
        assert!(
            pressure < 0.0,
            "cold env should reduce branching (negative pressure)"
        );
        assert!(pressure.is_finite());
    }

    #[test]
    fn allen_hot_no_pressure() {
        let pressure = allen_appendage_pressure(400.0, MORPHO_TARGET_TEMPERATURE);
        assert_eq!(pressure, 0.0);
    }

    #[test]
    fn allen_zero_target_returns_zero() {
        assert_eq!(allen_appendage_pressure(100.0, 0.0), 0.0);
    }

    #[test]
    fn allen_nan_returns_zero() {
        assert_eq!(allen_appendage_pressure(f32::NAN, 300.0), 0.0);
    }

    #[test]
    fn allen_max_cold_stress() {
        let pressure = allen_appendage_pressure(0.0, MORPHO_TARGET_TEMPERATURE);
        let expected = -ALLEN_LIMB_REDUCTION_SCALE;
        assert!((pressure - expected).abs() < 1e-6);
    }

    // --- E3: Wolff ---

    #[test]
    fn wolff_running_entity_increases_bond() {
        let high_load = 0.8;
        let current = 100.0;
        let result = use_driven_bone_density(high_load, current);
        assert!(result > current, "high load should increase bond energy");
    }

    #[test]
    fn wolff_sedentary_decreases_bond() {
        let low_load = 0.0;
        let current = 100.0;
        let result = use_driven_bone_density(low_load, current);
        assert!(result < current, "zero load should decrease bond energy");
    }

    #[test]
    fn wolff_homeostatic_load_no_change() {
        let current = 100.0;
        let result = use_driven_bone_density(WOLFF_HOMEOSTATIC_LOAD, current);
        assert!((result - current).abs() < 1e-6);
    }

    #[test]
    fn wolff_never_below_minimum() {
        let result = use_driven_bone_density(0.0, WOLFF_BOND_ENERGY_MIN);
        assert!(result >= WOLFF_BOND_ENERGY_MIN);
    }

    #[test]
    fn wolff_nan_load_returns_current() {
        let result = use_driven_bone_density(f32::NAN, 100.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn wolff_nan_current_returns_min() {
        let result = use_driven_bone_density(0.5, f32::NAN);
        assert_eq!(result, WOLFF_BOND_ENERGY_MIN);
    }

    #[test]
    fn wolff_result_is_finite() {
        let result = use_driven_bone_density(1.0, 500.0);
        assert!(result.is_finite());
    }
}
