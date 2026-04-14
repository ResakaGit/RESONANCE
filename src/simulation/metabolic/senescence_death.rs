//! Programmed senescence: age-based mortality.
//! Entities that exceed `max_viable_age` die. Entities approaching that age
//! have increasing mortality via Gompertz hazard (deterministic threshold).
//!
//! Phase: [`Phase::MetabolicLayer`], after `basal_drain_system`.

use bevy::prelude::*;

use crate::blueprint::equations::derived_thresholds as dt;
use crate::blueprint::equations::emergence::senescence::survival_probability;
use crate::events::{DeathCause, DeathEvent};
use crate::layers::{BaseEnergy, GompertzCache, SenescenceProfile};
use crate::runtime_platform::simulation_tick::SimulationClock;

/// Age-based death: hard limit + Gompertz hazard.
///
/// Uses `GompertzCache` precomputed death_tick when available (ADR-017).
/// Falls back to `survival_probability()` + `exp()` for entities without cache.
/// Hard age limit retained as safety net.
pub fn senescence_death_system(
    query: Query<(
        Entity,
        &BaseEnergy,
        &SenescenceProfile,
        Option<&GompertzCache>,
    )>,
    mut death_events: EventWriter<DeathEvent>,
    clock: Res<SimulationClock>,
) {
    for (entity, energy, senescence, gompertz) in &query {
        if energy.is_dead() {
            continue;
        }
        let age = senescence.age(clock.tick_id);

        // Hard age limit — unconditional death (safety net).
        if age >= senescence.max_viable_age {
            death_events.send(DeathEvent {
                entity,
                cause: DeathCause::Dissipation,
            });
            continue;
        }

        // Precomputed death tick (1 u64 comparison) or Gompertz exp() fallback.
        let should_die = if let Some(cache) = gompertz {
            cache.should_die(clock.tick_id)
        } else {
            let prob = survival_probability(
                age,
                senescence.senescence_coeff,
                senescence.senescence_coeff,
            );
            prob < dt::survival_probability_threshold()
        };
        if should_die {
            death_events.send(DeathEvent {
                entity,
                cause: DeathCause::Dissipation,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::emergence::senescence::survival_probability;

    #[test]
    fn young_entity_survives() {
        let prob = survival_probability(0, 0.0001, 0.0001);
        assert!(prob > dt::survival_probability_threshold());
    }

    #[test]
    fn very_old_entity_below_threshold() {
        let prob = survival_probability(40_000, 0.0001, 0.0001);
        assert!(prob < dt::survival_probability_threshold(), "prob={prob}");
    }

    #[test]
    fn survival_probability_decreases_with_age() {
        let young = survival_probability(100, 0.0001, 0.0001);
        let old = survival_probability(30_000, 0.0001, 0.0001);
        assert!(young > old);
    }
}
