//! Passive energy drain: basal metabolic cost per tick.
//! Every living entity pays a cost for existing. Without this, there is
//! no selective pressure to forage — entities survive indefinitely at qe=1.
//!
//! Phase: [`Phase::MetabolicLayer`], before `growth_budget_system`.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::senescence::age_dependent_dissipation;
use crate::layers::{BaseEnergy, EnergyOps, SenescenceProfile, SpatialVolume};
use crate::runtime_platform::simulation_tick::SimulationClock;

/// Base drain rate (qe/tick) for an entity with radius ≈ 1.0 and no senescence.
/// Calibrated against nucleus emission: 500 qe/s ÷ ~400 entities ≈ 1.25 qe/entity/tick.
/// At 0.5, an entity needs to forage to survive. At 0.05, drain is invisible.
const BASAL_RATE: f32 = 0.5;

/// Volume exponent: larger entities pay proportionally more.
/// `drain ∝ radius ^ VOLUME_EXPONENT`. Allometric scaling (Kleiber's 3/4 law approximation).
const VOLUME_EXPONENT: f32 = 0.75;

/// Passive energy drain — the cost of being alive.
pub fn basal_drain_system(
    mut ops: EnergyOps,
    query: Query<
        (Entity, &SpatialVolume, Option<&SenescenceProfile>),
        (With<BaseEnergy>, Without<crate::worldgen::EnergyNucleus>),
    >,
    clock: Res<SimulationClock>,
) {
    for (entity, volume, senescence) in &query {
        let Some(qe) = ops.qe(entity) else { continue };
        if qe <= 0.0 {
            continue;
        }
        let age_ticks = senescence
            .map(|s| s.age(clock.tick_id))
            .unwrap_or(0);
        let senescence_coeff = senescence
            .map(|s| s.senescence_coeff)
            .unwrap_or(0.0);
        let age_factor = age_dependent_dissipation(1.0, age_ticks, senescence_coeff);
        let vol_factor = volume.radius.max(0.01).powf(VOLUME_EXPONENT);
        let drain = BASAL_RATE * vol_factor * age_factor;
        if drain > 0.0 {
            ops.drain(entity, drain, crate::events::DeathCause::Dissipation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basal_rate_positive() {
        assert!(BASAL_RATE > 0.0);
    }

    #[test]
    fn larger_entity_drains_more() {
        let small = 0.5_f32.powf(VOLUME_EXPONENT);
        let large = 2.0_f32.powf(VOLUME_EXPONENT);
        assert!(large > small);
    }

    #[test]
    fn age_increases_drain() {
        let young = age_dependent_dissipation(1.0, 0, 0.0001);
        let old = age_dependent_dissipation(1.0, 10_000, 0.0001);
        assert!(old > young);
    }
}
