# Sprint BS-4 — Genetic Harness: Evolucion Real por Seleccion Masiva

**Modulo:** `src/batch/harness.rs`, `src/batch/genome.rs`, `src/batch/batch.rs`, `src/blueprint/equations/batch_fitness.rs` (nuevo)
**Tipo:** Genetic algorithm harness + tipos de genome + ecuaciones de fitness.
**Onda:** BS-3 → BS-4.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post BS-3)

- 33 systems completos (Tier 1 + 2 + 3). Simulacion funcional con ciclo de vida.
- `SimWorldFlat::tick()` corre un tick atomico con muerte, reproduccion, abiogenesis.
- `GenomeBlob::from_slot()` / `apply()` / `mutate()` basicos de BS-3.

---

## Objetivo

Implementar el loop evolutivo completo: inicializar N mundos con genomes variados, correr M ticks de evaluacion, calcular fitness por mundo, seleccionar elite, mutar/crossover, y repetir. Este sprint es el **core del producto** — lo que convierte la simulacion en evolucion real.

---

## Responsabilidades

### BS-4A: GenomeBlob completo

```rust
// src/batch/genome.rs

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GenomeBlob {
    pub archetype:      u8,
    pub trophic_class:  u8,
    pub growth_bias:    f32,
    pub mobility_bias:  f32,
    pub branching_bias: f32,
    pub resilience:     f32,
}

impl GenomeBlob {
    pub fn random(rng_state: u64) -> Self { ... }
    pub fn from_slot(slot: &EntitySlot) -> Self { ... }
    pub fn apply(&self, slot: &mut EntitySlot) { ... }
    pub fn mutate(&self, rng_state: u64, sigma: f32) -> Self { ... }
    pub fn crossover(&self, other: &Self, rng_state: u64) -> Self { ... }
    pub fn hash(&self) -> u64 { ... }
    pub fn distance(&self, other: &Self) -> f32 { ... }
}
```

**crossover:** Uniforme — para cada gen, 50% chance de tomar del padre A o B.
**distance:** Euclidean en espacio de 4 biases (para analisis de diversidad).

### BS-4B: FitnessReport

```rust
// src/batch/harness.rs

pub struct FitnessReport {
    pub world_index:       usize,
    pub survivors:         u8,
    pub total_qe:          f32,
    pub reproductions:     u16,
    pub max_trophic_level: u8,
    pub species_count:     u8,
    pub cultural_memes:    u8,
    pub coalition_count:   u8,
    pub institution_alive: bool,
    pub composite_fitness: f32,
}

impl FitnessReport {
    pub fn compute(world: &SimWorldFlat, weights: &[f32; 6]) -> Self {
        let survivors = world.alive_mask.count_ones() as u8;
        let reproductions = world.events.repro_len as u16;
        let species_count = count_frequency_bands(world);
        let max_trophic = max_trophic_chain(world);
        let memes = 0u8;       // TODO: cuando culture_transmission trackee
        let coalitions = 0u8;  // TODO: cuando cooperation_eval trackee

        let composite = equations::batch_fitness::composite_fitness(
            survivors, reproductions, species_count,
            max_trophic, memes, coalitions, weights,
        );

        Self {
            world_index: 0, survivors, total_qe: world.total_qe,
            reproductions, max_trophic_level: max_trophic,
            species_count, cultural_memes: memes,
            coalition_count: coalitions, institution_alive: false,
            composite_fitness: composite,
        }
    }
}

/// Cuenta bandas de frecuencia distintas (species proxy).
fn count_frequency_bands(world: &SimWorldFlat) -> u8 {
    let mut bands = [false; 16];  // 16 frequency bands
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let band = (world.entities[i].frequency_hz / 100.0).min(15.0) as usize;
        bands[band] = true;
    }
    bands.iter().filter(|&&b| b).count() as u8
}
```

### BS-4C: BatchConfig y WorldBatch

```rust
// src/batch/batch.rs

pub struct BatchConfig {
    pub world_count:     usize,
    pub ticks_per_eval:  u32,
    pub tick_rate_hz:    f32,
    pub mutation_sigma:  f32,
    pub elite_fraction:  f32,
    pub crossover_rate:  f32,
    pub max_generations: u32,
    pub seed:            u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            world_count: 10_000,
            ticks_per_eval: 10_000,
            tick_rate_hz: 20.0,
            mutation_sigma: 0.05,
            elite_fraction: 0.10,
            crossover_rate: 0.30,
            max_generations: 1000,
            seed: 42,
        }
    }
}

pub struct WorldBatch {
    pub worlds:     Vec<SimWorldFlat>,
    pub generation: u32,
    pub config:     BatchConfig,
}

impl WorldBatch {
    pub fn new(config: BatchConfig) -> Self { ... }
    pub fn tick_all(&mut self) { ... }
    pub fn run_evaluation(&mut self, ticks: u32) { ... }
}
```

### BS-4D: GeneticHarness

```rust
// src/batch/harness.rs

pub struct GeneticHarness {
    pub batch:  WorldBatch,
    pub config: BatchConfig,
    pub history: Vec<GenerationStats>,  // fitness history per generation
}

pub struct GenerationStats {
    pub generation:    u32,
    pub best_fitness:  f32,
    pub mean_fitness:  f32,
    pub worst_fitness: f32,
    pub diversity:     f32,  // mean pairwise genome distance in elite
    pub survivors_mean: f32,
    pub species_mean:  f32,
}

impl GeneticHarness {
    pub fn new(config: BatchConfig) -> Self { ... }

    /// Un paso generacional completo.
    pub fn step(&mut self) -> &GenerationStats { ... }

    /// Corre hasta max_generations o convergencia.
    pub fn run(&mut self) -> Vec<GenomeBlob> { ... }

    /// Extrae los N mejores genomes del batch actual.
    pub fn top_genomes(&self, n: usize) -> Vec<GenomeBlob> { ... }

    // --- internal ---
    fn evaluate(&mut self) -> Vec<FitnessReport> { ... }
    fn select_elite(&self, reports: &[FitnessReport]) -> Vec<usize> { ... }
    fn extract_genomes(&self, world_idx: usize) -> Vec<GenomeBlob> { ... }
    fn repopulate(&mut self, elite_genomes: &[Vec<GenomeBlob>]) { ... }
}
```

### BS-4E: Ecuaciones de fitness (blueprint)

```rust
// src/blueprint/equations/batch_fitness.rs (NUEVO)

/// Fitness ponderada. Todos los inputs normalizados internamente.
pub fn composite_fitness(
    survivors: u8, reproductions: u16, species: u8,
    trophic: u8, memes: u8, coalitions: u8,
    weights: &[f32; 6],
) -> f32 {
    let s = [
        survivors as f32 / MAX_ENTITIES as f32,
        (reproductions as f32).min(100.0) / 100.0,
        species as f32 / 16.0,
        trophic as f32 / 5.0,
        memes as f32 / 16.0,
        coalitions as f32 / 8.0,
    ];
    s.iter().zip(weights.iter()).map(|(v, w)| v * w).sum()
}

/// Tournament selection: pick k random, return index of best.
pub fn tournament_select(
    fitnesses: &[f32], k: usize, rng_state: u64,
) -> usize { ... }

/// Crossover uniforme entre dos genomes.
pub fn crossover_uniform(
    a: &[f32; 4], b: &[f32; 4], rng_state: u64,
) -> [f32; 4] { ... }
```

### BS-4F: Ecuaciones de determinismo (ampliar)

```rust
// src/blueprint/equations/determinism.rs — ampliar

/// PCG-like step: state → next state.
pub fn next_u64(state: u64) -> u64 {
    state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

/// Uniform f32 in [0, 1) from state.
pub fn unit_f32(state: u64) -> f32 {
    (state >> 40) as f32 / (1u64 << 24) as f32
}

/// Range f32 in [min, max) from state.
pub fn range_f32(state: u64, min: f32, max: f32) -> f32 {
    min + unit_f32(state) * (max - min)
}

/// Gaussian f32 via Box-Muller transform.
pub fn gaussian_f32(state: u64, sigma: f32) -> f32 {
    let u1 = unit_f32(state).max(1e-10);
    let u2 = unit_f32(next_u64(state));
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    z * sigma
}
```

---

## NO hace

- No implementa bridge Batch ↔ Bevy — eso es BS-5.
- No usa rayon — single-threaded. BS-6 paraleliza.
- No implementa early stopping por convergencia (futuro).
- No persiste genomes a disco — solo en memoria.

---

## Dependencias

- BS-3 — simulacion completa con lifecycle.
- `crate::blueprint::equations/determinism` — RNG (ampliar con `next_u64`, `gaussian_f32`).
- `crate::blueprint::equations/batch_fitness` — NUEVO: fitness + tournament + crossover.

---

## Criterios de aceptacion

### BS-4A (GenomeBlob)
- `GenomeBlob::random(seed)` produce genomes distintos para seeds distintos.
- `mutate` con `sigma=0` retorna genome identico.
- `crossover(a, b)` produce genome con genes de ambos padres.
- `hash(a) != hash(b)` si `a != b`.

### BS-4B (FitnessReport)
- `composite_fitness(64, 100, 16, 5, 16, 8, [1;6])` → maximo score.
- `composite_fitness(0, 0, 0, 0, 0, 0, [1;6])` → 0.0.

### BS-4D (GeneticHarness)
- 10K mundos × 100 generaciones: `best_fitness` es monotonicamente no-decreciente.
- Diversidad decrece con generaciones (convergencia).
- `top_genomes(10)` retorna 10 genomes distintos.

### BS-4F (Determinism)
- `next_u64(42) == next_u64(42)` siempre (determinista).
- `unit_f32` produce valores en `[0, 1)` para 1000 seeds.
- `gaussian_f32` tiene mean ≈ 0 y stddev ≈ sigma para 10K samples.

### General
- `cargo test --lib` sin regresion.
- 10K mundos × 100 gen completa en < 60 segundos single-threaded.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` §5 — genetic harness
- `src/blueprint/equations/determinism.rs` — hash functions existentes
- Algoritmo genetico: tournament selection + uniform crossover (standard GA)
