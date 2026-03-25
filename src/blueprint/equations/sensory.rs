//! D5: Sensory & perception — pure math.

use super::finite_helpers::finite_non_negative;
use crate::blueprint::constants::{
    SENSORY_NON_PREDATOR_FACTOR, SENSORY_PREDATOR_FACTOR, SENSORY_REFERENCE_QE,
    SENSORY_SPEED_THREAT_SCALE,
};

/// Detection range: range = sensitivity * sqrt(emitter_qe / noise_floor).
/// Higher entity qe → detectable from farther. Higher sensitivity → wider scan.
#[inline]
pub fn frequency_detection_range(sensitivity: f32, emitter_qe: f32, noise_floor: f32) -> f32 {
    let s = finite_non_negative(sensitivity);
    let qe = finite_non_negative(emitter_qe);
    let nf = finite_non_negative(noise_floor).max(f32::EPSILON);
    s * (qe / nf).sqrt()
}

/// Threat assessment: (qe / REF_QE) * (1 + speed * SCALE) * pred_factor / (1 + distance).
/// Higher qe, speed, predator status, or proximity → higher threat.
#[inline]
pub fn threat_level_assessment(
    entity_qe: f32,
    entity_speed: f32,
    is_predator: bool,
    distance: f32,
) -> f32 {
    let qe = finite_non_negative(entity_qe);
    let speed = finite_non_negative(entity_speed);
    let dist = finite_non_negative(distance);
    let pred_factor = if is_predator {
        SENSORY_PREDATOR_FACTOR
    } else {
        SENSORY_NON_PREDATOR_FACTOR
    };
    (qe / SENSORY_REFERENCE_QE) * (1.0 + speed * SENSORY_SPEED_THREAT_SCALE) * pred_factor
        / (1.0 + dist)
}

/// Food attractiveness: (qe * hunger) / (1 + distance^2).
/// Higher qe and hunger, lower distance → more attractive.
#[inline]
pub fn food_attractiveness(entity_qe: f32, distance: f32, hunger: f32) -> f32 {
    let qe = finite_non_negative(entity_qe);
    let h = finite_non_negative(hunger).min(1.0);
    let dist = finite_non_negative(distance);
    (qe * h) / (1.0 + dist * dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── frequency_detection_range ──

    #[test]
    fn detection_range_scales_with_qe() {
        let r_low = frequency_detection_range(3.0, 100.0, 1.0);
        let r_high = frequency_detection_range(3.0, 400.0, 1.0);
        assert!(r_high > r_low);
        assert!(r_low > 0.0);
    }

    #[test]
    fn detection_range_zero_sensitivity_returns_zero() {
        assert_eq!(frequency_detection_range(0.0, 100.0, 1.0), 0.0);
    }

    #[test]
    fn detection_range_zero_qe_returns_zero() {
        assert_eq!(frequency_detection_range(3.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn detection_range_scales_with_sensitivity() {
        let r1 = frequency_detection_range(1.0, 100.0, 1.0);
        let r3 = frequency_detection_range(3.0, 100.0, 1.0);
        assert!((r3 / r1 - 3.0).abs() < 1e-5);
    }

    #[test]
    fn detection_range_higher_noise_reduces_range() {
        let r_quiet = frequency_detection_range(3.0, 100.0, 1.0);
        let r_noisy = frequency_detection_range(3.0, 100.0, 4.0);
        assert!(r_quiet > r_noisy);
    }

    #[test]
    fn detection_range_nan_inputs_return_zero() {
        assert_eq!(frequency_detection_range(f32::NAN, 100.0, 1.0), 0.0);
        assert_eq!(frequency_detection_range(3.0, f32::NAN, 1.0), 0.0);
    }

    // ── threat_level_assessment ──

    #[test]
    fn threat_level_predator_at_close_range_is_high() {
        let threat = threat_level_assessment(1000.0, 2.0, true, 1.0);
        let non_pred = threat_level_assessment(1000.0, 2.0, false, 1.0);
        assert!(threat > non_pred);
        assert!(threat > 0.5);
    }

    #[test]
    fn threat_level_decreases_with_distance() {
        let close = threat_level_assessment(500.0, 1.0, true, 1.0);
        let far = threat_level_assessment(500.0, 1.0, true, 20.0);
        assert!(close > far);
    }

    #[test]
    fn threat_level_zero_qe_returns_zero() {
        assert_eq!(threat_level_assessment(0.0, 5.0, true, 1.0), 0.0);
    }

    #[test]
    fn threat_level_increases_with_speed() {
        let slow = threat_level_assessment(500.0, 0.0, false, 5.0);
        let fast = threat_level_assessment(500.0, 10.0, false, 5.0);
        assert!(fast > slow);
    }

    #[test]
    fn threat_level_negative_inputs_treated_as_zero() {
        let t = threat_level_assessment(-100.0, -5.0, true, -1.0);
        assert_eq!(t, 0.0);
    }

    // ── food_attractiveness ──

    #[test]
    fn food_attractiveness_scales_with_hunger() {
        let low_hunger = food_attractiveness(100.0, 5.0, 0.2);
        let high_hunger = food_attractiveness(100.0, 5.0, 0.9);
        assert!(high_hunger > low_hunger);
    }

    #[test]
    fn food_attractiveness_decreases_with_distance() {
        let close = food_attractiveness(100.0, 1.0, 0.5);
        let far = food_attractiveness(100.0, 10.0, 0.5);
        assert!(close > far);
    }

    #[test]
    fn food_attractiveness_zero_qe_returns_zero() {
        assert_eq!(food_attractiveness(0.0, 5.0, 0.5), 0.0);
    }

    #[test]
    fn food_attractiveness_zero_hunger_returns_zero() {
        assert_eq!(food_attractiveness(100.0, 5.0, 0.0), 0.0);
    }

    #[test]
    fn food_attractiveness_scales_with_qe() {
        let small = food_attractiveness(50.0, 5.0, 0.5);
        let large = food_attractiveness(200.0, 5.0, 0.5);
        assert!(large > small);
    }

    #[test]
    fn food_attractiveness_nan_returns_zero() {
        assert_eq!(food_attractiveness(f32::NAN, 5.0, 0.5), 0.0);
    }
}
