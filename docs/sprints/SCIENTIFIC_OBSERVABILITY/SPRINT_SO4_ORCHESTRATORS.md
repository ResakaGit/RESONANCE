# SO-4: HOF Orchestrators (Ablation, Ensemble, Sweep)

**Objetivo:** Funciones de orden superior que componen `GeneticHarness` + `PopulationCensus` + `ExportAdapter` en workflows científicos reutilizables. Cada orchestrator recibe funciones como parámetros — zero hardcode de lógica de análisis.

**Estado:** PENDIENTE
**Esfuerzo:** M (~200 LOC)
**Bloqueado por:** SO-3

---

## Principio: Composición > Configuración

```
El usuario no configura un "experimento de ablación".
El usuario COMPONE funciones puras:
  run_evolution × vary_parameter × collect_results × export
```

Cada orchestrator es un **fold** sobre un espacio de parámetros.

---

## Orchestrator 1: `ablate` — Sensitivity Analysis

```rust
// src/use_cases/orchestrators.rs (NUEVO)

/// Ablación de un parámetro: corre N experimentos variando un valor.
/// Parameter ablation: runs N experiments varying one value.
///
/// `modify_fn` recibe un `&mut BatchConfig` y el valor actual.
/// Zero hardcode de qué parámetro se varía — el caller lo decide con un closure.
pub fn ablate<F>(
    base_config: &BatchConfig,
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
            evolve_with_config(&cfg)
        })
        .collect()
}
```

**Uso:**
```rust
// Ablación de gravity:
let reports = ablate(&base, &[0.1, 0.5, 1.0, 2.0, 5.0], |cfg, g| {
    cfg.gravity = g;
});

// Ablación de mutation rate:
let reports = ablate(&base, &[0.0, 0.01, 0.05, 0.1, 0.5], |cfg, sigma| {
    cfg.mutation_sigma = sigma;
});
```

## Orchestrator 2: `ensemble` — Statistical Replication

```rust
/// Ensemble: corre el mismo experimento con N seeds distintas.
/// Ensemble: runs the same experiment with N different seeds.
///
/// Retorna reports individuales + estadísticas agregadas.
pub fn ensemble(
    base_config: &BatchConfig,
    n_seeds: usize,
) -> EnsembleReport {
    let reports: Vec<ExperimentReport> = (0..n_seeds)
        .map(|i| {
            let mut cfg = base_config.clone();
            cfg.seed = i as u64;
            evolve_with_config(&cfg)
        })
        .collect();

    EnsembleReport::from_reports(reports)
}

/// Estadísticas de un ensemble.
pub struct EnsembleReport {
    pub reports:        Vec<ExperimentReport>,
    pub mean_fitness:   f32,
    pub std_fitness:    f32,
    pub mean_diversity: f32,
    pub mean_species:   f32,
}

impl EnsembleReport {
    pub fn from_reports(reports: Vec<ExperimentReport>) -> Self {
        let n = reports.len() as f32;
        let fitnesses: Vec<f32> = reports.iter()
            .filter_map(|r| r.history.last())
            .map(|s| s.best_fitness)
            .collect();
        let mean_fitness = fitnesses.iter().sum::<f32>() / n.max(1.0);
        let var = fitnesses.iter()
            .map(|f| (f - mean_fitness).powi(2))
            .sum::<f32>() / n.max(1.0);
        Self {
            mean_fitness,
            std_fitness: var.sqrt(),
            mean_diversity: reports.iter()
                .filter_map(|r| r.history.last())
                .map(|s| s.diversity)
                .sum::<f32>() / n.max(1.0),
            mean_species: reports.iter()
                .filter_map(|r| r.history.last())
                .map(|s| s.species_mean)
                .sum::<f32>() / n.max(1.0),
            reports,
        }
    }
}
```

## Orchestrator 3: `sweep` — 2D Parameter Space

```rust
/// Sweep bidimensional: varía dos parámetros simultáneamente.
/// 2D parameter sweep: varies two parameters simultaneously.
///
/// Retorna una grilla de resultados indexada por (param_a, param_b).
pub fn sweep<Fa, Fb>(
    base_config: &BatchConfig,
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
                    evolve_with_config(&cfg)
                })
                .collect()
        })
        .collect()
}
```

**Uso:**
```rust
// Sweep gravity × solar_flux:
let grid = sweep(
    &base,
    &[0.1, 1.0, 5.0],           // gravity
    &[0.3, 1.0, 3.0],           // solar flux
    |cfg, g| cfg.gravity = g,
    |cfg, s| cfg.solar_flux_base = s,
);
// grid[0][2] = gravity=0.1, solar=3.0
```

## Orchestrator 4: `compare` — Multi-Universe

```rust
/// Compara universos con presets distintos. Cada uno corre ensemble de N seeds.
/// Compares universes with different presets. Each runs an ensemble of N seeds.
pub fn compare_presets(
    presets: &[(&str, BatchConfig)],
    seeds_per_preset: usize,
) -> Vec<(&str, EnsembleReport)> {
    presets
        .iter()
        .map(|(name, cfg)| (*name, ensemble(cfg, seeds_per_preset)))
        .collect()
}
```

---

## Propiedad: Toda constante derivada

Los orchestrators no definen constantes. Los parámetros que reciben vienen de `BatchConfig`, que a su vez se construye desde las 4 constantes fundamentales + presets. El orchestrator solo **compone**.

---

## Tests

```
// ablate
ablate_single_value_returns_one_report
ablate_multiple_values_returns_matching_count
ablate_closure_modifies_gravity_in_config
ablate_deterministic_same_seed_same_result

// ensemble
ensemble_one_seed_returns_one_report
ensemble_reports_count_matches_n_seeds
ensemble_mean_fitness_is_average_of_bests
ensemble_std_fitness_zero_if_all_identical

// sweep
sweep_1x1_returns_single_cell
sweep_3x3_returns_9_reports
sweep_grid_indexed_correctly

// compare
compare_two_presets_returns_two_entries
compare_preset_names_preserved

// HOF composition
ablate_then_export_csv_produces_valid_output
ensemble_then_distribution_extracts_fitness_spread
```

---

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/use_cases/orchestrators.rs` | **NUEVO** — ablate, ensemble, sweep, compare_presets |
| `src/use_cases/mod.rs` | + `pub mod orchestrators` |
