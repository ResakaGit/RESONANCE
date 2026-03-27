//! R2 — Determinism and Replay: integration tests.
//! Garantiza que misma configuración inicial → mismo estado final, verificable con hash.
//!
//! Escenario compartido: 3 pools con hijos EC de tipos variados.
//! Cada test construye el escenario desde cero dos o más veces y compara snapshots.

use bevy::prelude::*;
use resonance::blueprint::equations::{hash_f32_slice, snapshot_hash, snapshots_match};
use resonance::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolConservationLedger, PoolParentLink};
use resonance::simulation::competition_dynamics::{competition_dynamics_system, PoolDiagnostic};
use resonance::simulation::metabolic::pool_conservation::pool_conservation_system;
use resonance::simulation::metabolic::pool_distribution::{
    pool_dissipation_system, pool_distribution_system, pool_intake_system,
};
use resonance::simulation::metabolic::scale_composition::scale_composition_system;

// ─── App factory ─────────────────────────────────────────────────────────────

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

// ─── Scenario builder ────────────────────────────────────────────────────────

/// Construye escenario canónico: 3 pools con tipos de extracción variados.
/// Orden de spawn fijo → Entity ids reproducibles → snapshot canónico.
///
/// Pool A (5000 qe, cap 10000, intake 100, disip 0.01):
///   - 2 hijos Proportional
///   - 1 hijo Competitive (fitness 0.7)
///
/// Pool B (3000 qe, cap 8000, intake 80, disip 0.005):
///   - 1 hijo Greedy (param 500)
///   - 1 hijo Regulated (param 30)
///
/// Pool C (2000 qe, cap 5000, intake 50, disip 0.01):
///   - 1 hijo Competitive (fitness 0.4)
///   - 1 hijo Aggressive (fitness 0.5)
///
/// Retorna: `[pool_a, pool_b, pool_c, child_a1, child_a2, child_a3,
///            child_b1, child_b2, child_c1, child_c2]`
fn spawn_canonical_scenario(app: &mut App) -> [Entity; 10] {
    let pool_a = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 100.0, 0.01)).id();
    let pool_b = app.world_mut().spawn(EnergyPool::new(3000.0,  8000.0,  80.0, 0.005)).id();
    let pool_c = app.world_mut().spawn(EnergyPool::new(2000.0,  5000.0,  50.0, 0.01)).id();

    let child_a1 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_a, ExtractionType::Proportional, 0.0))).id();
    let child_a2 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_a, ExtractionType::Proportional, 0.0))).id();
    let child_a3 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_a, ExtractionType::Competitive,  0.7))).id();

    let child_b1 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_b, ExtractionType::Greedy,    500.0))).id();
    let child_b2 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_b, ExtractionType::Regulated,  30.0))).id();

    let child_c1 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_c, ExtractionType::Competitive, 0.4))).id();
    let child_c2 = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_c, ExtractionType::Aggressive,  0.5))).id();

    [pool_a, pool_b, pool_c, child_a1, child_a2, child_a3, child_b1, child_b2, child_c1, child_c2]
}

/// Extrae snapshot canónico de energías: pools (pool amount) + hijos (BaseEnergy.qe).
/// El orden de `entities` determina el orden del snapshot — usar siempre el mismo.
fn energy_snapshot(app: &App, entities: &[Entity; 10]) -> [f32; 10] {
    let pools  = &entities[0..3];
    let childs = &entities[3..10];

    let mut snap = [0.0f32; 10];
    for (i, &e) in pools.iter().enumerate() {
        snap[i] = app.world().get::<EnergyPool>(e).map(|p| p.pool()).unwrap_or(0.0);
    }
    for (i, &e) in childs.iter().enumerate() {
        snap[3 + i] = app.world().get::<BaseEnergy>(e).map(|b| b.qe()).unwrap_or(0.0);
    }
    snap
}

// ─── R2-1: same seed → same snapshot after 200 ticks ─────────────────────────

/// Construye el mismo escenario dos veces (3 pools, 7 hijos con extraction types variados),
/// corre 200 ticks cada uno y compara snapshots de EnergyPool + BaseEnergy bit a bit.
#[test]
fn determinism_same_seed_same_energy_snapshot() {
    let (hash_run1, hash_run2) = {
        let mut app1 = make_ec_app();
        let entities1 = spawn_canonical_scenario(&mut app1);
        for _ in 0..200 { app1.update(); }
        let snap1 = energy_snapshot(&app1, &entities1);

        let mut app2 = make_ec_app();
        let entities2 = spawn_canonical_scenario(&mut app2);
        for _ in 0..200 { app2.update(); }
        let snap2 = energy_snapshot(&app2, &entities2);

        assert!(
            snapshots_match(&snap1, &snap2),
            "R2-1: snapshots differ after 200 ticks\nrun1={snap1:?}\nrun2={snap2:?}",
        );
        (snapshot_hash(&snap1), snapshot_hash(&snap2))
    };
    assert_eq!(hash_run1, hash_run2, "R2-1: hashes must be equal");
}

// ─── R2-2: pool distribution order stable across runs ────────────────────────

/// 10 hijos en un único pool, 100 ticks. El pool residual final debe ser idéntico
/// entre dos corridas → confirma que el orden de distribución es estable.
#[test]
fn determinism_pool_distribution_order_stable() {
    fn run_10_children_100_ticks() -> f32 {
        let mut app = make_ec_app();
        let pool = app.world_mut().spawn(EnergyPool::new(2000.0, 5000.0, 50.0, 0.01)).id();
        for i in 0..10 {
            let et = match i % 5 {
                0 => ExtractionType::Proportional,
                1 => ExtractionType::Greedy,
                2 => ExtractionType::Competitive,
                3 => ExtractionType::Regulated,
                _ => ExtractionType::Aggressive,
            };
            let param = (i as f32) * 10.0 + 5.0;
            app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, et, param)));
        }
        for _ in 0..100 { app.update(); }
        app.world().get::<EnergyPool>(pool).map(|p| p.pool()).unwrap_or(0.0)
    }

    let pool_run1 = run_10_children_100_ticks();
    let pool_run2 = run_10_children_100_ticks();

    assert_eq!(
        pool_run1.to_bits(), pool_run2.to_bits(),
        "R2-2: pool residual differs between runs: run1={pool_run1} run2={pool_run2}",
    );
}

// ─── R2-3: 1000 ticks, 3 runs → same final hash ──────────────────────────────

/// Corre el mismo escenario canónico 3 veces durante 1000 ticks.
/// Los 3 hashes de snapshot final deben ser idénticos.
#[test]
fn determinism_1000_ticks_no_divergence() {
    let mut hashes = [0u64; 3];

    for (run_idx, h) in hashes.iter_mut().enumerate() {
        let mut app = make_ec_app();
        let entities = spawn_canonical_scenario(&mut app);
        for _ in 0..1000 { app.update(); }
        let snap = energy_snapshot(&app, &entities);
        *h = snapshot_hash(&snap);
        let _ = run_idx; // silence unused warning
    }

    assert_eq!(
        hashes[0], hashes[1],
        "R2-3: run0 vs run1 diverged: {} != {}",
        hashes[0], hashes[1],
    );
    assert_eq!(
        hashes[1], hashes[2],
        "R2-3: run1 vs run2 diverged: {} != {}",
        hashes[1], hashes[2],
    );
}

// ─── R2-4: different entity count → different hash (control negativo) ─────────

/// Control negativo: mismo escenario pero con distinto número de hijos.
/// Los hashes deben ser DISTINTOS, confirma que el hash discrimina estado diferente.
#[test]
fn determinism_different_entity_count_different_hash() {
    // Escenario A: escenario canónico (7 hijos)
    let hash_canonical = {
        let mut app = make_ec_app();
        let entities = spawn_canonical_scenario(&mut app);
        for _ in 0..200 { app.update(); }
        let snap = energy_snapshot(&app, &entities);
        snapshot_hash(&snap)
    };

    // Escenario B: solo pool_a con 2 hijos Proportional (subset, diferente resultado)
    let hash_reduced = {
        let mut app = make_ec_app();
        let pool = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 100.0, 0.01)).id();
        let c1   = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Proportional, 0.0))).id();
        let c2   = app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Proportional, 0.0))).id();
        for _ in 0..200 { app.update(); }
        let snap = [
            app.world().get::<EnergyPool>(pool).map(|p| p.pool()).unwrap_or(0.0),
            app.world().get::<BaseEnergy>(c1).map(|b| b.qe()).unwrap_or(0.0),
            app.world().get::<BaseEnergy>(c2).map(|b| b.qe()).unwrap_or(0.0),
        ];
        hash_f32_slice(&snap)
    };

    assert_ne!(
        hash_canonical, hash_reduced,
        "R2-4: control negativo fallido — escenarios distintos produjeron el mismo hash",
    );
}
