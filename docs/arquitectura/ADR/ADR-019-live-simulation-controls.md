# ADR-019: Live Simulation Controls — Pause, Speed, Reset, Map Selector

**Estado:** Aceptado
**Fecha:** 2026-04-12
**Contexto:** LAB_UI_REFACTOR sprint LR-2, ADR-018

## Contexto

El lab binary tiene un modo Live 2D que muestra la simulacion en tiempo real pero
sin controles: no se puede pausar, cambiar velocidad, resetear ni cambiar mapa.
El mapa se lee de `RESONANCE_MAP` env var al inicio y no hay forma de cambiarlo
sin reiniciar el proceso.

La simulacion ya tiene `GameState::Paused` declarado pero no usado. El pipeline
gate en `pipeline.rs:71` es `in_state(Playing).and(in_state(Active))` — si el
estado cambia a `Paused`, el pipeline se detiene automaticamente.

## Decisiones

### D1: Pause via GameState existente (NO inventar estados nuevos)

`GameState::Paused` ya existe en `states.rs:14`. El pipeline ya lo respeta
implicitamente: el gate `in_state(Playing)` excluye `Paused`.

**Implementacion:**
- Lab binary escribe `NextState<GameState>` directamente (es un binary, no un plugin)
- Toggle: `Playing` <-> `Paused`
- `PlayState` es `SubState` de `Playing` — al pausar se preserva automaticamente
- Al resumir, vuelve a `Playing` y `PlayState::Active` sigue activo

**NO crear:** eventos de pause, sistemas de pause, ni recursos de pause. El state
machine de Bevy ya lo resuelve.

### D2: Speed via Time<Fixed> period (NO crear resources de speed)

Bevy 0.15 permite cambiar el period de `Time<Fixed>` en runtime:
```rust
time_fixed.set_timestep(Duration::from_secs_f64(1.0 / (base_hz * scale)));
```

Cambiar el timestep del FixedUpdate schedule hace que Bevy ejecute mas o menos
ticks por frame. Esto es:
- Determinista: cada tick sigue teniendo el mismo dt
- Transparente: ningun sistema necesita saber que la velocidad cambio
- Sin side effects: el clock avanza igual, solo cambian cuantos ticks por frame

**Rango:** 0.25x a 4x (15 Hz a 240 Hz). Mas alla de 4x el frame budget explota.

**NO crear:** `SimulationSpeed` resource, ni escalar dt manualmente, ni tocar
`advance_simulation_clock_system`. La solucion nativa de Bevy es superior.

### D3: Reset como re-warmup (exclusive system)

Reset requiere:
1. Despawn todas las entidades con `BaseEnergy` (materialized + offspring)
2. Preservar nuclei (`EnergyNucleus` sin `BaseEnergy`)
3. Reset `EnergyFieldGrid` cells a zero
4. Reset `NutrientFieldGrid` cells a zero
5. Reset `SimulationClock` y `SimulationElapsed` a zero
6. Transicion `PlayState::Active` -> `PlayState::Warmup`
7. Re-run warmup (propagation + materialization)
8. Transicion `PlayState::Warmup` -> `PlayState::Active`

Esto es un exclusive system porque necesita `&mut World` para re-usar la
logica de warmup existente. Se dispara con un `Event<ResetWorldEvent>`.

**Riesgo mitigado:** El warmup existente ya funciona como exclusive system en
startup. Re-usamos la misma logica. No duplicamos codigo: extraemos una fn
`run_warmup_sequence(world)` usable desde startup Y desde reset.

### D4: Map selector como Reset + re-load config

Cambiar mapa = cargar nueva `MapConfig` + reset completo. La secuencia es:

1. UI selecciona nuevo slug
2. System lee el slug, carga `MapConfig` desde `assets/maps/{slug}.ron`
3. Despawn nuclei existentes
4. Replace resources: `MapConfig`, `ActiveMapName`, `EnergyFieldGrid`, `NutrientFieldGrid`
5. Re-spawn nuclei desde nueva config
6. Re-seed field si la config lo requiere
7. Run warmup
8. Transition to Active

Esto extiende el exclusive system de D3 con pasos adicionales de carga.

**Funcion pura de carga:** `load_map_config_from_slug(slug: &str) -> Result<MapConfig, String>`
ya existe implicitamente en `map_config.rs`. Se expone como pub fn que recibe un
slug directamente (hoy `selected_map_path_from_env` lee env var, creamos una variante
que recibe el slug como parametro).

### D5: Que NO tocar

- Pipeline phase chain (`pipeline.rs:71-84`) — intocable
- `advance_simulation_clock_system` — intocable
- Startup chain en `simulation_plugin.rs:65-83` — intocable
- `warmup_system` internals — se extrae, no se modifica

## Archivos a modificar

| Archivo | Cambio |
|---------|--------|
| `src/worldgen/map_config.rs` | +`load_map_config_from_slug(slug)` (4 lineas, wrapper de parse) |
| `src/worldgen/mod.rs` | Re-export `load_map_config_from_slug` |
| `src/worldgen/field_grid.rs` | +`reset_cells(&mut self)` (clear all cells to default) |
| `src/worldgen/systems/startup.rs` | Extraer `run_warmup_sequence(world)` de `worldgen_warmup_system` |
| `src/bin/lab.rs` | Controles Live: pause button, speed slider, reset button, map dropdown |

## Archivos nuevos

Ninguno. Todo se integra en archivos existentes.

## Consecuencias

### Se gana
- Pause/resume con 1 click (usa state machine existente)
- Speed 0.25x-4x sin romper determinismo
- Reset sin reiniciar proceso
- Cambio de mapa en caliente (25 mapas disponibles)
- Zero recursos/eventos/sistemas nuevos para pause y speed
- Reset reutiliza warmup existente (zero duplicacion)

### Se pierde
- Nada. No hay tradeoffs negativos reales.

### Riesgo
- Reset exclusive system bloquea 1-2 frames durante warmup (~100-200ms)
- Mitigacion: warmup tipico son 50-100 ticks, ~50ms. Aceptable para reset manual.

## No viola axiomas

- Axioma 2 (Pool Invariant): Reset regenera campo desde nuclei, conservacion OK
- Axioma 4 (Dissipation): Warmup aplica disipacion normalmente
- Axioma 5 (Conservation): Reset es un "big bang" nuevo, no viola conservacion
- Speed change no afecta dt por tick, solo cuantos ticks por frame
