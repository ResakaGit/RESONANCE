//! B3: Cambrian Explosion — measure morphological innovation rate per generation.
//!
//! Tracks how many new peak configurations appear each generation.
//! An "explosion" = sudden spike in innovation rate.

use crate::use_cases::evolve_with;
use crate::use_cases::presets::UniversePreset;

/// Result of a Cambrian Explosion analysis.
#[derive(Debug)]
pub struct CambrianReport {
    pub preset_name:       &'static str,
    pub generations:       u32,
    /// Diversity per generation (from harness.history).
    pub diversity_curve:   Vec<f32>,
    /// Species count per generation.
    pub species_curve:     Vec<f32>,
    /// Generation where max diversity increase happened.
    pub explosion_gen:     Option<u32>,
    /// Max diversity delta between consecutive generations.
    pub max_diversity_delta: f32,
    /// Whether an "explosion" was detected (delta > threshold).
    pub explosion_detected: bool,
}

/// Run evolution and detect Cambrian-style morphological explosions.
///
/// An explosion = generation where `diversity[g] - diversity[g-1] > threshold`.
pub fn run(
    preset: &UniversePreset,
    seed: u64,
    worlds: usize,
    gens: u32,
    ticks: u32,
    explosion_threshold: f32,
) -> CambrianReport {
    let report = evolve_with(preset, seed, worlds, gens, ticks, 12);

    let diversity_curve: Vec<f32> = report.history.iter().map(|s| s.diversity).collect();
    let species_curve: Vec<f32> = report.history.iter().map(|s| s.species_mean).collect();

    let (explosion_gen, max_delta) = diversity_curve.windows(2)
        .enumerate()
        .map(|(i, w)| (i + 1, w[1] - w[0]))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, d)| (Some(i as u32), d))
        .unwrap_or((None, 0.0));

    CambrianReport {
        preset_name: preset.name,
        generations: gens,
        diversity_curve,
        species_curve,
        explosion_gen,
        max_diversity_delta: max_delta,
        explosion_detected: max_delta > explosion_threshold,
    }
}
