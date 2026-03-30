# Track: SCIENTIFIC_OBSERVABILITY — Infraestructura para los 9 Use Cases

**Objetivo:** Construir la capa de observabilidad que habilita TODOS los use cases científicos sin tocar la simulación. Lineage tracking, time-series export, HOF orchestrators, ensemble statistics. Zero impacto en precisión. Zero acoplamiento con el motor.

**Estado:** ACTIVO (2026-03-30)
**Bloqueado por:** Nada
**Desbloquea:** B1 Paper Machine, B2 Speciation, B3 Cambrian, B4 Debate, Epidemiology, Astrobiology

---

## Principio rector

```
La observabilidad es ORTOGONAL a la simulación.
No modifica un tick. No altera un bit de output.
Solo OBSERVA, REGISTRA, y COMPONE.
```

Los 9 use cases no necesitan código de simulación nuevo — necesitan **lentes** sobre lo que ya existe.

---

## Auditoría: Qué existe vs qué falta

| Capacidad | Estado | Gap |
|-----------|--------|-----|
| Correr evolución | ✅ `GeneticHarness` | — |
| Fitness por generación | ✅ `GenerationStats` | — |
| Top genomes | ✅ `top_genomes()` | Solo top N, no distribución |
| Save/load genomes binario | ✅ `bridge::save/load` | Sin metadata (gen, lineage) |
| Determinismo | ✅ INV-4 verified | — |
| HOF experiments | ✅ `evolve_with`, `compare_universes` | Sin ablation, sin ensemble |
| **Time-series per-entity** | ❌ | **CRITICAL** — no hay snapshots poblacionales |
| **Lineage tracking** | ❌ | **CRITICAL** — parentesco se pierde en repopulación |
| **CSV/JSON export** | ❌ | **CRITICAL** — solo binario y terminal |
| **Ensemble statistics** | ❌ | Comparación manual de reports |
| **Parameter ablation HOF** | ❌ | Loop manual por config |
| **World state snapshots** | ❌ | `EntitySlot` es repr(C) pero sin serialización |

---

## Sprints (5)

| Sprint | Descripción | Esfuerzo | Bloqueado por | Estado |
|--------|-------------|----------|---------------|--------|
| **SO-1** | Lineage tracking (LineageId + TrackedGenome) | S | — | ✅ 10 tests |
| **SO-2** | Population snapshot (EntitySnapshot + PopulationCensus) | M | SO-1 | ✅ 8 tests |
| **SO-3** | Export pipeline (CSV + JSON stateless adapters) | M | SO-2 | ✅ 9 tests |
| **SO-4** | HOF orchestrators (ablate, ensemble, sweep) | M | SO-3 | ✅ 5 tests |
| **SO-5** | CSV export integrado en binaries (fermi, cancer, convergence) | M | SO-4 | ✅ 3 binaries wired |

**Esfuerzo:** S = <100 LOC, M = 100-300 LOC, L = 300+ LOC
**Total estimado:** ~800 LOC + ~200 LOC tests

---

## Dependencias

```
SO-1 ──→ SO-2 ──→ SO-3 ──→ SO-4 ──→ SO-5
```

Cadena lineal. Cada sprint produce la herramienta que el siguiente consume.

---

## Arquitectura: 3 capas ortogonales

```
┌─────────────────────────────────────────────────┐
│  SO-5: Use-Case Binaries                        │
│  fermi.rs · speciation.rs · cancer.rs           │
│  epidemiology.rs · convergence.rs               │
│  (src/bin/ — orquestadores de alto nivel)        │
├─────────────────────────────────────────────────┤
│  SO-4: HOF Orchestrators                        │
│  ablate() · ensemble() · sweep() · compare()    │
│  (src/use_cases/orchestrators.rs — composición)  │
├─────────────────────────────────────────────────┤
│  SO-3: Export Adapters                          │
│  CsvWriter · JsonWriter · BinaryWriter          │
│  (src/use_cases/export.rs — stateless writers)   │
├─────────────────────────────────────────────────┤
│  SO-2: Population Snapshot                      │
│  PopulationCensus · EntitySnapshot              │
│  (src/batch/census.rs — per-gen capture)         │
├─────────────────────────────────────────────────┤
│  SO-1: Lineage Tracking                         │
│  LineageId · ParentRef                          │
│  (src/batch/lineage.rs — parentesco en genome)   │
├─────────────────────────────────────────────────┤
│  EXISTING: GeneticHarness · WorldBatch · arena  │
│  (src/batch/ — no se modifica, solo se extiende) │
└─────────────────────────────────────────────────┘
```

**Regla:** Cada capa solo importa la inferior. Nunca al revés. Nunca lateral.

---

## Documentos

| Documento | Contenido |
|-----------|-----------|
| [SPRINT_SO1](./SPRINT_SO1_LINEAGE.md) | Lineage tracking + parent metadata |
| [SPRINT_SO2](./SPRINT_SO2_CENSUS.md) | Population snapshot per generation |
| [SPRINT_SO3](./SPRINT_SO3_EXPORT.md) | CSV/JSON stateless export adapters |
| [SPRINT_SO4](./SPRINT_SO4_ORCHESTRATORS.md) | HOF orchestrators (ablation, ensemble, sweep) |
| [SPRINT_SO5](./SPRINT_SO5_USE_CASE_BINARIES.md) | 5 binarios científicos |
