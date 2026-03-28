//! B2: Allopatric Speciation — does reproductive isolation emerge without programming it?
//!
//! Two populations evolve isolated (different seeds on same preset).
//! After N generations, measure frequency divergence via `interference()`.

use crate::use_cases::evolve_with;
use crate::use_cases::presets::UniversePreset;
use crate::batch::bridge;
use crate::batch::genome::GenomeBlob;
use crate::blueprint::equations;

/// Result of a speciation experiment.
#[derive(Debug)]
pub struct SpeciationReport {
    pub preset_name:       &'static str,
    pub generations:       u32,
    pub pop_a_genomes:     Vec<GenomeBlob>,
    pub pop_b_genomes:     Vec<GenomeBlob>,
    /// Mean frequency of population A's top genomes.
    pub mean_freq_a:       f32,
    /// Mean frequency of population B's top genomes.
    pub mean_freq_b:       f32,
    /// Interference between mean frequencies (1.0 = identical, 0.0 = orthogonal, -1.0 = destructive).
    pub cross_interference: f32,
    /// Whether populations diverged enough to be "reproductively isolated".
    pub speciated:          bool,
}

/// Run two isolated populations and measure divergence.
///
/// "Isolation" = different seeds (no gene flow). Same physical laws.
/// Speciation detected when cross-population interference < threshold.
pub fn run(
    preset: &UniversePreset,
    seed_a: u64,
    seed_b: u64,
    gens: u32,
    ticks: u32,
    speciation_threshold: f32,
) -> SpeciationReport {
    let report_a = evolve_with(preset, seed_a, 100, gens, ticks, 12);
    let report_b = evolve_with(preset, seed_b, 100, gens, ticks, 12);

    // Use bridge's canonical frequency mapping (single source of truth).
    let mean_freq = |genomes: &[GenomeBlob]| -> f32 {
        if genomes.is_empty() { return 0.0; }
        let sum: f32 = genomes.iter()
            .map(|g| bridge::genome_to_components(g).2.frequency_hz())
            .sum();
        sum / genomes.len() as f32
    };

    let freq_a = mean_freq(&report_a.top_genomes);
    let freq_b = mean_freq(&report_b.top_genomes);

    let interference = equations::freq_interference(freq_a, freq_b);

    SpeciationReport {
        preset_name: preset.name,
        generations: gens,
        pop_a_genomes: report_a.top_genomes,
        pop_b_genomes: report_b.top_genomes,
        mean_freq_a: freq_a,
        mean_freq_b: freq_b,
        cross_interference: interference,
        speciated: interference < speciation_threshold,
    }
}

