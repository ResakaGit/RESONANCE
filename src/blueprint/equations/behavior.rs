use crate::blueprint::constants::{
    BEHAVIOR_ACTION_COUNT, FLEE_RESILIENCE_SCALE, FORAGE_MAX_RANGE, HUNT_MAX_RANGE,
    HUNT_QE_REFERENCE, REPRODUCE_BIOMASS_THRESHOLD, THREAT_LEVEL_DIVISOR, THREAT_POWER_CLAMP,
};

/// Computes (hunger_fraction, energy_ratio) from engine buffer state.
pub fn assess_energy(buffer_level: f32, buffer_cap: f32) -> (f32, f32) {
    let ratio = if buffer_cap > 0.0 {
        buffer_level / buffer_cap
    } else {
        0.0
    };
    let energy_ratio = ratio.clamp(0.0, 1.0);
    (1.0 - energy_ratio, energy_ratio)
}

/// Normalized threat level from hostile/self energy ratio.
pub fn threat_level(hostile_qe: f32, self_biomass: f32) -> f32 {
    let relative_power = if self_biomass > 0.0 {
        (hostile_qe / self_biomass).clamp(0.0, THREAT_POWER_CLAMP)
    } else {
        THREAT_POWER_CLAMP
    };
    (relative_power / THREAT_LEVEL_DIVISOR).min(1.0)
}

/// E1: Utility score for foraging.
/// `hunger` ∈ [0,1], `distance` ≥ 0, `urgency_bias` ≥ 0.
pub fn utility_forage(hunger: f32, distance: f32, urgency_bias: f32) -> f32 {
    let proximity = (1.0 - distance / FORAGE_MAX_RANGE).max(0.0);
    (hunger * proximity * (1.0 + urgency_bias)).max(0.0)
}

/// E2: Utility score for fleeing.
/// `threat_level` ∈ [0,1], `distance` ≥ 0, `detection_range` > 0, `resilience` ∈ [0,1].
pub fn utility_flee(
    threat_level: f32,
    distance: f32,
    detection_range: f32,
    resilience: f32,
) -> f32 {
    if detection_range <= 0.0 {
        return 0.0;
    }
    let proximity = (1.0 - distance / detection_range).max(0.0);
    let damping = (1.0 - resilience * FLEE_RESILIENCE_SCALE).max(0.0);
    (threat_level * proximity * damping).max(0.0)
}

/// E3: Utility score for hunting.
/// `prey_qe` ≥ 0, `distance` ≥ 0, `energy_available` ∈ [0,1], `mobility_bias` ∈ [0,1].
pub fn utility_hunt(prey_qe: f32, distance: f32, energy_available: f32, mobility_bias: f32) -> f32 {
    let value = (prey_qe / HUNT_QE_REFERENCE).min(1.0);
    let proximity = (1.0 - distance / HUNT_MAX_RANGE).max(0.0);
    let energy_factor = energy_available.clamp(0.0, 1.0);
    (value * proximity * mobility_bias * energy_factor).max(0.0)
}

/// E4: Utility score for reproduction.
/// `biomass` ≥ 0, `viability` ∈ [0,1], `maturity_progress` ∈ [0,1].
pub fn utility_reproduce(biomass: f32, viability: f32, maturity_progress: f32) -> f32 {
    let readiness = (biomass / REPRODUCE_BIOMASS_THRESHOLD).min(1.0);
    (readiness * viability * maturity_progress).max(0.0)
}

/// E5: Selects the action index with highest score (deterministic tie-break: lower index wins).
pub fn select_best_action(scores: &[f32; BEHAVIOR_ACTION_COUNT]) -> usize {
    scores
        .iter()
        .enumerate()
        .skip(1)
        .fold((0, scores[0]), |(best_idx, best_score), (i, &score)| {
            if score > best_score {
                (i, score)
            } else {
                (best_idx, best_score)
            }
        })
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_forage_zero_deficit_returns_zero() {
        assert_eq!(utility_forage(0.0, 5.0, 0.0), 0.0);
    }

    #[test]
    fn utility_forage_max_deficit_max_proximity_returns_high() {
        let u = utility_forage(1.0, 0.0, 0.5);
        assert!(u > 1.0);
    }

    #[test]
    fn utility_forage_out_of_range_returns_zero() {
        assert_eq!(utility_forage(1.0, FORAGE_MAX_RANGE + 1.0, 0.5), 0.0);
    }

    #[test]
    fn utility_forage_half_deficit_half_range() {
        let u = utility_forage(0.5, FORAGE_MAX_RANGE / 2.0, 0.0);
        assert!((u - 0.25).abs() < 1e-5);
    }

    #[test]
    fn utility_flee_no_threat_returns_zero() {
        assert_eq!(utility_flee(0.0, 5.0, 15.0, 0.5), 0.0);
    }

    #[test]
    fn utility_flee_close_threat_high_level_returns_max() {
        let u = utility_flee(1.0, 0.0, 15.0, 0.0);
        assert!((u - 1.0).abs() < 1e-5);
    }

    #[test]
    fn utility_flee_zero_detection_range_returns_zero() {
        assert_eq!(utility_flee(1.0, 0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn utility_flee_resilience_reduces_score() {
        let low_res = utility_flee(0.8, 3.0, 15.0, 0.0);
        let high_res = utility_flee(0.8, 3.0, 15.0, 0.8);
        assert!(low_res > high_res);
    }

    #[test]
    fn utility_hunt_far_prey_returns_low() {
        let u = utility_hunt(500.0, HUNT_MAX_RANGE - 0.1, 1.0, 1.0);
        assert!(u < 0.05);
    }

    #[test]
    fn utility_hunt_close_valuable_prey_returns_high() {
        let u = utility_hunt(500.0, 0.0, 1.0, 1.0);
        assert!((u - 1.0).abs() < 1e-5);
    }

    #[test]
    fn utility_hunt_zero_energy_returns_zero() {
        assert_eq!(utility_hunt(500.0, 5.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn utility_hunt_zero_mobility_returns_zero() {
        assert_eq!(utility_hunt(500.0, 5.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn utility_reproduce_low_biomass_returns_low() {
        let u = utility_reproduce(100.0, 1.0, 1.0);
        assert!(u < 0.2);
    }

    #[test]
    fn utility_reproduce_full_readiness() {
        let u = utility_reproduce(REPRODUCE_BIOMASS_THRESHOLD, 1.0, 1.0);
        assert!((u - 1.0).abs() < 1e-5);
    }

    #[test]
    fn utility_reproduce_zero_viability_returns_zero() {
        assert_eq!(utility_reproduce(1000.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn select_best_action_deterministic_tiebreak() {
        let scores = [0.5, 0.5, 0.3, 0.1, 0.0];
        assert_eq!(select_best_action(&scores), 0);
    }

    #[test]
    fn select_best_action_picks_highest() {
        let scores = [0.1, 0.3, 0.9, 0.2, 0.0];
        assert_eq!(select_best_action(&scores), 2);
    }

    #[test]
    fn select_best_action_all_zero() {
        let scores = [0.0; BEHAVIOR_ACTION_COUNT];
        assert_eq!(select_best_action(&scores), 0);
    }

    #[test]
    fn select_best_action_last_index_highest() {
        let scores = [0.0, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(select_best_action(&scores), 4);
    }

    #[test]
    fn assess_energy_full_buffer_no_hunger() {
        let (hunger, ratio) = assess_energy(100.0, 100.0);
        assert!((hunger - 0.0).abs() < 1e-5);
        assert!((ratio - 1.0).abs() < 1e-5);
    }

    #[test]
    fn assess_energy_empty_buffer_max_hunger() {
        let (hunger, ratio) = assess_energy(0.0, 100.0);
        assert!((hunger - 1.0).abs() < 1e-5);
        assert!((ratio - 0.0).abs() < 1e-5);
    }

    #[test]
    fn assess_energy_zero_cap_returns_max_hunger() {
        let (hunger, ratio) = assess_energy(50.0, 0.0);
        assert!((hunger - 1.0).abs() < 1e-5);
        assert!((ratio - 0.0).abs() < 1e-5);
    }

    #[test]
    fn threat_level_equal_power_returns_half() {
        let t = threat_level(500.0, 500.0);
        assert!((t - 0.5).abs() < 1e-5);
    }

    #[test]
    fn threat_level_zero_biomass_returns_max() {
        let t = threat_level(100.0, 0.0);
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn threat_level_weak_hostile_returns_low() {
        let t = threat_level(100.0, 1000.0);
        assert!(t < 0.1);
    }
}
