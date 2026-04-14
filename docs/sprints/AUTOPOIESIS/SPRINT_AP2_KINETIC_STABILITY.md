# Sprint AP-2: Kinetic Stability — Reconstrucción vs decay (Pross)

**ADR:** —
**Esfuerzo:** 0.5 semanas
**Bloqueado por:** AP-1
**Desbloquea:** AP-5

## Contexto

Detectar una RAF no implica que persista. Addy Pross (cap. 5 del paper) define **estabilidad dinámica cinética**:

> Algo es estable porque se reconstruye más rápido de lo que se destruye.

Es la métrica que distingue una llama de una piedra, y una célula de un cristal. Sin esto, AP-5 no tiene criterio cuantitativo.

## Principio

```
Para cada Closure C en celda:
  reconstruction_rate = Σ_r∈C rate_r × stoich_out
  decay_rate         = Σ_s∈C [s] × DISSIPATION_LIQUID
  kinetic_stability  = reconstruction_rate / max(decay_rate, ε)

Estado:
  K > 1.0  → persistente (se reconstruye más rápido que decae)
  K ≈ 1.0  → metaestable
  K < 1.0  → declinante (extinción inminente)
```

`K` es la constante de Pross. Pure fn.

## Entregable

1. `kinetic_stability(closure: &Closure, species: &[f32], network: &ReactionNetwork, freq: f32) → f32` — pure fn
2. `ClosureMetrics { id_hash, k_stability, age_ticks, total_qe }` — Component (SparseSet)
3. `closure_metrics_system` — every 10 ticks, escribe métricas por closure detectada
4. Histórico circular (ring buffer, N=128 ticks) por closure para detectar tendencia

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `kinetic_stability` pure fn | `src/blueprint/equations/raf.rs` | 6 |
| 2 | `ClosureMetrics` component | `src/layers/closure.rs` | 2 |
| 3 | Ring buffer `KStabilityHistory` | `src/layers/closure.rs` | 3 |
| 4 | `closure_metrics_system` | `src/simulation/chemical/closure_metrics.rs` | 2 integration |

## Criterios de aceptación

- [ ] Closure con k=1.0, food abundante → K > 1.0 estable
- [ ] Closure aislada del food → K decae monotónicamente a 0 en O(1/DISSIPATION_LIQUID) ticks
- [ ] Closure con catálisis frequency-aligned → K mayor que misma closure desalineada
- [ ] Property: K es invariante a permutaciones de reacciones en C (orden no importa)
- [ ] Pure fn: zero allocations, zero side effects
