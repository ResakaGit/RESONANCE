//! Probe: planta fotosintética — photosynthesis + allometric growth.
//!
//! Verifica que la fotosíntesis genera qe y que el crecimiento radial
//! ocurre cuando hay biomasa disponible.

use bevy::prelude::*;
use resonance::blueprint::IdGenerator;
use resonance::entities::archetypes::spawn_planta_demo;
use resonance::events::DeathEvent;
use resonance::layers::{
    BaseEnergy, GrowthBudget, GrowthIntent, IrradianceReceiver, SpatialVolume,
};
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::simulation::lifecycle::allometric_growth::allometric_growth_system;
use resonance::simulation::metabolic::photosynthesis::photosynthetic_contribution_system;

// ── App factories ────────────────────────────────────────────────────────────

/// App con fotosíntesis solamente.
fn photosynthesis_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeathEvent>();
    app.add_systems(Update, photosynthetic_contribution_system);
    app
}

/// App con crecimiento alométrico solamente.
fn growth_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeathEvent>();
    app.add_systems(Update, allometric_growth_system);
    app
}

/// App con fotosíntesis → crecimiento encadenados.
fn full_planta_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeathEvent>();
    app.add_systems(
        Update,
        (
            photosynthetic_contribution_system,
            allometric_growth_system.after(photosynthetic_contribution_system),
        ),
    );
    app
}

fn spawn_planta_in(app: &mut App) -> Entity {
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();
    let entity = {
        let mut commands = app.world_mut().commands();
        let e = spawn_planta_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
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

// ── Photosynthesis ───────────────────────────────────────────────────────────

/// Con irradiancia inyectada manualmente, la fotosíntesis debe incrementar qe.
#[test]
fn photosynthesis_increases_qe_with_irradiance() {
    let mut app = photosynthesis_app();
    let entity = spawn_planta_in(&mut app);

    // Inyectar irradiancia manualmente (el irradiance_update_system
    // necesita nuclei + almanac, pero aquí probamos solo la contribución).
    if let Some(mut ir) = app.world_mut().get_mut::<IrradianceReceiver>(entity) {
        ir.photon_density = 25.0;
        ir.absorbed_fraction = 0.75;
    }

    let initial_qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();

    run_ticks(&mut app, 200);

    let final_qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();
    assert!(
        final_qe > initial_qe,
        "photosynthesis should increase qe: initial={initial_qe}, final={final_qe}",
    );
}

/// Sin irradiancia, la fotosíntesis no inyecta energía extra.
#[test]
fn photosynthesis_zero_irradiance_no_qe_increase() {
    let mut app = photosynthesis_app();
    let entity = spawn_planta_in(&mut app);

    // photon_density=0 por defecto en spawn_planta_demo
    let initial_qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();

    run_ticks(&mut app, 100);

    let final_qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();
    assert!(
        (final_qe - initial_qe).abs() < 0.01,
        "without irradiance, qe should stay constant: initial={initial_qe}, final={final_qe}",
    );
}

// ── Allometric Growth ────────────────────────────────────────────────────────

/// Con biomasa disponible > 0, el radio debe crecer.
#[test]
fn allometric_growth_increases_radius_with_budget() {
    let mut app = growth_app();
    let entity = spawn_planta_in(&mut app);

    let initial_radius = app.world().get::<SpatialVolume>(entity).unwrap().radius;

    // Dar biomasa + GrowthIntent (normalmente generado por growth_budget_system)
    if let Some(mut gb) = app.world_mut().get_mut::<GrowthBudget>(entity) {
        gb.biomass_available = 50.0;
    }
    app.world_mut()
        .entity_mut(entity)
        .insert(GrowthIntent::new(0.01, 1.0, 1.0));

    run_ticks(&mut app, 100);

    let final_radius = app.world().get::<SpatialVolume>(entity).unwrap().radius;
    assert!(
        final_radius > initial_radius,
        "radius should grow with budget: initial={initial_radius}, final={final_radius}",
    );
}

/// Sin biomasa, el radio no crece.
#[test]
fn allometric_growth_no_budget_no_growth() {
    let mut app = growth_app();
    let entity = spawn_planta_in(&mut app);

    let initial_radius = app.world().get::<SpatialVolume>(entity).unwrap().radius;

    // biomass_available defaults to something from spawn; force it to 0
    if let Some(mut gb) = app.world_mut().get_mut::<GrowthBudget>(entity) {
        gb.biomass_available = 0.0;
    }

    run_ticks(&mut app, 100);

    let final_radius = app.world().get::<SpatialVolume>(entity).unwrap().radius;
    assert!(
        (final_radius - initial_radius).abs() < 0.01,
        "without budget, radius should not grow: initial={initial_radius}, final={final_radius}",
    );
}

// ── No-NaN ───────────────────────────────────────────────────────────────────

/// Ningún tick produce NaN en la planta con pipeline completo.
#[test]
fn no_nan_across_300_ticks() {
    let mut app = full_planta_app();
    let entity = spawn_planta_in(&mut app);

    // Dar irradiancia para activar fotosíntesis
    if let Some(mut ir) = app.world_mut().get_mut::<IrradianceReceiver>(entity) {
        ir.photon_density = 20.0;
        ir.absorbed_fraction = 0.75;
    }

    for tick in 0..300 {
        app.update();
        let qe = app.world().get::<BaseEnergy>(entity).unwrap().qe();
        assert!(!qe.is_nan(), "NaN in BaseEnergy at tick {tick}");
        assert!(qe >= 0.0, "negative qe={qe} at tick {tick}");

        let r = app.world().get::<SpatialVolume>(entity).unwrap().radius;
        assert!(!r.is_nan(), "NaN in radius at tick {tick}");
        assert!(r >= 0.0, "negative radius={r} at tick {tick}");
    }
}

// ── Supervivencia ────────────────────────────────────────────────────────────

/// 2 plantas (mismo patrón que demo_planta) sobreviven 300 ticks.
#[test]
fn two_plantas_survive_300_ticks() {
    let mut app = full_planta_app();
    let layout = SimWorldTransformParams::default();
    let mut id_gen = IdGenerator::default();

    let entities: Vec<Entity> = {
        let mut commands = app.world_mut().commands();
        let es = vec![
            spawn_planta_demo(&mut commands, &mut id_gen, Vec2::new(-2.5, -1.5), &layout),
            spawn_planta_demo(&mut commands, &mut id_gen, Vec2::new(2.5, -1.5), &layout),
        ];
        drop(commands);
        es
    };
    app.update(); // flush

    run_ticks(&mut app, 300);

    for (i, &entity) in entities.iter().enumerate() {
        let Some(energy) = app.world().get::<BaseEnergy>(entity) else {
            panic!("planta {i} disappeared after 300 ticks");
        };
        assert!(
            energy.qe() >= 0.0,
            "planta {i}: negative qe={}",
            energy.qe()
        );
    }
}
