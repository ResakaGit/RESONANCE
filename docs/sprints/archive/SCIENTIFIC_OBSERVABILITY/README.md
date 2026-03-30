# Track: SCIENTIFIC_OBSERVABILITY

**Estado:** ✅ ARCHIVADO (2026-03-30) — 5/5 sprints completados. 32 tests. 0 DEBT.

Los documentos `SPRINT_SO1`…`SO5` se eliminaron al cerrar el track; la especificación queda en los comments de cada archivo fuente y en `CLAUDE.md`.

---

## Entregables

| Sprint | Entregable | Archivo | Tests |
|--------|-----------|---------|-------|
| SO-1 | `LineageId` + `TrackedGenome` (FNV-1a u64, zero float truncation) | `src/batch/lineage.rs` | 10 |
| SO-2 | `EntitySnapshot` + `PopulationCensus` (alive_mask capture, HOF distribution/mean) | `src/batch/census.rs` | 8 |
| SO-3 | `write_entity_csv`, `write_generation_csv` (zero-alloc), `export_history_csv`, JSON | `src/use_cases/export.rs` | 9 |
| SO-4 | `ablate(closure)`, `ensemble()`, `sweep(2 closures)`, `aggregate_ensemble` (pura) | `src/use_cases/orchestrators.rs` | 5 |
| SO-5 | `--out` CSV en `fermi.rs`, `cancer_therapy.rs`, `convergence.rs` | `src/bin/` | — |

## Arquitectura

```
Binaries (--out flag) → HOF Orchestrators → Export Adapters → Census → Lineage → GeneticHarness
```

Cada capa solo importa la inferior. Stateless. Zero IO en adapters. Zero precision loss.
