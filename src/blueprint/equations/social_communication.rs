use bevy::math::Vec2;

use super::finite_helpers::finite_non_negative;
use crate::blueprint::constants::*;

/// Fuerza de cohesión hacia el centroide del pack.
/// `force = COHESION_STRENGTH × (distance - rest_distance) × direction`
#[inline]
pub fn pack_cohesion_force(member_pos: Vec2, centroid: Vec2, rest_distance: f32) -> Vec2 {
    let diff = centroid - member_pos;
    let distance = diff.length();
    if distance < f32::EPSILON {
        return Vec2::ZERO;
    }
    let direction = diff / distance;
    let rest = finite_non_negative(rest_distance);
    let magnitude = COHESION_STRENGTH * (distance - rest);
    direction * magnitude
}

/// Score de dominancia: `qe × radius × (1 + resilience × DOMINANCE_RESILIENCE_WEIGHT)`.
#[inline]
pub fn dominance_contest_score(qe: f32, radius: f32, resilience: f32) -> f32 {
    let q = finite_non_negative(qe);
    let r = finite_non_negative(radius);
    let res = finite_non_negative(resilience);
    q * r * (1.0 + res * DOMINANCE_RESILIENCE_WEIGHT)
}

/// Bonus de caza cooperativa: `√pack_size × COOPERATIVE_HUNT_SCALE`.
/// Diminishing returns: 2→1.41×, 4→2×, 9→3×.
#[inline]
pub fn pack_hunt_bonus(pack_size: u32, _prey_qe: f32) -> f32 {
    let size = pack_size.max(1) as f32;
    size.sqrt() * COOPERATIVE_HUNT_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cohesion_force_zero_when_at_centroid() {
        let pos = Vec2::new(5.0, 5.0);
        let force = pack_cohesion_force(pos, pos, 3.0);
        assert!(force.length() < f32::EPSILON);
    }

    #[test]
    fn cohesion_force_points_toward_centroid() {
        let pos = Vec2::ZERO;
        let centroid = Vec2::new(10.0, 0.0);
        let force = pack_cohesion_force(pos, centroid, 3.0);
        assert!(force.x > 0.0, "force should point toward centroid: {force:?}");
    }

    #[test]
    fn cohesion_force_repels_when_closer_than_rest() {
        let pos = Vec2::ZERO;
        let centroid = Vec2::new(1.0, 0.0);
        let force = pack_cohesion_force(pos, centroid, 5.0);
        // distance(1) < rest(5) → magnitude negative → force points away
        assert!(force.x < 0.0, "should repel when closer than rest: {force:?}");
    }

    #[test]
    fn cohesion_force_magnitude_scales_with_distance() {
        let pos = Vec2::ZERO;
        let near = Vec2::new(5.0, 0.0);
        let far = Vec2::new(20.0, 0.0);
        let f_near = pack_cohesion_force(pos, near, 3.0).length();
        let f_far = pack_cohesion_force(pos, far, 3.0).length();
        assert!(f_far > f_near, "farther = stronger pull: near={f_near}, far={f_far}");
    }

    #[test]
    fn dominance_score_zero_qe_returns_zero() {
        assert_eq!(dominance_contest_score(0.0, 2.0, 0.5), 0.0);
    }

    #[test]
    fn dominance_score_zero_radius_returns_zero() {
        assert_eq!(dominance_contest_score(100.0, 0.0, 0.5), 0.0);
    }

    #[test]
    fn dominance_score_resilience_increases_score() {
        let low_res = dominance_contest_score(100.0, 2.0, 0.0);
        let high_res = dominance_contest_score(100.0, 2.0, 1.0);
        assert!(high_res > low_res, "high_res={high_res} > low_res={low_res}");
    }

    #[test]
    fn dominance_score_nan_safe() {
        let result = dominance_contest_score(f32::NAN, 2.0, 0.5);
        assert!(result.is_finite());
        assert_eq!(result, 0.0);
    }

    #[test]
    fn dominance_score_negative_inputs_safe() {
        let result = dominance_contest_score(-10.0, -2.0, -1.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn pack_hunt_bonus_single_returns_scale() {
        let bonus = pack_hunt_bonus(1, 100.0);
        assert!((bonus - COOPERATIVE_HUNT_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn pack_hunt_bonus_four_returns_two_x() {
        let bonus = pack_hunt_bonus(4, 100.0);
        assert!((bonus - 2.0 * COOPERATIVE_HUNT_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn pack_hunt_bonus_nine_returns_three_x() {
        let bonus = pack_hunt_bonus(9, 100.0);
        assert!((bonus - 3.0 * COOPERATIVE_HUNT_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn pack_hunt_bonus_zero_pack_clamps_to_one() {
        let bonus = pack_hunt_bonus(0, 100.0);
        assert!((bonus - COOPERATIVE_HUNT_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn pack_hunt_bonus_diminishing_returns() {
        let b2 = pack_hunt_bonus(2, 100.0);
        let b3 = pack_hunt_bonus(3, 100.0);
        let b4 = pack_hunt_bonus(4, 100.0);
        let delta_2_3 = b3 - b2;
        let delta_3_4 = b4 - b3;
        assert!(delta_3_4 < delta_2_3, "diminishing: Δ(3→4)={delta_3_4} < Δ(2→3)={delta_2_3}");
    }
}
