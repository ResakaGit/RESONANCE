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

/// Tournament selection: sample `k` random indices, return the one with best fitness.
///
/// Deterministic — uses `rng_state` for index sampling.
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
        let a = tournament_select(&fitnesses, 3, 42);
        let b = tournament_select(&fitnesses, 3, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn tournament_select_tends_toward_best() {
        let fitnesses = [0.0, 0.0, 0.0, 0.0, 1.0]; // idx 4 is best
        let mut best_count = 0;
        for seed in 0..100 {
            if tournament_select(&fitnesses, 3, seed * 7 + 13) == 4 {
                best_count += 1;
            }
        }
        assert!(best_count > 20, "should frequently select best: {best_count}/100");
    }

    #[test]
    fn crossover_uniform_mixes_parents() {
        let a = [0.0; 4];
        let b = [1.0; 4];
        let child = crossover_uniform(&a, &b, 42);
        let has_a = child.iter().any(|&v| v == 0.0);
        let has_b = child.iter().any(|&v| v == 1.0);
        assert!(has_a || has_b, "should contain genes from at least one parent");
    }

    #[test]
    fn crossover_uniform_deterministic() {
        let a = [0.1, 0.2, 0.3, 0.4];
        let b = [0.9, 0.8, 0.7, 0.6];
        assert_eq!(crossover_uniform(&a, &b, 42), crossover_uniform(&a, &b, 42));
    }
}
