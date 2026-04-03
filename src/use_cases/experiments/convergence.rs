//! D2: Convergent Evolution Detector — do different seeds find the same solution?

use crate::batch::genome::GenomeBlob;
use crate::use_cases::presets::UniversePreset;
use crate::use_cases::{ExperimentReport, evolve_with};

/// Result of convergence analysis.
#[derive(Debug)]
pub struct ConvergenceReport {
    pub n_seeds: usize,
    pub mean_distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    /// Fraction of seed-pairs with distance < threshold (converged).
    pub convergence_rate: f32,
    pub top_genomes: Vec<GenomeBlob>,
}

/// Run N seeds with same preset, compare final top genomes.
///
/// If most seeds converge to similar genomes → the solution is an attractor.
/// If they diverge → the fitness landscape has multiple optima.
pub fn run(
    preset: &UniversePreset,
    n_seeds: usize,
    gens: u32,
    ticks: u32,
    convergence_threshold: f32,
) -> ConvergenceReport {
    let reports: Vec<ExperimentReport> = (0..n_seeds)
        .map(|i| evolve_with(preset, i as u64, 100, gens, ticks, 12))
        .collect();

    // Extract top genome from each seed
    let tops: Vec<GenomeBlob> = reports
        .iter()
        .filter_map(|r| r.top_genomes.first().copied())
        .collect();

    // Pairwise distance matrix
    let mut distances = Vec::new();
    for i in 0..tops.len() {
        for j in (i + 1)..tops.len() {
            distances.push(tops[i].distance(&tops[j]));
        }
    }

    let mean = if distances.is_empty() {
        0.0
    } else {
        distances.iter().sum::<f32>() / distances.len() as f32
    };
    let min = distances.iter().copied().fold(f32::MAX, f32::min);
    let max = distances.iter().copied().fold(0.0f32, f32::max);
    let converged = distances
        .iter()
        .filter(|&&d| d < convergence_threshold)
        .count();
    let convergence_rate = if distances.is_empty() {
        0.0
    } else {
        converged as f32 / distances.len() as f32
    };

    ConvergenceReport {
        n_seeds,
        mean_distance: mean,
        min_distance: if min == f32::MAX { 0.0 } else { min },
        max_distance: max,
        convergence_rate,
        top_genomes: tops,
    }
}
