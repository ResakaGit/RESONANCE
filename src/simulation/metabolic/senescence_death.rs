//! Programmed senescence: age-based mortality.
//! Entities that exceed `max_viable_age` die. Entities approaching that age
//! have increasing mortality via Gompertz hazard (deterministic threshold).
//!
//! Phase: [`Phase::MetabolicLayer`], after `basal_drain_system`.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::senescence::survival_probability;
use crate::events::{DeathCause, DeathEvent};
use crate::layers::{BaseEnergy, SenescenceProfile};
use crate::runtime_platform::simulation_tick::SimulationClock;

/// Survival probability threshold below which the entity dies.
/// At 0.05, ~5% of the theoretical population at that age would die per tick.
/// Deterministic: same age + same coeff = same outcome. No RNG needed.
const SURVIVAL_THRESHOLD: f32 = 0.05;

/// Age-based death: hard limit + Gompertz hazard.
pub fn senescence_death_system(
    query: Query<(Entity, &BaseEnergy, &SenescenceProfile)>,
    mut death_events: EventWriter<DeathEvent>,
    clock: Res<SimulationClock>,
) {
    for (entity, energy, senescence) in &query {
        if energy.is_dead() {
            continue;
        }
        let age = senescence.age(clock.tick_id);

        // Hard age limit — unconditional death.
        if age >= senescence.max_viable_age {
            death_events.send(DeathEvent { entity, cause: DeathCause::Dissipation });
            continue;
        }

        // Gompertz hazard — increasing death probability with age.
        let prob = survival_probability(age, senescence.senescence_coeff, senescence.senescence_coeff);
        if prob < SURVIVAL_THRESHOLD {
            death_events.send(DeathEvent { entity, cause: DeathCause::Dissipation });
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
        assert!(prob > SURVIVAL_THRESHOLD);
    }

    #[test]
    fn very_old_entity_below_threshold() {
        let prob = survival_probability(40_000, 0.0001, 0.0001);
        assert!(prob < SURVIVAL_THRESHOLD, "prob={prob}");
    }

    #[test]
    fn survival_probability_decreases_with_age() {
        let young = survival_probability(100, 0.0001, 0.0001);
        let old = survival_probability(30_000, 0.0001, 0.0001);
        assert!(young > old);
    }
}
