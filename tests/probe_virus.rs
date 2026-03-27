//! Probe: virus + células huésped — parasitismo energético.
//!
//! Verifica que en la escena virus+hosts (misma composición que `demo_virus`),
//! la simulación corre sin NaN y las entidades sobreviven.
//!
//! NOTA: el virus usa `AlchemicalInjector` (L8) pero NO tiene `SpellMarker`,
//! por lo que `catalysis_spatial_filter_system` no lo procesa como spell.
//! El drenaje parasítico ocurre vía `collision_interference_system` cuando
//! virus y host se solapan en el `SpatialIndex`. Este probe verifica la
//! estabilidad de la escena y documenta el estado real del mecanismo.

use bevy::prelude::*;
use resonance::blueprint::IdGenerator;
use resonance::entities::archetypes::{spawn_celula, spawn_virus};
use resonance::events::{DeathEvent, HungerEvent};
use resonance::layers::{AlchemicalInjector, BaseEnergy, TrophicState};
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

struct VirusScene {
    hosts: Vec<Entity>,
    viruses: Vec<Entity>,
}

fn spawn_scene(app: &mut App) -> VirusScene {
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();

    let (hosts, viruses) = {
        let mut commands = app.world_mut().commands();

        let hosts: Vec<Entity> = [
            Vec2::new(-2.0, 0.5),
            Vec2::new(-2.0, -0.5),
            Vec2::new(-1.2, 0.0),
            Vec2::new(-1.5, 1.0),
        ]
        .iter()
        .map(|&pos| spawn_celula(&mut commands, &mut id_gen, pos, &layout))
        .collect();

        let viruses: Vec<Entity> = [Vec2::new(1.2, 0.2), Vec2::new(1.5, -0.3)]
            .iter()
            .map(|&pos| spawn_virus(&mut commands, &mut id_gen, pos, &layout))
            .collect();

        drop(commands);
        (hosts, viruses)
    };
    app.update(); // flush commands

    VirusScene { hosts, viruses }
}

fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

// ── Composición de capas ─────────────────────────────────────────────────────

/// Virus tiene AlchemicalInjector (L8), hosts no.
#[test]
fn virus_has_injector_hosts_do_not() {
    let mut app = make_app();
    let scene = spawn_scene(&mut app);

    for &virus in &scene.viruses {
        assert!(
            app.world().get::<AlchemicalInjector>(virus).is_some(),
            "virus must have AlchemicalInjector",
        );
    }
    for &host in &scene.hosts {
        assert!(
            app.world().get::<AlchemicalInjector>(host).is_none(),
            "host must NOT have AlchemicalInjector",
        );
    }
}

// ── Supervivencia ────────────────────────────────────────────────────────────

/// Hosts sobreviven 100 ticks (sin collision_interference, no hay drenaje).
#[test]
fn hosts_survive_100_ticks() {
    let mut app = make_app();
    let scene = spawn_scene(&mut app);

    run_ticks(&mut app, 100);

    for (i, &host) in scene.hosts.iter().enumerate() {
        let Some(energy) = app.world().get::<BaseEnergy>(host) else {
            panic!("host {i} disappeared after 100 ticks");
        };
        assert!(energy.qe() >= 0.0, "host {i}: negative qe={}", energy.qe());
    }
}

/// Virus sobreviven 100 ticks.
#[test]
fn viruses_survive_100_ticks() {
    let mut app = make_app();
    let scene = spawn_scene(&mut app);

    run_ticks(&mut app, 100);

    for (i, &virus) in scene.viruses.iter().enumerate() {
        let Some(energy) = app.world().get::<BaseEnergy>(virus) else {
            panic!("virus {i} disappeared after 100 ticks");
        };
        assert!(energy.qe() >= 0.0, "virus {i}: negative qe={}", energy.qe());
    }
}

// ── Satiation ────────────────────────────────────────────────────────────────

/// Virus tiene TrophicState (carnivore/parasite). Satiation decae.
#[test]
fn virus_satiation_decays() {
    let mut app = make_app();
    let scene = spawn_scene(&mut app);

    let virus = scene.viruses[0];
    let initial = app.world().get::<TrophicState>(virus).unwrap().satiation;

    run_ticks(&mut app, 100);

    let after = app.world().get::<TrophicState>(virus).unwrap().satiation;
    assert!(
        after < initial,
        "virus satiation must decay: initial={initial}, after={after}",
    );
}

// ── No-NaN ───────────────────────────────────────────────────────────────────

/// Ningún tick produce NaN en hosts ni virus.
#[test]
fn no_nan_across_100_ticks() {
    let mut app = make_app();
    let scene = spawn_scene(&mut app);

    let all: Vec<Entity> = scene
        .hosts
        .iter()
        .chain(scene.viruses.iter())
        .copied()
        .collect();

    for tick in 0..100 {
        app.update();
        for &entity in &all {
            let qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();
            assert!(!qe.is_nan(), "NaN at tick {tick}");
            assert!(qe >= 0.0, "negative qe={qe} at tick {tick}");
        }
    }
}

// ── Escena completa (6 entidades) ────────────────────────────────────────────

/// La escena completa (4 hosts + 2 virus) corre 200 ticks sin crash.
#[test]
fn full_scene_runs_200_ticks_without_panic() {
    let mut app = make_app();
    let _scene = spawn_scene(&mut app);
    run_ticks(&mut app, 200);
    // Si llegamos aquí sin panic, la escena es estable.
}
