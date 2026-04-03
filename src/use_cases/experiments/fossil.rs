//! C1: Fossil Record — genome snapshots per generation for timeline visualization.
//!
//! Evolves and captures the top genome at each generation, enabling
//! morphological replay via mesh interpolation.

use crate::batch::batch::BatchConfig;
use crate::batch::genome::GenomeBlob;
use crate::batch::harness::{GenerationStats, GeneticHarness};
use crate::use_cases::presets::UniversePreset;
use std::time::Instant;

/// One fossil: genome + stats at a generation.
#[derive(Debug, Clone)]
pub struct FossilSnapshot {
    pub generation: u32,
    pub genome: GenomeBlob,
    pub fitness: f32,
    pub diversity: f32,
    pub species: f32,
}

/// Complete fossil record: one snapshot per generation.
#[derive(Debug)]
pub struct FossilRecord {
    pub preset_name: &'static str,
    pub seed: u64,
    pub fossils: Vec<FossilSnapshot>,
    pub history: Vec<GenerationStats>,
    pub wall_time_ms: u64,
}

/// Evolve and capture top genome at every generation.
///
/// Unlike `evolve_with` which only returns final genomes, this preserves
/// the complete morphological timeline for slider-based replay.
pub fn run(
    preset: &UniversePreset,
    seed: u64,
    worlds: usize,
    gens: u32,
    ticks: u32,
) -> FossilRecord {
    preset.apply();

    let config = BatchConfig {
        world_count: worlds,
        ticks_per_eval: ticks,
        tick_rate_hz: 20.0,
        mutation_sigma: 0.0,
        elite_fraction: 0.03,
        crossover_rate: 0.30,
        max_generations: gens,
        seed,
        initial_entities: 12,
        fitness_weights: [1.0, 1.0, 4.0, 3.0, 1.0, 1.0],
        tournament_k: 2,
    };

    let start = Instant::now();
    let mut harness = GeneticHarness::new(config);
    let mut fossils = Vec::with_capacity(gens as usize);

    for _ in 0..gens {
        let stats = harness.step();
        let top = harness.top_genomes(1);
        if let Some(genome) = top.into_iter().next() {
            fossils.push(FossilSnapshot {
                generation: stats.generation,
                genome,
                fitness: stats.best_fitness,
                diversity: stats.diversity,
                species: stats.species_mean,
            });
        }
    }

    FossilRecord {
        preset_name: preset.name,
        seed,
        fossils,
        history: harness.history,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}
