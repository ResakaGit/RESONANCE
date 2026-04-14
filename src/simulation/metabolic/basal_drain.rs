//! Passive energy drain: basal metabolic cost per tick.
//! Every living entity pays a cost for existing. Without this, there is
//! no selective pressure to forage — entities survive indefinitely at qe=1.
//!
//! Phase: [`Phase::MetabolicLayer`], before `growth_budget_system`.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::senescence::age_dependent_dissipation;
use crate::layers::{BaseEnergy, EnergyOps, KleiberCache, SenescenceProfile, SpatialVolume};
use crate::runtime_platform::simulation_tick::SimulationClock;

use crate::blueprint::equations::derived_thresholds as dt;

/// Passive energy drain — the cost of being alive.
///
/// Uses `KleiberCache` dirty-flag to avoid `powf(0.75)` per-tick (ADR-017).
/// Fallback to inline `powf` for entities spawned without cache (transition safety).
pub fn basal_drain_system(
    mut ops: EnergyOps,
    mut query: Query<
        (
            Entity,
            &SpatialVolume,
            Option<&SenescenceProfile>,
            Option<&mut KleiberCache>,
        ),
        (With<BaseEnergy>, Without<crate::worldgen::EnergyNucleus>),
    >,
    clock: Res<SimulationClock>,
) {
    for (entity, volume, senescence, kleiber) in &mut query {
        let Some(qe) = ops.qe(entity) else { continue };
        if qe <= 0.0 {
            continue;
        }
        let age_ticks = senescence.map(|s| s.age(clock.tick_id)).unwrap_or(0);
        let senescence_coeff = senescence.map(|s| s.senescence_coeff).unwrap_or(0.0);
        let age_factor = age_dependent_dissipation(1.0, age_ticks, senescence_coeff);
        let vol_factor = if let Some(mut cache) = kleiber {
            cache.update(volume.radius);
            cache.vol_factor()
        } else {
            volume.radius.max(0.01).powf(dt::KLEIBER_EXPONENT)
        };
        let drain = dt::basal_drain_rate() * vol_factor * age_factor;
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
        assert!(dt::basal_drain_rate() > 0.0);
    }

    #[test]
    fn larger_entity_drains_more() {
        let small = 0.5_f32.powf(dt::KLEIBER_EXPONENT);
        let large = 2.0_f32.powf(dt::KLEIBER_EXPONENT);
        assert!(large > small);
    }

    #[test]
    fn age_increases_drain() {
        let young = age_dependent_dissipation(1.0, 0, 0.0001);
        let old = age_dependent_dissipation(1.0, 10_000, 0.0001);
        assert!(old > young);
    }
}
