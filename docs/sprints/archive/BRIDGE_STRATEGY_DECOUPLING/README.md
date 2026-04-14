# Track: BRIDGE_STRATEGY_DECOUPLING — Cache × Normalización × Estrategia

**Objetivo:** Desacoplar normalización de atención en el Bridge Optimizer. Cache exacto sin pérdida de precisión. TDD coherente.

**Estado:** ✅ COMPLETADO (2026-04-13) — 3/7 completados, 4 cancelados. ADR-017.
**Bloqueado por:** Nada (track independiente)
**Desbloquea:** Performance (powf/exp elimination), convergence skip (epigenetics, shape)

---

## Completado (archivado en código)

| Sprint | Entregable | Archivo |
|--------|-----------|---------|
| BS-2 ✅ | CompetitionNormBridge wired (5 macros), hot reload phase reset | `bridge/{context_fill,metrics,presets}.rs` |
| BS-3 ✅ | `exact_cache.rs` (kleiber, gompertz, alignment), KleiberCache, GompertzCache, Converged\<T\>, shape_cache_signature extraída | `equations/exact_cache.rs`, `layers/{kleiber,gompertz,converged}.rs` |

---

## Reestructurado (ADR-017, 2026-04-12)

CACHE_STRATEGY_DESIGN_V2 demostró que solo 1/8 bloques se beneficia del bridge cache.
Los otros 7 se optimizan con dirty flags, precompute y convergence detection directos.

| Sprint | Estado | Razón |
|--------|--------|-------|
| BS-1 | **Cancelado** | `BridgeConfig.enabled` ya cubre Passthrough vs Concentration |
| BS-4 | **Cancelado** | Reemplazado por integración directa (ADR-017: 4 cambios) |
| BS-5 | **Completado** | 5 tests de integración (kleiber×radii, gompertz×exact, gompertz×boundary, converged×invalidation, converged×edge) |
| BS-6 | **Cancelado** | NormPipeline sin razón de ser — V2 eliminó variantes |
| BS-7 | **Cancelado** | Dependía de BS-6 |

### ADR-017: 4 cambios concretos

```
1. KleiberCache  → basal_drain_system     (elimina powf per-tick)
2. GompertzCache → senescence_death_system (elimina exp per-tick)
3. Converged<EpigeneticState>  → epigenetic_adaptation_system (skip 85%)
4. Converged<ShapeParams>      → shape_optimization_system    (skip 90%)
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
