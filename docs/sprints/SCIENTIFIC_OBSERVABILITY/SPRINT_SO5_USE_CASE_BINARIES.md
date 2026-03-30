# SO-5: Use-Case Binaries

**Objetivo:** 5 binarios científicos standalone que componen los orchestrators (SO-4) con los adapters (SO-3) para producir resultados publicables. Cada binario es un one-liner conceptual que conecta hipótesis → simulación → datos.

**Estado:** PENDIENTE
**Esfuerzo:** L (~300 LOC total, ~60 per binary)
**Bloqueado por:** SO-4

---

## Principio

```
Cada binary = 1 orquestador + 1 export adapter + 1 output path.
Zero lógica de simulación. Zero constantes nuevas. Solo composición.
```

---

## Binary 1: `fermi` — Paradoja de Fermi

**Pregunta:** "¿En cuántos universos aleatorios emerge vida compleja?"

```rust
// src/bin/fermi.rs

fn main() {
    let args = parse_args();  // --seeds 100000 --gens 100 --ticks 500 --out fermi.csv

    // Sweep: perturbación aleatoria de las 4 constantes fundamentales
    let reports = ablate(&base_config(), &seeds_f32(args.seeds), |cfg, seed| {
        let mut rng = seed as u64;
        cfg.gravity *= perturb(&mut rng, 0.1, 10.0);
        cfg.solar_flux_base *= perturb(&mut rng, 0.1, 5.0);
        cfg.dissipation_solid *= perturb(&mut rng, 0.5, 2.0);
        cfg.seed = seed as u64;
    });

    // Análisis: ¿cuántos produjeron species_count > 3?
    let complex = reports.iter()
        .filter(|r| r.history.last().map(|s| s.species_mean > 3.0).unwrap_or(false))
        .count();

    println!("Fermi result: {complex}/{} universes produced complex life ({:.1}%)",
        reports.len(), complex as f32 / reports.len() as f32 * 100.0);

    // Export CSV con todos los runs
    let csv = export_history::<CsvAdapter>(&reports);
    std::fs::write(&args.out, csv).expect("write failed");
}
```

**Output:** CSV con parámetros + resultado (vida sí/no) por seed. Directamente importable en Python/R.

**Constantes:** Todas derivadas. `perturb()` escala las 4 fundamentales, no inventa nuevas.

---

## Binary 2: `speciation` — Aislamiento Reproductivo

**Pregunta:** "¿Emerge especiación si hay una barrera geográfica?"

```rust
// src/bin/speciation.rs

fn main() {
    let args = parse_args();  // --gens 200 --barrier-width 4 --out speciation.csv

    let ensemble_report = ensemble(&barrier_config(args.barrier_width), args.seeds);

    // Medir divergencia de frecuencia entre lado izquierdo y derecho
    for report in &ensemble_report.reports {
        for census in report.censuses.as_ref().unwrap() {
            let (left, right) = split_by_position(census, grid_midpoint);
            let freq_divergence = mean_frequency(&left) - mean_frequency(&right);
            // Export: gen, divergence, species_left, species_right
        }
    }
}
```

**Constantes derivadas:** `barrier_width` solo modifica el mapa RON (nutrient=0 en franja), no constantes de simulación.

---

## Binary 3: `cancer` — Tumor Evolution

**Pregunta:** "¿Cómo evoluciona resistencia a quimioterapia?"

```rust
// src/bin/cancer.rs

fn main() {
    let args = parse_args();  // --treatment-tick 100 --treatment-radius 5 --out cancer.csv

    // Fase 1: crecer tumor (alta growth, baja dissipation)
    let tumor_config = cancer_config(args.treatment_tick, args.treatment_radius);

    // Sweep: tratamiento a distintos ticks
    let reports = ablate(&tumor_config, &treatment_ticks(50, 200, 10), |cfg, tick| {
        cfg.asteroid_interval = tick as u32;  // "quimio" = asteroid localizado
    });

    // Export: ¿cuántas células sobreviven post-tratamiento? ¿resilience media?
    for r in &reports {
        let post = r.censuses.as_ref().and_then(|c| c.last());
        // survivors, mean_resilience, mean_mobility (escape = metastasis)
    }
}
```

**Mapeo axiomático:**
- Tumor = entity con `dissipation ≈ 0`, `growth_bias = 1.0` (Axiom 4 violado por la enfermedad)
- Quimio = asteroid localizado (Axiom 5: destrucción conserva energía al grid)
- Resistencia = evolución de `resilience` bajo presión selectiva (Axiom 3)
- Metástasis = alta `mobility_bias` escapando del radio de impacto (Axiom 7: distance attenuation)

---

## Binary 4: `epidemiology` — Propagación de Enfermedad

**Pregunta:** "¿Cómo se propaga una perturbación de frecuencia en una población?"

```rust
// src/bin/epidemiology.rs

fn main() {
    let args = parse_args();  // --pathogen-freq 500 --pop-freq 100 --virulence 0.8

    // Población homogénea + 1 "paciente cero" con frecuencia distinta
    let epi_config = epidemiology_config(args.pathogen_freq, args.pop_freq, args.virulence);

    // Ensemble para statistics
    let report = ensemble(&epi_config, args.seeds);

    // Tracking: ¿cuántas entidades "infectadas" (frequency shifted > threshold) por tick?
    // La interferencia (Axiom 8) entre pathogen-freq y pop-freq determina transmisión
}
```

**Mapeo axiomático:**
- Infección = frequency interference destructiva (Axiom 8)
- Transmisión = entrainment hacia la frecuencia del patógeno (Axiom 8)
- Inmunidad = alta coherencia resiste entrainment (Axiom 8)
- Cuarentena = barrera espacial (Axiom 7: distance attenuation)

**Constantes:** Zero nuevas. Frequency del patógeno y la población son bandas del Almanac existente.

---

## Binary 5: `convergence` — Evolución Convergente

**Pregunta:** "¿Seeds distintas llegan a la misma solución?"

```rust
// src/bin/convergence.rs

fn main() {
    let args = parse_args();  // --seeds 100 --gens 500 --out convergence.csv

    let report = ensemble(&base_config(), args.seeds);

    // Comparar genomas finales entre seeds
    let top_genomes: Vec<_> = report.reports.iter()
        .flat_map(|r| r.top_genomes.iter())
        .collect();

    // Clustering: ¿cuántos clusters hay? (K-means en 4D bias space)
    // Si cluster_count < seeds → convergencia detectada
    let clusters = cluster_genomes(&top_genomes, distance_threshold);
}
```

---

## Función helper compartida: `base_config()`

```rust
/// Config base derivada de las 4 constantes fundamentales.
/// Base config derived from the 4 fundamental constants.
///
/// Zero hardcode: usa `derived_thresholds` para calibrar todo.
fn base_config() -> BatchConfig {
    BatchConfig {
        world_count: 100,
        ticks_per_eval: 500,
        max_generations: 200,
        seed: 42,
        ..Default::default()
    }
}
```

`Default::default()` hereda todas las constantes de `batch/constants.rs`, que a su vez se derivan de los 4 fundamentales.

---

## Tests (por binary)

```
// Fermi
fermi_100_seeds_runs_without_panic
fermi_deterministic_same_seed_same_complex_count
fermi_zero_seeds_produces_empty_report

// Speciation
speciation_barrier_zero_no_divergence
speciation_barrier_max_frequency_diverges
speciation_ensemble_deterministic

// Cancer
cancer_no_treatment_tumor_grows
cancer_early_treatment_more_survivors
cancer_late_treatment_resistance_higher

// Epidemiology
epidemiology_same_frequency_no_spread
epidemiology_different_frequency_interference
epidemiology_quarantine_reduces_spread

// Convergence
convergence_single_seed_single_cluster
convergence_100_seeds_fewer_clusters_than_seeds
```

---

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/bin/fermi.rs` | **NUEVO** |
| `src/bin/speciation.rs` | **NUEVO** |
| `src/bin/cancer.rs` | **NUEVO** |
| `src/bin/epidemiology.rs` | **NUEVO** |
| `src/bin/convergence.rs` | **NUEVO** |
| `src/use_cases/presets.rs` | **NUEVO** — `cancer_config()`, `epidemiology_config()`, `barrier_config()` |

---

## Invariantes transversales

1. **Zero constantes nuevas** — todo se deriva de `BatchConfig::default()` + las 4 fundamentales
2. **Zero modificación de simulación** — los binarios solo componen + exportan
3. **Determinismo** — mismo seed → mismos resultados → reproducibilidad científica
4. **Stateless** — cada función recibe inputs, retorna outputs, zero estado global
5. **Axiom compliance** — cada mapeo (tumor, patógeno, barrera) usa axiomas existentes, no los extiende
