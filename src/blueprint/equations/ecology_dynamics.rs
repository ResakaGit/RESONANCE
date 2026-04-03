//! D9: Ecological Dynamics — pure equations (E1-E4).

use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::TrophicClass;
use crate::blueprint::constants::{
    ABIOGENESIS_PRESSURE_SCALE, CARRYING_CAPACITY_QE_FACTOR, SUCCESSION_EARLY_TICKS,
    SUCCESSION_MID_TICKS, SUCCESSION_PIONEER_TICKS,
};

/// Ecological succession phase (not a component — return value only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SuccessionStage {
    Pioneer,
    Early,
    Mid,
    Climax,
}

/// E1: Carrying capacity of a grid cell.
/// `K = floor(cell_qe × nutrient_total / (QE_FACTOR × cell_size²))`
pub fn carrying_capacity(cell_qe: f32, nutrient_total: f32, cell_size: f32) -> u32 {
    if cell_size <= 0.0 || !cell_size.is_finite() {
        return 0;
    }
    let numerator = finite_non_negative(cell_qe) * finite_non_negative(nutrient_total);
    let denominator = CARRYING_CAPACITY_QE_FACTOR * cell_size * cell_size;
    let k = numerator / denominator;
    if k.is_finite() { k.max(0.0) as u32 } else { 0 }
}

/// E2: Reproduction pressure in `[0, 1]`.
/// 1.0 = cell empty (max pressure to reproduce), 0.0 = at/over capacity.
pub fn reproduction_pressure(local_population: u32, carrying_capacity: u32) -> f32 {
    if carrying_capacity == 0 {
        return 0.0;
    }
    finite_unit(1.0 - (local_population as f32 / carrying_capacity as f32).min(1.0))
}

/// E3: Succession stage from elapsed ticks and dominant trophic class.
/// Higher trophic dominants accelerate succession.
pub fn succession_stage(
    time_since_disturbance: u32,
    dominant_trophic: TrophicClass,
) -> SuccessionStage {
    let baseline = if time_since_disturbance < SUCCESSION_PIONEER_TICKS {
        SuccessionStage::Pioneer
    } else if time_since_disturbance < SUCCESSION_EARLY_TICKS {
        SuccessionStage::Early
    } else if time_since_disturbance < SUCCESSION_MID_TICKS {
        SuccessionStage::Mid
    } else {
        SuccessionStage::Climax
    };
    let trophic_floor = match dominant_trophic {
        TrophicClass::Carnivore | TrophicClass::Detritivore => SuccessionStage::Mid,
        TrophicClass::Herbivore | TrophicClass::Omnivore => SuccessionStage::Early,
        TrophicClass::PrimaryProducer => SuccessionStage::Pioneer,
    };
    baseline.max(trophic_floor)
}

/// E4: Modulated abiogenesis threshold — lower when reproduction pressure is high.
/// `threshold = base / (1.0 + pressure × SCALE)`
pub fn abiogenesis_modulated_threshold(base: f32, reproduction_pressure: f32) -> f32 {
    let pressure = finite_unit(reproduction_pressure);
    let base = finite_non_negative(base);
    finite_non_negative(base / (1.0 + pressure * ABIOGENESIS_PRESSURE_SCALE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrying_capacity_zero_nutrients_returns_zero() {
        assert_eq!(carrying_capacity(100.0, 0.0, 1.0), 0);
    }

    #[test]
    fn carrying_capacity_high_qe_high_nutrients_returns_many() {
        // K = floor(500 * 4.0 / (10.0 * 1²)) = floor(200) = 200
        assert_eq!(carrying_capacity(500.0, 4.0, 1.0), 200);
    }

    #[test]
    fn carrying_capacity_zero_cell_size_returns_zero() {
        assert_eq!(carrying_capacity(100.0, 1.0, 0.0), 0);
    }

    #[test]
    fn carrying_capacity_nan_returns_zero() {
        assert_eq!(carrying_capacity(f32::NAN, 1.0, 1.0), 0);
        assert_eq!(carrying_capacity(100.0, f32::NAN, 1.0), 0);
        assert_eq!(carrying_capacity(100.0, 1.0, f32::NAN), 0);
    }

    #[test]
    fn carrying_capacity_negative_cell_size_returns_zero() {
        assert_eq!(carrying_capacity(100.0, 1.0, -1.0), 0);
    }

    #[test]
    fn reproduction_pressure_at_capacity_is_zero() {
        assert!((reproduction_pressure(10, 10) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reproduction_pressure_empty_cell_is_one() {
        assert!((reproduction_pressure(0, 10) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reproduction_pressure_zero_k_returns_zero() {
        assert!((reproduction_pressure(5, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reproduction_pressure_over_capacity_clamped_to_zero() {
        assert!((reproduction_pressure(20, 10) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn succession_starts_as_pioneer() {
        assert_eq!(
            succession_stage(0, TrophicClass::PrimaryProducer),
            SuccessionStage::Pioneer,
        );
    }

    #[test]
    fn succession_reaches_climax_after_threshold() {
        assert_eq!(
            succession_stage(SUCCESSION_MID_TICKS, TrophicClass::PrimaryProducer),
            SuccessionStage::Climax,
        );
    }

    #[test]
    fn succession_carnivore_accelerates_to_mid() {
        // Time says Pioneer (tick 0), but Carnivore dominance promotes to Mid.
        assert_eq!(
            succession_stage(0, TrophicClass::Carnivore),
            SuccessionStage::Mid,
        );
    }

    #[test]
    fn succession_herbivore_accelerates_to_early() {
        assert_eq!(
            succession_stage(0, TrophicClass::Herbivore),
            SuccessionStage::Early,
        );
    }

    #[test]
    fn succession_baseline_early_not_demoted_by_producer() {
        assert_eq!(
            succession_stage(SUCCESSION_PIONEER_TICKS, TrophicClass::PrimaryProducer),
            SuccessionStage::Early,
        );
    }

    #[test]
    fn abiogenesis_threshold_lower_when_empty() {
        let base = 100.0;
        let full = abiogenesis_modulated_threshold(base, 0.0);
        let empty = abiogenesis_modulated_threshold(base, 1.0);
        assert!(
            empty < full,
            "empty cell should lower threshold: {empty} < {full}"
        );
    }

    #[test]
    fn abiogenesis_threshold_zero_base_is_zero() {
        assert!((abiogenesis_modulated_threshold(0.0, 1.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn abiogenesis_threshold_nan_base_returns_zero() {
        assert!((abiogenesis_modulated_threshold(f32::NAN, 0.5) - 0.0).abs() < f32::EPSILON);
    }
}
