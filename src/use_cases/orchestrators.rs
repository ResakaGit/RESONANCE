//! HOF orchestrators — composición de experimentos sin hardcode.
//! HOF orchestrators — experiment composition without hardcode.
//!
//! Cada orchestrator recibe closures que definen qué variar y cómo medir.
//! Zero lógica de simulación. Zero constantes nuevas. Solo composición funcional.
//! Stateless: funciones puras que reciben config, retornan reports.

use crate::batch::batch::BatchConfig;
use crate::use_cases::ExperimentReport;
use crate::use_cases::presets::UniversePreset;

// ─── Ablation ───────────────────────────────────────────────────────────────

/// Ablación de un parámetro: corre N experimentos variando un valor.
/// Parameter ablation: runs N experiments varying one value.
///
/// `modify_fn` recibe `&mut BatchConfig` y el valor actual — zero hardcode de qué parámetro.
pub fn ablate<F>(
    base_config: &BatchConfig,
    preset: &UniversePreset,
    values: &[f32],
    modify_fn: F,
) -> Vec<ExperimentReport>
where
    F: Fn(&mut BatchConfig, f32),
{
    values
        .iter()
        .map(|&v| {
            let mut cfg = base_config.clone();
            modify_fn(&mut cfg, v);
            run_with_config(&cfg, preset)
        })
        .collect()
}

// ─── Ensemble ───────────────────────────────────────────────────────────────

/// Estadísticas agregadas de un ensemble de experimentos.
/// Aggregated statistics from an ensemble of experiments.
#[derive(Debug, Clone)]
pub struct EnsembleReport {
    pub reports:        Vec<ExperimentReport>,
    pub mean_fitness:   f32,
    pub std_fitness:    f32,
    pub mean_diversity: f32,
    pub mean_species:   f32,
}

/// Ensemble: corre el mismo experimento con N seeds distintas.
/// Ensemble: runs the same experiment with N different seeds.
pub fn ensemble(
    base_config: &BatchConfig,
    preset: &UniversePreset,
    n_seeds: usize,
) -> EnsembleReport {
    let reports: Vec<ExperimentReport> = (0..n_seeds)
        .map(|i| {
            let mut cfg = base_config.clone();
            cfg.seed = i as u64;
            run_with_config(&cfg, preset)
        })
        .collect();

    aggregate_ensemble(reports)
}

/// Agrega reports en EnsembleReport. Función pura (testeable sin correr simulación).
/// Aggregates reports into EnsembleReport. Pure function (testable without running simulation).
pub fn aggregate_ensemble(reports: Vec<ExperimentReport>) -> EnsembleReport {
    // Solo reports con history no-vacía contribuyen a las estadísticas.
    let last_stats: Vec<_> = reports.iter()
        .filter_map(|r| r.history.last())
        .collect();
    let n = last_stats.len() as f32;

    if n < 1.0 {
        return EnsembleReport {
            reports,
            mean_fitness: 0.0, std_fitness: 0.0,
            mean_diversity: 0.0, mean_species: 0.0,
        };
    }

    let mean_fitness = last_stats.iter().map(|s| s.best_fitness).sum::<f32>() / n;
    let var = last_stats.iter()
        .map(|s| (s.best_fitness - mean_fitness).powi(2))
        .sum::<f32>() / n;
    let mean_diversity = last_stats.iter().map(|s| s.diversity).sum::<f32>() / n;
    let mean_species = last_stats.iter().map(|s| s.species_mean).sum::<f32>() / n;

    EnsembleReport {
        reports,
        mean_fitness,
        std_fitness: var.sqrt(),
        mean_diversity,
        mean_species,
    }
}

// ─── Sweep 2D ───────────────────────────────────────────────────────────────

/// Sweep bidimensional: varía dos parámetros simultáneamente.
/// 2D parameter sweep: varies two parameters simultaneously.
///
/// Retorna grilla `[len_a][len_b]` de reports.
pub fn sweep<Fa, Fb>(
    base_config: &BatchConfig,
    preset: &UniversePreset,
    values_a: &[f32],
    values_b: &[f32],
    modify_a: Fa,
    modify_b: Fb,
) -> Vec<Vec<ExperimentReport>>
where
    Fa: Fn(&mut BatchConfig, f32),
    Fb: Fn(&mut BatchConfig, f32),
{
    values_a
        .iter()
        .map(|&va| {
            values_b
                .iter()
                .map(|&vb| {
                    let mut cfg = base_config.clone();
                    modify_a(&mut cfg, va);
                    modify_b(&mut cfg, vb);
                    run_with_config(&cfg, preset)
                })
                .collect()
        })
        .collect()
}

// ─── Internal helper ────────────────────────────────────────────────────────

/// Delega a `evolve_with_config` — preserva TODOS los campos de config.
/// Delegates to `evolve_with_config` — preserves ALL config fields.
fn run_with_config(config: &BatchConfig, preset: &UniversePreset) -> ExperimentReport {
    crate::use_cases::evolve_with_config(config, preset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::harness::GenerationStats;

    fn mock_report(fitness: f32, diversity: f32, species: f32) -> ExperimentReport {
        ExperimentReport {
            preset_name: "test",
            seed: 0,
            generations: 1,
            ticks_per_gen: 10,
            world_count: 1,
            top_genomes: vec![],
            history: vec![GenerationStats {
                generation: 0,
                best_fitness: fitness,
                mean_fitness: fitness * 0.5,
                worst_fitness: 0.0,
                diversity,
                survivors_mean: 4.0,
                species_mean: species,
                gene_count_mean: 8.0,
                metabolic_graph_rate: 0.0,
                protein_function_rate: 0.0,
                codon_count_mean: 0.0,
                multicellular_rate: 0.0,
            }],
            wall_time_ms: 0,
        }
    }

    // ── aggregate_ensemble (pure, no simulation) ──

    #[test]
    fn aggregate_empty_returns_zeros() {
        let e = aggregate_ensemble(vec![]);
        assert_eq!(e.mean_fitness, 0.0);
        assert_eq!(e.std_fitness, 0.0);
    }

    #[test]
    fn aggregate_single_report_zero_std() {
        let e = aggregate_ensemble(vec![mock_report(10.0, 0.5, 3.0)]);
        assert!((e.mean_fitness - 10.0).abs() < 1e-4);
        assert!((e.std_fitness - 0.0).abs() < 1e-4);
    }

    #[test]
    fn aggregate_two_reports_correct_mean() {
        let e = aggregate_ensemble(vec![
            mock_report(10.0, 0.5, 2.0),
            mock_report(20.0, 0.7, 4.0),
        ]);
        assert!((e.mean_fitness - 15.0).abs() < 1e-4);
        assert!((e.mean_diversity - 0.6).abs() < 1e-4);
        assert!((e.mean_species - 3.0).abs() < 1e-4);
    }

    #[test]
    fn aggregate_std_correct_for_known_values() {
        // Values: 10, 20. Mean = 15. Var = ((10-15)² + (20-15)²)/2 = 25. Std = 5.
        let e = aggregate_ensemble(vec![
            mock_report(10.0, 0.0, 0.0),
            mock_report(20.0, 0.0, 0.0),
        ]);
        assert!((e.std_fitness - 5.0).abs() < 1e-3);
    }

    #[test]
    fn aggregate_reports_preserved() {
        let e = aggregate_ensemble(vec![
            mock_report(10.0, 0.0, 0.0),
            mock_report(20.0, 0.0, 0.0),
        ]);
        assert_eq!(e.reports.len(), 2);
    }
}
