# Track — Energy Competition (EC)

**Estado:** **EC-1–EC-8 cerrados** (2026-03; ver `src/blueprint/equations/energy_competition/` y `src/simulation/metabolic/`).

Los documentos `SPRINT_EC1`…`EC8` se **eliminaron** al cerrar el track.

## Qué implementa

Pools de energía jerárquicos con 5 tipos de extracción (Proporcional, Greedy, Competitiva, Agresiva, Regulada). Conservación estricta `Σ extraído ≤ disponible` por tick. Dinámicas Lotka-Volterra emergentes sin lógica explícita. Escala-invariante (Matryoshka): célula, órgano, organismo, población usan la misma mecánica.

## Módulos en código

- `src/blueprint/equations/energy_competition/` — ecuaciones puras (pool, extracción, dinámica, escala)
- `src/simulation/metabolic/pool_distribution.rs` — EC-4: intake → distribución → disipación
- `src/simulation/metabolic/pool_conservation.rs` — EC-6: ledger de conservación post-tick
- `src/simulation/metabolic/competition_dynamics.rs` — EC-5: `PoolDiagnostic`, matriz de competencia
- `src/simulation/metabolic/scale_composition.rs` — EC-7: fitness inferido cross-scale
- `src/layers/pool_ledger.rs` — `PoolConservationLedger` (SparseSet)
- `src/world/demos/competition_arena.rs` — demo startup (EC-8)
- `assets/maps/competition_arena.ron` — mapa 32×32 con 3 nuclei (EC-8)
- `tests/energy_competition_integration.rs` — 6 acceptance tests (EC-8)
- `benches/energy_competition_bench.rs` — benchmarks pool distribution (EC-8)

**Backlog:** ninguno → [`../README.md`](../README.md).
