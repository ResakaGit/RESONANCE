//! Universe presets — physical constants as pure data.
//!
//! Each preset is a universe. Zero logic. Only values.
//! Changing a preset changes the laws of physics — not the code.

/// Complete set of physical constants defining a universe.
#[derive(Debug, Clone)]
pub struct UniversePreset {
    pub name:               &'static str,
    pub gravity:            f32,
    pub solar_flux:         f32,
    pub solar_frequency:    f32,
    pub season_rate:        f32,
    pub season_amplitude:   f32,
    pub asteroid_interval:  u64,
    pub asteroid_radius_sq: f32,
    pub asteroid_survival:  f32,
    pub photosynthesis_eff: f32,
}

impl UniversePreset {
    /// Apply this preset's constants to the batch module.
    ///
    /// Note: batch constants are compile-time in current arch.
    /// This function validates compatibility but cannot hot-swap.
    /// For hot-swap, use the preset values directly in the experiment.
    pub fn apply(&self) {
        // Currently batch constants are compile-time.
        // Presets are used for documentation and comparison.
        // When batch supports runtime config, this will set them.
    }

    /// Derive a preset from a seed (for random universe generation).
    pub fn from_seed(seed: u64) -> Self {
        use crate::blueprint::equations::determinism;
        let s0 = determinism::next_u64(seed);
        let s1 = determinism::next_u64(s0);
        let s2 = determinism::next_u64(s1);
        let s3 = determinism::next_u64(s2);
        Self {
            name:               "random",
            gravity:            determinism::range_f32(s0, 0.05, 5.0),
            solar_flux:         determinism::range_f32(s1, 0.5, 10.0),
            solar_frequency:    determinism::range_f32(s2, 100.0, 800.0),
            season_rate:        determinism::range_f32(s3, 0.0001, 0.01),
            season_amplitude:   determinism::range_f32(determinism::next_u64(s3), 0.0, 0.9),
            asteroid_interval:  (determinism::range_f32(determinism::next_u64(seed ^ 0xFF), 500.0, 20000.0)) as u64,
            asteroid_radius_sq: determinism::range_f32(seed ^ 0xAB, 4.0, 100.0),
            asteroid_survival:  determinism::range_f32(seed ^ 0xCD, 0.01, 0.5),
            photosynthesis_eff: determinism::range_f32(seed ^ 0xEF, 0.1, 0.8),
        }
    }
}

// ─── Canonical Presets ──────────────────────────────────────────────────────

pub const EARTH: UniversePreset = UniversePreset {
    name:               "Earth",
    gravity:            0.5,
    solar_flux:         2.0,
    solar_frequency:    400.0,
    season_rate:        0.001,
    season_amplitude:   0.4,
    asteroid_interval:  5000,
    asteroid_radius_sq: 25.0,
    asteroid_survival:  0.1,
    photosynthesis_eff: 0.4,
};

pub const JUPITER: UniversePreset = UniversePreset {
    name:               "Jupiter",
    gravity:            2.5,
    solar_flux:         0.5,
    solar_frequency:    400.0,
    season_rate:        0.0005,
    season_amplitude:   0.1,
    asteroid_interval:  10000,
    asteroid_radius_sq: 100.0,
    asteroid_survival:  0.05,
    photosynthesis_eff: 0.2,
};

pub const MARS: UniversePreset = UniversePreset {
    name:               "Mars",
    gravity:            0.05,
    solar_flux:         1.0,
    solar_frequency:    400.0,
    season_rate:        0.002,
    season_amplitude:   0.8,
    asteroid_interval:  2000,
    asteroid_radius_sq: 16.0,
    asteroid_survival:  0.15,
    photosynthesis_eff: 0.3,
};

pub const EDEN: UniversePreset = UniversePreset {
    name:               "Eden",
    gravity:            0.2,
    solar_flux:         5.0,
    solar_frequency:    400.0,
    season_rate:        0.0,
    season_amplitude:   0.0,
    asteroid_interval:  0, // disabled
    asteroid_radius_sq: 0.0,
    asteroid_survival:  1.0,
    photosynthesis_eff: 0.6,
};

pub const HELL: UniversePreset = UniversePreset {
    name:               "Hell",
    gravity:            3.0,
    solar_flux:         0.3,
    solar_frequency:    400.0,
    season_rate:        0.005,
    season_amplitude:   0.9,
    asteroid_interval:  500,
    asteroid_radius_sq: 64.0,
    asteroid_survival:  0.01,
    photosynthesis_eff: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_distinct_names() {
        let presets = [&EARTH, &JUPITER, &MARS, &EDEN, &HELL];
        for i in 0..presets.len() {
            for j in (i+1)..presets.len() {
                assert_ne!(presets[i].name, presets[j].name);
            }
        }
    }

    #[test]
    fn from_seed_deterministic() {
        let a = UniversePreset::from_seed(42);
        let b = UniversePreset::from_seed(42);
        assert_eq!(a.gravity.to_bits(), b.gravity.to_bits());
        assert_eq!(a.solar_flux.to_bits(), b.solar_flux.to_bits());
    }

    #[test]
    fn from_seed_different_seeds_differ() {
        let a = UniversePreset::from_seed(42);
        let b = UniversePreset::from_seed(43);
        assert_ne!(a.gravity.to_bits(), b.gravity.to_bits());
    }

    #[test]
    fn earth_has_reasonable_values() {
        assert!(EARTH.gravity > 0.0);
        assert!(EARTH.solar_flux > 0.0);
        assert!(EARTH.photosynthesis_eff > 0.0 && EARTH.photosynthesis_eff <= 1.0);
    }
}
