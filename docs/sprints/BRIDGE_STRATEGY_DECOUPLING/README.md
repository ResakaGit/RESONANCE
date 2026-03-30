# Track: BRIDGE_STRATEGY_DECOUPLING — Cache × Normalización × Estrategia

**Objetivo:** Desacoplar normalización de atención en el Bridge Optimizer. Cache exacto sin pérdida de precisión. TDD coherente.

**Estado:** ACTIVO (2026-03-30) — 2/7 completados, 5 pendientes.
**Bloqueado por:** Nada (track independiente)
**Desbloquea:** Percepciones metafísicas, LOD por entidad, cache adaptativo

---

## Completado (archivado en código)

| Sprint | Entregable | Archivo |
|--------|-----------|---------|
| BS-2 ✅ | CompetitionNormBridge wired (5 macros), hot reload phase reset | `bridge/{context_fill,metrics,presets}.rs` |
| BS-3 ✅ | `exact_cache.rs` (kleiber, gompertz, alignment), KleiberCache, GompertzCache, Converged\<T\>, shape_cache_signature extraída | `equations/exact_cache.rs`, `layers/{kleiber,gompertz,converged}.rs` |

---

## Pendiente

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| BS-1 | NormStrategy enum + desacople | M | — |
| BS-4 | 6 bridges nuevos (basal, senescence, awakening, rad, shape, epi) | L | BS-1 |
| BS-5 | TDD: tests unitarios + integración tier 1 | L | BS-4 |
| BS-6 | HOF composition (NormPipeline) | M | BS-1 |
| BS-7 | RON presets para estrategias + validación keys | S | BS-6 |

```
BS-1 ──→ BS-4 ──→ BS-5
  │
  └──→ BS-6 ──→ BS-7
```

---

## Documentos

| Documento | Contenido |
|-----------|-----------|
| [CACHE_STRATEGY_DESIGN_V2.md](./CACHE_STRATEGY_DESIGN_V2.md) | Zero precision loss: precompute, dirty flag, memoización exacta, lookup table |
| [SPRINT_BS1](./SPRINT_BS1_NORM_STRATEGY_ENUM.md) | NormStrategy enum + desacople normalización |
| [SPRINT_BS4](./SPRINT_BS4_NEW_BRIDGES.md) | 6 bridges nuevos para sistemas sin cache |
| [SPRINT_BS5](./SPRINT_BS5_TDD_COVERAGE.md) | TDD: tests unitarios + integración tier 1 |
| [SPRINT_BS6](./SPRINT_BS6_HOF_COMPOSITION.md) | HOF composition (NormPipeline) |
| [SPRINT_BS7](./SPRINT_BS7_RON_PRESETS_VALIDATION.md) | RON presets + validación keys |
