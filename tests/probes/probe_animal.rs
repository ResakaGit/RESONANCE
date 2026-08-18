//! Probe: animal herbívoro — behavior + trophic cycle.
//!
//! Verifica que la saciedad decae, el comportamiento se activa,
//! y el animal no colapsa en 200 ticks.

use bevy::prelude::*;
use resonance::blueprint::IdGenerator;
use resonance::entities::archetypes::{spawn_animal_demo, spawn_planta_demo};
use resonance::events::{DeathEvent, HungerEvent, PreyConsumedEvent, ThreatDetectedEvent};
use resonance::layers::{BaseEnergy, BehaviorIntent, TrophicState};
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::simulation::metabolic::trophic::trophic_satiation_decay_system;
use resonance::simulation::thermodynamic::pre_physics::engine_processing_system;
use resonance::world::SpatialIndex;

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeathEvent>();
    app.add_event::<HungerEvent>();
    app.add_event::<PreyConsumedEvent>();
    app.add_event::<ThreatDetectedEvent>();
    app.init_resource::<SpatialIndex>();
    app.insert_resource(SimWorldTransformParams::default());
    app.add_systems(
        Update,
        (
            engine_processing_system,
            trophic_satiation_decay_system.after(engine_processing_system),
        ),
    );
    app
}

fn spawn_scene(app: &mut App) -> (Entity, Vec<Entity>) {
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();

    let (animal, plants) = {
        let mut commands = app.world_mut().commands();
        let plants: Vec<Entity> = [
            Vec2::new(-5.0, 2.0),
            Vec2::new(3.0, -3.0),
            Vec2::new(0.0, 5.0),
        ]
        .iter()
        .map(|&pos| spawn_planta_demo(&mut commands, &mut id_gen, pos, &layout))
        .collect();

        let animal = spawn_animal_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        drop(commands);
        (animal, plants)
    };
    app.update(); // flush commands
    (animal, plants)
}

fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

// ── Satiation ────────────────────────────────────────────────────────────────

/// Animal comienza con satiation=0.3; después de 200 ticks debe haber decaído.
#[test]
fn animal_satiation_decays() {
    let mut app = make_app();
    let (animal, _) = spawn_scene(&mut app);

    let initial = app.world().get::<TrophicState>(animal).unwrap().satiation;
    assert!(
        (initial - 0.3).abs() < 0.05,
        "initial satiation ~0.3, got {initial}"
    );

    run_ticks(&mut app, 200);

    let after = app.world().get::<TrophicState>(animal).unwrap().satiation;
    assert!(
        after < initial,
        "satiation must decay: initial={initial}, after={after}",
    );
}

// ── Supervivencia ────────────────────────────────────────────────────────────

/// Animal sobrevive 200 ticks con sus plantas (misma escena que demo_animal).
#[test]
fn animal_survives_200_ticks() {
    let mut app = make_app();
    let (animal, _) = spawn_scene(&mut app);

    run_ticks(&mut app, 200);

    let energy = app.world().get::<BaseEnergy>(animal);
    assert!(energy.is_some(), "animal must exist after 200 ticks");
    let qe = energy.unwrap().qe();
    assert!(qe >= 0.0, "animal qe must be non-negative: {qe}");
}

/// Las plantas fuente trófica también sobreviven.
#[test]
fn food_plants_survive_200_ticks() {
    let mut app = make_app();
    let (_, plants) = spawn_scene(&mut app);

    run_ticks(&mut app, 200);

    for (i, &plant) in plants.iter().enumerate() {
        let Some(energy) = app.world().get::<BaseEnergy>(plant) else {
            panic!("food plant {i} disappeared after 200 ticks");
        };
        assert!(energy.qe() >= 0.0, "plant {i}: negative qe={}", energy.qe());
    }
}

// ── No-NaN ───────────────────────────────────────────────────────────────────

/// Ningún tick produce NaN en la escena animal+plantas.
#[test]
fn no_nan_across_200_ticks() {
    let mut app = make_app();
    let (animal, plants) = spawn_scene(&mut app);

    let all_entities: Vec<Entity> = std::iter::once(animal).chain(plants).collect();

    for tick in 0..200 {
        app.update();
        for &entity in &all_entities {
            let qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();
            assert!(!qe.is_nan(), "NaN in entity at tick {tick}");
            assert!(qe >= 0.0, "negative qe={qe} at tick {tick}");
        }
    }
}

// ── BehaviorIntent ───────────────────────────────────────────────────────────

/// BehaviorIntent debe existir en el animal (insertado por spawn).
#[test]
fn animal_has_behavior_intent() {
    let mut app = make_app();
    let (animal, _) = spawn_scene(&mut app);

    let intent = app.world().get::<BehaviorIntent>(animal);
    assert!(
        intent.is_some(),
        "animal must have BehaviorIntent component"
    );
}
