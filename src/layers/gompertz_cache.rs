//! Cache del tick exacto de muerte por Gompertz.
//! Exact Gompertz death-tick cache.
//!
//! Precomputa algebraicamente el tick en que `survival_probability < exp(-2)`.
//! Elimina `exp()` per-tick en `senescence_death_system` — reemplaza con 1 comparación u64.
//! SparseSet: solo entidades con `SenescenceProfile`.

use bevy::prelude::*;

/// Tick exacto de muerte precomputado. SparseSet, co-localizado con SenescenceProfile.
/// Precomputed exact death tick. SparseSet, co-located with SenescenceProfile.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct GompertzCache {
    death_tick: u64,
}

impl GompertzCache {
    /// Construye el cache computando el tick exacto de muerte.
    /// Builds cache by computing the exact death tick.
    #[inline]
    pub fn from_senescence(
        birth_tick: u64,
        base_dissipation: f32,
        senescence_coeff: f32,
        max_viable_age: u64,
    ) -> Self {
        Self {
            death_tick: crate::blueprint::equations::exact_cache::exact_death_tick(
                birth_tick,
                base_dissipation,
                senescence_coeff,
                max_viable_age,
            ),
        }
    }

    /// Tick exacto en que la entidad muere.
    /// Exact tick at which the entity dies.
    #[inline]
    pub fn death_tick(&self) -> u64 {
        self.death_tick
    }

    /// ¿Debe morir en este tick?
    /// Should entity die at this tick?
    #[inline]
    pub fn should_die(&self, current_tick: u64) -> bool {
        current_tick >= self.death_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::derived_thresholds as dt;

    #[test]
    fn fauna_cache_death_tick_within_max_age() {
        let c = GompertzCache::from_senescence(
            0,
            dt::senescence_coeff_fauna(),
            dt::senescence_coeff_fauna(),
            dt::max_age_fauna(),
        );
        assert!(c.death_tick() <= dt::max_age_fauna());
        assert!(c.death_tick() > 0);
    }

    #[test]
    fn should_die_before_death_tick_returns_false() {
        let c = GompertzCache::from_senescence(100, 0.02, 0.02, 200);
        assert!(!c.should_die(100));
    }

    #[test]
    fn should_die_at_death_tick_returns_true() {
        let c = GompertzCache::from_senescence(0, 0.02, 0.02, 200);
        assert!(c.should_die(c.death_tick()));
    }

    #[test]
    fn should_die_after_death_tick_returns_true() {
        let c = GompertzCache::from_senescence(0, 0.02, 0.02, 200);
        assert!(c.should_die(c.death_tick() + 1));
    }

    #[test]
    fn birth_offset_shifts_death_tick() {
        let a = GompertzCache::from_senescence(0, 0.02, 0.02, 200);
        let b = GompertzCache::from_senescence(1000, 0.02, 0.02, 200);
        assert_eq!(b.death_tick() - a.death_tick(), 1000);
    }

    #[test]
    fn zero_coeff_dies_at_max_age() {
        let c = GompertzCache::from_senescence(0, 0.01, 0.0, 500);
        assert_eq!(c.death_tick(), 500);
    }

    #[test]
    fn nan_inputs_die_at_max_age() {
        let c = GompertzCache::from_senescence(0, f32::NAN, f32::NAN, 500);
        assert_eq!(c.death_tick(), 500);
    }

    #[test]
    fn should_die_default_at_tick_zero_returns_false() {
        let c = GompertzCache::from_senescence(0, 0.02, 0.02, 200);
        assert!(!c.should_die(0), "should survive at tick 0");
    }
}
