//! Input → Event flow integration test.
//! Validates that keyboard input correctly translates to movement intent
//! and that grimoire slot selection fires SlotActivatedEvent.

use bevy::prelude::*;
use bevy::input::ButtonInput;
use resonance::blueprint::recipes::EffectRecipe;
use resonance::events::AbilitySelectionEvent;
use resonance::layers::{
    AbilityCastSpec, AbilityOutput, AbilitySlot, Grimoire, ModifiedField, WillActuator,
};
use resonance::simulation::PlayerControlled;
use resonance::simulation::input::{
    SlotActivatedEvent, grimoire_slot_selection_system, will_input_system,
};

fn make_input_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<SlotActivatedEvent>();
    app.add_event::<AbilitySelectionEvent>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app
}

// ─── Movement intent ─────────────────────────────────────────────────────────

#[test]
fn will_input_wasd_sets_movement_intent() {
    let mut app = make_input_app();
    app.add_systems(Update, will_input_system);

    let e = app
        .world_mut()
        .spawn((PlayerControlled, WillActuator::default()))
        .id();

    // Press W (up).
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.update();

    let actuator = app.world().get::<WillActuator>(e).unwrap();
    assert!(
        actuator.movement_intent().y > 0.0,
        "W should set +Y intent, got {:?}",
        actuator.movement_intent()
    );
}

#[test]
fn will_input_no_keys_zeroes_intent() {
    let mut app = make_input_app();
    app.add_systems(Update, will_input_system);

    let e = app
        .world_mut()
        .spawn((PlayerControlled, WillActuator::default()))
        .id();

    // No keys pressed.
    app.update();

    let actuator = app.world().get::<WillActuator>(e).unwrap();
    assert_eq!(
        actuator.movement_intent(),
        Vec2::ZERO,
        "no keys => zero intent"
    );
}

#[test]
fn will_input_opposite_keys_cancel() {
    let mut app = make_input_app();
    app.add_systems(Update, will_input_system);

    let e = app
        .world_mut()
        .spawn((PlayerControlled, WillActuator::default()))
        .id();

    // Press both W and S (cancel vertical).
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyW);
        input.press(KeyCode::KeyS);
    }
    app.update();

    let actuator = app.world().get::<WillActuator>(e).unwrap();
    assert!(
        actuator.movement_intent().length() < 1e-5,
        "opposite keys should cancel, got {:?}",
        actuator.movement_intent()
    );
}

#[test]
fn will_input_diagonal_is_normalized() {
    let mut app = make_input_app();
    app.add_systems(Update, will_input_system);

    let e = app
        .world_mut()
        .spawn((PlayerControlled, WillActuator::default()))
        .id();

    // Press W + D (diagonal).
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::KeyW);
        input.press(KeyCode::KeyD);
    }
    app.update();

    let actuator = app.world().get::<WillActuator>(e).unwrap();
    let len = actuator.movement_intent().length();
    assert!(
        (len - 1.0).abs() < 1e-4,
        "diagonal should be normalized, got length {len}"
    );
}

// ─── Grimoire slot selection ─────────────────────────────────────────────────

fn make_test_grimoire() -> Grimoire {
    let mut g = Grimoire::default();
    g.push_ability(AbilitySlot {
        name: String::from("test_ability"),
        output: AbilityOutput::SelfBuff {
            effect: EffectRecipe {
                field: ModifiedField::DissipationMultiplier,
                magnitude: 1.0,
                fuel_qe: 10.0,
                dissipation: 0.1,
            },
        },
        cast: AbilityCastSpec::default(),
    });
    g
}

#[test]
fn grimoire_slot_q_fires_slot_activated_event() {
    let mut app = make_input_app();
    app.add_systems(Update, grimoire_slot_selection_system);

    let e = app
        .world_mut()
        .spawn((
            PlayerControlled,
            make_test_grimoire(),
            WillActuator::default(),
        ))
        .id();

    // Press Q (slot 0). just_pressed requires transition from not-pressed.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyQ);
    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<SlotActivatedEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 1, "pressing Q should fire SlotActivatedEvent");
    assert_eq!(events[0].caster, e);
    assert_eq!(events[0].slot_index, 0);
}

#[test]
fn grimoire_no_key_no_event() {
    let mut app = make_input_app();
    app.add_systems(Update, grimoire_slot_selection_system);

    app.world_mut().spawn((
        PlayerControlled,
        make_test_grimoire(),
        WillActuator::default(),
    ));

    // No keys pressed.
    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<SlotActivatedEvent>>()
        .drain()
        .collect();

    assert!(events.is_empty(), "no key => no event");
}

#[test]
fn grimoire_empty_no_event_even_with_key() {
    let mut app = make_input_app();
    app.add_systems(Update, grimoire_slot_selection_system);

    // Empty grimoire (no abilities).
    app.world_mut().spawn((
        PlayerControlled,
        Grimoire::default(),
        WillActuator::default(),
    ));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyQ);
    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<SlotActivatedEvent>>()
        .drain()
        .collect();

    assert!(events.is_empty(), "empty grimoire should not fire event");
}
