//! Pathfinding: multi-waypoint steering with obstacles (pure math, no Bevy).
//! Tests path_follow_step_xz with complex polylines.

use bevy::prelude::{Vec2, Vec3};
use resonance::simulation::pathfinding::core::path_follow_step_xz;

#[test]
fn multi_waypoint_traversal_reaches_each_in_order() {
    let waypoints = [
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 10.0),
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];
    let reach = 0.5;

    // Start at origin, heading toward first waypoint.
    let step0 = path_follow_step_xz(Vec2::ZERO, &waypoints, 0, reach);
    assert!(!step0.path_finished);
    assert_eq!(step0.next_index, 0);
    assert!(
        (step0.movement_xz - Vec2::X).length() < 0.01,
        "should head +X"
    );

    // Close to first waypoint — skips to second.
    let step1 = path_follow_step_xz(Vec2::new(9.8, 0.0), &waypoints, 0, reach);
    assert!(!step1.path_finished);
    assert_eq!(step1.next_index, 1, "should advance to wp 1");

    // Close to second waypoint — skips to third.
    let step2 = path_follow_step_xz(Vec2::new(10.0, 9.8), &waypoints, 1, reach);
    assert!(!step2.path_finished);
    assert_eq!(step2.next_index, 2, "should advance to wp 2");

    // Close to third — skips to fourth.
    let step3 = path_follow_step_xz(Vec2::new(0.2, 10.0), &waypoints, 2, reach);
    assert!(!step3.path_finished);
    assert_eq!(step3.next_index, 3, "should advance to wp 3");

    // At final waypoint — path finished.
    let step4 = path_follow_step_xz(Vec2::new(0.0, 0.2), &waypoints, 3, reach);
    assert!(step4.path_finished, "should finish at last waypoint");
}

#[test]
fn zigzag_path_direction_changes_correctly() {
    let waypoints = [
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 5.0),
        Vec3::new(10.0, 0.0, 5.0),
    ];
    let reach = 0.3;

    // Heading east to first waypoint.
    let s0 = path_follow_step_xz(Vec2::ZERO, &waypoints, 0, reach);
    assert!(
        s0.movement_xz.x > 0.5,
        "should head +X: {:?}",
        s0.movement_xz
    );

    // At first wp, heading north to second.
    let s1 = path_follow_step_xz(Vec2::new(5.0, 0.1), &waypoints, 0, reach);
    assert!(
        s1.movement_xz.y > 0.5,
        "should head +Z: {:?}",
        s1.movement_xz
    );
    assert_eq!(s1.next_index, 1);

    // At second wp, heading east again to third.
    let s2 = path_follow_step_xz(Vec2::new(5.1, 5.0), &waypoints, 1, reach);
    assert!(
        s2.movement_xz.x > 0.5,
        "should head +X again: {:?}",
        s2.movement_xz
    );
    assert_eq!(s2.next_index, 2);
}

#[test]
fn large_reach_radius_skips_nearby_waypoints() {
    let waypoints = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
    ];
    let reach = 3.5;

    // All first 3 waypoints are within reach from origin — should skip to wp 3.
    let step = path_follow_step_xz(Vec2::ZERO, &waypoints, 0, reach);
    assert!(!step.path_finished);
    assert_eq!(step.next_index, 3, "should skip to wp 3");
}

#[test]
fn diagonal_movement_is_normalized() {
    let waypoints = [Vec3::new(10.0, 0.0, 10.0)];
    let step = path_follow_step_xz(Vec2::ZERO, &waypoints, 0, 0.1);
    let len = step.movement_xz.length();
    assert!(
        (len - 1.0).abs() < 1e-4,
        "direction should be unit length, got {len}"
    );
}

#[test]
fn negative_reach_radius_treated_as_zero() {
    let waypoints = [Vec3::new(0.0, 0.0, 0.0)];
    // Agent right on the waypoint with negative reach — should still detect arrival.
    let step = path_follow_step_xz(Vec2::ZERO, &waypoints, 0, -5.0);
    assert!(
        step.path_finished,
        "negative reach should be clamped to 0, agent at wp"
    );
}

#[test]
fn index_beyond_waypoints_finishes_immediately() {
    let waypoints = [Vec3::new(5.0, 0.0, 0.0)];
    let step = path_follow_step_xz(Vec2::ZERO, &waypoints, 5, 0.5);
    assert!(step.path_finished);
    assert_eq!(step.movement_xz, Vec2::ZERO);
}
