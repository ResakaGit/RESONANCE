# Sprint Q5 — Partir SimulationPlugin

**Tipo:** Refactor — modularizacion.
**Severidad:** MEDIA — god plugin con 40+ sistemas, acoplamiento cruzado.
**Onda:** A — Sin dependencias duras.

## Objetivo

Descomponer `SimulationPlugin` (~227 lineas en snapshot actual, 40+ sistemas en el schedule, 20+ resources en `build`, 9 eventos en `build`) en sub-plugins por subsistema, donde cada modulo registra sus propios sistemas.

## Hallazgo

**Archivo:** `src/plugins/simulation_plugin.rs` (orquestador + `register_simulation_pipeline` + `register_visual_pipeline`)

### Problema actual (resumen fiel al codigo)

```text
SimulationPlugin::build(&self, app):
  // Estado sim / worldgen / almanaque / grid / caches (lista larga: ver archivo)
  app.init_resource::<V6RuntimeConfig>()
  app.init_resource::<SpatialIndex>()
  app.init_resource::<CatalysisStrikeScratch>()
  // ... + insert_resource EnergyFieldGrid, init_asset ElementDef, etc.

  // 9 eventos (ejemplos reales)
  app.add_event::<CollisionEvent>()
  app.add_event::<CatalysisEvent>()
  app.add_event::<WorldgenMutationEvent>()
  // ... ver lineas 47-55 en el archivo fuente

  // Pipeline: register_simulation_pipeline (~110 lineas) + Startup chain + visual en Update
  register_simulation_pipeline(app, FixedUpdate);
  register_visual_pipeline(app);
```

**Problemas:**
- Conoce internals de `physics`, `reactions`, `worldgen`, `element_layer2`, `structural_runtime`, `pre_physics`, `containment`.
- Cualquier refactor en un subsistema requiere tocar este archivo.
- Imposible desactivar un subsistema sin comentar lineas.
- Nuevo desarrollador no sabe donde empieza/termina cada subsistema.

## Responsabilidades

### Nuevos sub-plugins (o funciones `register_*`)

| Sub-plugin | Responsabilidad | Resources | Sistemas (nombres reales en `simulation_plugin.rs` hoy) |
|------------|-----------------|-----------|----------|
| `PhysicsSubPlugin` | Fase `Phase::Physics` | — | `dissipation_system`, `movement_will_drag_system`, `movement_integrate_transform_system`, `update_spatial_index_after_move_system`, `tension_field_system`, `collision_interference_system` |
| `ReactionsSubPlugin` | Fase `Phase::Reactions` | `CatalysisStrikeScratch` | `state_transitions_system`, `catalysis_scan_system`, `catalysis_apply_energy_system`, `catalysis_aftermath_system`, `homeostasis_system` |
| `WorldgenSubPlugin` | Campo / materializacion / visual (gran parte ya en `worldgen_prephysics` + `register_visual_pipeline`) | `EnergyFieldGrid`, caches perf, etc. | ver `register_prephysics_worldgen_through_delta`, Startup chain, `register_visual_pipeline` |
| `PrePhysicsSubPlugin` | Containment, structural, engine, resonancia, percepcion | — | cadena `containment` → `structural_constraint` → `contained_thermal_transfer` → `reset_resonance_overlay` → `resonance_link` → `sync_injector_projected_qe` → `engine_processing` → `perception` |
| `LayerSyncSubPlugin` | Input + sync frecuencia/elemento | — | `ensure_element_id`, `derive_frequency`, `sync_element_id`, `grimoire_cast_intent` (+ `almanac_hot_reload` si se agrupa aqui) |

### SimulationPlugin simplificado (objetivo de diseno, no snapshot actual)

```text
SimulationPlugin::build(&self, app):
  // Solo recursos / eventos realmente compartidos entre subsistemas (lista a cerrar en implementacion)
  app.init_resource::<SpatialIndex>()
  // ...

  // Delegar a sub-plugins o register_* por modulo
  app.add_plugins(PrePhysicsSubPlugin)
  app.add_plugins(PhysicsSubPlugin)
  app.add_plugins(ReactionsSubPlugin)
  app.add_plugins(LayerSyncSubPlugin)
  // Reloj sim y tick: hoy viven en V6RuntimeConfig + SimulationClockSet; no inventar tipos que no existan en el crate
```

### Patron por sub-plugin

Cada subsistema en `src/simulation/` expone una funcion publica:
```text
// Patron: una funcion register_* por subsistema; copiar orden y .chain() desde
// register_simulation_pipeline (Phase::Physics) sin alterar sistemas ni dependencias.
pub fn register_physics_systems<S: ScheduleLabel + Clone>(app: &mut App, schedule: S) { ... }
```

O alternativamente, cada uno es un Plugin propio.

## Tacticas

- **Empezar por el subsistema mas independiente.** `reactions` es buen candidato: solo necesita CatalysisStrikeScratch y queries. Extraer primero, validar patron, luego aplicar al resto.
- **Mantener el orden de fases.** Los `.in_set(Phase::*)` y `.chain()` deben preservarse exactamente. El orden de ejecucion no debe cambiar.
- **Un commit por subsistema extraido.** Extraer reactions → test → commit. Extraer physics → test → commit. Etc.
- **No mover archivos.** Los sistemas siguen en `src/simulation/physics.rs`, `reactions.rs`, etc. Solo cambia DONDE se registran.
- **SimulationPlugin queda como orquestador.** No desaparece — solo delega. Sigue siendo el entry point para `app.add_plugins(SimulationPlugin)`.

### Tambien: limpiar plugin wrappers vacios

7 archivos en `src/plugins/` son solo `pub use` hacia `crate::v6::...` (un re-export por archivo):
- `camera_controller_3d_plugin.rs`
- `click_to_move_plugin.rs`
- `debug_observability_plugin.rs`
- `input_capture_plugin.rs`
- `render_bridge_3d_plugin.rs`
- `simulation_tick_plugin.rs`
- `scenario_isolation_plugin.rs`

Consolidar en `plugins/mod.rs` (`pub use crate::v6::...::FooPlugin`) y eliminar los modulos wrapper, ajustando `mod` en `mod.rs`.

## NO hace

- No cambia logica de ningun sistema.
- No modifica el orden de ejecucion del pipeline.
- No agrega nuevos sistemas.
- No cambia aserciones de tests existentes; agregar smoke de `SimulationPlugin` solo si el criterio de aceptacion lo exige (ver abajo).

## Dependencias

- Ninguna hard. Recomendado despues de Q1 para trabajar sobre codigo limpio.

## Criterio de aceptacion

- Test: `SimulationPlugin` tiene menos de 50 lineas (vs ~227 actuales en snapshot).
- Test: cada subsistema registra sus propios sistemas.
- Test: `cargo test` y `cargo build` pasan.
- Test: los tests de `src/simulation/regression.rs` no cubren el plugin completo; si hace falta golden de schedule, agregar smoke explicito que monte `SimulationPlugin`.
- Test: los 7 archivos wrapper de re-export estan eliminados.
