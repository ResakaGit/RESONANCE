# Use Case Architecture — HOFs + Contracts + Encapsulation

## Insight

Todos los use cases son pipelines de 4 etapas:

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Config  │ →  │  Engine  │ →  │ Results  │ →  │ Present  │
│          │    │          │    │          │    │          │
│ seed     │    │ evolve   │    │ genomes  │    │ 3D view  │
│ constants│    │ tick     │    │ stats    │    │ CSV      │
│ preset   │    │ select   │    │ worlds   │    │ terminal │
│ map      │    │ mutate   │    │ metrics  │    │ audio    │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

Hoy: cada `src/bin/*.rs` reimplementa las 4 etapas inline.
Propuesta: 4 traits + HOFs que componen cualquier use case.

---

## Contratos (Traits)

### 1. `Experiment` — qué se corre

```rust
// src/use_cases/mod.rs

/// Define un experimento: config → results.
/// Stateless. Pure. El mismo config produce el mismo resultado (determinismo).
pub trait Experiment {
    type Config: Clone;
    type Result;

    /// Corre el experimento completo. Pure function.
    fn run(config: &Self::Config) -> Self::Result;
}
```

Cada use case implementa `Experiment` con su config y resultado específico.
El motor de evolución (`GeneticHarness`) es un detalle de implementación — los
use cases no lo ven directamente.

### 2. `Presenter` — qué se hace con los resultados

```rust
/// Consume resultados y produce output (terminal, 3D, file, audio).
pub trait Presenter<R> {
    type Output;

    fn present(&self, results: &R) -> Self::Output;
}
```

### 3. `UniversePreset` — constantes del universo como valor

```rust
/// Combinación nombrada de constantes físicas.
/// Cada preset es un universo alternativo.
pub struct UniversePreset {
    pub name:                &'static str,
    pub gravity:             f32,
    pub solar_flux:          f32,
    pub solar_frequency:     f32,
    pub season_rate:         f32,
    pub season_amplitude:    f32,
    pub asteroid_interval:   u64,
    pub asteroid_radius_sq:  f32,
    pub asteroid_survival:   f32,
    pub dissipation_floor:   f32,
    pub photosynthesis_eff:  f32,
}
```

### 4. `ExperimentReport` — resultado estandarizado

```rust
/// Resultado estándar de cualquier experimento evolutivo.
pub struct ExperimentReport {
    pub preset:       UniversePreset,
    pub seed:         u64,
    pub generations:  u32,
    pub ticks_per_gen: u32,
    pub world_count:  usize,
    pub top_genomes:  Vec<GenomeBlob>,
    pub history:      Vec<GenerationStats>,
    pub wall_time_ms: u64,
}
```

---

## HOFs (Higher Order Functions)

### `evolve_with` — el HOF central

```rust
/// HOF: configura universo + evoluciona + retorna reporte.
///
/// Todos los use cases llaman a esta función.
/// Es la única que conoce GeneticHarness. Los callers no.
pub fn evolve_with(
    preset: &UniversePreset,
    seed: u64,
    worlds: usize,
    generations: u32,
    ticks: u32,
    entities: u8,
) -> ExperimentReport {
    let config = preset.to_batch_config(seed, worlds, generations, ticks, entities);
    let start = Instant::now();
    let mut harness = GeneticHarness::new(config);
    let genomes = harness.run();
    ExperimentReport {
        preset: preset.clone(),
        seed,
        generations,
        ticks_per_gen: ticks,
        world_count: worlds,
        top_genomes: genomes,
        history: harness.history,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}
```

### `compare_universes` — HOF para investigación

```rust
/// HOF: corre N universos con presets diferentes, compara resultados.
///
/// Use cases: Fermi Paradox, Debate Settler, Convergence Detector.
pub fn compare_universes(
    presets: &[UniversePreset],
    seeds_per_preset: usize,
    generations: u32,
    ticks: u32,
) -> Vec<(UniversePreset, Vec<ExperimentReport>)> {
    presets.iter().map(|preset| {
        let reports: Vec<ExperimentReport> = (0..seeds_per_preset)
            .map(|i| evolve_with(preset, i as u64, 100, generations, ticks, 16))
            .collect();
        (preset.clone(), reports)
    }).collect()
}
```

### `evolve_and_present` — HOF para viewer

```rust
/// HOF: evoluciona + presenta con cualquier Presenter.
pub fn evolve_and_present<P: Presenter<ExperimentReport>>(
    preset: &UniversePreset,
    seed: u64,
    worlds: usize,
    generations: u32,
    ticks: u32,
    presenter: &P,
) -> P::Output {
    let report = evolve_with(preset, seed, worlds, generations, ticks, 16);
    presenter.present(&report)
}
```

---

## Presets canónicos

```rust
pub const EARTH: UniversePreset = UniversePreset {
    name: "Earth",
    gravity: 0.5,
    solar_flux: 2.0,
    solar_frequency: 400.0,
    season_rate: 0.001,
    season_amplitude: 0.4,
    asteroid_interval: 5000,
    asteroid_radius_sq: 25.0,
    asteroid_survival: 0.1,
    dissipation_floor: 0.001,
    photosynthesis_eff: 0.4,
};

pub const JUPITER: UniversePreset = UniversePreset {
    name: "Jupiter",
    gravity: 2.5,      // 5× Earth
    solar_flux: 0.5,    // lejos del sol
    season_amplitude: 0.1,
    ..EARTH
};

pub const MARS: UniversePreset = UniversePreset {
    name: "Mars",
    gravity: 0.05,      // baja gravedad
    solar_flux: 1.0,     // menos sol
    season_amplitude: 0.8, // estaciones extremas
    asteroid_interval: 2000, // más impactos
    ..EARTH
};

pub const EDEN: UniversePreset = UniversePreset {
    name: "Eden",
    gravity: 0.2,
    solar_flux: 5.0,     // abundancia
    season_amplitude: 0.0, // sin estaciones
    asteroid_interval: 0, // sin asteroides (0 = desactivado)
    dissipation_floor: 0.0001,
    ..EARTH
};

pub const HELL: UniversePreset = UniversePreset {
    name: "Hell",
    gravity: 3.0,
    solar_flux: 0.3,
    season_amplitude: 0.9,
    asteroid_interval: 500, // extinción constante
    asteroid_survival: 0.01, // 99% muerte
    dissipation_floor: 0.01,
    ..EARTH
};
```

---

## Cómo cada use case se encapsula

### A1. Versus Arena

```rust
pub fn versus(genome_file_a: &Path, genome_file_b: &Path, preset: &UniversePreset) {
    let genomes_a = bridge::load_genomes(genome_file_a).unwrap();
    let genomes_b = bridge::load_genomes(genome_file_b).unwrap();
    // spawn both in same world with different factions
    // tick until one faction loses all qe
    // report winner
}
```

2 inputs (archivos .bin). 1 output (quién ganó). Zero configuración interna.

### A2. Lab de Universos

```rust
pub fn lab(preset: &UniversePreset, seed: u64) -> ExperimentReport {
    evolve_with(preset, seed, 200, 100, 3000, 16)
}

// Uso:
let earth = lab(&EARTH, 42);
let jupiter = lab(&JUPITER, 42);
// Comparar: earth.top_genomes vs jupiter.top_genomes
```

1 preset + 1 seed. El HOF hace todo.

### B1. Fermi Paradox

```rust
pub fn fermi(n_universes: usize) -> FermiReport {
    let presets = generate_random_presets(n_universes);
    let results = compare_universes(&presets, 1, 100, 1000);
    let life_count = results.iter()
        .filter(|(_, reports)| reports[0].top_genomes.len() > 3)
        .count();
    FermiReport {
        total: n_universes,
        with_life: life_count,
        probability: life_count as f32 / n_universes as f32,
    }
}
```

1 número (cuántos universos). 1 output (probabilidad de vida).

### C1. Fossil Record

```rust
pub fn fossil_record(preset: &UniversePreset, seed: u64, gens: u32) -> Vec<GenomeBlob> {
    // Modificar harness para guardar top genome POR generación
    let config = preset.to_batch_config(seed, 200, gens, 3000, 16);
    let mut harness = GeneticHarness::new(config);
    let mut timeline = Vec::with_capacity(gens as usize);
    for _ in 0..gens {
        harness.step();
        timeline.push(harness.top_genomes(1)[0]);
    }
    timeline // Vec de gens genomes, uno por generación
}
```

1 preset + 1 seed + N gens. Output: secuencia de genomes para morphing.

### D1. Personal Universe

```rust
pub fn personal_universe(birthday: &str) -> ExperimentReport {
    let seed = hash_string_to_u64(birthday);
    evolve_with(&EARTH, seed, 100, 50, 2000, 12)
}
```

1 string. 1 universo. 5 LOC.

---

## Dónde vive cada cosa

```
src/
├── use_cases/                    ← NUEVO: HOFs + traits + presets
│   ├── mod.rs                    ← Experiment, Presenter, ExperimentReport traits
│   ├── presets.rs                ← EARTH, JUPITER, MARS, EDEN, HELL + UniversePreset
│   ├── evolve.rs                ← evolve_with(), compare_universes()
│   ├── presenters/
│   │   ├── mod.rs
│   │   ├── terminal.rs          ← Print stats to stdout
│   │   ├── csv.rs               ← Export to CSV
│   │   ├── bevy_viewer.rs       ← Open 3D Bevy window
│   │   ├── stl_export.rs        ← Export meshes to STL
│   │   └── audio.rs             ← Sonification (future)
│   └── experiments/
│       ├── mod.rs
│       ├── versus.rs            ← A1
│       ├── lab.rs               ← A2
│       ├── survival.rs          ← A3
│       ├── fermi.rs             ← B1
│       ├── speciation.rs        ← B2
│       ├── debate.rs            ← B4
│       ├── fossil_record.rs     ← C1
│       ├── personal.rs          ← D1
│       └── convergence.rs       ← D2
│
├── batch/                        ← SIN CAMBIOS — motor puro
│   ├── harness.rs               ← GeneticHarness (detalle interno)
│   ├── arena.rs                 ← SimWorldFlat (detalle interno)
│   └── ...
│
├── blueprint/equations/          ← SIN CAMBIOS — math pura
│
├── bin/                          ← SIMPLIFICADO — thin wrappers
│   ├── evolve.rs                ← cli::parse_args() → evolve_with() → terminal_presenter
│   ├── evolve_and_view.rs       ← cli::parse_args() → evolve_with() → bevy_presenter
│   ├── versus.rs                ← NUEVO: cli → versus::run()
│   ├── fermi.rs                 ← NUEVO: cli → fermi::run()
│   └── personal.rs              ← NUEVO: cli → personal::run()
```

---

## Principios

1. **Los binarios son thin wrappers.** Parsean args, llaman un HOF, pasan un presenter.
   Nunca más de 30 LOC por binario.

2. **Los experiments no conocen Bevy.** Reciben config pura, retornan data pura.
   Son testables sin GPU.

3. **Los presenters no conocen batch.** Reciben `ExperimentReport`, producen output.
   Son intercambiables (terminal ↔ CSV ↔ 3D ↔ audio).

4. **Los presets son valores, no código.** `JUPITER` es una struct con 11 floats.
   No tiene comportamiento. No tiene lógica. Solo datos.

5. **`evolve_with` es el único punto de contacto con el motor.**
   Ningún use case toca `GeneticHarness` directamente. Si el motor cambia,
   solo `evolve_with` se modifica.

6. **Composición sobre herencia.** `versus = load + evolve_with × 2 + compare`.
   `fermi = generate_presets + compare_universes + aggregate`. No hay jerarquía de clases.

---

## Invariantes

- **INV-UC1:** Todo `Experiment::run()` es determinista. Mismo config → mismo result.
- **INV-UC2:** Ningún experiment importa Bevy. `use_cases/` es Bevy-free.
- **INV-UC3:** Ningún presenter modifica resultados. Son read-only consumers.
- **INV-UC4:** `UniversePreset` no contiene lógica — solo constantes.
- **INV-UC5:** Los binarios en `src/bin/` tienen ≤30 LOC cada uno.

---

## Testing

```rust
#[test]
fn evolve_with_deterministic() {
    let a = evolve_with(&EARTH, 42, 10, 5, 100, 4);
    let b = evolve_with(&EARTH, 42, 10, 5, 100, 4);
    assert_eq!(a.top_genomes[0].hash(), b.top_genomes[0].hash());
}

#[test]
fn different_presets_different_results() {
    let earth = evolve_with(&EARTH, 42, 10, 5, 100, 4);
    let jupiter = evolve_with(&JUPITER, 42, 10, 5, 100, 4);
    assert_ne!(earth.top_genomes[0].hash(), jupiter.top_genomes[0].hash());
}

#[test]
fn preset_to_batch_config_preserves_values() {
    let config = EARTH.to_batch_config(42, 100, 50, 3000, 16);
    assert_eq!(config.seed, 42);
    assert_eq!(config.world_count, 100);
}
```
