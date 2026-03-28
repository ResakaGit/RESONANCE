//! Pure fitness equations for the batch genetic harness.
//!
//! All functions are stateless. No Bevy dependency.

use super::determinism;

/// Composite fitness — weighted sum of normalized metrics.
///
/// Each input normalized to [0,1] internally. `weights` scale per-metric.
/// `fitness = Σ (normalized_i × weight_i)`.
pub fn composite_fitness(
    survivors: u8,
    reproductions: u16,
    species: u8,
    trophic: u8,
    memes: u8,
    coalitions: u8,
    weights: &[f32; 6],
) -> f32 {
    let normalized = [
        survivors as f32 / 64.0,
        (reproductions as f32).min(100.0) / 100.0,
        species as f32 / 16.0,
        trophic as f32 / 5.0,
        memes as f32 / 16.0,
        coalitions as f32 / 8.0,
    ];
    normalized.iter().zip(weights.iter()).map(|(v, w)| v * w).sum()
}

/// Intra-world genome diversity: mean pairwise distance among biases.
///
/// Axiom 6 compliant: measures a natural outcome (how different survivors are),
/// does not prescribe what they should be.
/// Returns [0, 2] — 0 = monoculture, 2 = max diversity.
pub fn genome_diversity(biases: &[[f32; 4]]) -> f32 {
    if biases.len() < 2 { return 0.0; }
    let mut sum = 0.0_f32;
    let mut count = 0u32;
    for i in 0..biases.len() {
        for j in (i + 1)..biases.len() {
            let d = euclidean_4d(&biases[i], &biases[j]);
            sum += d;
            count += 1;
        }
    }
    if count > 0 { sum / count as f32 } else { 0.0 }
}

/// Euclidean distance in 4D bias space.
fn euclidean_4d(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let mut sq = 0.0;
    for k in 0..4 { let d = a[k] - b[k]; sq += d * d; }
    sq.sqrt()
}

/// Tournament selection: sample `k` random indices, return the one with best fitness.
pub fn tournament_select(fitnesses: &[f32], k: usize, rng_state: u64) -> usize {
    if fitnesses.is_empty() { return 0; }
    let k = k.max(1).min(fitnesses.len());
    let mut best_idx = 0;
    let mut best_val = f32::NEG_INFINITY;
    let mut s = rng_state;
    for _ in 0..k {
        s = determinism::next_u64(s);
        let idx = (determinism::unit_f32(s) * fitnesses.len() as f32) as usize;
        let idx = idx.min(fitnesses.len() - 1);
        if fitnesses[idx] > best_val {
            best_val = fitnesses[idx];
            best_idx = idx;
        }
    }
    best_idx
}

/// Uniform crossover on 4 float biases. Each gene: 50% from `a`, 50% from `b`.
pub fn crossover_uniform(a: &[f32; 4], b: &[f32; 4], rng_state: u64) -> [f32; 4] {
    let mut result = [0.0; 4];
    let mut s = rng_state;
    for i in 0..4 {
        s = determinism::next_u64(s);
        result[i] = if determinism::unit_f32(s) < 0.5 { a[i] } else { b[i] };
    }
    result
}

// ─── Self-adaptive mutation (Schwefel 1981) ─────────────────────────────────

/// Tau parameter for self-adaptive sigma: `1 / sqrt(2 × d)` where d=4 biases.
/// Schwefel (1981): controls the mutation rate of the mutation rate itself.
pub const SELF_ADAPTIVE_TAU: f32 = 0.354; // 1/sqrt(8)

/// Mutate sigma first, then biases. Schwefel self-adaptation.
///
/// `sigma' = sigma × exp(tau × N(0,1))`
/// `bias'  = bias + sigma' × N(0,1)`
/// Sigma is clamped to [min, max] to prevent collapse or explosion.
pub fn self_adaptive_mutate(
    biases: &[f32; 4],
    sigma: f32,
    rng_state: u64,
    sigma_min: f32,
    sigma_max: f32,
) -> ([f32; 4], f32) {
    let mut s = rng_state;
    // Mutate sigma first (log-normal)
    s = determinism::next_u64(s);
    let new_sigma = (sigma * (SELF_ADAPTIVE_TAU * determinism::gaussian_f32(s, 1.0)).exp())
        .clamp(sigma_min, sigma_max);
    // Mutate biases with new sigma
    let mut out = *biases;
    for bias in &mut out {
        s = determinism::next_u64(s);
        *bias = (*bias + determinism::gaussian_f32(s, new_sigma)).clamp(0.0, 1.0);
    }
    (out, new_sigma)
}

// ─── Genome → Geometry Influence mapping ────────────────────────────────────

/// Branching plan derived from genome biases.
///
/// `branch_count`: number of sub-branches (from branching_bias).
/// `branch_angles`: azimuthal angle per branch (evenly distributed).
/// `branch_length_fraction`: fraction of trunk length per branch.
/// `branch_radius_fraction`: fraction of trunk radius per branch.
#[derive(Debug, Clone)]
pub struct BranchPlan {
    pub count:           usize,
    pub attach_fractions: [f32; 8],
    pub angles:          [f32; 8],
    pub length_fraction: f32,
    pub radius_fraction: f32,
    pub flexibility:     f32,
}

/// Derive trunk geometry parameters from genome biases.
///
/// Pure function. No Bevy, no ECS, no side effects.
/// Returns `(length, radius, tilt, resistance, detail, segments)`.
pub fn trunk_params_from_genome(
    growth_bias: f32,
    mobility_bias: f32,
    branching_bias: f32,
    resilience: f32,
) -> (f32, f32, f32, f32, f32, u32) {
    let length    = 0.5 + growth_bias * 3.5;
    let radius    = 0.15 + (1.0 - growth_bias) * 0.6;
    let tilt      = mobility_bias * 1.2;
    let resistance = 0.1 + resilience * 0.9;
    let detail    = 0.4 + branching_bias * 0.6;
    let segments  = 6 + (branching_bias * 18.0) as u32;
    (length, radius, tilt, resistance, detail, segments)
}

/// Derive branch plan from genome biases.
///
/// Pure function. `branch_count = floor(branching_bias × 5)`.
/// Branches are evenly distributed azimuthally.
pub fn branch_plan_from_genome(
    branching_bias: f32,
    _growth_bias: f32,
    resilience: f32,
) -> BranchPlan {
    let count = (branching_bias * 5.0) as usize;
    let mut attach_fractions = [0.0f32; 8];
    let mut angles = [0.0f32; 8];
    for b in 0..count.min(8) {
        attach_fractions[b] = (b as f32 + 1.0) / (count as f32 + 1.0);
        angles[b] = (b as f32 / count.max(1) as f32) * std::f32::consts::TAU;
    }
    BranchPlan {
        count: count.min(8),
        attach_fractions,
        angles,
        length_fraction: 0.3 * (1.0 - resilience * 0.5),
        radius_fraction: 0.4,
        flexibility: 1.0 - resilience * 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_fitness_zero_inputs_zero_output() {
        assert_eq!(composite_fitness(0, 0, 0, 0, 0, 0, &[1.0; 6]), 0.0);
    }

    #[test]
    fn composite_fitness_max_inputs() {
        let f = composite_fitness(64, 100, 16, 5, 16, 8, &[1.0; 6]);
        assert!((f - 6.0).abs() < 1e-3, "max fitness should be ~6.0, got {f}");
    }

    #[test]
    fn composite_fitness_weights_scale() {
        let a = composite_fitness(32, 0, 0, 0, 0, 0, &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let b = composite_fitness(32, 0, 0, 0, 0, 0, &[2.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!((b - 2.0 * a).abs() < 1e-5);
    }

    #[test]
    fn tournament_select_returns_valid_index() {
        let fitnesses = [0.1, 0.5, 0.9, 0.3];
        let idx = tournament_select(&fitnesses, 3, 42);
        assert!(idx < fitnesses.len());
    }

    #[test]
    fn tournament_select_deterministic() {
        let fitnesses = [0.1, 0.5, 0.9, 0.3, 0.7];
        assert_eq!(tournament_select(&fitnesses, 3, 42), tournament_select(&fitnesses, 3, 42));
    }

    #[test]
    fn tournament_select_tends_toward_best() {
        let fitnesses = [0.0, 0.0, 0.0, 0.0, 1.0];
        let mut best_count = 0;
        for seed in 0..100 {
            if tournament_select(&fitnesses, 3, seed * 7 + 13) == 4 { best_count += 1; }
        }
        assert!(best_count > 20, "should frequently select best: {best_count}/100");
    }

    #[test]
    fn crossover_uniform_deterministic() {
        let a = [0.1, 0.2, 0.3, 0.4];
        let b = [0.9, 0.8, 0.7, 0.6];
        assert_eq!(crossover_uniform(&a, &b, 42), crossover_uniform(&a, &b, 42));
    }

    // ── genome_diversity ────────────────────────────────────────────────────

    #[test]
    fn diversity_monoculture_is_zero() {
        let biases = [[0.5, 0.5, 0.5, 0.5]; 5];
        assert_eq!(genome_diversity(&biases), 0.0);
    }

    #[test]
    fn diversity_max_is_two() {
        let biases = [[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]];
        assert!((genome_diversity(&biases) - 2.0).abs() < 1e-3);
    }

    #[test]
    fn diversity_single_entity_is_zero() {
        assert_eq!(genome_diversity(&[[0.5, 0.5, 0.5, 0.5]]), 0.0);
    }

    #[test]
    fn diversity_increases_with_spread() {
        let narrow = [[0.5, 0.5, 0.5, 0.5], [0.51, 0.51, 0.51, 0.51]];
        let wide = [[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]];
        assert!(genome_diversity(&wide) > genome_diversity(&narrow));
    }
}
