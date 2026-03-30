# Track: BRIDGE_STRATEGY_DECOUPLING — Cache × Normalización × Estrategia

**Objetivo:** Desacoplar normalización de atención en el Bridge Optimizer. Crear mapa de estrategias con Concentration como default. Corregir bugs, eliminar código muerto, extender cache a todos los sistemas cacheables. TDD coherente.

**Estado:** ACTIVO (2026-03-30)
**Bloqueado por:** Nada (track independiente)
**Desbloquea:** Percepciones metafísicas (apostantes/tendencias), LOD por entidad, cache adaptativo

---

## Auditoría pre-sprint

| Categoría | Estado actual | Objetivo | Sprint |
|-----------|---------------|----------|--------|
| Normalización acoplada a Bridgeable | Hardcodeada por impl | Enum inyectable por config | BS-1 |
| CompetitionNormBridge sin lifecycle | BUG: no en context_fill ni metrics | ✅ FIXED (wired en 5 macros) | BS-2 |
| Hot reload no resetea BridgePhaseState | BUG: transiciones espurias | ✅ FIXED (reset en pre-Active) | BS-2 |
| ~~PerformanceCachePolicy~~ | ~~Código muerto~~ → activamente usado en shape inference | ~~Eliminar~~ CANCELADO | — |
| MetabolicGraph.total_entropy_rate | Campo derivado almacenado (Regla 13) | Compute on demand | BS-3 |
| BridgeMetrics 8 campos | Excede espíritu de max-4 | Split en 2 structs | BS-3 |
| BridgeLayerRow.recommendations: Vec\<String\> | Heap en struct de reporte | Enum flags | BS-3 |
| 6 sistemas sin cache (basal, senescence, awakening, radiation, shape, epigenetic) | Computan cada tick | Bridges nuevos | BS-4 |
| 18 archivos equations sin tests | 0 cobertura | Unit tests TDD | BS-5 |
| ~110 sistemas sin integration tests | 0 cobertura | MinimalPlugins tests tier 1 | BS-5 |
| shape_cache_signature inline | Bitwise ops en sistema | ✅ FIXED (extraído a equations/) | BS-3 |
| ~~expect() en ops.rs hot path~~ | ~~Panic posible~~ → solo en tests | FALSO POSITIVO | — |

---

## Sprints (7)

| Sprint | Descripción | Archivos tocados | Esfuerzo | Bloqueado por |
|--------|-------------|------------------|----------|---------------|
| **BS-1** | NormStrategy enum + desacople | bridge/{strategy,config,decorator,macros}.rs | M | — |
| **BS-2** ✅ | Bug fixes (CompetitionNorm, hot reload) | bridge/{context_fill,metrics,presets}.rs | S | — |
| **BS-3** ✅ parcial | Exact cache components + shape_cache extraction | equations/exact_cache.rs, layers/{kleiber,gompertz,converged}.rs | S | — |
| **BS-4** | Bridges nuevos (6 sistemas) | bridge/impls/, simulation/{metabolic,awakening,worldgen} | L | BS-1 |
| **BS-5** | TDD: tests unitarios + integración tier 1 | tests/, blueprint/equations/, simulation/ | L | BS-4 |
| **BS-6** | HOF composition para strategy stacking | bridge/strategy.rs | M | BS-1 |
| **BS-7** | Preset RON para estrategias + validación keys | bridge/presets/, assets/ | S | BS-6 |

**Esfuerzo:** S = <100 LOC, M = 100-300 LOC, L = 300+ LOC

---

## Dependencias

```
BS-1 ──→ BS-4 ──→ BS-5
  │
  └──→ BS-6 ──→ BS-7

BS-2 (independiente, hacer primero)
BS-3 (independiente, hacer primero)
```

**Orden óptimo:** BS-2 → BS-3 → BS-1 → BS-4 → BS-6 → BS-5 → BS-7

---

## Documentos

| Documento | Contenido |
|-----------|-----------|
| ~~[CACHE_STRATEGY_DESIGN.md](./CACHE_STRATEGY_DESIGN.md)~~ | ~~V1: 3 variantes por bloque con normalización~~ SUPERSEDED — referencia de investigación física |
| [CACHE_STRATEGY_DESIGN_V2.md](./CACHE_STRATEGY_DESIGN_V2.md) | **V2: Zero precision loss** — solo patrones que no pierden bits (precompute, dirty flag, memoización exacta, lookup table). 8 bloques, ~200 LOC. |
| [SPRINT_BS1](./SPRINT_BS1_NORM_STRATEGY_ENUM.md) | NormStrategy enum + desacople normalización |
| [SPRINT_BS2](./SPRINT_BS2_BUG_FIXES.md) | Bug fixes (CompetitionNorm, hot reload) |
| [SPRINT_BS3](./SPRINT_BS3_DOD_CLEANUP.md) | Limpieza DoD (campos derivados, math inline) |
| [SPRINT_BS4](./SPRINT_BS4_NEW_BRIDGES.md) | 6 bridges nuevos para sistemas sin cache |
| [SPRINT_BS5](./SPRINT_BS5_TDD_COVERAGE.md) | TDD: tests unitarios + integración tier 1 |
| [SPRINT_BS6](./SPRINT_BS6_HOF_COMPOSITION.md) | HOF composition (NormPipeline) |
| [SPRINT_BS7](./SPRINT_BS7_RON_PRESETS_VALIDATION.md) | RON presets + validación keys |
