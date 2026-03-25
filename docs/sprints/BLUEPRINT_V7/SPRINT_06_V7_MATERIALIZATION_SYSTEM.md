# Sprint 06 — V7 Sistema de Materialización ECS

**Módulo:** `src/worldgen/systems/materialization.rs`
**Tipo:** Sistema ECS — conecta funciones puras con el mundo Bevy.
**Onda:** C — Depende de Sprint 02 (propagation math) + Sprint 03 (materialization rules).

## Objetivo

Crear los sistemas que leen el `EnergyFieldGrid` y spawnean/despawnean entidades en el mundo según las reglas de materialización.

## Responsabilidades

### Sistema: materialization_full_system

- Recorrer todas las celdas del grid.
- Para cada celda, llamar a `materialize_cell` (Sprint 03).
- Si retorna `Some` y la celda no tiene entidad materializada → `Commands::spawn`.
- Si retorna `None` y la celda tiene entidad materializada → `Commands::entity().despawn()`.
- Asignar componente `Materialized` con coordenadas de celda y arquetipo.
- Asignar componentes base: `Transform`, `BaseEnergy`, `SpatialVolume`, `OscillatorySignature`, `MatterCoherence` según el arquetipo.
- Usar para el warmup inicial y reconstrucción completa.

### Sistema: materialization_delta_system

- Mantener un bitset o lista de celdas "dirty" (que cambiaron desde el último tick).
- Solo procesar celdas dirty: comparar resultado de `materialize_cell` con lo que ya hay.
- Spawn si pasó de `None` a `Some`.
- Despawn si pasó de `Some` a `None`.
- Actualizar componentes si el arquetipo cambió (raro pero posible).
- Usar para runtime tick-a-tick.

### Responsabilidad compartida: archetype_to_components

- Función (o lookup table) que dado un `WorldArchetype` retorna los componentes base que la entidad materializada debe tener.
- No todas las entidades materializadas tienen las mismas capas: una montaña tiene materia sólida y volumen grande; una niebla tiene volumen grande pero no materia.
- Reutilizar `EntityBuilder` existente donde tenga sentido.

## Tácticas

- **Presupuesto de spawn por frame.** No spawnear más de N entidades por tick (ej. 50). Si hay más celdas pendientes, distribuir en ticks sucesivos. Esto previene frame drops en cambios masivos del campo (ej. warmup, destrucción de núcleo grande).
- **Dirty tracking en el grid, no en los sistemas.** Cuando `propagate_nuclei_system` o `dissipate_field_system` modifican una celda, marcar la celda como dirty en el propio `EnergyFieldGrid`. El sistema delta solo itera dirty cells. Resetear dirty flags al final del frame.
- **Para el warmup, usar `materialization_full_system`.** Es más simple y se ejecuta una sola vez. Para runtime, usar `materialization_delta_system` que es incremental.
- **Las entidades materializadas deben usar componentes existentes de `layers/`.** No crear componentes nuevos para "árboles" o "rocas" — un árbol es una entidad con `BaseEnergy` + `SpatialVolume` + `OscillatorySignature` (Terra) + `MatterCoherence` (Solid). La diferencia entre un árbol y una roca es la densidad y el volumen, no un tipo especial.
- **Guardar la referencia `Entity` en la celda.** `EnergyCell.materialized_entity = Some(entity)` permite despawnear sin queries costosas.
- **Limpiar `materialized_entity` cuando la entidad muere por otros medios.** Si un `DeathEvent` se emite para una entidad materializada, el sistema debe limpiar la referencia en la celda. Registrar un observer o leer `DeathEvent` en el mismo sistema.

## NO hace

- No calcula propagación (Sprint 04 ya hizo esto).
- No asigna propiedades visuales (Sprint 08).
- No resuelve compuestos (Sprint 10).
- No carga MapConfig (Sprint 11).

## Dependencias

- Sprint 01 (tipos).
- Sprint 02 (funciones de propagación — indirectamente, via el grid ya populado).
- Sprint 03 (funciones de materialización — llamadas directamente).
- Sprint 04 (`EnergyFieldGrid` resource ya existente y populado).
- `entities/builder.rs` (`EntityBuilder`).
- `layers/*` (componentes para las entidades materializadas).

## Criterio de aceptación

- Test: grid 5×5 con un núcleo Terra en el centro. Tras warmup + full materialization, las celdas con qe suficiente tienen entidades spawneadas con `Materialized` component.
- Test: las entidades spawneadas tienen `OscillatorySignature` con frecuencia cercana a la del elemento dominante.
- Test: las entidades spawneadas tienen `MatterCoherence` con el estado correcto según temperatura.
- Test: remover el núcleo → tras N ticks de disipación → delta system despawnea las entidades.
- Test: el presupuesto de spawn limita a N entidades por tick (no se excede).
- Test: `materialized_entity` en la celda apunta a la entidad correcta.
- `cargo test` pasa.

## Referencia

`docs/design/V7.md` sección 6. `docs/arquitectura/blueprint_v7.md` sección 4.
