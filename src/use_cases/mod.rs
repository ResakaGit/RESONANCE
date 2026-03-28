//! Use case layer — HOFs + contracts that compose the batch engine.
//!
//! Zero Bevy. Zero batch internals exposed. Pure functions in, data out.
//! Binaries are thin wrappers: parse args → call HOF → pass to presenter.

pub mod cli;
pub mod experiments;
pub mod presets;
pub mod presenters;

use crate::batch::genome::GenomeBlob;
use crate::batch::harness::GenerationStats;

/// Result of any evolutionary experiment. Owned, serializable, Bevy-free.
#[derive(Debug, Clone)]
pub struct ExperimentReport {
    pub preset_name:   &'static str,
    pub seed:          u64,
    pub generations:   u32,
    pub ticks_per_gen: u32,
    pub world_count:   usize,
    pub top_genomes:   Vec<GenomeBlob>,
    pub history:       Vec<GenerationStats>,
    pub wall_time_ms:  u64,
}

/// The single HOF that touches the engine. Everything else composes this.
///
/// Deterministic: same preset + seed → identical report (INV-UC1).
pub fn evolve_with(
    preset: &presets::UniversePreset,
    seed: u64,
    worlds: usize,
    generations: u32,
    ticks: u32,
    entities: u8,
) -> ExperimentReport {
    use crate::batch::batch::BatchConfig;
    use crate::batch::harness::GeneticHarness;
    use std::time::Instant;

    let config = BatchConfig {
        world_count:      worlds,
        ticks_per_eval:   ticks,
        tick_rate_hz:     20.0,
        mutation_sigma:   0.0, // self-adaptive
        elite_fraction:   0.03,
        crossover_rate:   0.30,
        max_generations:  generations,
        seed,
        initial_entities: entities,
        fitness_weights:  [1.0, 1.0, 4.0, 3.0, 1.0, 1.0],
        tournament_k:     2,
    };

    // Apply preset constants to batch module
    preset.apply();

    let start = Instant::now();
    let mut harness = GeneticHarness::new(config);
    let genomes = harness.run();

    ExperimentReport {
        preset_name: preset.name,
        seed,
        generations,
        ticks_per_gen: ticks,
        world_count: worlds,
        top_genomes: genomes,
        history: harness.history,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

/// Compare multiple universe presets. Each preset runs `seeds_per` independent experiments.
pub fn compare_universes(
    presets: &[presets::UniversePreset],
    seeds_per: usize,
    generations: u32,
    ticks: u32,
) -> Vec<(&'static str, Vec<ExperimentReport>)> {
    presets.iter().map(|preset| {
        let reports: Vec<ExperimentReport> = (0..seeds_per)
            .map(|i| evolve_with(preset, i as u64, 100, generations, ticks, 12))
            .collect();
        (preset.name, reports)
    }).collect()
}
