# Sprint BS-6 — Parallel + Tuning: rayon, SIMD, Benchmark 1M

**Modulo:** `src/batch/pipeline.rs` (expandir), `Cargo.toml`, `benches/batch_benchmark.rs`
**Tipo:** Performance engineering — paralelismo + benchmarks.
**Onda:** BS-5 → BS-6.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post BS-5)

- Simulacion batch completa: 33 systems, genetic harness, genome bridge.
- Todo funcional single-threaded.
- 1M mundos corren pero lento (~seconds per tick sin paralelismo).

---

## Objetivo

Paralelizar `WorldBatch::tick_all()` con rayon, optimizar hot loops para auto-vectorizacion SIMD, y validar el performance gate: **1M mundos × 1 tick < 20ms en 64 cores**.

---

## Responsabilidades

### BS-6A: rayon integration

```rust
// src/batch/pipeline.rs — modificar tick_all

use rayon::prelude::*;

impl WorldBatch {
    pub fn tick_all(&mut self) {
        self.worlds.par_iter_mut().for_each(|world| {
            SCRATCH.with(|s| {
                let mut scratch = s.borrow_mut();
                world.tick(&mut scratch);
            });
        });
    }
}
```

**Cargo.toml:**
```toml
[dependencies]
rayon = "1.10"
```

**Invariante INV-B4:** Mundos no comparten estado mutable. `par_iter_mut` es safe porque cada `SimWorldFlat` es independiente y `ScratchPad` es thread-local.

### BS-6B: SIMD-friendly inner loops

Asegurar que los Tier 1 systems auto-vectoricen. Tecnicas:

1. **Loop bounds conocidos en compile time:** `for i in 0..MAX_ENTITIES` donde `MAX_ENTITIES = 64`.
2. **No branching en inner loop:** pre-filtrar con `alive_mask` fuera del SIMD loop, o usar mask operations.
3. **`repr(C)` alignment:** `EntitySlot` alineado a 64 bytes (cache line).
4. **Scalar f32 operations:** sin `f64`, sin conversiones.

```rust
/// Ejemplo optimizado: dissipation con pre-filtered indices.
pub fn dissipation_simd(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mask = world.alive_mask;
    let mut m = mask;
    while m != 0 {
        let i = m.trailing_zeros() as usize;
        m &= m - 1;  // clear lowest set bit
        let e = &mut world.entities[i];
        let loss = e.qe * e.dissipation * dt;
        e.qe -= loss;
    }
}
```

**Verificar auto-vectorizacion:** `cargo asm --lib --simplify batch::systems::dissipation` — buscar instrucciones SSE/AVX.

### BS-6C: Benchmark suite

```rust
// benches/batch_benchmark.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_tick_single_world(c: &mut Criterion) {
    let mut world = SimWorldFlat::new(42, 0.05);
    populate_demo(&mut world, 32);
    let mut scratch = ScratchPad::new();
    c.bench_function("single_world_tick", |b| {
        b.iter(|| world.tick(&mut scratch))
    });
}

fn bench_tick_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_tick");
    for count in [1_000, 10_000, 100_000, 1_000_000] {
        let mut batch = WorldBatch::new(BatchConfig {
            world_count: count, ..Default::default()
        });
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, _| b.iter(|| batch.tick_all()),
        );
    }
    group.finish();
}

fn bench_genetic_step(c: &mut Criterion) {
    let mut harness = GeneticHarness::new(BatchConfig {
        world_count: 1_000,
        ticks_per_eval: 100,
        ..Default::default()
    });
    c.bench_function("genetic_step_1k", |b| {
        b.iter(|| harness.step())
    });
}

criterion_group!(benches, bench_tick_single_world, bench_tick_batch, bench_genetic_step);
criterion_main!(benches);
```

### BS-6D: Performance profiling y tuning

1. **Profile con flamegraph:** `cargo flamegraph --bench batch_benchmark`.
2. **Identificar top 3 hotspots** — probablemente collision (N²) y trophic_predation.
3. **Optimizar:**
   - Collision: usar `alive_mask` bitops para skip rapidamente.
   - Trophic: pre-sort entities por position para early exit.
   - Culture: batch meme candidates, reduce comparisons.
4. **Memory bandwidth:** `perf stat` para verificar cache hit ratio.

### BS-6E: CI benchmark gate

```yaml
# .github/workflows/bench.yml (o CI equivalente)
# Gate: 1M mundos × 1 tick debe completar en < 20ms
# Measured on: [specify hardware baseline]
```

---

## Performance targets

| Metric | Target | Medido en |
|--------|--------|-----------|
| Single world tick (32 entities) | < 5 µs | M1/M2 Mac |
| 1K worlds tick | < 500 µs | 8 cores |
| 10K worlds tick | < 2 ms | 8 cores |
| 100K worlds tick | < 10 ms | 64 cores |
| **1M worlds tick** | **< 20 ms** | **64 cores** |
| Genetic step (1K worlds, 100 ticks) | < 100 ms | 8 cores |
| Memory: 1M worlds × 64 entities | < 15 GB | — |

---

## NO hace

- No implementa GPU compute — futuro post-BS-6.
- No modifica ecuaciones — solo optimiza como se llaman.
- No cambia la semantica de ningun system.

---

## Dependencias

- BS-5 — simulacion completa + bridge.
- `rayon 1.10` — unica dependencia externa nueva.
- `criterion 0.5` — dev-dependency para benchmarks.

---

## Criterios de aceptacion

### Performance
- **1M worlds × 1 tick < 20ms** en hardware con 64 cores (o extrapolacion lineal desde 8 cores).
- Single world tick < 10 µs.
- Zero allocations in hot loop (verified via custom allocator or `perf`).

### Correctness
- Resultados paralelos bit-identical a single-threaded (INV-B1 determinismo).
- `cargo test` sin regresion — mismos resultados con y sin rayon.

### Benchmarks
- `benches/batch_benchmark.rs` ejecutable con `cargo bench`.
- Flamegraph generado y hotspots documentados.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` §10 — performance targets
- rayon docs: https://docs.rs/rayon/latest/rayon/
- criterion docs: https://docs.rs/criterion/latest/criterion/
