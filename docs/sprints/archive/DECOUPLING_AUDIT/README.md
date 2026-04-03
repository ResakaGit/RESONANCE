# Track: DECOUPLING_AUDIT — Architectural Coupling Elimination

**Objetivo:** Eliminar todos los acoplamientos arquitectónicos detectados en la auditoría exhaustiva del 2026-04-01. Restablecer la pureza de `blueprint/equations/`, la ortogonalidad de las 14 capas, y las fronteras entre simulation/worldgen/rendering.

**Estado:** ✅ ARCHIVADO (parcial, 2026-04-01) — 3/5 sprints implementados y validados (DC-1, DC-3, DC-4). 2 pendientes (DC-2, DC-5).
**Bloqueado por:** Nada (track independiente)
**Desbloquea:** Mantenibilidad a largo plazo, onboarding de contribuidores, reducción de blast radius en refactors futuros

---

## Auditoría post-implementación

| Categoría | Antes | Después | Estado |
|-----------|-------|---------|--------|
| Equations importing layers/ (enums) | 41 | **0** | ✅ DC-1 |
| Systems >5 components | 4 | 4 | Pendiente (DC-2) |
| Inline math in systems | 8 | **0** | ✅ DC-4C |
| Worldgen controlling sim state | 2 transitions | **0** | ✅ DC-3 |
| Rendering→simulation::sensory import | 1 | **0** | ✅ DC-4B |
| terrain_blocks_vision in simulation/ | 1 def | **0** (movido a equations/) | ✅ DC-4A |
| EnergyFieldGrid direct mutation from sim | 6+ systems | 6+ | Pendiente (DC-5) |
| Worldgen chaining sim systems | 6 systems | 6 | Pendiente (DC-5) |
| Hardcoded 0.016 dt | 1 | **0** (usa time.delta_secs) | ✅ Auditoría |
| Dead code (orphaned spawns, unused fns) | 7 items | **0** | ✅ Auditoría |
| COMMENSALISM_INTAKE hardcoded | 1 | **0** (derivado de DISSIPATION_SOLID) | ✅ Auditoría |
| Element band constants scattered | 18+ | **centralizadas** (element_bands.rs) | ✅ Auditoría |
| Cargo warnings | 4 | **0** | ✅ Auditoría |

**Resultado validado:** `cargo test` = 3,113 passed, 0 failed. `cargo check` = 0 warnings.

---

## 5 Sprints

| Sprint | Descripción | Archivos | Esfuerzo | Estado |
|--------|-------------|----------|----------|--------|
| [DC-1](SPRINT_DC1_DOMAIN_ENUM_EXTRACTION.md) | Extraer enums de dominio (MatterState, OrganRole, TrophicClass, LifecycleStage) de `layers/` a `blueprint/` | ~25 archivos | Medio | ✅ |
| [DC-2](SPRINT_DC2_SHAPE_INFERENCE_DECOMPOSITION.md) | Descomponer `entity_shape_inference_system` (15→3×5 componentes) | 4 archivos | Medio | Pendiente |
| [DC-3](SPRINT_DC3_STATE_REPATRIATION.md) | Repatriar transiciones GameState/PlayState de worldgen a simulation | 5 archivos | Bajo | ✅ |
| [DC-4](SPRINT_DC4_PURE_MATH_BOUNDARY.md) | Mover `terrain_blocks_vision` a equations, desacoplar rendering↔AttentionGrid, extraer inline math | 12 archivos | Medio | ✅ |
| [DC-5](SPRINT_DC5_SIM_WORLDGEN_BOUNDARY.md) | Encapsular mutaciones de EnergyFieldGrid, extraer system chaining de worldgen | 8 archivos | Alto | Pendiente |

**Implementados: DC-1, DC-3, DC-4 (Wave 0 + Wave 1). Pendientes: DC-2 (shape decomp), DC-5 (sim↔worldgen boundary).**

---

## Grafo de dependencias

```
DC-1 (enums)  ───────┬──→  DC-2 (shape decomp)
                     └──→  DC-4 (math boundary)
                                    │
DC-3 (state repatriation) ──→  DC-5 (sim↔worldgen boundary)
```

## Matriz de paralelismo

| | DC-1 | DC-2 | DC-3 | DC-4 | DC-5 |
|---|---|---|---|---|---|
| **DC-1** | — | blocks | — | blocks | — |
| **DC-2** | blocked | — | parallel | parallel | parallel |
| **DC-3** | parallel | parallel | — | parallel | blocks |
| **DC-4** | blocked | parallel | parallel | — | parallel |
| **DC-5** | parallel | parallel | blocked | parallel | — |

**Ondas de ejecución:**
- **Wave 0:** DC-1 + DC-3 (paralelos, zero conflicto — distintos archivos)
- **Wave 1:** DC-2 + DC-4 (paralelos entre sí, ambos post-DC-1)
- **Wave 2:** DC-5 (post-DC-3, puede solapar con Wave 1 si DC-3 ya cerró)

---

## Invariantes del track

1. **Zero test regression.** Cada sprint debe pasar `cargo test` completo antes de merge. Si un test falla, el sprint no cierra — se arregla el test o se revisa el diseño.
2. **Zero nueva deuda.** No se introduce `// DEBT:` como parte de la solución. Si el diseño requiere deuda, se replantea.
3. **Backward-compatible imports.** DC-1 usa `pub use` re-exports para que ningún consumidor externo cambie su import path. Los viejos paths siguen compilando. Se eliminan en un commit separado al final.
4. **No trait objects en hot paths.** Si la indirección cuesta rendimiento medible (>1% en bench), se usa enum dispatch o generics.
5. **Stateless first.** Toda función nueva es pura (input→output, sin side effects). Los sistemas ECS son el único lugar donde se muta estado.
6. **HOFs para orquestación.** Donde hay composición de comportamiento, se usa `Fn(Input) -> Output` como parámetro, no herencia ni trait objects.

---

## Estrategia de testing (3 capas)

### Capa 1: Unitario (pure functions)

```
blueprint/equations/  — cada fn tiene #[cfg(test)] mod tests
                       — inputs extremos: qe=0, radius=0, frequency=0
                       — property: output ∈ [expected_min, expected_max]
                       — determinismo: f(x) == f(x) siempre
```

**Patrón:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_blocks_vision_same_cell_returns_false() {
        let terrain = mock_flat_terrain(10, 10, 0.0);
        assert!(!terrain_blocks_vision(Vec2::ZERO, Vec2::ZERO, &terrain));
    }

    #[test]
    fn terrain_blocks_vision_hill_between_returns_true() {
        let mut terrain = mock_flat_terrain(10, 10, 0.0);
        terrain.set_altitude(5, 5, 100.0); // Hill
        assert!(terrain_blocks_vision(Vec2::new(0.0, 0.0), Vec2::new(9.0, 9.0), &terrain));
    }
}
```

### Capa 2: Integración (systems aislados)

```
simulation/*/tests/  — MinimalPlugins app
                     — spawn SOLO componentes necesarios
                     — UN update()
                     — assert delta en componentes output
```

**Patrón:**
```rust
#[test]
fn cache_validation_system_skips_on_hit() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, shape_cache_validation_system);

    let entity = app.world_mut().spawn((
        HasInferredShape,
        ShapeInferred,
        PerformanceCachePolicy::stable(0xABCD),
    )).id();

    app.update(); // Should NOT trigger rebuild

    let policy = app.world().get::<PerformanceCachePolicy>(entity).unwrap();
    assert_eq!(policy.dependency_signature, 0xABCD, "Signature unchanged = cache hit");
}
```

### Capa 3: Orquestación (pipeline end-to-end)

```
tests/integration/  — full pipeline con phases registradas
                    — spawn entidad completa (via archetypes)
                    — N updates (steady-state)
                    — assert: conservación, ordering, no panics
```

**Patrón HOF para test harness:**
```rust
/// HOF: ejecuta un test de pipeline con configuración inyectada.
fn run_pipeline_test<F, A>(setup: F, assert_fn: A)
where
    F: FnOnce(&mut App),
    A: FnOnce(&World),
{
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    setup(&mut app);
    for _ in 0..10 { app.update(); }
    assert_fn(app.world());
}

#[test]
fn shape_pipeline_produces_mesh_for_mobile_entity() {
    run_pipeline_test(
        |app| { spawn_animal_demo(app.world_mut()); },
        |world| {
            let count = world.query_filtered::<&ShapeInferred, With<HasInferredShape>>()
                .iter(world).count();
            assert!(count > 0, "At least one entity should have inferred shape");
        },
    );
}
```

---

## Patrones de programación aplicados

| Patrón | Cuándo | Justificación de complejidad |
|--------|--------|------------------------------|
| **Newtype re-export** (`pub use`) | DC-1: migración de enums | Zero-cost abstraction. Permite migrar sin romper consumidores. Se elimina cuando todos los imports apunten al nuevo path. |
| **SystemParam adapter** | DC-2: queries compuestos | Encapsula queries multi-componente detrás de una interfaz tipada. Reduce query width visible. Vale la pena cuando >3 sistemas comparten el mismo patrón de acceso. |
| **HOF orchestrator** | DC-2, DC-5: composición de sistemas | `Fn(Input) -> Output` como parámetro permite testear cada stage aislado y componer en pipeline. Vale la pena vs. un god-system monolítico. |
| **Contract Resource** | DC-4: AttentionGrid | Resource intermedia que rendering lee sin importar de simulation. Ownership claro: simulation escribe, rendering lee. Complejidad mínima (un struct). |
| **Facade method** | DC-5: EnergyFieldGrid API | `grid.register_entity(cx, cy, id)` en vez de acceso directo a `cell_mut().field = val`. Encapsula invariantes. Vale la pena cuando >3 sistemas mutan el mismo grid. |
| **State machine extraction** | DC-3: lifecycle de estados | Mover transiciones a un solo owner (simulation plugin). Reduce surface area de bugs de state. |

---

## Criterios de cierre del track

- [x] `cargo test` — 3,113 passed, 0 failures, 0 warnings
- [x] `grep -r "use crate::layers::" src/blueprint/equations/` — **0 resultados** (DC-1)
- [ ] `entity_shape_inference_system` — max 5 componentes por query (DC-2 pendiente)
- [x] `worldgen/systems/startup.rs` — **0 imports de `simulation::states` en producción** (DC-3)
- [x] `rendering/quantized_color/systems.rs` — **0 imports de `simulation::sensory`** (DC-4B)
- [ ] `grep "\.cell_xy_mut\|\.cell_mut" src/simulation/` — 39 accesos directos (DC-5 pendiente)
- [x] Ningún `// DEBT:` introducido por este track

---

## Archivos creados por este track

| Archivo | Sprint | Propósito |
|---------|--------|-----------|
| `src/blueprint/domain_enums.rs` | DC-1 | Source of truth: MatterState, OrganRole, TrophicClass, LifecycleStage, GeometryPrimitive, MAX_ORGANS_PER_ENTITY |
| `src/simulation/lifecycle/state_transitions.rs` | DC-3 | enter_game_state_playing_system + transition_to_active_system (5 tests) |
| `src/worldgen/contracts.rs::WorldgenReady` | DC-3 | Resource señalizadora post-warmup |
| `src/blueprint/equations/vision.rs` | DC-4A | terrain_blocks_vision + raycast_cells_exclusive (5 tests) |
| `src/runtime_platform/contracts/mod.rs::AttentionGrid` | DC-4B | Contract resource (sim escribe, rendering lee) |
| `src/blueprint/equations/core_physics/::density_from_qe_radius` | DC-4C | Función pura de densidad |
| `src/blueprint/equations/emergence/epigenetics::gene_expression_threshold` | DC-4C | Umbral epigenético derivado (2 tests) |
| `src/blueprint/constants/element_bands.rs` | Auditoría | FREQ_TERRA/IGNIS/etc + BAND_TERRA/IGNIS/etc centralizadas |

## Correcciones adicionales (auditoría post-implementación)

| Fix | Archivo | Impacto |
|-----|---------|---------|
| Hardcoded 0.016 dt → `time.delta_secs()` | `epigenetic_adaptation.rs` | Timestep decoupling restaurado |
| COMMENSALISM = DISSIPATION_SOLID | `symbiosis_effect.rs` | Derivado de constante fundamental |
| MUTUALISM = 2× DISSIPATION_SOLID | `symbiosis_effect.rs` | Derivado de constante fundamental |
| Dead fn `dimension_base_frequency` | `pathway_inhibitor_exp.rs` | Eliminada |
| Dead spawns: dummy, tension, adaptive, lava_knight | `heroes.rs`, `world_entities.rs` | Eliminados (zero consumers) |
| Dead constants: EMERGENT_INITIAL_RADIUS etc. | `abiogenesis/constants.rs` | Eliminados |
| 4 cargo warnings eliminated | `pathway_inhibitor_exp.rs` | 0 warnings total |

---

## Archivos clave (referencia rápida)

| Módulo | Archivos principales | Sprints que lo tocan |
|--------|---------------------|---------------------|
| `blueprint/equations/` | 45+ domain files | DC-1, DC-4 |
| `layers/coherence.rs` | MatterState definition | DC-1 |
| `layers/organ.rs` | OrganRole, LifecycleStage | DC-1 |
| `layers/inference.rs` | TrophicClass | DC-1 |
| `simulation/lifecycle/entity_shape_inference.rs` | 15-component system | DC-2 |
| `simulation/thermodynamic/physics.rs` | terrain_blocks_vision | DC-4 |
| `simulation/thermodynamic/sensory.rs` | AttentionGrid | DC-4 |
| `rendering/quantized_color/systems.rs` | factor_precision_system | DC-4 |
| `worldgen/systems/startup.rs` | GameState/PlayState transitions | DC-3 |
| `worldgen/systems/prephysics.rs` | 6-system chaining | DC-5 |
| `plugins/simulation_plugin.rs` | Startup chain registration | DC-3, DC-5 |
