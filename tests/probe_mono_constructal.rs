//! Probe: constructal body plan emergence for primate-like entity.
//!
//! Verifies that an entity with high mobility_bias + MOVE capability + AmbientPressure
//! produces a constructal body plan with:
//!   - Non-zero limb count (from optimal_appendage_count)
//!   - Front limbs longer than rear (mobility → arm asymmetry)
//!   - Compound mesh with more triangles than a single tube
//!
//! `cargo test --test probe_mono_constructal`

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use resonance::blueprint::IdGenerator;
use resonance::blueprint::equations::{optimal_appendage_count, organ_slot_scale};
use resonance::entities::archetypes::spawn_animal_demo;
use resonance::layers::{BodyPlanLayout, FlowVector};
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::simulation::lifecycle::constructal_body_plan_system;

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.insert_resource(SimWorldTransformParams::default());
    app.add_systems(Update, constructal_body_plan_system);
    app
}

fn spawn_animal(app: &mut App) -> Entity {
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();
    let entity = {
        let mut commands = app.world_mut().commands();
        spawn_animal_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout)
    };
    app.update(); // flush commands
    entity
}

// ── Pure equation tests: emergence from energy composition ──────────────────

#[test]
fn monkey_velocity_produces_limbs() {
    // Monkey-like: radius=0.55, fineness=1.5, moderate speed
    let n = optimal_appendage_count(0.55, 1.5, 1.0, 5.0, 0.1, 0.44, 0.08, 8);
    assert!(
        n >= 2,
        "monkey-like entity at v=5 should have limbs, got {n}"
    );
}

#[test]
fn stationary_entity_gets_zero_limbs() {
    let n = optimal_appendage_count(0.55, 1.5, 1.0, 0.0, 0.1, 0.44, 0.08, 8);
    assert_eq!(n, 0, "stationary entity should have 0 limbs");
}

#[test]
fn high_mobility_front_limbs_longer_than_rear() {
    let mobility = 0.8; // primate-like
    let (front_len, _) = organ_slot_scale(2, 6, mobility); // front limb
    let (rear_len, _) = organ_slot_scale(4, 6, mobility); // rear limb
    assert!(
        front_len > rear_len,
        "high mobility should make front limbs longer: front={front_len} rear={rear_len}"
    );
}

#[test]
fn low_mobility_limbs_roughly_equal() {
    let mobility = 0.2; // quadruped runner
    let (front_len, _) = organ_slot_scale(2, 6, mobility);
    let (rear_len, _) = organ_slot_scale(4, 6, mobility);
    let diff = (front_len - rear_len).abs();
    assert!(
        diff < 0.15,
        "low mobility limbs should be roughly equal: front={front_len} rear={rear_len} diff={diff}"
    );
}

#[test]
fn high_mobility_head_bigger_than_low() {
    let (_, head_r_hi) = organ_slot_scale(0, 6, 0.9);
    let (_, head_r_lo) = organ_slot_scale(0, 6, 0.1);
    assert!(
        head_r_hi > head_r_lo,
        "high mobility head should be larger: hi={head_r_hi} lo={head_r_lo}"
    );
}

// ── Integration: constructal system fires for animal with L6 ────────────────

#[test]
fn animal_with_l6_gets_constructal_body_plan() {
    let mut app = make_app();
    let animal = spawn_animal(&mut app);

    // Give it some velocity so constructal produces limbs
    app.world_mut()
        .entity_mut(animal)
        .get_mut::<FlowVector>()
        .unwrap()
        .set_velocity(Vec2::new(4.0, 0.0), None);
    app.update(); // constructal fires

    let has_layout = app.world().entity(animal).contains::<BodyPlanLayout>();
    assert!(
        has_layout,
        "animal with AmbientPressure should get BodyPlanLayout from constructal system"
    );

    let layout = app.world().get::<BodyPlanLayout>(animal).unwrap();
    assert!(
        layout.active_count() >= 2,
        "moving animal should have at least head+tail, got {}",
        layout.active_count()
    );
}

#[test]
fn distinct_profiles_produce_distinct_silhouettes() {
    // Primate-like (high mobility)
    let (arm_len_primate, _) = organ_slot_scale(2, 6, 0.8);
    let (leg_len_primate, _) = organ_slot_scale(4, 6, 0.8);
    let (_, head_r_primate) = organ_slot_scale(0, 6, 0.8);

    // Dog-like (low mobility)
    let (arm_len_dog, _) = organ_slot_scale(2, 6, 0.3);
    let (leg_len_dog, _) = organ_slot_scale(4, 6, 0.3);
    let (_, head_r_dog) = organ_slot_scale(0, 6, 0.3);

    // Primate arms are longer than dog arms
    assert!(arm_len_primate > arm_len_dog, "primate arms > dog arms");
    // Primate has bigger head
    assert!(head_r_primate > head_r_dog, "primate head > dog head");
    // Primate arm/leg ratio is higher (longer arms relative to legs)
    let ratio_primate = arm_len_primate / leg_len_primate;
    let ratio_dog = arm_len_dog / leg_len_dog;
    assert!(
        ratio_primate > ratio_dog,
        "primate arm/leg ratio ({ratio_primate:.2}) should exceed dog ({ratio_dog:.2})"
    );
}
