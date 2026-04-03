use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::*;

/// Probabilidad de éxito de predación: `base + speed_advantage × scale`, clamped [0,1].
/// `predator_speed / prey_speed > 1` → ventaja; `distance` penaliza linealmente.
#[inline]
pub fn predation_success_probability(
    predator_speed: f32,
    prey_speed: f32,
    distance: f32,
    terrain_factor: f32,
) -> f32 {
    let pred_spd = finite_non_negative(predator_speed);
    let prey_spd = finite_non_negative(prey_speed).max(PREY_SPEED_FLOOR);
    let dist = finite_non_negative(distance);
    let terrain = finite_unit(terrain_factor);
    let speed_ratio = (pred_spd / prey_spd - 1.0).max(0.0);
    let speed_bonus = speed_ratio * PREDATION_SPEED_ADVANTAGE_SCALE;
    let distance_penalty =
        (dist * PREDATION_DISTANCE_PENALTY_SCALE).min(PREDATION_DISTANCE_PENALTY_MAX);
    let raw = (PREDATION_BASE_SUCCESS + speed_bonus - distance_penalty) * terrain;
    raw.clamp(0.0, 1.0)
}

/// qe transferida de presa a predador, modulada por bond_energy (resistencia) y eficiencia.
/// `bond_energy` alto → la presa resiste más, menos transferencia.
#[inline]
pub fn prey_qe_transfer(prey_qe: f32, bond_energy: f32, assimilation_efficiency: f32) -> f32 {
    let qe = finite_non_negative(prey_qe);
    let bond = finite_non_negative(bond_energy);
    let efficiency = finite_unit(assimilation_efficiency);
    let resistance = 1.0 / (1.0 + bond * PREY_BOND_RESISTANCE_SCALE);
    (qe * resistance * efficiency).max(0.0)
}

/// qe extraída de una celda de nutrientes durante foraging: `min(cell_qe × intake_rate × dt, drain_max)`.
#[inline]
pub fn foraging_intake_from_field(cell_qe: f32, intake_rate: f32, dt: f32) -> f32 {
    let cq = finite_non_negative(cell_qe);
    let rate = finite_non_negative(intake_rate);
    let delta = finite_non_negative(dt);
    (cq * rate * delta).min(FORAGING_CELL_DRAIN_MAX).max(0.0)
}

/// Decay lineal de saciedad por tick.
#[inline]
pub fn satiation_decay(current: f32, dt: f32) -> f32 {
    let c = finite_unit(current);
    let delta = finite_non_negative(dt);
    (c - SATIATION_DECAY_RATE * delta).max(0.0)
}

/// qe devuelta al suelo por descomposición de un cadáver.
#[inline]
pub fn decomposition_nutrient_return(corpse_qe: f32, efficiency: f32) -> f32 {
    let qe = finite_non_negative(corpse_qe);
    let eff = finite_unit(efficiency);
    (qe * eff).max(0.0)
}

/// Promedio normalizado de nutrientes de una celda.
#[inline]
pub fn nutrient_cell_average(carbon: f32, nitrogen: f32, phosphorus: f32, water: f32) -> f32 {
    let c = finite_unit(carbon);
    let n = finite_unit(nitrogen);
    let p = finite_unit(phosphorus);
    let w = finite_unit(water);
    (c + n + p + w) * 0.25
}

/// Ganancia de saciedad proporcional al qe asimilado.
#[inline]
pub fn satiation_gain_from_meal(assimilated_qe: f32) -> f32 {
    let a = finite_non_negative(assimilated_qe);
    (MEAL_SATIATION_GAIN * (a / FORAGING_CELL_DRAIN_MAX)).min(1.0)
}

/// Fracción de drenaje de una celda de nutrientes proporcional al intake.
#[inline]
pub fn nutrient_drain_fraction(intake: f32, cell_qe: f32) -> f32 {
    let i = finite_non_negative(intake);
    let c = finite_non_negative(cell_qe).max(CELL_QE_EPSILON);
    (i / c).min(1.0)
}

/// Delta de regeneración de nutrientes en grid por descomposición de cadáver.
#[inline]
pub fn decomposition_grid_delta(nutrient_return_qe: f32) -> f32 {
    let nr = finite_non_negative(nutrient_return_qe);
    (nr * DECOMPOSITION_GRID_RETURN_SCALE).min(DECOMPOSITION_GRID_RETURN_MAX)
}

/// qe bruta a drenar de la presa (antes de asimilación). La presa pierde este monto total.
#[inline]
pub fn predation_raw_drain(prey_qe: f32, bond_energy: f32) -> f32 {
    let qe = finite_non_negative(prey_qe);
    let bond = finite_non_negative(bond_energy);
    let resistance = 1.0 / (1.0 + bond * PREY_BOND_RESISTANCE_SCALE);
    (qe * resistance).max(0.0)
}

/// qe neta que el predador asimila de lo drenado (waste heat = drained - asimilado).
#[inline]
pub fn predation_assimilated(drained_qe: f32, efficiency: f32) -> f32 {
    let d = finite_non_negative(drained_qe);
    let eff = finite_unit(efficiency);
    (d * eff).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predation_success_faster_predator_higher_probability() {
        let fast = predation_success_probability(10.0, 5.0, 1.0, 1.0);
        let slow = predation_success_probability(5.0, 5.0, 1.0, 1.0);
        assert!(fast > slow, "fast={fast} should exceed slow={slow}");
    }

    #[test]
    fn predation_success_far_distance_returns_low() {
        let close = predation_success_probability(8.0, 5.0, 0.5, 1.0);
        let far = predation_success_probability(8.0, 5.0, 10.0, 1.0);
        assert!(close > far, "close={close} should exceed far={far}");
    }

    #[test]
    fn predation_success_clamps_to_unit() {
        let result = predation_success_probability(100.0, 1.0, 0.0, 1.0);
        assert!(result <= 1.0);
        assert!(result >= 0.0);
    }

    #[test]
    fn predation_success_nan_inputs_safe() {
        let result = predation_success_probability(f32::NAN, 5.0, 1.0, 1.0);
        assert!(result.is_finite());
    }

    #[test]
    fn predation_success_inf_inputs_safe() {
        let result = predation_success_probability(f32::INFINITY, 5.0, 1.0, 1.0);
        assert!(result.is_finite());
        assert!(result <= 1.0);
    }

    #[test]
    fn predation_success_terrain_zero_returns_zero() {
        let result = predation_success_probability(10.0, 5.0, 1.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn predation_success_negative_prey_speed_safe() {
        let result = predation_success_probability(10.0, -5.0, 1.0, 1.0);
        assert!(result.is_finite());
        assert!(result >= 0.0);
    }

    #[test]
    fn prey_qe_transfer_high_bond_reduces_transfer() {
        let low_bond = prey_qe_transfer(100.0, 0.0, 0.2);
        let high_bond = prey_qe_transfer(100.0, 200.0, 0.2);
        assert!(
            low_bond > high_bond,
            "low_bond={low_bond} > high_bond={high_bond}"
        );
    }

    #[test]
    fn prey_qe_transfer_zero_qe_returns_zero() {
        assert_eq!(prey_qe_transfer(0.0, 50.0, 0.2), 0.0);
    }

    #[test]
    fn prey_qe_transfer_nan_safe() {
        let result = prey_qe_transfer(f32::NAN, 50.0, 0.2);
        assert!(result.is_finite());
        assert_eq!(result, 0.0);
    }

    #[test]
    fn foraging_intake_respects_cell_drain_max() {
        let result = foraging_intake_from_field(1000.0, 1.0, 1.0);
        assert!(result <= FORAGING_CELL_DRAIN_MAX);
    }

    #[test]
    fn foraging_intake_zero_cell_returns_zero() {
        assert_eq!(foraging_intake_from_field(0.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn foraging_intake_nan_safe() {
        let result = foraging_intake_from_field(f32::NAN, 1.0, 1.0);
        assert!(result.is_finite());
    }

    #[test]
    fn satiation_decay_reduces_over_time() {
        let before = 0.8;
        let after = satiation_decay(before, 1.0);
        assert!(after < before);
        assert!(after >= 0.0);
    }

    #[test]
    fn satiation_decay_clamps_to_zero() {
        let result = satiation_decay(0.001, 100.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn satiation_decay_dt_zero_no_change() {
        let result = satiation_decay(0.5, 0.0);
        assert!((result - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn decomposition_nutrient_return_positive() {
        let result = decomposition_nutrient_return(100.0, DECOMPOSER_ASSIMILATION);
        assert!(result > 0.0);
        assert!((result - 15.0).abs() < 0.01);
    }

    #[test]
    fn decomposition_nutrient_return_zero_corpse() {
        assert_eq!(decomposition_nutrient_return(0.0, 0.15), 0.0);
    }

    #[test]
    fn decomposition_nutrient_return_nan_safe() {
        let result = decomposition_nutrient_return(f32::NAN, 0.15);
        assert!(result.is_finite());
        assert_eq!(result, 0.0);
    }

    #[test]
    fn nutrient_cell_average_uniform() {
        let avg = nutrient_cell_average(0.8, 0.8, 0.8, 0.8);
        assert!((avg - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn nutrient_cell_average_nan_safe() {
        let avg = nutrient_cell_average(f32::NAN, 0.5, 0.5, 0.5);
        assert!(avg.is_finite());
    }

    #[test]
    fn satiation_gain_from_meal_bounded() {
        let gain = satiation_gain_from_meal(100.0);
        assert!(gain <= 1.0);
        assert!(gain > 0.0);
    }

    #[test]
    fn nutrient_drain_fraction_bounded() {
        let frac = nutrient_drain_fraction(10.0, 0.5);
        assert!(frac <= 1.0);
        assert!(frac > 0.0);
    }

    #[test]
    fn predation_raw_drain_bond_reduces() {
        let no_bond = predation_raw_drain(100.0, 0.0);
        let high_bond = predation_raw_drain(100.0, 200.0);
        assert!(no_bond > high_bond);
    }

    #[test]
    fn predation_assimilated_fraction_of_drained() {
        let drained = 50.0;
        let assimilated = predation_assimilated(drained, CARNIVORE_ASSIMILATION);
        assert!(assimilated < drained);
        assert!((assimilated - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn decomposition_grid_delta_bounded() {
        let delta = decomposition_grid_delta(1000.0);
        assert!(delta <= DECOMPOSITION_GRID_RETURN_MAX);
    }

    #[test]
    fn trophic_net_qe_delta_maintenance_exceeds_intake_is_negative() {
        use crate::blueprint::equations::trophic_net_qe_delta;
        let result = trophic_net_qe_delta(1.0, 5.0, 0.5);
        assert!(result < 0.0, "net delta should be negative: {result}");
    }
}
