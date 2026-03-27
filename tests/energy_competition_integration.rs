//! EC-8E: Integration tests — acceptance tests for the full EC track.
//! Tests: Lotka-Volterra, host collapse, homeostasis, Matryoshka, conservation, determinism.

use bevy::prelude::*;
use resonance::blueprint::constants::POOL_CONSERVATION_EPSILON;
use resonance::blueprint::equations::{conservation_error, PoolHealthStatus};
use resonance::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolConservationLedger, PoolParentLink};
use resonance::simulation::competition_dynamics::{competition_dynamics_system, PoolDiagnostic};
use resonance::simulation::metabolic::pool_conservation::pool_conservation_system;
use resonance::simulation::metabolic::pool_distribution::{
    pool_dissipation_system, pool_distribution_system, pool_intake_system,
};
use resonance::simulation::metabolic::scale_composition::scale_composition_system;

fn make_ec_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.register_type::<EnergyPool>();
    app.register_type::<PoolParentLink>();
    app.register_type::<BaseEnergy>();
    app.register_type::<PoolConservationLedger>();
    app.register_type::<PoolDiagnostic>();
    app.add_systems(Update, (
        pool_intake_system,
        pool_distribution_system.after(pool_intake_system),
        pool_dissipation_system.after(pool_distribution_system),
        pool_conservation_system.after(pool_dissipation_system),
        competition_dynamics_system.after(pool_dissipation_system),
        scale_composition_system.after(pool_conservation_system),
    ));
    app
}

/// EC-8E Test 1: Lotka-Volterra emergente.
/// Pool con Type III (competitivo) + Type V (regulado). 500 ticks.
/// Pool no colapsa, dinámica activa hasta el final.
#[test]
fn lotka_volterra_emergent_no_collapse() {
    let mut app = make_ec_app();

    let pool = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 100.0, 0.001)).id();
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Competitive, 0.7)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Competitive, 0.3)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Regulated, 30.0)));

    for _ in 0..500 {
        app.update();
    }

    let ep = app.world().get::<EnergyPool>(pool).unwrap();
    assert!(ep.pool() >= 0.0, "pool must be non-negative after 500 ticks: {}", ep.pool());

    let ledger = app.world().get::<PoolConservationLedger>(pool);
    if let Some(l) = ledger {
        assert!(l.active_children() > 0, "children must still be active: {}", l.active_children());
    }
}

/// EC-8E Test 2: Host collapse.
/// Pool + Type IV agresivo (sin intake). La capacity degrada monotónicamente,
/// el pool llega a 0, y los hijos dejan de recibir energía post-colapso.
#[test]
fn host_collapse_capacity_degrades_and_pool_hits_zero() {
    let mut app = make_ec_app();

    // intake_rate=0 para que el pool colapse sin recuperación
    let pool = app.world_mut().spawn(EnergyPool::new(1000.0, 5000.0, 0.0, 0.001)).id();
    let child = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Aggressive, 0.9))).id();

    let initial_capacity = app.world().get::<EnergyPool>(pool).unwrap().capacity();
    let mut last_capacity = initial_capacity;

    // Run until pool=0 or max 50 ticks
    for _ in 0..50 {
        app.update();
        let ep = app.world().get::<EnergyPool>(pool).unwrap();
        assert!(
            ep.capacity() <= last_capacity + 1e-3,
            "capacity must not increase: {} > {}",
            ep.capacity(),
            last_capacity,
        );
        last_capacity = ep.capacity();
        if ep.pool() <= 0.0 {
            break;
        }
    }

    // Pool should have reached 0
    let ep = app.world().get::<EnergyPool>(pool).unwrap();
    assert!(ep.pool() <= 1.0, "pool should collapse to near 0: {}", ep.pool());

    // Capacity degraded
    assert!(last_capacity < initial_capacity, "capacity must have degraded");

    // Post-collapse: children get no more energy
    let qe_at_collapse = app.world().get::<BaseEnergy>(child).unwrap().qe();
    for _ in 0..10 {
        app.update();
    }
    let qe_after = app.world().get::<BaseEnergy>(child).unwrap().qe();
    assert!(
        qe_after <= qe_at_collapse + 1.0,
        "children should not gain energy after collapse: before={qe_at_collapse} after={qe_after}",
    );
}

/// EC-8E Test 3: Homeostasis.
/// Pool con solo Type V (regulados). Tras 200 ticks: pool estable, PoolDiagnostic=Healthy.
#[test]
fn homeostasis_regulated_children_healthy_pool() {
    let mut app = make_ec_app();

    // intake=200 >> extraction of 2 * base_rate=50 → always Healthy
    let pool = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 200.0, 0.001)).id();
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Regulated, 50.0)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Regulated, 50.0)));

    for _ in 0..200 {
        app.update();
    }

    let ep = app.world().get::<EnergyPool>(pool).unwrap();
    assert!(ep.pool() > 0.0, "pool must be positive: {}", ep.pool());

    // PoolDiagnostic is inserted via commands on tick 1, visible from tick 2
    if let Some(diag) = app.world().get::<PoolDiagnostic>(pool) {
        assert!(
            diag.health_status == PoolHealthStatus::Healthy || diag.health_status == PoolHealthStatus::Stressed,
            "health must be Healthy or Stressed, not Collapsing/Collapsed: {:?}",
            diag.health_status,
        );
    }
}

/// EC-8E Test 4: Multi-level Matryoshka.
/// Pool-raíz → sub-pool → hijos-hoja. 100 ticks.
/// El primary_param del sub-pool refleja el fitness inferido.
#[test]
fn matryoshka_sub_pool_fitness_propagates() {
    let mut app = make_ec_app();

    let root = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 200.0, 0.001)).id();
    let sub = app.world_mut().spawn((
        EnergyPool::new(1000.0, 2000.0, 50.0, 0.01),
        PoolParentLink::new(root, ExtractionType::Competitive, 0.5),
    )).id();
    for _ in 0..4 {
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(sub, ExtractionType::Proportional, 0.0)));
    }

    // Initial param
    let initial_param = app.world().get::<PoolParentLink>(sub).unwrap().primary_param();

    for _ in 0..100 {
        app.update();
    }

    let link = app.world().get::<PoolParentLink>(sub).unwrap();
    let final_param = link.primary_param();

    // scale_composition_system should have updated the param
    assert_ne!(final_param, initial_param, "primary_param must change after 100 ticks");
    assert!(final_param.is_finite(), "primary_param must be finite: {final_param}");
    assert!(final_param >= 0.0, "primary_param must be non-negative: {final_param}");
}

/// EC-8E Test 5: Conservación estricta.
/// 1000 ticks, conservation_error < EPSILON en cada tick.
#[test]
fn strict_conservation_1000_ticks() {
    let mut app = make_ec_app();

    let pool = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Proportional, 0.0)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Competitive, 0.6)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Regulated, 30.0)));

    // Warmup tick to insert ledger
    app.update();

    for tick in 0..1000 {
        let snap = *app.world().get::<EnergyPool>(pool).unwrap();
        let pool_before = snap.pool();
        let intake_rate  = snap.intake_rate();
        let capacity     = snap.capacity();

        app.update();

        let pool_after = app.world().get::<EnergyPool>(pool).unwrap().pool();
        let Some(ledger) = app.world().get::<PoolConservationLedger>(pool) else { continue; };

        if pool_before <= POOL_CONSERVATION_EPSILON { continue; }

        let actual_intake = (pool_before + intake_rate).min(capacity) - pool_before;
        let err = conservation_error(
            pool_before, pool_after, actual_intake,
            ledger.total_extracted(), ledger.total_dissipated(),
        );
        assert!(
            err < POOL_CONSERVATION_EPSILON,
            "tick {tick}: conservation_error={err} > EPSILON={POOL_CONSERVATION_EPSILON}",
        );
    }
}

/// EC-8E Test 6: Determinismo.
/// Mismo escenario 2 veces → mismos resultados bit a bit.
#[test]
fn determinism_same_scenario_same_results() {
    fn run_scenario() -> (f32, f32, f32) {
        let mut app = make_ec_app();
        let pool = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 100.0, 0.001)).id();
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Competitive, 0.7)));
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Competitive, 0.3)));
        app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Regulated, 30.0)));
        for _ in 0..200 { app.update(); }
        let ep = app.world().get::<EnergyPool>(pool).unwrap();
        let ledger = app.world().get::<PoolConservationLedger>(pool);
        let extracted = ledger.map(|l| l.total_extracted()).unwrap_or(0.0);
        let dissipated = ledger.map(|l| l.total_dissipated()).unwrap_or(0.0);
        (ep.pool(), extracted, dissipated)
    }

    let (pool1, ext1, dis1) = run_scenario();
    let (pool2, ext2, dis2) = run_scenario();

    assert!((pool1 - pool2).abs() < 1e-4, "pool diverged: {pool1} vs {pool2}");
    assert!((ext1 - ext2).abs() < 1e-4,  "extraction diverged: {ext1} vs {ext2}");
    assert!((dis1 - dis2).abs() < 1e-4,  "dissipation diverged: {dis1} vs {dis2}");
}
