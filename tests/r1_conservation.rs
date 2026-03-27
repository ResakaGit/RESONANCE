//! R1 — Units and Conservation: integration tests.
//! Verifica:
//!   1. Ningún BaseEnergy/EnergyPool tiene NaN o Inf tras 1000 ticks.
//!   2. conservation_error < CONSERVATION_ERROR_TOLERANCE en cada pool con ledger (500 ticks).
//!   3. Ninguna entidad tiene qe < 0 tras extracción agresiva (100 ticks).

use bevy::prelude::*;
use resonance::blueprint::constants::{CONSERVATION_ERROR_TOLERANCE, POOL_CONSERVATION_EPSILON};
use resonance::blueprint::equations::{conservation_error, global_conservation_error, has_invalid_values, is_valid_qe};
use resonance::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolConservationLedger, PoolParentLink};
use resonance::simulation::competition_dynamics::{competition_dynamics_system, PoolDiagnostic};
use resonance::simulation::metabolic::pool_conservation::pool_conservation_system;
use resonance::simulation::metabolic::pool_distribution::{
    pool_dissipation_system, pool_distribution_system, pool_intake_system,
};
use resonance::simulation::metabolic::scale_composition::scale_composition_system;

// ─── Shared app factory ───────────────────────────────────────────────────────

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

// ─── R1-1: No NaN after 1000 ticks ───────────────────────────────────────────

/// Scenario: 3 pools, múltiples hijos por pool (competition_arena).
/// Invariant: BaseEnergy.qe y EnergyPool.pool() siempre finitos y ≥ 0.
#[test]
fn test_no_nan_after_1000_ticks() {
    let mut app = make_ec_app();

    // Pool A: 3 hijos proportional
    let pool_a = app.world_mut().spawn(EnergyPool::new(5000.0, 10000.0, 100.0, 0.01)).id();
    for _ in 0..3 {
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(pool_a, ExtractionType::Proportional, 0.0),
        ));
    }

    // Pool B: 2 hijos competitive
    let pool_b = app.world_mut().spawn(EnergyPool::new(3000.0, 8000.0, 80.0, 0.005)).id();
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_b, ExtractionType::Competitive, 0.7)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_b, ExtractionType::Competitive, 0.3)));

    // Pool C: 1 hijo regulated + 1 greedy
    let pool_c = app.world_mut().spawn(EnergyPool::new(2000.0, 5000.0, 50.0, 0.01)).id();
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_c, ExtractionType::Regulated, 40.0)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool_c, ExtractionType::Greedy, 200.0)));

    let pools = [pool_a, pool_b, pool_c];

    for tick in 0..1000 {
        app.update();

        // Verificar EnergyPool
        for &pool_entity in &pools {
            let ep = app.world().get::<EnergyPool>(pool_entity).unwrap();
            assert!(
                ep.pool().is_finite(),
                "tick {tick}: pool_entity={pool_entity:?} pool=NaN/Inf: {}",
                ep.pool(),
            );
            assert!(
                ep.pool() >= 0.0,
                "tick {tick}: pool_entity={pool_entity:?} pool negative: {}",
                ep.pool(),
            );
        }

        // Verificar BaseEnergy de todos los hijos
        let invalid_entities: Vec<(Entity, f32)> = app
            .world_mut()
            .query::<(Entity, &BaseEnergy)>()
            .iter(app.world())
            .filter(|(_, e)| !is_valid_qe(e.qe()))
            .map(|(ent, e)| (ent, e.qe()))
            .collect();
        assert!(
            invalid_entities.is_empty(),
            "tick {tick}: entities with invalid qe: {invalid_entities:?}",
        );
    }
}

// ─── R1-2: Pool conservation invariant ───────────────────────────────────────

/// Scenario: pool único con 4 hijos de tipos mixtos. 500 ticks.
/// Invariant: conservation_error < CONSERVATION_ERROR_TOLERANCE en cada tick con ledger.
#[test]
fn test_pool_conservation_invariant_holds() {
    let mut app = make_ec_app();

    let pool = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Proportional, 0.0)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Competitive, 0.6)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Regulated, 30.0)));
    app.world_mut().spawn((BaseEnergy::new(0.0), PoolParentLink::new(pool, ExtractionType::Greedy, 150.0)));

    // Warmup: inserta el ledger
    app.update();

    let mut violations = 0u32;

    for tick in 0..500 {
        let snap = *app.world().get::<EnergyPool>(pool).unwrap();
        let pool_before = snap.pool();
        let intake_rate  = snap.intake_rate();
        let capacity     = snap.capacity();

        app.update();

        let pool_after = app.world().get::<EnergyPool>(pool).unwrap().pool();
        let Some(ledger) = app.world().get::<PoolConservationLedger>(pool) else {
            continue;
        };

        // Saltear ticks donde pool_before ≤ epsilon (pool casi vacío)
        if pool_before <= POOL_CONSERVATION_EPSILON {
            continue;
        }

        let actual_intake = (pool_before + intake_rate).min(capacity) - pool_before;
        let err = conservation_error(
            pool_before, pool_after, actual_intake,
            ledger.total_extracted(), ledger.total_dissipated(),
        );

        assert!(
            err < CONSERVATION_ERROR_TOLERANCE,
            "tick {tick}: conservation_error={err} >= TOLERANCE={CONSERVATION_ERROR_TOLERANCE} \
             before={pool_before} after={pool_after} \
             extracted={} dissipated={}",
            ledger.total_extracted(), ledger.total_dissipated(),
        );
        violations += (err >= CONSERVATION_ERROR_TOLERANCE) as u32;
    }

    assert_eq!(violations, 0, "conservation violations detected");
}

// ─── R1-3: Energy never negative ─────────────────────────────────────────────

/// Scenario: entidades con qe=1.0 bajo extracción agresiva (aggression=0.9).
/// Invariant: qe ≥ 0 en todos los hijos después de 100 ticks.
#[test]
fn test_energy_never_negative() {
    let mut app = make_ec_app();

    // Pool pequeño con intake nulo para forzar colapso rápido
    let pool = app.world_mut().spawn(EnergyPool::new(1.0, 100.0, 0.0, 0.01)).id();

    let mut children = Vec::new();
    for _ in 0..5 {
        let child = app.world_mut().spawn((
            BaseEnergy::new(1.0),
            PoolParentLink::new(pool, ExtractionType::Aggressive, 0.9),
        )).id();
        children.push(child);
    }

    for tick in 0..100 {
        app.update();

        for &child in &children {
            let Some(energy) = app.world().get::<BaseEnergy>(child) else {
                continue;
            };
            assert!(
                energy.qe() >= 0.0,
                "tick {tick}: child={child:?} qe negative: {}",
                energy.qe(),
            );
            assert!(
                energy.qe().is_finite(),
                "tick {tick}: child={child:?} qe not finite: {}",
                energy.qe(),
            );
        }
    }
}

// ─── R1-4: global_conservation_error on multi-pool snapshot ──────────────────

/// Verifica que global_conservation_error retorna 0 cuando la suma de extracciones
/// cabe dentro del pool disponible (invariante del algoritmo EC-4B).
#[test]
fn test_global_conservation_error_zero_on_valid_extraction() {
    let available = 1000.0_f32;
    let extracted = [300.0_f32, 250.0, 200.0];
    let err = global_conservation_error(available, &extracted);
    assert_eq!(err, 0.0, "no overshoot → error must be 0, got {err}");
}

/// Verifica que global_conservation_error detecta sobreextracción.
#[test]
fn test_global_conservation_error_detects_overshoot() {
    let available = 500.0_f32;
    let extracted = [300.0_f32, 300.0]; // sum=600 > 500
    let err = global_conservation_error(available, &extracted);
    assert!(err > 0.0, "overshoot must yield error > 0, got {err}");
    assert!((err - 100.0).abs() < 1e-4, "expected err=100.0, got {err}");
}

// ─── R1-5: has_invalid_values guard on pool snapshot ─────────────────────────

/// Verifica que has_invalid_values detecta NaN en un snapshot de pools reales.
#[test]
fn test_has_invalid_values_nan_detection() {
    let mut snapshot = [100.0_f32, 200.0, 300.0];
    assert!(!has_invalid_values(&snapshot), "all finite → no invalids");
    snapshot[1] = f32::NAN;
    assert!(has_invalid_values(&snapshot), "NaN must be detected");
}
