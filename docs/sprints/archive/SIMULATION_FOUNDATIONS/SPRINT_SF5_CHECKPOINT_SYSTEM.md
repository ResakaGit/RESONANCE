# Sprint SF-5 — Checkpoint Save/Load System

**Modulo:** `src/simulation/checkpoint.rs`, `src/plugins/simulation_plugin.rs` (extension)
**Tipo:** Sistemas ECS (on-demand save, Startup load).
**Onda:** A — Requiere SF-2.
**Estado:** ⏳ Pendiente

## Contexto: que ya existe

SF-2 agrega `Serialize, Deserialize` a todos los tipos core y define `WorldCheckpoint` + `build_checkpoint()` + `checkpoint_to_ron()` + `checkpoint_from_ron()`. Todo como funciones puras en `blueprint/checkpoint.rs`.

**Lo que NO existe:**
1. **Sistema de save.** Nada recolecta el estado del mundo y lo escribe a disco.
2. **Sistema de load.** Nada lee un checkpoint y reconstruye el mundo.
3. **Trigger de save.** No hay mecanismo para solicitar un checkpoint (key, command, tick interval).
4. **Entity reconstruction.** No hay pipeline para reconstruir entidades desde `EntitySnapshot`.

## Objetivo

Sistemas para guardar y cargar checkpoints. Save es on-demand (trigger via Resource flag o env var). Load es al startup si se provee archivo.

**Resultado:** `RESONANCE_CHECKPOINT_SAVE=100 cargo run` guarda un checkpoint cada 100 ticks. `RESONANCE_CHECKPOINT_LOAD=/tmp/resonance_checkpoint_100.ron cargo run` carga ese estado y continua.

## Responsabilidades

### SF-5A: Resource `CheckpointConfig`

```rust
/// Configuracion de checkpointing. Insertado condicionalmente.
#[derive(Resource, Debug, Clone)]
pub struct CheckpointConfig {
    pub save_interval: u32,          // 0 = manual only, >0 = cada N ticks
    pub output_dir: String,          // Directorio destino
    pub load_path: Option<String>,   // Si Some, cargar al startup
    pub format: CheckpointFormat,    // Ron o Json
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointFormat { Ron, Json }
```

Inicializado desde env vars:
- `RESONANCE_CHECKPOINT_SAVE=100` → `save_interval = 100`
- `RESONANCE_CHECKPOINT_LOAD=/path/file.ron` → `load_path = Some(...)`
- `RESONANCE_CHECKPOINT_DIR=/tmp` → `output_dir = "/tmp"`

### SF-5B: `checkpoint_save_system` (FixedUpdate)

```rust
/// Guarda checkpoint a disco cuando toca.
pub fn checkpoint_save_system(
    config: Option<Res<CheckpointConfig>>,
    clock: Res<SimulationClock>,
    grid: Option<Res<EnergyFieldGrid>>,
    map_name: Option<Res<ActiveMapName>>,
    query: Query<(Entity, &Transform, Option<&BaseEnergy>, Option<&SpatialVolume>,
                  Option<&OscillatorySignature>, Option<&MatterCoherence>)>,
    mut last_save: Local<u64>,
) { ... }
```

- **Phase:** `Phase::MorphologicalLayer` (final del tick — estado completo).
- Guard: `config.is_none()` → return.
- Guard: `save_interval == 0 || clock.tick_id % save_interval != 0` → return.
- Guard: `clock.tick_id == *last_save` → return (idempotente).
- Algoritmo:
  1. `build_checkpoint()` desde query + grid.
  2. `checkpoint_to_ron()` o `checkpoint_to_json()`.
  3. `std::fs::write(path, content)`.
  4. `info!("checkpoint saved: {} entities, tick {}", checkpoint.entities.len(), clock.tick_id)`.
  5. `*last_save = clock.tick_id`.

Path: `{output_dir}/resonance_checkpoint_{tick}.{ext}`.

### SF-5C: `checkpoint_load_startup_system` (Startup)

```rust
/// Carga checkpoint al inicio si esta configurado.
pub fn checkpoint_load_startup_system(
    mut commands: Commands,
    config: Option<Res<CheckpointConfig>>,
) { ... }
```

- **Schedule:** `Startup`, before `load_map_config_startup_system`.
- Guard: `config.is_none() || config.load_path.is_none()` → return.
- Algoritmo:
  1. `std::fs::read_to_string(path)`.
  2. `checkpoint_from_ron(&content)` → `WorldCheckpoint`.
  3. Insert `EnergyFieldGrid` from checkpoint.
  4. Set `SimulationClock.tick_id = checkpoint.tick`.
  5. Spawn entidades desde `EntitySnapshot`:
     - `commands.spawn((Transform::from_xyz(...), BaseEnergy::new(qe), ...))`.
     - Solo layers presentes en el snapshot (Options).
  6. Set `ActiveMapName` from checkpoint.
  7. `info!("checkpoint loaded: {} entities, tick {}", count, tick)`.

### SF-5D: Registro en plugin

- `CheckpointConfig` se inicializa en `SimulationPlugin::build()` si env vars presentes.
- `checkpoint_save_system` en `FixedUpdate / Phase::MorphologicalLayer` (ultimo, after abiogenesis).
- `checkpoint_load_startup_system` en `Startup` (before worldgen warmup).

## Tacticas

- **Env var gating.** Sin env var, cero overhead. No se inserta el Resource.
- **RON por defecto.** Human-readable, debuggeable.
- **Una entidad = una fila.** `EntitySnapshot` es flat, sin nesting profundo.
- **Skip auxiliary components.** MetabolicGraph, OrganManifest, Grimoire se reconstruyen post-load via inference systems. El checkpoint guarda solo el estado fisico minimo.

## NO hace

- No implementa undo/redo.
- No implementa networking/multiplayer sync.
- No implementa diff-based incremental checkpoints.
- No serializa rendering state (meshes, materials).

## Dependencias

- SF-2 (`WorldCheckpoint`, `build_checkpoint`, `checkpoint_to_ron`, `checkpoint_from_ron`).
- `simulation/pipeline.rs` — Phase para registro.
- `worldgen/field_grid.rs` — `EnergyFieldGrid` (insert).
- `runtime_platform/simulation_tick.rs` — `SimulationClock` (set tick).

## Criterios de aceptacion

### SF-5B (Save)
- Test: con config + grid + 3 entidades + tick=100 → archivo existe, parseable.
- Test: save_interval=0 → nunca guarda.
- Test: tick != multiple de interval → no guarda.
- Test: dos saves al mismo tick → idempotente (1 archivo).

### SF-5C (Load)
- Test (MinimalPlugins): insertar checkpoint RON como file mock → tras startup → grid dimensions correctas.
- Test: entidades spawneadas con BaseEnergy correcta.
- Test: SimulationClock.tick_id = checkpoint.tick.
- Test: load_path=None → no carga (startup normal).

### SF-5D (Roundtrip)
- Test: save tick 100 → load → run 10 ticks → save tick 110. Comparar: load original → run 10 ticks → save. Los checkpoints de tick 110 deben ser **identicos** (determinismo).

### General
- `cargo test --lib` sin regresion.

## Referencias

- SF-2 — `WorldCheckpoint`, funciones de serialization
- `src/worldgen/systems/startup.rs` — startup sequence
- `src/simulation/pipeline.rs` — Phase registration
