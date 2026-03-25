# D0: Reparaciones — Completar Systems Parciales

**Prioridad**: P0 (antes de todo lo demás)
**Estimación**: 12 systems a completar/verificar

---

## Inventario de Parciales

| # | System | Archivo | Estado | Qué Falta |
|---|--------|---------|--------|-----------|
| 1 | `irradiance_update_system` | photosynthesis.rs | Throttled OK | Verificar cobertura: ¿procesa TODOS los receivers en N/128 frames? |
| 2 | `osmotic_diffusion_system` | osmosis.rs | Throttled OK | Verificar: ¿respeta LOD correctamente? ¿frequency mixing es estable? |
| 3 | `catalysis_math_strategy_system` | reactions.rs | Logic continues | Leer completo, verificar que TODOS los AbilityOutput variants están cubiertos |
| 4 | `entropy_ledger_system` | morphogenesis.rs | Partial | Completar materialización de EntropyLedger desde DAG output |
| 5 | `shape_optimization_system` | morphogenesis.rs | MG-4 partial | Verificar bounded_fineness_descent converge. Test con extremos |
| 6 | `surface_rugosity_system` | morphogenesis.rs | MG-7 partial | Verificar Q/V ratio → rugosity → detail_multiplier pipeline |
| 7 | `albedo_inference_system` | morphogenesis.rs | MG-5 partial | Verificar irradiance_effective → albedo_luminosity_blend |
| 8 | `evolution_surrogate_enqueue_system` | evolution_surrogate.rs | Complex | Verificar multi-scenario evaluation. ¿Deterministic? |
| 9 | `evolution_surrogate_tick_system` | evolution_surrogate.rs | Complex | Verificar fitness aggregation. ¿Converge? |
| 10 | `reproduction_spawn_system` | reproduction.rs | Throttled | Verificar: radius threshold, mutation, offspring placement |
| 11 | `abiogenesis_system` | abiogenesis.rs | Throttled | Verificar: scoring, phenotype inference, spawn placement |
| 12 | `pathfinding_compute_system` | pathfinding/systems.rs | Threaded | Verificar: navmesh invalidation, path caching |

---

## Componentes — Estado Verificado (Post-Audit Cruzado)

**RESULTADO: 0 componentes huérfanos.** El audit inicial fue falso positivo.

| # | Component | Layer | Estado Real | System Activo |
|---|-----------|-------|-------------|---------------|
| 1 | `AlchemicalForge` | L5 | ACTIVO | Data component con métodos (`creation_efficiency()`, `discover()`, `master()`). Accedido por lógica de grimoire. |
| 2 | `TensionField` | L11 | ACTIVO | `tension_field_system` en `structural_runtime.rs:98` — acumula aceleraciones gravitatorias/magnéticas. Phase::AtomicLayer. |
| 3 | `Homeostasis` | L12 | ACTIVO | `homeostasis_system` en `structural_runtime.rs:184` — adapta frecuencia hacia pressure del host. Emite `HomeostasisAdaptEvent`. Phase::ChemicalLayer. |
| 4 | `VisionBlocker` | L9 aux | ACTIVO | Marker de filtro ECS en `fog_of_war.rs:68` — `Without<VisionBlocker>` excluye entidades que bloquean visión. |
| 5 | `PerformanceCachePolicy` | aux | PLANNED | Declara contrato operativo para sistemas V5. Sin system activo aún. Solo tests. Mantener. |

---

## Protocolo de Reparación

Para cada system parcial:

1. **Leer** el archivo completo (no solo primeras líneas)
2. **Identificar** qué falta: ¿lógica incompleta? ¿branches sin cubrir? ¿TODO comments?
3. **Escribir** test que exponga el gap
4. **Completar** la implementación
5. **Verificar** con checklist completo

Para componentes huérfanos:

1. **Grep** `Query<.*ComponentName` en toda la base de código
2. Si 0 resultados → confirmar orphan
3. Decidir: implementar o eliminar
4. Si eliminar: remover de LayersPlugin, EntityBuilder, archetypes
