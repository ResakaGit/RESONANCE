//! ET-7: Programmed Senescence — SenescenceProfile component. Capa T2-3.

use bevy::prelude::*;

/// Capa T2-3: SenescenceProfile — parámetros de mortalidad intrínseca.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SenescenceProfile {
    pub tick_birth: u64,
    pub senescence_coeff: f32,
    pub max_viable_age: u64,
    pub strategy: u8, // 0=Iteroparous, 1=Semelparous
}

impl Default for SenescenceProfile {
    fn default() -> Self {
        Self {
            tick_birth: 0,
            senescence_coeff: 0.0001,
            max_viable_age: 50_000,
            strategy: 0,
        }
    }
}

impl SenescenceProfile {
    /// Edad en ticks desde el nacimiento.
    pub fn age(&self, current_tick: u64) -> u64 {
        current_tick.saturating_sub(self.tick_birth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_age_at_tick_zero_is_zero() {
        let s = SenescenceProfile::default();
        assert_eq!(s.age(0), 0);
    }

    #[test]
    fn age_simple_difference() {
        let s = SenescenceProfile {
            tick_birth: 100,
            ..Default::default()
        };
        assert_eq!(s.age(350), 250);
    }

    #[test]
    fn age_current_before_birth_saturates_to_zero() {
        let s = SenescenceProfile {
            tick_birth: 1000,
            ..Default::default()
        };
        assert_eq!(s.age(500), 0);
    }

    #[test]
    fn default_strategy_is_iteroparous() {
        let s = SenescenceProfile::default();
        assert_eq!(s.strategy, 0);
    }

    #[test]
    fn default_max_viable_age() {
        let s = SenescenceProfile::default();
        assert_eq!(s.max_viable_age, 50_000);
    }
}
