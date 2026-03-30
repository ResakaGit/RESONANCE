//! Use case layer — HOFs + contracts that compose the batch engine.
//!
//! Zero Bevy. Zero batch internals exposed. Pure functions in, data out.
//! Binaries are thin wrappers: parse args → call HOF → pass to presenter.

pub mod cli;
pub mod experiments;
pub mod export;
pub mod orchestrators;
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

/// Ejecuta un experimento con config completa. Single source of truth.
/// Runs an experiment with full config. Single source of truth.
///
/// Deterministic: same config + preset → identical report (INV-UC1).
pub fn evolve_with_config(
    config: &crate::batch::batch::BatchConfig,
    preset: &presets::UniversePreset,
) -> ExperimentReport {
    use crate::batch::harness::GeneticHarness;
    use std::time::Instant;

    preset.apply();
    let start = Instant::now();
    let mut harness = GeneticHarness::new(config.clone());
    let genomes = harness.run();

    ExperimentReport {
        preset_name: preset.name,
        seed: config.seed,
        generations: config.max_generations,
        ticks_per_gen: config.ticks_per_eval,
        world_count: config.world_count,
        top_genomes: genomes,
        history: harness.history,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

/// Convenience wrapper con parámetros principales. Defaults razonables para el resto.
/// Convenience wrapper with main parameters. Reasonable defaults for the rest.
///
/// Para control total, usar `evolve_with_config`.
pub fn evolve_with(
    preset: &presets::UniversePreset,
    seed: u64,
    worlds: usize,
    generations: u32,
    ticks: u32,
    entities: u8,
) -> ExperimentReport {
    use crate::batch::batch::BatchConfig;

    let config = BatchConfig {
        world_count: worlds,
        ticks_per_eval: ticks,
        max_generations: generations,
        seed,
        initial_entities: entities,
        ..Default::default()
    };
    evolve_with_config(&config, preset)
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
