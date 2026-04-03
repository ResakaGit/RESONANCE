//! WorldBatch — N worlds for parallel evaluation.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::genome::GenomeBlob;
use crate::batch::scratch::ScratchPad;
use crate::blueprint::equations::determinism;

/// Configuration for a batch evolutionary experiment.
#[derive(Clone, Debug)]
pub struct BatchConfig {
    pub world_count: usize,
    pub ticks_per_eval: u32,
    pub tick_rate_hz: f32,
    pub mutation_sigma: f32,
    pub elite_fraction: f32,
    pub crossover_rate: f32,
    pub max_generations: u32,
    pub seed: u64,
    pub initial_entities: u8,
    pub fitness_weights: [f32; 6],
    /// Tournament selection size (Blickle 1996). k=2 for diversity.
    pub tournament_k: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            world_count: 100,
            ticks_per_eval: 500,
            tick_rate_hz: 20.0,
            mutation_sigma: 0.0,  // 0 = self-adaptive (Schwefel 1981)
            elite_fraction: 0.03, // 3% elite (Eiben & Smith 2015)
            crossover_rate: 0.30,
            max_generations: 100,
            seed: 42,
            initial_entities: 8,
            // weights: survivors, reproductions, species, trophic_depth, memes, coalitions
            fitness_weights: [1.0, 1.0, 4.0, 3.0, 1.0, 1.0],
            tournament_k: 2,
        }
    }
}

/// Batch of N worlds for evaluation and selection.
pub struct WorldBatch {
    pub worlds: Vec<SimWorldFlat>,
    pub generation: u32,
    pub config: BatchConfig,
}

impl WorldBatch {
    /// Create N worlds, each seeded with varied genomes.
    pub fn new(config: BatchConfig) -> Self {
        let dt = 1.0 / config.tick_rate_hz;
        let worlds: Vec<SimWorldFlat> = (0..config.world_count)
            .map(|i| {
                let world_seed = determinism::next_u64(config.seed ^ (i as u64));
                let mut w = SimWorldFlat::new(world_seed, dt);
                // Populate with initial entities using varied genomes
                for e in 0..config.initial_entities {
                    let genome_seed = determinism::next_u64(world_seed ^ (e as u64));
                    let genome = GenomeBlob::random(genome_seed);
                    let mut slot = EntitySlot::default();
                    genome.apply(&mut slot);
                    slot.qe = 30.0;
                    slot.radius = 0.5;
                    slot.dissipation = 0.01;
                    slot.frequency_hz = determinism::range_f32(genome_seed, 100.0, 900.0);
                    slot.engine_max = 20.0;
                    slot.input_valve = 0.5;
                    slot.output_valve = 0.5;
                    let s1 = determinism::next_u64(genome_seed);
                    let s2 = determinism::next_u64(s1);
                    slot.position = [
                        determinism::range_f32(s1, 1.0, 15.0),
                        determinism::range_f32(s2, 1.0, 15.0),
                    ];
                    w.spawn(slot);
                }
                // Seed nutrient grid
                for cell in &mut w.nutrient_grid {
                    *cell = 5.0;
                }
                w.update_total_qe();
                w
            })
            .collect();

        Self {
            worlds,
            generation: 0,
            config,
        }
    }

    /// Advance all worlds by one tick. Uses rayon for data-parallel execution.
    ///
    /// Each world is independent (INV-B4). ScratchPad is thread-local (zero contention).
    pub fn tick_all(&mut self) {
        use rayon::prelude::*;
        self.worlds.par_iter_mut().for_each(|world| {
            THREAD_SCRATCH.with(|cell| {
                let mut scratch = cell.borrow_mut();
                world.tick(&mut scratch);
            });
        });
    }

    /// Advance all worlds by one tick, single-threaded (for determinism comparison).
    pub fn tick_all_sequential(&mut self) {
        let mut scratch = ScratchPad::new();
        for world in &mut self.worlds {
            world.tick(&mut scratch);
        }
    }

    /// Run `ticks` ticks across all worlds (parallel, analytical stepping).
    pub fn run_evaluation(&mut self, ticks: u32) {
        use rayon::prelude::*;
        // Use tick_fast for analytical acceleration
        self.worlds.par_iter_mut().for_each(|world| {
            THREAD_SCRATCH.with(|cell| {
                let mut scratch = cell.borrow_mut();
                world.tick_fast(&mut scratch, ticks);
            });
        });
    }
}

use std::cell::RefCell;

// DEBT: thread_local for rayon zero-contention parallelism. Each thread owns its
// ScratchPad — no shared mutable state. Required because rayon par_iter_mut needs
// per-thread scratch without Arc<Mutex>. Batch-only; never used in Bevy runtime.
thread_local! {
    static THREAD_SCRATCH: RefCell<ScratchPad> = RefCell::new(ScratchPad::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::constants::MAX_ENTITIES;

    #[test]
    fn new_batch_creates_n_worlds() {
        let config = BatchConfig {
            world_count: 10,
            ..Default::default()
        };
        let batch = WorldBatch::new(config);
        assert_eq!(batch.worlds.len(), 10);
    }

    #[test]
    fn each_world_has_initial_entities() {
        let config = BatchConfig {
            world_count: 5,
            initial_entities: 4,
            ..Default::default()
        };
        let batch = WorldBatch::new(config);
        for w in &batch.worlds {
            assert_eq!(w.entity_count, 4);
        }
    }

    #[test]
    fn worlds_have_different_seeds() {
        let config = BatchConfig {
            world_count: 3,
            ..Default::default()
        };
        let batch = WorldBatch::new(config);
        assert_ne!(batch.worlds[0].seed, batch.worlds[1].seed);
        assert_ne!(batch.worlds[1].seed, batch.worlds[2].seed);
    }

    #[test]
    fn tick_all_advances_all_worlds() {
        let config = BatchConfig {
            world_count: 5,
            initial_entities: 2,
            ..Default::default()
        };
        let mut batch = WorldBatch::new(config);
        batch.tick_all();
        for w in &batch.worlds {
            assert_eq!(w.tick_id, 1);
        }
    }

    #[test]
    fn run_evaluation_advances_n_ticks() {
        let config = BatchConfig {
            world_count: 3,
            initial_entities: 2,
            ticks_per_eval: 10,
            ..Default::default()
        };
        let mut batch = WorldBatch::new(config);
        batch.run_evaluation(10);
        for w in &batch.worlds {
            assert_eq!(w.tick_id, 10);
        }
    }

    // ── BS-6: Parallel correctness ──────────────────────────────────────────

    #[test]
    fn parallel_matches_sequential_tick_ids() {
        let config = BatchConfig {
            world_count: 20,
            initial_entities: 4,
            ..Default::default()
        };
        let mut par = WorldBatch::new(config.clone());
        let mut seq = WorldBatch::new(config);
        par.tick_all();
        seq.tick_all_sequential();
        for (p, s) in par.worlds.iter().zip(seq.worlds.iter()) {
            assert_eq!(p.tick_id, s.tick_id);
        }
    }

    #[test]
    fn parallel_matches_sequential_energy() {
        let config = BatchConfig {
            world_count: 20,
            initial_entities: 4,
            ..Default::default()
        };
        let mut par = WorldBatch::new(config.clone());
        let mut seq = WorldBatch::new(config);
        for _ in 0..10 {
            par.tick_all();
            seq.tick_all_sequential();
        }
        for (i, (p, s)) in par.worlds.iter().zip(seq.worlds.iter()).enumerate() {
            assert_eq!(
                p.total_qe.to_bits(),
                s.total_qe.to_bits(),
                "world {i}: parallel={} sequential={}",
                p.total_qe,
                s.total_qe,
            );
        }
    }

    #[test]
    fn parallel_matches_sequential_alive_mask() {
        let config = BatchConfig {
            world_count: 10,
            initial_entities: 6,
            ..Default::default()
        };
        let mut par = WorldBatch::new(config.clone());
        let mut seq = WorldBatch::new(config);
        for _ in 0..20 {
            par.tick_all();
            seq.tick_all_sequential();
        }
        for (i, (p, s)) in par.worlds.iter().zip(seq.worlds.iter()).enumerate() {
            assert_eq!(
                p.alive_mask, s.alive_mask,
                "world {i}: parallel mask={:#b} sequential mask={:#b}",
                p.alive_mask, s.alive_mask,
            );
        }
    }

    #[test]
    fn parallel_matches_sequential_entity_qe() {
        let config = BatchConfig {
            world_count: 10,
            initial_entities: 4,
            ..Default::default()
        };
        let mut par = WorldBatch::new(config.clone());
        let mut seq = WorldBatch::new(config);
        for _ in 0..10 {
            par.tick_all();
            seq.tick_all_sequential();
        }
        for (wi, (pw, sw)) in par.worlds.iter().zip(seq.worlds.iter()).enumerate() {
            for ei in 0..MAX_ENTITIES {
                assert_eq!(
                    pw.entities[ei].qe.to_bits(),
                    sw.entities[ei].qe.to_bits(),
                    "world {wi} entity {ei}: parallel qe={} sequential qe={}",
                    pw.entities[ei].qe,
                    sw.entities[ei].qe,
                );
            }
        }
    }

    #[test]
    fn parallel_tick_all_scales_to_1000_worlds() {
        let config = BatchConfig {
            world_count: 1000,
            initial_entities: 4,
            ..Default::default()
        };
        let mut batch = WorldBatch::new(config);
        batch.tick_all(); // should not panic
        for w in &batch.worlds {
            assert_eq!(w.tick_id, 1);
        }
    }
}
