# DC-3: Worldgen State Repatriation — Ownership de GameState/PlayState

**Objetivo:** Mover las transiciones de `GameState` y `PlayState` de `worldgen/systems/startup.rs` a `simulation/`, estableciendo un owner único para el state machine del juego.

**Estado:** PENDIENTE
**Esfuerzo:** B (~5 archivos, ~60 LOC movidos)
**Bloqueado por:** —
**Desbloquea:** DC-5 (sim↔worldgen boundary)

---

## Problema

Hoy, worldgen controla directamente las transiciones de estado del juego:

```
worldgen/systems/startup.rs:207  → next.set(GameState::Playing)
worldgen/systems/startup.rs:324  → next.set(PlayState::Active)
```

**Violación de principio:** El state machine del juego (`GameState`, `PlayState`) es responsabilidad de `simulation/`. Worldgen no debería saber qué estados existen ni cuándo transicionar. Su responsabilidad es "generar el mundo" — no "decidir cuándo el mundo está listo para jugar".

**Secuencia actual (problemática):**
```
Startup schedule (.chain()):
  ...
  spawn_nuclei_from_map_config_system     [worldgen]
  seed_nutrient_field_from_nuclei_system  [worldgen]
  enter_game_state_playing_system         [worldgen — CONTROLA GameState]
  worldgen_warmup_system                  [worldgen — sync loop N ticks]
  mark_play_state_active_system           [worldgen — CONTROLA PlayState]
```

**Riesgo:** Si mañana se añade otro módulo que necesita inicializar antes de `PlayState::Active` (ej: netcode warmup, AI bootstrap), worldgen no debería ser el gatekeeper.

---

## Diseño

### Principio: Worldgen emite señal de "mundo listo". Simulation decide qué hacer con ella.

```
ANTES:
  worldgen → set(GameState::Playing)    // worldgen controla
  worldgen → warmup loop
  worldgen → set(PlayState::Active)     // worldgen controla

DESPUÉS:
  simulation → set(GameState::Playing)  // simulation controla
  worldgen → warmup loop                // worldgen solo genera
  worldgen → insert WorldgenReady       // worldgen señaliza
  simulation → observa WorldgenReady → set(PlayState::Active)  // simulation decide
```

### Resource señalizadora: `WorldgenReady`

```rust
// worldgen/mod.rs (o worldgen/contracts.rs)

/// Señaliza que el worldgen completó su warmup y el mundo está materializado.
/// Insertada por worldgen_warmup_system. Leída por simulation para transicionar estado.
///
/// Resource (no Event) porque es un estado persistente: una vez ready, siempre ready.
#[derive(Resource, Debug, Default)]
pub struct WorldgenReady {
    /// Tick en el que se completó el warmup.
    pub completed_at_tick: u64,
}
```

**Por qué Resource y no Event:**
- Es un estado, no un instante. Una vez el mundo está listo, sigue listo.
- Los Events se consumen. Si simulation no los lee en el tick correcto, se pierden.
- Resource permite `Option<Res<WorldgenReady>>` — si no existe, worldgen no terminó.

### Sistema repatriado: `transition_to_active_system`

```rust
// simulation/lifecycle/state_transitions.rs — NUEVO

/// Transiciona a PlayState::Active cuando worldgen está listo.
/// Owner único del state machine del juego.
pub fn transition_to_active_system(
    ready: Option<Res<WorldgenReady>>,
    state: Res<State<PlayState>>,
    mut next: ResMut<NextState<PlayState>>,
) {
    if ready.is_some() && *state.get() == PlayState::Warmup {
        next.set(PlayState::Active);
    }
}
```

**Características:**
- Stateless: lee Resource, compara estado, transiciona si aplica
- Idempotente: si ya es Active, no hace nada
- Extensible: si mañana se añade `NetcodeReady`, se añade otro guard:
  ```rust
  if ready.is_some() && netcode.is_some() && *state == Warmup { ... }
  ```

### Migración de `enter_game_state_playing_system`

```rust
// ANTES (worldgen/systems/startup.rs:207):
pub fn enter_game_state_playing_system(
    mut next: ResMut<NextState<GameState>>,
) {
    next.set(GameState::Playing);
}

// DESPUÉS: se mueve a simulation/lifecycle/state_transitions.rs
// Mismo código, diferente owner module.
pub fn enter_game_state_playing_system(
    mut next: ResMut<NextState<GameState>>,
) {
    next.set(GameState::Playing);
}
```

### Migración de `mark_play_state_active_system`

```rust
// ANTES (worldgen/systems/startup.rs:324):
pub fn mark_play_state_active_system(
    mut next: ResMut<NextState<PlayState>>,
) {
    next.set(PlayState::Active);
}

// DESPUÉS: ELIMINADO. Reemplazado por transition_to_active_system.
// worldgen_warmup_system ahora inserta WorldgenReady en vez de transicionar.
```

### Cambio en `worldgen_warmup_system`

```rust
// ANTES (worldgen/systems/startup.rs:211-237):
pub fn worldgen_warmup_system(world: &mut World) {
    let ticks = world.resource::<WorldgenWarmupConfig>().ticks;
    // ... warmup loop ...
    materialization_full_world(world);
    // (retorna, y el siguiente chained system hace set(PlayState::Active))
}

// DESPUÉS:
pub fn worldgen_warmup_system(world: &mut World) {
    let ticks = world.resource::<WorldgenWarmupConfig>().ticks;
    // ... warmup loop (INTACTO) ...
    materialization_full_world(world);

    // Señalizar que worldgen completó
    world.insert_resource(WorldgenReady {
        completed_at_tick: ticks as u64,
    });
}
```

### Cambio en `plugins/simulation_plugin.rs`

```rust
// ANTES:
app.add_systems(Startup, (
    // ... worldgen systems ...
    enter_game_state_playing_system,         // worldgen/
    worldgen_warmup_system,
    mark_play_state_active_system,           // worldgen/
).chain());

// DESPUÉS:
app.add_systems(Startup, (
    // ... worldgen systems ...
    enter_game_state_playing_system,         // simulation/ (repatriado)
    worldgen_warmup_system,                  // worldgen/ (inserta WorldgenReady)
    transition_to_active_system,             // simulation/ (lee WorldgenReady)
).chain());
```

---

## Plan de ejecución (3 commits atómicos)

### Commit 1: Crear `WorldgenReady` + `transition_to_active_system`

- Crear `worldgen/contracts.rs` (si no existe) con `WorldgenReady`
- Crear `simulation/lifecycle/state_transitions.rs` con:
  - `enter_game_state_playing_system` (movido de worldgen)
  - `transition_to_active_system` (nuevo)
- Tests unitarios para ambos sistemas
- **Test:** `cargo check` pasa

### Commit 2: Migrar worldgen_warmup_system

- Modificar `worldgen_warmup_system` para insertar `WorldgenReady` al final
- Eliminar `mark_play_state_active_system` de `worldgen/systems/startup.rs`
- Eliminar `enter_game_state_playing_system` de `worldgen/systems/startup.rs`
- **Test:** `cargo check` pasa (puede haber warnings de dead code)

### Commit 3: Rewire plugin + cleanup

- Actualizar `plugins/simulation_plugin.rs`:
  - Import desde `simulation::lifecycle::state_transitions` en vez de `worldgen::systems::startup`
  - Reemplazar `mark_play_state_active_system` por `transition_to_active_system`
- Eliminar `use crate::simulation::states::*` de `worldgen/systems/startup.rs`
- **Test:** `cargo test` completo — 0 failures

---

## Testing

### Capa 1: Unitario

```rust
// simulation/lifecycle/state_transitions.rs — tests

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn transition_to_active_when_worldgen_ready() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayState>();
        app.insert_resource(WorldgenReady { completed_at_tick: 100 });
        app.add_systems(Update, transition_to_active_system);

        // Force into Playing + Warmup
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update(); // Apply GameState transition

        app.update(); // Run transition_to_active_system

        let state = app.world().resource::<State<PlayState>>();
        assert_eq!(*state.get(), PlayState::Active);
    }

    #[test]
    fn transition_noop_without_worldgen_ready() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayState>();
        // NO WorldgenReady inserted
        app.add_systems(Update, transition_to_active_system);

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        app.update();

        let state = app.world().resource::<State<PlayState>>();
        assert_eq!(*state.get(), PlayState::Warmup,
            "Should remain Warmup without WorldgenReady");
    }

    #[test]
    fn transition_idempotent_when_already_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayState>();
        app.insert_resource(WorldgenReady { completed_at_tick: 50 });
        app.add_systems(Update, transition_to_active_system);

        // Transition to Active
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        app.update();

        // Run again — should not panic or double-transition
        app.update();
        let state = app.world().resource::<State<PlayState>>();
        assert_eq!(*state.get(), PlayState::Active);
    }
}
```

### Capa 2: Integración (startup sequence)

```rust
// tests/integration/dc3_state_repatriation.rs

#[test]
fn startup_sequence_transitions_to_active() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));

    // Simulate enough updates for startup chain to complete
    for _ in 0..5 { app.update(); }

    let game = app.world().resource::<State<GameState>>();
    assert_eq!(*game.get(), GameState::Playing);

    let play = app.world().resource::<State<PlayState>>();
    assert_eq!(*play.get(), PlayState::Active);

    assert!(app.world().get_resource::<WorldgenReady>().is_some());
}
```

### Capa 3: Orquestación

```rust
/// HOF: test harness para startup lifecycle.
fn run_startup_test<A>(assert_fn: A)
where
    A: FnOnce(&World),
{
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    for _ in 0..10 { app.update(); }
    assert_fn(app.world());
}

#[test]
fn worldgen_no_longer_imports_simulation_states() {
    // Meta-test: verify at compile time that worldgen doesn't depend on states
    // This is enforced by the grep check in CI, but also by the module structure
    run_startup_test(|world| {
        assert!(world.get_resource::<WorldgenReady>().is_some(),
            "WorldgenReady should exist after startup");
    });
}
```

---

## Integración al codebase

### Lo que se CREA
- `worldgen/contracts.rs` (o extend existente) — `WorldgenReady` resource
- `simulation/lifecycle/state_transitions.rs` — 2 sistemas (enter_playing + transition_to_active)

### Lo que se MUEVE
- `enter_game_state_playing_system`: worldgen/startup.rs → simulation/lifecycle/state_transitions.rs
- Import en simulation_plugin.rs: worldgen → simulation

### Lo que se ELIMINA
- `mark_play_state_active_system` de worldgen/startup.rs
- `use crate::simulation::states::*` de worldgen/startup.rs (tras cleanup)

### Lo que NO cambia
- `worldgen_warmup_system` loop (solo se añade 1 línea: insert WorldgenReady)
- `GameState` y `PlayState` definiciones (siguen en simulation/states.rs)
- Phase gates (run_if conditions en pipeline.rs)
- Cualquier otro sistema de worldgen

### Invariante preservado

```
GameState::Loading → Playing  : set en Startup, ANTES de warmup
PlayState::Warmup → Active    : set en Startup, DESPUÉS de warmup
ThermodynamicLayer runs in Warmup: YES (propagation needs it)
Other phases run only in Active: YES (unchanged)
```

La secuencia temporal es idéntica. Solo cambia quién ejecuta la transición.

---

## Scope definido

**Entra:**
- Crear `WorldgenReady` resource en worldgen/
- Mover `enter_game_state_playing_system` a simulation/
- Reemplazar `mark_play_state_active_system` por `transition_to_active_system`
- Actualizar warmup para insertar WorldgenReady
- Rewire en simulation_plugin.rs
- Tests de las 3 capas

**NO entra:**
- Refactor de worldgen_warmup_system loop (queda sync, misma lógica)
- Mover worldgen systems de prephysics (eso es DC-5)
- Crear estados nuevos (Victory, PostGame — fuera de scope)
- Cambiar Phase gates en pipeline.rs

---

## Criterios de cierre

- [ ] `cargo test` — 0 failures
- [ ] `grep "GameState\|PlayState" src/worldgen/systems/startup.rs` — 0 resultados (solo WorldgenReady)
- [ ] `WorldgenReady` resource existe tras startup
- [ ] `transition_to_active_system` tiene 3+ tests (ready, not-ready, idempotent)
- [ ] startup sequence funcional (integration test pasa)
- [ ] Ningún `// DEBT:` introducido
