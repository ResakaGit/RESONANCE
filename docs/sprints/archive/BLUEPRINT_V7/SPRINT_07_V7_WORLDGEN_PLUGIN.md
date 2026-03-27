# Sprint 07 — V7 WorldgenPlugin (Wiring y Orquestación)

**Módulo:** `src/plugins/worldgen_plugin.rs`
**Tipo:** Plugin ECS — orquestación, registro, orden de sistemas.
**Onda:** C — Depende de Sprint 04 (field grid) + Sprint 06 (materialization system).

## Objetivo

Crear el plugin que registra todos los recursos, componentes, eventos y sistemas de V7 en el orden correcto dentro del pipeline de Bevy.

## Responsabilidades

- Registrar `EnergyFieldGrid` como resource (`init_resource` o insert en startup).
- Registrar tipos para reflection: `EnergyNucleus`, `Materialized`, `EnergyVisual`, `WorldArchetype`.
- Registrar sistemas de campo en `Phase::PrePhysics` (o equivalente en `FixedUpdate`):
  1. `propagate_nuclei_system`
  2. `dissipate_field_system`
  3. `derive_cell_state_system`
  - Estos tres deben correr en cadena (chain), en ese orden.
- Registrar sistemas de materialización en `Phase::PostPhysics`:
  4. `materialization_delta_system`
- Registrar sistemas visuales en `Update` (no en `FixedUpdate`):
  5. `visual_derivation_system`
- Sistema de startup: `init_worldgen_system` que crea el grid con las dimensiones del mapa.
- Exponer el plugin como `WorldgenPlugin` agregable en `main.rs`.

## Tácticas

- **Seguir el patrón de `SimulationPlugin` existente.** Usa `SystemSet` y `.chain()` para garantizar orden determinista. V7 añade sus sistemas al mismo pipeline sin romper el orden existente.
- **Los sistemas de campo corren ANTES del pipeline de simulación.** El campo debe estar actualizado cuando los sistemas de gameplay leen `AmbientPressure` o el `SpatialIndex`. Insertar en `Phase::PrePhysics` antes del `containment_system` existente.
- **El sistema de materialización corre DESPUÉS de la simulación.** Las entidades se spawnean/despawnean después de que toda la física se resolvió. Esto evita que una entidad recién materializada interfiera con colisiones del tick actual.
- **El sistema visual corre en `Update`, no en `FixedUpdate`.** La derivación visual es cosmética — no afecta la simulación y puede correr a frame rate variable.
- **Feature flag `v7_worldgen`.** Poner el plugin detrás de un feature flag en `Cargo.toml` para que sea opt-in. Esto permite que el pipeline V6 siga funcionando sin V7 hasta que esté estable.
- **No romper `spawn_demo_level` existente.** El plugin debe coexistir con el nivel demo. En este sprint, el grid se crea vacío si no hay MapConfig. El nivel demo sigue funcionando con biomas hardcodeados.
- **Registrar un `WorldgenPhase` SystemSet propio.** Esto facilita habilitar/deshabilitar todo V7 con un run condition.

## NO hace

- No define la lógica de los sistemas (ya implementada en sprints 04 y 06).
- No carga MapConfig (Sprint 11).
- No implementa warmup (Sprint 09).

## Dependencias

- Sprint 04 (sistemas de campo + EnergyFieldGrid).
- Sprint 06 (sistemas de materialización).
- `plugins/simulation_plugin.rs` (para integrar en el pipeline existente).
- `plugins/layers_plugin.rs` (para registrar tipos reflectivos).

## Criterio de aceptación

- `cargo check --features v7_worldgen` compila sin errores.
- El plugin se puede agregar a `main.rs` sin romper el pipeline existente.
- Los sistemas de campo corren en el orden correcto (propagate → dissipate → derive).
- Los sistemas de materialización corren después de la simulación.
- Con el plugin activo pero sin núcleos, el grid se inicializa vacío y no hay materialización (no-op seguro).
- `spawn_demo_level` sigue funcionando con o sin el plugin.
- `cargo test` pasa.

## Referencia

`docs/design/V7.md` sección 6. `docs/arquitectura/blueprint_plugins.md`.
