//! EC-6B: Verificación de conservación de pools post-distribución.
//!
//! STATUS: IMPLEMENTED, NOT REGISTERED. Used in integration tests only.
//! Designed for Phase::MetabolicLayer after pool_distribution_system, but
//! no plugin wires it into the schedule.

use bevy::prelude::*;

use crate::blueprint::constants::POOL_CONSERVATION_EPSILON;
use crate::layers::{EnergyPool, PoolConservationLedger};

/// Verifica invariantes de conservación post-distribución.
pub fn pool_conservation_system(
    query: Query<(Entity, &PoolConservationLedger, &EnergyPool)>,
) {
    for (entity, ledger, pool) in &query {
        debug_assert!(
            pool.pool() >= 0.0,
            "EC-6: negative pool on {entity:?}: {}",
            pool.pool(),
        );
        debug_assert!(
            ledger.total_extracted() >= 0.0,
            "EC-6: negative extraction on {entity:?}: {}",
            ledger.total_extracted(),
        );
        debug_assert!(
            ledger.total_dissipated() >= 0.0,
            "EC-6: negative dissipation on {entity:?}: {}",
            ledger.total_dissipated(),
        );

        let expected_net = pool.intake_rate() - ledger.total_extracted() - ledger.total_dissipated();
        debug_assert!(
            (ledger.net_delta() - expected_net).abs() < POOL_CONSERVATION_EPSILON,
            "EC-6: net_delta mismatch on {entity:?}: got={} expected={expected_net}",
            ledger.net_delta(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::POOL_CONSERVATION_EPSILON;
    use crate::blueprint::equations::conservation_error;
    use crate::layers::{BaseEnergy, ExtractionType, PoolParentLink};
    use crate::simulation::metabolic::pool_distribution::{
        pool_dissipation_system, pool_distribution_system, pool_intake_system,
    };

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.register_type::<EnergyPool>();
        app.register_type::<PoolParentLink>();
        app.register_type::<BaseEnergy>();
        app.register_type::<PoolConservationLedger>();
        app.add_systems(Update, (
            pool_intake_system,
            pool_distribution_system.after(pool_intake_system),
            pool_dissipation_system.after(pool_distribution_system),
            pool_conservation_system.after(pool_dissipation_system),
        ));
        app
    }

    #[test]
    fn ledger_inserted_after_distribution() {
        let mut app = make_app();
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        app.update(); // distribution inserts ledger via commands
        app.update(); // commands flushed, ledger visible + updated

        let ledger = app.world().get::<PoolConservationLedger>(parent);
        assert!(ledger.is_some(), "ledger should exist after distribution");
        let ledger = ledger.unwrap();
        assert!(ledger.total_extracted() > 0.0, "children extracted: {}", ledger.total_extracted());
        assert_eq!(ledger.active_children(), 2);
    }

    #[test]
    fn ledger_net_delta_correct() {
        let mut app = make_app();
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        app.update();
        app.update();

        let pool = app.world().get::<EnergyPool>(parent).unwrap();
        let ledger = app.world().get::<PoolConservationLedger>(parent).unwrap();
        let expected_net = pool.intake_rate() - ledger.total_extracted() - ledger.total_dissipated();
        assert!(
            (ledger.net_delta() - expected_net).abs() < 1e-6,
            "net_delta={} expected={expected_net}",
            ledger.net_delta(),
        );
    }

    #[test]
    fn ledger_idempotent_structure() {
        let mut app = make_app();
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.01)).id();
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        for _ in 0..4 {
            app.update();
        }

        let ledger = app.world().get::<PoolConservationLedger>(parent).unwrap();
        assert!(ledger.total_extracted() >= 0.0);
        assert!(ledger.total_dissipated() >= 0.0);
        assert_eq!(ledger.active_children(), 1);
    }

    #[test]
    fn ledger_active_children_count() {
        let mut app = make_app();
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
        for _ in 0..4 {
            app.world_mut().spawn((
                BaseEnergy::new(0.0),
                PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
            ));
        }

        app.update();
        app.update();

        let ledger = app.world().get::<PoolConservationLedger>(parent).unwrap();
        assert_eq!(ledger.active_children(), 4);
    }

    #[test]
    fn conservation_100_ticks_below_epsilon() {
        let mut app = make_app();
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01)).id();
        for _ in 0..4 {
            app.world_mut().spawn((
                BaseEnergy::new(0.0),
                PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
            ));
        }

        app.update(); // init

        for tick in 0..100 {
            let pool_snap = *app.world().get::<EnergyPool>(parent).unwrap();
            let pool_before = pool_snap.pool();
            let intake_rate = pool_snap.intake_rate();
            let capacity    = pool_snap.capacity();

            app.update();

            let pool_after = app.world().get::<EnergyPool>(parent).unwrap().pool();
            let Some(ledger) = app.world().get::<PoolConservationLedger>(parent) else {
                continue;
            };

            let actual_intake = (pool_before + intake_rate).min(capacity) - pool_before;
            let err = conservation_error(
                pool_before, pool_after, actual_intake,
                ledger.total_extracted(), ledger.total_dissipated(),
            );
            if pool_before > POOL_CONSERVATION_EPSILON {
                assert!(
                    err < POOL_CONSERVATION_EPSILON,
                    "tick {tick}: err={err} before={pool_before} after={pool_after}"
                );
            }
        }
    }

    #[test]
    fn conservation_aggressive_type_iv_does_not_violate() {
        let mut app = make_app();
        let parent = app.world_mut().spawn(EnergyPool::new(1000.0, 5000.0, 200.0, 0.01)).id();
        app.world_mut().spawn((
            BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Aggressive, 0.5),
        ));

        app.update();

        for tick in 0..50 {
            let pool_snap = *app.world().get::<EnergyPool>(parent).unwrap();
            let pool_before = pool_snap.pool();
            let intake_rate = pool_snap.intake_rate();
            let capacity    = pool_snap.capacity();

            app.update();

            let pool_after = app.world().get::<EnergyPool>(parent).unwrap().pool();
            let Some(ledger) = app.world().get::<PoolConservationLedger>(parent) else {
                continue;
            };

            let actual_intake = (pool_before + intake_rate).min(capacity) - pool_before;
            let err = conservation_error(
                pool_before, pool_after, actual_intake,
                ledger.total_extracted(), ledger.total_dissipated(),
            );
            if pool_before > POOL_CONSERVATION_EPSILON {
                assert!(
                    err < POOL_CONSERVATION_EPSILON,
                    "aggressive tick {tick}: err={err}"
                );
            }
        }
    }

    #[test]
    fn stress_10_pools_50_children_zero_violations() {
        let mut app = make_app();
        let mut parents = Vec::new();
        for _ in 0..10 {
            let parent = app.world_mut().spawn(
                EnergyPool::new(1000.0, 5000.0, 100.0, 0.01)
            ).id();
            for _ in 0..5 {
                app.world_mut().spawn((
                    BaseEnergy::new(0.0),
                    PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
                ));
            }
            parents.push(parent);
        }

        for _ in 0..1000 {
            app.update();
        }

        for &parent in &parents {
            let pool = app.world().get::<EnergyPool>(parent).unwrap();
            assert!(pool.pool() >= 0.0, "pool negative after stress");
            assert!(pool.pool() <= pool.capacity(), "pool exceeds capacity");
        }
    }
}
