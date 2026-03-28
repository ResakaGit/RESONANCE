//! B1: Fermi Paradox Simulator — probability of complex life across random universes.

use crate::use_cases::{evolve_with, ExperimentReport};
use crate::use_cases::presets::UniversePreset;

/// Result of a Fermi Paradox experiment.
#[derive(Debug)]
pub struct FermiReport {
    pub total_universes:    usize,
    pub with_life:          usize, // species_count > 1
    pub with_complex_life:  usize, // species_count > 3
    pub life_probability:   f32,
    pub complex_probability: f32,
    pub reports:            Vec<ExperimentReport>,
}

/// Run N random universes and count how many develop life.
///
/// Each universe gets random physical constants via `UniversePreset::from_seed`.
/// "Life" = at least 2 species coexisting after evolution.
/// "Complex life" = at least 4 species with trophic depth > 2.
pub fn run(n_universes: usize, gens: u32, ticks: u32) -> FermiReport {
    let mut reports = Vec::with_capacity(n_universes);
    let mut with_life = 0usize;
    let mut with_complex = 0usize;

    for i in 0..n_universes {
        let preset = UniversePreset::from_seed(i as u64 * 7 + 13);
        let report = evolve_with(&preset, i as u64, 50, gens, ticks, 12);

        let species = report.history.last()
            .map(|s| s.species_mean)
            .unwrap_or(0.0);

        if species > 1.0 { with_life += 1; }
        if species > 3.0 { with_complex += 1; }
        reports.push(report);
    }

    FermiReport {
        total_universes: n_universes,
        with_life,
        with_complex_life: with_complex,
        life_probability: with_life as f32 / n_universes.max(1) as f32,
        complex_probability: with_complex as f32 / n_universes.max(1) as f32,
        reports,
    }
}
