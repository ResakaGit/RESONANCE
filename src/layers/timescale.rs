//! ET-10: Multiple Timescales (Baldwin Effect) — TimescaleAdapter component. Capa T3-1.

use bevy::prelude::*;

/// Capa T3-1: TimescaleAdapter — integra cuatro velocidades de cambio fenotípico.
/// genetic: τ_g ≈ 10⁵. epigenetic: τ_e ≈ 10³. cultural: τ_c ≈ 10⁴. learned: τ_a ≈ 10².
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct TimescaleAdapter {
    pub genetic_baseline: f32,
    pub epigenetic_offset: f32,
    pub cultural_offset: f32,
    pub learned_offset: f32,
}

impl Default for TimescaleAdapter {
    fn default() -> Self {
        Self {
            genetic_baseline: 1.0,
            epigenetic_offset: 0.0,
            cultural_offset: 0.0,
            learned_offset: 0.0,
        }
    }
}

impl TimescaleAdapter {
    /// Fenotipo efectivo total.
    pub fn effective(&self) -> f32 {
        self.genetic_baseline + self.epigenetic_offset + self.cultural_offset + self.learned_offset
    }
    /// Offsets totales sobre el baseline.
    pub fn total_plasticity(&self) -> f32 {
        self.epigenetic_offset + self.cultural_offset + self.learned_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_effective_equals_genetic_baseline() {
        let t = TimescaleAdapter::default();
        assert!((t.effective() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn default_plasticity_is_zero() {
        let t = TimescaleAdapter::default();
        assert_eq!(t.total_plasticity(), 0.0);
    }

    #[test]
    fn effective_sums_all_four_timescales() {
        let t = TimescaleAdapter {
            genetic_baseline: 1.0,
            epigenetic_offset: 0.1,
            cultural_offset: 0.2,
            learned_offset: 0.3,
        };
        assert!((t.effective() - 1.6).abs() < 1e-5);
    }

    #[test]
    fn plasticity_excludes_genetic_baseline() {
        let t = TimescaleAdapter {
            genetic_baseline: 999.0,
            epigenetic_offset: 0.1,
            cultural_offset: 0.2,
            learned_offset: 0.3,
        };
        assert!((t.total_plasticity() - 0.6).abs() < 1e-5);
    }

    #[test]
    fn negative_offsets_reduce_effective() {
        let t = TimescaleAdapter {
            genetic_baseline: 1.0,
            epigenetic_offset: -0.5,
            cultural_offset: 0.0,
            learned_offset: 0.0,
        };
        assert!((t.effective() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn large_values_remain_finite() {
        let t = TimescaleAdapter {
            genetic_baseline: 1e30,
            epigenetic_offset: 1e30,
            cultural_offset: 1e30,
            learned_offset: 1e30,
        };
        assert!(t.effective().is_finite());
        assert!(t.total_plasticity().is_finite());
    }
}
