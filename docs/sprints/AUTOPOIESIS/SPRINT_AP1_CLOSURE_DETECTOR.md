# Sprint AP-1: Autocatalytic Closure Detector — Encontrar RAF (Kauffman)

**ADR:** — (algoritmo puro, sin decisión arquitectónica nueva)
**Esfuerzo:** 1 semana
**Bloqueado por:** AP-0
**Desbloquea:** AP-2

## Contexto

Tener una red de reacciones no demuestra autopoiesis. Falta detectar si un subconjunto de reacciones es **RAF (Reflexively Autocatalytic and Food-generated)** — Kauffman 1986, refinado por Hordijk & Steel 2004:

> Un subconjunto R' ⊆ R es RAF si:
> 1. **Reflexivamente autocatalítico:** cada reacción r ∈ R' es catalizada por algún producto de R' (o por el food set F).
> 2. **Generado por food:** todo reactivo de toda r ∈ R' es producible desde F mediante reacciones en R'.

Es la formalización matemática del "bucle cerrado" del cap. 5 del paper.

## Principio

Estructura como grafo bipartito (reacciones ↔ especies). Algoritmo de punto-fijo de Hordijk-Steel:

```
1. R₀ ← R, S₀ ← all species
2. iterate:
   - eliminar reacciones cuyos reactivos no estén en (F ∪ products(R_i))
   - eliminar reacciones sin catalizador en (F ∪ products(R_i))
   - eliminar especies que no son producidas ni están en F
3. fixed point → RAF
```

Complejidad: O(R · S) por iteración, ≤R iteraciones → O(R² · S). Para R≤256, S≤32 es <2ms — apto para tick.

## Entregable

1. `find_raf(network: &ReactionNetwork, food: &[SpeciesId]) → Vec<ReactionId>` — pure fn
2. `raf_closures(network, food) → Vec<Closure>` — descompone RAF en SCCs (Tarjan) si hay múltiples ciclos disjuntos
3. `Closure { reactions: Vec<ReactionId>, species: Vec<SpeciesId>, hash: u64 }` — para tracking inter-tick
4. `closure_detection_system` — corre cada N=100 ticks (no hot path), escribe `Resource<DetectedClosures>`
5. `food_set_from_grid(cell, threshold) → Vec<SpeciesId>` — inferir food del estado actual

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `Closure` struct + hash estable | `src/layers/closure.rs` | 3 |
| 2 | `find_raf` Hordijk-Steel | `src/blueprint/equations/raf.rs` | 8 |
| 3 | `raf_closures` SCC decomposition | `src/blueprint/equations/raf.rs` | 5 |
| 4 | `food_set_from_grid` | `src/blueprint/equations/raf.rs` | 3 |
| 5 | `DetectedClosures` resource | `src/resources/detected_closures.rs` | 1 |
| 6 | `closure_detection_system` (every 100 ticks) | `src/simulation/chemical/closure_detection.rs` | 3 integration |

## Criterios de aceptación

- [ ] `raf_minimal.ron` (3 reacciones cíclicas) → 1 closure detectada con `len=3`
- [ ] Red sin ciclo → 0 closures
- [ ] Red con 2 ciclos disjuntos → 2 closures distintas
- [ ] Hash estable: misma red en orden distinto → mismo hash
- [ ] Determinismo: misma seed → mismas closures
- [ ] Performance: red de 256 reacciones < 5ms (criterion bench)
- [ ] Property test: si `find_raf(R, F) = R'`, entonces todo r ∈ R' tiene reactivos producibles desde F via R'
