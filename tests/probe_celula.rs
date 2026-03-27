//! Probe: célula eucariota — engine processing + satiation decay.
//!
//! Verifica que el ciclo metabólico base ocurre cuando se corren los sistemas
//! de simulación con una célula real (mismas capas que `demo_celula`).

use bevy::prelude::*;
use resonance::blueprint::IdGenerator;
use resonance::entities::archetypes::spawn_celula;
use resonance::events::{DeathEvent, HungerEvent};
use resonance::layers::{AlchemicalEngine, BaseEnergy, TrophicState};
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::simulation::metabolic::trophic::trophic_satiation_decay_system;
use resonance::simulation::thermodynamic::pre_physics::engine_processing_system;

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeathEvent>();
    app.add_event::<HungerEvent>();
    app.add_systems(
        Update,
        (
            engine_processing_system,
            trophic_satiation_decay_system.after(engine_processing_system),
        ),
    );
    app
}

fn spawn_celula_in(app: &mut App) -> Entity {
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();
    let entity = {
        let mut commands = app.world_mut().commands();
        let e = spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        drop(commands);
        e
    };
    app.update(); // flush commands
    entity
}

fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

// ── Satiation ────────────────────────────────────────────────────────────────

/// Célula comienza con satiation=0.5; después de 200 ticks debe haber decaído.
#[test]
fn satiation_decays_over_200_ticks() {
    let mut app = make_app();
    let entity = spawn_celula_in(&mut app);

    let initial = app.world().get::<TrophicState>(entity).unwrap().satiation;
    assert!((initial - 0.5).abs() < 0.01, "initial satiation should be ~0.5, got {initial}");

    run_ticks(&mut app, 200);

    let after = app.world().get::<TrophicState>(entity).unwrap().satiation;
    assert!(
        after < initial,
        "satiation must decay: initial={initial}, after 200 ticks={after}",
    );
}

// ── Supervivencia ────────────────────────────────────────────────────────────

/// Célula con qe=150 debe sobrevivir 200 ticks sin colapsar.
#[test]
fn survives_200_ticks_qe_positive() {
    let mut app = make_app();
    let entity = spawn_celula_in(&mut app);

    run_ticks(&mut app, 200);

    let energy = app.world().get::<BaseEnergy>(entity);
    assert!(energy.is_some(), "célula entity must still exist after 200 ticks");
    let qe = energy.unwrap().qe();
    assert!(qe >= 0.0, "qe must be non-negative: {qe}");
}

// ── Engine ───────────────────────────────────────────────────────────────────

/// Engine buffer no debe ser NaN después de 200 ticks de procesamiento.
#[test]
fn engine_buffer_stable_after_200_ticks() {
    let mut app = make_app();
    let entity = spawn_celula_in(&mut app);

    run_ticks(&mut app, 200);

    let engine = app.world().get::<AlchemicalEngine>(entity).unwrap();
    let buf = engine.buffer_level();
    assert!(!buf.is_nan(), "engine buffer must not be NaN");
    assert!(buf >= 0.0, "engine buffer must be non-negative: {buf}");
}

// ── No-NaN invariante ────────────────────────────────────────────────────────

/// Ningún tick produce NaN en BaseEnergy.
#[test]
fn no_nan_in_energy_across_200_ticks() {
    let mut app = make_app();
    let entity = spawn_celula_in(&mut app);

    for tick in 0..200 {
        app.update();
        let qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();
        assert!(!qe.is_nan(), "NaN in BaseEnergy at tick {tick}");
        assert!(qe >= 0.0, "negative qe={qe} at tick {tick}");
    }
}

// ── Múltiples células ────────────────────────────────────────────────────────

/// 3 células (mismo patrón que demo_celula) todas sobreviven.
#[test]
fn three_celulas_all_survive_200_ticks() {
    let mut app = make_app();
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();

    let positions = [
        Vec2::new(-0.6, 0.0),
        Vec2::new(0.6, 0.0),
        Vec2::new(0.0, 0.8),
    ];

    let entities: Vec<Entity> = {
        let mut commands = app.world_mut().commands();
        let es: Vec<Entity> = positions
            .iter()
            .map(|&pos| spawn_celula(&mut commands, &mut id_gen, pos, &layout))
            .collect();
        drop(commands);
        es
    };
    app.update(); // flush

    run_ticks(&mut app, 200);

    for (i, &entity) in entities.iter().enumerate() {
        let Some(energy) = app.world().get::<BaseEnergy>(entity) else {
            panic!("célula {i} disappeared after 200 ticks");
        };
        assert!(energy.qe() >= 0.0, "célula {i} has negative qe: {}", energy.qe());
        assert!(!energy.qe().is_nan(), "célula {i} has NaN qe");
    }
}
