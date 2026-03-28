//! A1: Versus Arena — two evolved ecosystems compete.

use crate::batch::bridge;
use crate::batch::genome::GenomeBlob;
use std::path::Path;

/// Result of a versus match.
#[derive(Debug)]
pub struct VersusResult {
    pub winner:      &'static str, // "A", "B", or "draw"
    pub qe_a:        f32,
    pub qe_b:        f32,
    pub survivors_a: u8,
    pub survivors_b: u8,
    pub genomes_a:   Vec<GenomeBlob>,
    pub genomes_b:   Vec<GenomeBlob>,
}

/// Load two genome files and report their stats.
///
/// Full simulation versus requires Bevy (spawn in arena, tick, measure).
/// This function provides the genome comparison layer.
pub fn load_competitors(path_a: &Path, path_b: &Path) -> Option<(Vec<GenomeBlob>, Vec<GenomeBlob>)> {
    let a = bridge::load_genomes(path_a).ok()?;
    let b = bridge::load_genomes(path_b).ok()?;
    Some((a, b))
}

/// Compare genome fitness potential between two teams.
///
/// Quick heuristic: sum of biases as proxy for evolved quality.
/// Full versus requires spawning in a shared world (Bevy layer).
pub fn compare_potential(genomes_a: &[GenomeBlob], genomes_b: &[GenomeBlob]) -> VersusResult {
    let potential = |g: &GenomeBlob| g.growth_bias + g.resilience + g.mobility_bias * 0.5;
    let score_a: f32 = genomes_a.iter().map(potential).sum();
    let score_b: f32 = genomes_b.iter().map(potential).sum();
    let winner = if (score_a - score_b).abs() < 0.1 { "draw" }
                 else if score_a > score_b { "A" }
                 else { "B" };
    VersusResult {
        winner,
        qe_a: score_a,
        qe_b: score_b,
        survivors_a: genomes_a.len() as u8,
        survivors_b: genomes_b.len() as u8,
        genomes_a: genomes_a.to_vec(),
        genomes_b: genomes_b.to_vec(),
    }
}
