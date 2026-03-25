use crate::blueprint::constants::{
    LOCOMOTION_KINETIC_FACTOR, SLOPE_COST_SCALE, STAMINA_BASE_RECOVERY,
};
use crate::layers::MatterState;

/// E1: Kinetic energy cost of locomotion.
/// `E_locomotion = KINETIC_FACTOR × mass × speed² × terrain_factor`
pub fn locomotion_energy_cost(mass: f32, speed: f32, terrain_factor: f32) -> f32 {
    if mass <= 0.0 || speed <= 0.0 || terrain_factor <= 0.0 {
        return 0.0;
    }
    LOCOMOTION_KINETIC_FACTOR * mass * speed * speed * terrain_factor
}

/// E2: Terrain locomotion factor from slope, viscosity, and matter state.
/// `f_terrain = (1 + slope × SLOPE_COST_SCALE) × viscosity × state_multiplier`
pub fn terrain_locomotion_factor(slope: f32, viscosity: f32, matter_state: MatterState) -> f32 {
    let state_multiplier = match matter_state {
        MatterState::Solid  => 1.0,
        MatterState::Liquid => 1.5,
        MatterState::Gas    => 0.8,
        MatterState::Plasma => 2.0,
    };
    let slope_clamped = slope.max(0.0);
    (1.0 + slope_clamped * SLOPE_COST_SCALE) * viscosity.max(0.0) * state_multiplier
}

/// E3: Stamina recovery rate, quadratic with buffer fullness.
/// `recovery = BASE_RECOVERY × (current_buffer / max_buffer)²`
pub fn stamina_recovery_rate(current_buffer: f32, max_buffer: f32) -> f32 {
    if max_buffer <= 0.0 {
        return 0.0;
    }
    let ratio = (current_buffer / max_buffer).clamp(0.0, 1.0);
    STAMINA_BASE_RECOVERY * ratio * ratio
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{LOCOMOTION_KINETIC_FACTOR, SLOPE_COST_SCALE, STAMINA_BASE_RECOVERY};

    // --- locomotion_energy_cost ---

    #[test]
    fn locomotion_cost_zero_speed_returns_zero() {
        assert_eq!(locomotion_energy_cost(100.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn locomotion_cost_zero_mass_returns_zero() {
        assert_eq!(locomotion_energy_cost(0.0, 5.0, 1.0), 0.0);
    }

    #[test]
    fn locomotion_cost_zero_terrain_returns_zero() {
        assert_eq!(locomotion_energy_cost(100.0, 5.0, 0.0), 0.0);
    }

    #[test]
    fn locomotion_cost_negative_inputs_return_zero() {
        assert_eq!(locomotion_energy_cost(-10.0, 5.0, 1.0), 0.0);
        assert_eq!(locomotion_energy_cost(100.0, -5.0, 1.0), 0.0);
        assert_eq!(locomotion_energy_cost(100.0, 5.0, -1.0), 0.0);
    }

    #[test]
    fn locomotion_cost_doubles_with_sqrt2_speed() {
        let base = locomotion_energy_cost(100.0, 1.0, 1.0);
        let doubled = locomotion_energy_cost(100.0, std::f32::consts::SQRT_2, 1.0);
        assert!((doubled - 2.0 * base).abs() < 1e-5);
    }

    #[test]
    fn locomotion_cost_quadruples_with_double_speed() {
        let base = locomotion_energy_cost(100.0, 1.0, 1.0);
        let quad = locomotion_energy_cost(100.0, 2.0, 1.0);
        assert!((quad - 4.0 * base).abs() < 1e-5);
    }

    #[test]
    fn locomotion_cost_scales_linearly_with_mass() {
        let cost_100 = locomotion_energy_cost(100.0, 3.0, 1.0);
        let cost_200 = locomotion_energy_cost(200.0, 3.0, 1.0);
        assert!((cost_200 - 2.0 * cost_100).abs() < 1e-5);
    }

    #[test]
    fn locomotion_cost_scales_linearly_with_terrain() {
        let cost_1 = locomotion_energy_cost(100.0, 3.0, 1.0);
        let cost_2 = locomotion_energy_cost(100.0, 3.0, 2.0);
        assert!((cost_2 - 2.0 * cost_1).abs() < 1e-5);
    }

    #[test]
    fn locomotion_cost_known_value() {
        // K=0.002, m=500, s=4, t=1 → 0.002 × 500 × 16 × 1 = 16.0
        let cost = locomotion_energy_cost(500.0, 4.0, 1.0);
        assert!((cost - LOCOMOTION_KINETIC_FACTOR * 500.0 * 16.0).abs() < 1e-5);
    }

    // --- terrain_locomotion_factor ---

    #[test]
    fn terrain_factor_flat_neutral_solid_is_one() {
        let f = terrain_locomotion_factor(0.0, 1.0, MatterState::Solid);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn terrain_factor_uphill_costs_more() {
        let flat = terrain_locomotion_factor(0.0, 1.0, MatterState::Solid);
        let hill = terrain_locomotion_factor(0.5, 1.0, MatterState::Solid);
        assert!(hill > flat);
    }

    #[test]
    fn terrain_factor_steep_slope_formula() {
        // slope=1.0 (45°), viscosity=1.0, solid → (1 + 1.0 × 1.5) × 1.0 × 1.0 = 2.5
        let f = terrain_locomotion_factor(1.0, 1.0, MatterState::Solid);
        assert!((f - (1.0 + SLOPE_COST_SCALE)).abs() < 1e-5);
    }

    #[test]
    fn terrain_factor_liquid_higher_than_solid() {
        let solid  = terrain_locomotion_factor(0.3, 1.0, MatterState::Solid);
        let liquid = terrain_locomotion_factor(0.3, 1.0, MatterState::Liquid);
        assert!(liquid > solid);
    }

    #[test]
    fn terrain_factor_gas_lower_than_solid() {
        let solid = terrain_locomotion_factor(0.3, 1.0, MatterState::Solid);
        let gas   = terrain_locomotion_factor(0.3, 1.0, MatterState::Gas);
        assert!(gas < solid);
    }

    #[test]
    fn terrain_factor_plasma_highest() {
        let solid  = terrain_locomotion_factor(0.3, 1.0, MatterState::Solid);
        let plasma = terrain_locomotion_factor(0.3, 1.0, MatterState::Plasma);
        assert!(plasma > solid);
        assert!((plasma / solid - 2.0).abs() < 1e-5);
    }

    #[test]
    fn terrain_factor_high_viscosity_increases_cost() {
        let low  = terrain_locomotion_factor(0.0, 1.0, MatterState::Solid);
        let high = terrain_locomotion_factor(0.0, 2.0, MatterState::Solid);
        assert!((high - 2.0 * low).abs() < 1e-5);
    }

    #[test]
    fn terrain_factor_zero_viscosity_returns_zero() {
        let f = terrain_locomotion_factor(0.5, 0.0, MatterState::Solid);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn terrain_factor_negative_slope_clamped_to_zero() {
        let f = terrain_locomotion_factor(-0.5, 1.0, MatterState::Solid);
        let flat = terrain_locomotion_factor(0.0, 1.0, MatterState::Solid);
        assert!((f - flat).abs() < 1e-5);
    }

    #[test]
    fn terrain_factor_negative_viscosity_clamped_to_zero() {
        let f = terrain_locomotion_factor(0.5, -1.0, MatterState::Solid);
        assert_eq!(f, 0.0);
    }

    // --- stamina_recovery_rate ---

    #[test]
    fn stamina_recovery_full_buffer_returns_base() {
        let r = stamina_recovery_rate(100.0, 100.0);
        assert!((r - STAMINA_BASE_RECOVERY).abs() < 1e-5);
    }

    #[test]
    fn stamina_recovery_empty_buffer_returns_zero() {
        let r = stamina_recovery_rate(0.0, 100.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn stamina_recovery_half_buffer_quarter_rate() {
        // ratio=0.5, recovery = BASE × 0.5² = BASE × 0.25
        let r = stamina_recovery_rate(50.0, 100.0);
        assert!((r - STAMINA_BASE_RECOVERY * 0.25).abs() < 1e-5);
    }

    #[test]
    fn stamina_recovery_zero_max_returns_zero() {
        let r = stamina_recovery_rate(50.0, 0.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn stamina_recovery_negative_max_returns_zero() {
        let r = stamina_recovery_rate(50.0, -10.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn stamina_recovery_over_full_clamps_to_base() {
        let r = stamina_recovery_rate(200.0, 100.0);
        assert!((r - STAMINA_BASE_RECOVERY).abs() < 1e-5);
    }

    #[test]
    fn stamina_recovery_quadratic_not_linear() {
        let quarter = stamina_recovery_rate(25.0, 100.0);
        let half    = stamina_recovery_rate(50.0, 100.0);
        // quarter: BASE × 0.0625, half: BASE × 0.25
        // ratio should be 4:1, not 2:1 (quadratic)
        assert!((half / quarter - 4.0).abs() < 1e-4);
    }
}
