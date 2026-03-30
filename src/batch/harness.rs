//! GeneticHarness — evolutionary loop: evaluate → select → reproduce → repeat.

use crate::batch::arena::SimWorldFlat;
use crate::batch::batch::{BatchConfig, WorldBatch};
use crate::batch::constants::MAX_ENTITIES;
use crate::batch::genome::GenomeBlob;
use crate::blueprint::equations::{batch_fitness, determinism, variable_genome};

// ─── FitnessReport ──────────────────────────────────────────────────────────

/// Fitness evaluation of a single world after N ticks.
#[derive(Debug, Clone)]
pub struct FitnessReport {
    pub world_index:       usize,
    pub survivors:         u8,
    pub total_qe:          f32,
    pub reproductions:     u16,
    pub max_trophic_level: u8,
    pub species_count:     u8,
    pub composite_fitness: f32,
}

impl FitnessReport {
    /// Evaluate a world's fitness using weighted composite score.
    ///
    /// Axiom 6: includes intra-world genome diversity — measures how different
    /// survivors are from each other. Worlds with diverse survivors score higher.
    pub fn compute(world: &SimWorldFlat, world_index: usize, weights: &[f32; 6]) -> Self {
        let survivors = world.alive_mask.count_ones() as u8;
        let reproductions = world.events.repro_len as u16;
        let species_count = count_frequency_bands(world);
        let max_trophic = max_trophic_chain(world);

        // Collect biases of all survivors for diversity measurement
        let mut biases_buf = [[0.0f32; 4]; 64];
        let mut bias_count = 0usize;
        let mut mask = world.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            biases_buf[bias_count] = [
                world.entities[i].growth_bias,
                world.entities[i].mobility_bias,
                world.entities[i].branching_bias,
                world.entities[i].resilience,
            ];
            bias_count += 1;
        }
        let diversity = batch_fitness::genome_diversity(&biases_buf[..bias_count]);

        // Diversity bonus: normalized to [0,1] (max euclidean in 4D = 2.0)
        let diversity_norm = (diversity / 2.0).min(1.0);

        // Genome complexity bonus: mean gene count normalized to [0,1] (max = 32).
        // Axiom 6: worlds where genomes grew are more complex → higher fitness.
        let complexity_mean = if bias_count > 0 {
            let total_genes: f32 = {
                let mut sum = 0.0f32;
                let mut m = world.alive_mask;
                while m != 0 {
                    let idx = m.trailing_zeros() as usize;
                    m &= m - 1;
                    sum += world.genomes[idx].gene_count() as f32;
                }
                sum
            };
            (total_genes / bias_count as f32) / 32.0 // normalized [0,1]
        } else { 0.0 };

        let mut composite = batch_fitness::composite_fitness(
            survivors, reproductions, species_count,
            max_trophic, 0, 0, weights,
        );
        // Add diversity bonus (scaled by species weight — same axis of importance)
        composite += diversity_norm * weights[2];
        // Add complexity bonus (scaled moderately — reward but don't dominate)
        composite += complexity_mean * weights[3]; // uses trophic weight as proxy

        Self {
            world_index, survivors, total_qe: world.total_qe,
            reproductions, max_trophic_level: max_trophic,
            species_count, composite_fitness: composite,
        }
    }
}

fn count_frequency_bands(world: &SimWorldFlat) -> u8 {
    let mut bands = [false; 16];
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let band = (world.entities[i].frequency_hz / 100.0).min(15.0) as usize;
        bands[band] = true;
    }
    bands.iter().filter(|&&b| b).count() as u8
}

/// Count distinct trophic roles alive (0-5). Rewards coexistence.
fn max_trophic_chain(world: &SimWorldFlat) -> u8 {
    let mut roles = [false; 5];
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let tc = world.entities[i].trophic_class.min(4) as usize;
        roles[tc] = true;
    }
    roles.iter().filter(|&&r| r).count() as u8
}

// ─── GenerationStats ────────────────────────────────────────────────────────

/// Statistics for one generation — recorded in `GeneticHarness.history`.
#[derive(Debug, Clone)]
pub struct GenerationStats {
    pub generation:           u32,
    pub best_fitness:         f32,
    pub mean_fitness:         f32,
    pub worst_fitness:        f32,
    pub diversity:            f32,
    pub survivors_mean:       f32,
    pub species_mean:         f32,
    /// Mean gene count across all alive entities (VG observability).
    pub gene_count_mean:       f32,
    /// Fraction of entities with a functional MetabolicGraph (MGN observability).
    pub metabolic_graph_rate:  f32,
    /// Fraction of entities with a functional protein (PF observability).
    pub protein_function_rate: f32,
    /// Mean codon count (PD-5 observability).
    pub codon_count_mean:      f32,
    /// Fraction of entities in colonies ≥ 3 (MC-5 observability).
    pub multicellular_rate:    f32,
}

// ─── GeneticHarness ─────────────────────────────────────────────────────────

/// Evolutionary loop: evaluate → select → reproduce → repeat.
pub struct GeneticHarness {
    pub batch:   WorldBatch,
    pub config:  BatchConfig,
    pub history: Vec<GenerationStats>,
}

impl GeneticHarness {
    pub fn new(config: BatchConfig) -> Self {
        let batch = WorldBatch::new(config.clone());
        Self { batch, config: config.clone(), history: Vec::new() }
    }

    /// One generational step: evaluate → select elite → repopulate with mutation.
    pub fn step(&mut self) -> GenerationStats {
        // 1. Evaluate
        self.batch.run_evaluation(self.config.ticks_per_eval);

        // 2. Score all worlds
        let mut reports: Vec<FitnessReport> = self.batch.worlds.iter()
            .enumerate()
            .map(|(i, w)| FitnessReport::compute(w, i, &self.config.fitness_weights))
            .collect();

        // 3. Sort by fitness descending
        reports.sort_unstable_by(|a, b|
            b.composite_fitness.partial_cmp(&a.composite_fitness).unwrap_or(std::cmp::Ordering::Equal)
        );

        // 4. Stats
        let best = reports.first().map(|r| r.composite_fitness).unwrap_or(0.0);
        let worst = reports.last().map(|r| r.composite_fitness).unwrap_or(0.0);
        let mean = if reports.is_empty() { 0.0 } else {
            reports.iter().map(|r| r.composite_fitness).sum::<f32>() / reports.len() as f32
        };
        let survivors_mean = if reports.is_empty() { 0.0 } else {
            reports.iter().map(|r| r.survivors as f32).sum::<f32>() / reports.len() as f32
        };
        let species_mean = if reports.is_empty() { 0.0 } else {
            reports.iter().map(|r| r.species_count as f32).sum::<f32>() / reports.len() as f32
        };

        // 5. Select elite
        let elite_n = ((self.config.world_count as f32 * self.config.elite_fraction) as usize).max(1);
        let elite_indices: Vec<usize> = reports[..elite_n.min(reports.len())]
            .iter()
            .map(|r| r.world_index)
            .collect();

        // 6. Extract genomes from elite worlds (both GenomeBlob + VariableGenome)
        let elite_genomes: Vec<Vec<GenomeBlob>> = elite_indices.iter()
            .map(|&wi| self.extract_genomes(wi))
            .collect();
        let elite_vgenomes: Vec<Vec<variable_genome::VariableGenome>> = elite_indices.iter()
            .map(|&wi| self.extract_variable_genomes(wi))
            .collect();

        // 7. Compute diversity
        let diversity = compute_diversity(&elite_genomes);

        // 7b. Complexity observability
        let cx = compute_complexity_stats(&self.batch.worlds);

        // 8. Repopulate with variable genomes (gene duplication/deletion propagated)
        self.repopulate_variable(&elite_genomes, &elite_vgenomes);

        self.batch.generation += 1;
        let stats = GenerationStats {
            generation: self.batch.generation,
            best_fitness: best, mean_fitness: mean, worst_fitness: worst,
            diversity, survivors_mean, species_mean,
            gene_count_mean: cx.gene_count_mean,
            metabolic_graph_rate: cx.metabolic_graph_rate,
            protein_function_rate: cx.protein_function_rate,
            codon_count_mean: cx.codon_count_mean,
            multicellular_rate: cx.multicellular_rate,
        };
        self.history.push(stats.clone());
        stats
    }

    /// Run until max_generations, return top genomes.
    pub fn run(&mut self) -> Vec<GenomeBlob> {
        for _ in 0..self.config.max_generations {
            self.step();
        }
        self.top_genomes(10)
    }

    /// Extract the best N genomes from the current generation (post-evaluation).
    pub fn top_genomes(&self, n: usize) -> Vec<GenomeBlob> {
        let mut reports: Vec<FitnessReport> = self.batch.worlds.iter()
            .enumerate()
            .map(|(i, w)| FitnessReport::compute(w, i, &self.config.fitness_weights))
            .collect();
        reports.sort_unstable_by(|a, b|
            b.composite_fitness.partial_cmp(&a.composite_fitness).unwrap_or(std::cmp::Ordering::Equal)
        );
        reports.iter()
            .take(n)
            .flat_map(|r| self.extract_genomes(r.world_index))
            .take(n)
            .collect()
    }

    fn extract_genomes(&self, world_idx: usize) -> Vec<GenomeBlob> {
        let w = &self.batch.worlds[world_idx];
        let mut genomes = Vec::new();
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            genomes.push(GenomeBlob::from_slot(&w.entities[i]));
        }
        genomes
    }

    fn extract_variable_genomes(&self, world_idx: usize) -> Vec<variable_genome::VariableGenome> {
        let w = &self.batch.worlds[world_idx];
        let mut genomes = Vec::new();
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            genomes.push(w.genomes[i]);
        }
        genomes
    }

    fn repopulate_variable(
        &mut self,
        elite_genomes: &[Vec<GenomeBlob>],
        elite_vgenomes: &[Vec<variable_genome::VariableGenome>],
    ) {
        if elite_genomes.is_empty() { return; }
        let flat_elite: Vec<GenomeBlob> = elite_genomes.iter().flatten().copied().collect();
        let flat_vg: Vec<variable_genome::VariableGenome> = elite_vgenomes.iter().flatten().copied().collect();
        if flat_elite.is_empty() { return; }

        let dt = 1.0 / self.config.tick_rate_hz;
        let generation_id = self.batch.generation as u64;

        for (wi, world) in self.batch.worlds.iter_mut().enumerate() {
            let world_seed = determinism::next_u64(self.config.seed ^ generation_id ^ (wi as u64));
            *world = SimWorldFlat::new(world_seed, dt);

            for e in 0..self.config.initial_entities {
                let e_seed = determinism::next_u64(world_seed ^ (e as u64));

                // Select parent(s) via tournament
                let fitnesses: Vec<f32> = flat_elite.iter()
                    .map(|g| g.growth_bias + g.resilience)
                    .collect();
                let p1_idx = batch_fitness::tournament_select(&fitnesses, self.config.tournament_k, e_seed);
                let parent1 = flat_elite[p1_idx];

                // Variable genome: crossover or clone+mutate (with gene duplication/deletion)
                let parent_vg = if p1_idx < flat_vg.len() { flat_vg[p1_idx] } else {
                    variable_genome::from_genome_blob(parent1.growth_bias, parent1.mobility_bias, parent1.branching_bias, parent1.resilience, parent1.sigma)
                };
                let child_vg = if determinism::unit_f32(e_seed) < self.config.crossover_rate {
                    let p2_seed = determinism::next_u64(e_seed);
                    let p2_idx = batch_fitness::tournament_select(&fitnesses, 3, p2_seed);
                    let parent2_vg = if p2_idx < flat_vg.len() { flat_vg[p2_idx] } else { parent_vg };
                    let crossed = variable_genome::crossover_variable(&parent_vg, &parent2_vg, e_seed);
                    variable_genome::mutate_variable(&crossed, e_seed)
                } else {
                    variable_genome::mutate_variable(&parent_vg, e_seed)
                };

                // Derive GenomeBlob from VariableGenome (effective biases for EntitySlot)
                let (biases, sigma) = variable_genome::to_genome_blob_biases(&child_vg);
                let genome = GenomeBlob {
                    growth_bias: biases[0], mobility_bias: biases[1],
                    branching_bias: biases[2], resilience: biases[3],
                    sigma, ..parent1
                };

                let mut slot = crate::batch::arena::EntitySlot::default();
                genome.apply(&mut slot);
                slot.qe = 30.0;
                slot.radius = 0.5;
                slot.dissipation = 0.01;
                slot.frequency_hz = determinism::range_f32(e_seed, 100.0, 900.0);
                slot.engine_max = 20.0;
                slot.input_valve = 0.5;
                slot.output_valve = 0.5;
                let s1 = determinism::next_u64(e_seed);
                let s2 = determinism::next_u64(s1);
                slot.position = [
                    determinism::range_f32(s1, 1.0, 15.0),
                    determinism::range_f32(s2, 1.0, 15.0),
                ];
                if let Some(idx) = world.spawn(slot) {
                    world.genomes[idx] = child_vg;
                }
            }
            for cell in &mut world.nutrient_grid { *cell = 5.0; }
            world.update_total_qe();
        }
    }
}

fn compute_diversity(elite_genomes: &[Vec<GenomeBlob>]) -> f32 {
    let flat: Vec<GenomeBlob> = elite_genomes.iter().flatten().copied().collect();
    if flat.len() < 2 { return 0.0; }
    let mut sum = 0.0_f32;
    let mut count = 0u32;
    for i in 0..flat.len() {
        for j in (i + 1)..flat.len() {
            sum += flat[i].distance(&flat[j]);
            count += 1;
        }
    }
    if count > 0 { sum / count as f32 } else { 0.0 }
}

/// Complexity stats: genes, codons, graphs, proteins, colonies.
struct ComplexityStats {
    gene_count_mean: f32,
    metabolic_graph_rate: f32,
    protein_function_rate: f32,
    codon_count_mean: f32,
    multicellular_rate: f32,
}

fn compute_complexity_stats(worlds: &[SimWorldFlat]) -> ComplexityStats {
    use crate::blueprint::equations::{metabolic_genome, multicellular, protein_fold};

    let mut total_genes = 0u64;
    let mut total_codons = 0u64;
    let mut total_entities = 0u64;
    let mut with_graph = 0u64;
    let mut with_protein = 0u64;
    let mut in_colony = 0u64;

    for w in worlds {
        // Colony detection for this world (lightweight: just count, don't modulate)
        let mut adjacency = [[false; MAX_ENTITIES]; MAX_ENTITIES];
        let mut mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            // Check pairs by proximity (simplified: radius overlap)
            let mut mask2 = if i >= 127 { 0 } else { w.alive_mask & !((1u128 << (i + 1)) - 1) };
            while mask2 != 0 {
                let j = mask2.trailing_zeros() as usize;
                mask2 &= mask2 - 1;
                let dx = w.entities[i].position[0] - w.entities[j].position[0];
                let dy = w.entities[i].position[1] - w.entities[j].position[1];
                let dist = (dx * dx + dy * dy).sqrt();
                let contact = w.entities[i].radius + w.entities[j].radius;
                if dist < contact * 2.0 {
                    let aff = multicellular::adhesion_affinity(
                        w.entities[i].frequency_hz, w.entities[j].frequency_hz,
                        dist, w.entities[i].radius, w.entities[j].radius,
                    );
                    if multicellular::should_bond(aff) {
                        adjacency[i][j] = true; adjacency[j][i] = true;
                    }
                }
            }
        }
        let colonies = multicellular::detect_colonies(&adjacency, w.alive_mask);

        mask = w.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            total_entities += 1;
            total_genes += w.genomes[i].gene_count() as u64;
            total_codons += w.codon_genomes[i].codon_count() as u64;

            let expr = w.entities[i].expression_mask;
            if metabolic_genome::metabolic_graph_from_variable_genome(&w.genomes[i], &expr).is_ok() {
                with_graph += 1;
            }
            let phenotype = protein_fold::compute_protein_phenotype(&w.genomes[i], w.seed ^ i as u64);
            if phenotype.function.is_some() { with_protein += 1; }
            if colonies.colony_id[i] > 0 { in_colony += 1; }
        }
    }

    let n = total_entities.max(1) as f32;
    ComplexityStats {
        gene_count_mean: total_genes as f32 / n,
        metabolic_graph_rate: with_graph as f32 / n,
        protein_function_rate: with_protein as f32 / n,
        codon_count_mean: total_codons as f32 / n,
        multicellular_rate: in_colony as f32 / n,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> BatchConfig {
        BatchConfig {
            world_count: 20,
            ticks_per_eval: 50,
            initial_entities: 4,
            max_generations: 5,
            ..Default::default()
        }
    }

    // ── FitnessReport ───────────────────────────────────────────────────────

    #[test]
    fn fitness_empty_world_is_zero() {
        let w = SimWorldFlat::new(0, 0.05);
        let r = FitnessReport::compute(&w, 0, &[1.0; 6]);
        assert_eq!(r.composite_fitness, 0.0);
        assert_eq!(r.survivors, 0);
    }

    #[test]
    fn fitness_more_survivors_higher_score() {
        let config = BatchConfig { world_count: 2, initial_entities: 2, ..Default::default() };
        let batch = WorldBatch::new(config);
        let r0 = FitnessReport::compute(&batch.worlds[0], 0, &[1.0; 6]);
        let empty = FitnessReport::compute(&SimWorldFlat::new(0, 0.05), 1, &[1.0; 6]);
        assert!(r0.composite_fitness > empty.composite_fitness);
    }

    // ── GeneticHarness ──────────────────────────────────────────────────────

    #[test]
    fn harness_step_records_stats() {
        let config = small_config();
        let mut harness = GeneticHarness::new(config);
        let stats = harness.step();
        assert_eq!(stats.generation, 1);
        assert!(stats.best_fitness >= stats.mean_fitness);
        assert!(stats.mean_fitness >= stats.worst_fitness);
        assert_eq!(harness.history.len(), 1);
    }

    #[test]
    fn harness_step_resets_worlds() {
        let config = small_config();
        let mut harness = GeneticHarness::new(config);
        harness.step();
        // After repopulation, worlds should be reset (tick_id = 0)
        for w in &harness.batch.worlds {
            assert_eq!(w.tick_id, 0, "worlds should be freshly reset after step");
        }
    }

    #[test]
    fn harness_best_fitness_non_decreasing() {
        let config = BatchConfig {
            world_count: 30,
            ticks_per_eval: 100,
            initial_entities: 6,
            max_generations: 5,
            ..Default::default()
        };
        let mut harness = GeneticHarness::new(config);
        let mut prev_best = f32::NEG_INFINITY;
        for _ in 0..5 {
            let stats = harness.step();
            assert!(
                stats.best_fitness >= prev_best - 1.0,
                "best fitness should not regress significantly: {} < {}",
                stats.best_fitness, prev_best,
            );
            prev_best = stats.best_fitness;
        }
    }

    #[test]
    fn harness_top_genomes_returns_requested_count() {
        let config = small_config();
        let mut harness = GeneticHarness::new(config);
        harness.step();
        let top = harness.top_genomes(5);
        assert!(top.len() <= 5);
        assert!(!top.is_empty());
    }

    #[test]
    fn harness_run_completes() {
        let config = BatchConfig {
            world_count: 10,
            ticks_per_eval: 20,
            initial_entities: 3,
            max_generations: 3,
            ..Default::default()
        };
        let mut harness = GeneticHarness::new(config);
        let result = harness.run();
        assert!(!result.is_empty());
        assert_eq!(harness.history.len(), 3);
    }

    #[test]
    fn harness_deterministic() {
        let config = small_config();
        let mut h1 = GeneticHarness::new(config.clone());
        let mut h2 = GeneticHarness::new(config);
        let s1 = h1.step();
        let s2 = h2.step();
        assert_eq!(s1.best_fitness.to_bits(), s2.best_fitness.to_bits());
        assert_eq!(s1.mean_fitness.to_bits(), s2.mean_fitness.to_bits());
    }

    // ── WorldBatch integration ──────────────────────────────────────────────

    #[test]
    fn batch_worlds_survive_evaluation() {
        let config = BatchConfig {
            world_count: 5,
            ticks_per_eval: 100,
            initial_entities: 4,
            ..Default::default()
        };
        let mut batch = WorldBatch::new(config);
        batch.run_evaluation(100);
        // At least some worlds should have survivors
        let total_survivors: u32 = batch.worlds.iter()
            .map(|w| w.alive_mask.count_ones())
            .sum();
        assert!(total_survivors > 0, "some entities should survive 100 ticks");
    }

    #[test]
    fn diversity_decreases_or_is_finite() {
        let config = BatchConfig {
            world_count: 15,
            ticks_per_eval: 30,
            initial_entities: 4,
            max_generations: 3,
            ..Default::default()
        };
        let mut harness = GeneticHarness::new(config);
        for _ in 0..3 { harness.step(); }
        for stats in &harness.history {
            assert!(stats.diversity.is_finite(), "diversity must be finite");
        }
    }
}
