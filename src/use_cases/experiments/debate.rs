//! B4: Debate Settler — does cooperation inevitably emerge?
//!
//! Run many seeds, count how many develop cooperation, complexity, and altruism.
//! Statistical answer to fundamental evolutionary questions.

use crate::use_cases::evolve_with;
use crate::use_cases::presets::UniversePreset;

/// Result of a cooperation emergence experiment.
#[derive(Debug)]
pub struct DebateReport {
    pub preset_name:        &'static str,
    pub n_seeds:            usize,
    pub generations:        u32,
    /// Fraction of seeds where species_count > 2 (life emerged).
    pub life_rate:          f32,
    /// Fraction of seeds where diversity increased over time (complexity grew).
    pub complexity_rate:    f32,
    /// Fraction of seeds where final diversity > initial (cooperation signal).
    pub cooperation_signal: f32,
    /// Per-seed final stats: (best_fitness, diversity, species_mean).
    pub seed_results:       Vec<(f32, f32, f32)>,
}

/// Run N seeds and aggregate evolutionary outcomes.
pub fn run(
    preset: &UniversePreset,
    n_seeds: usize,
    gens: u32,
    ticks: u32,
) -> DebateReport {
    let mut life_count = 0usize;
    let mut complexity_count = 0usize;
    let mut cooperation_count = 0usize;
    let mut seed_results = Vec::with_capacity(n_seeds);

    for i in 0..n_seeds {
        let report = evolve_with(preset, i as u64, 50, gens, ticks, 12);

        let first_div = report.history.first().map(|s| s.diversity).unwrap_or(0.0);
        let last = report.history.last();
        let final_species = last.map(|s| s.species_mean).unwrap_or(0.0);
        let final_diversity = last.map(|s| s.diversity).unwrap_or(0.0);
        let final_fitness = last.map(|s| s.best_fitness).unwrap_or(0.0);

        if final_species > 2.0 { life_count += 1; }
        if final_diversity > first_div { complexity_count += 1; }
        if final_diversity > first_div * 1.1 { cooperation_count += 1; }

        seed_results.push((final_fitness, final_diversity, final_species));
    }

    let n = n_seeds.max(1) as f32;
    DebateReport {
        preset_name: preset.name,
        n_seeds,
        generations: gens,
        life_rate: life_count as f32 / n,
        complexity_rate: complexity_count as f32 / n,
        cooperation_signal: cooperation_count as f32 / n,
        seed_results,
    }
}
