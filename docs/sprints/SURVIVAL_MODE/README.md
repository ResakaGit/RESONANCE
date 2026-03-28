# Track: SURVIVAL_MODE — Jugar como una criatura evolucionada

Cargar genomes evolucionados → spawnar en arena Bevy → controlar una entidad →
sobrevivir el mayor tiempo posible. Score = ticks survived. Muere → game over.

**Invariante:** El player solo controla `WillActuator.movement_intent`. La física decide si sobrevive.
Zero god-mode. Zero excepciones. Mismos axiomas para player y AI.

---

## Sprints (3)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [SV-1](SPRINT_SV1_INPUT_WIRING.md) | Input Wiring | Bajo | — | ✅ Completado (2026-03-28) |
| [SV-2](SPRINT_SV2_SURVIVAL_BINARY.md) | Survival Binary | Medio | SV-1 ✅ | `cargo run --bin survival` carga genomes + spawns + play |
| [SV-3](SPRINT_SV3_GAME_OVER.md) | Game Over | Bajo | SV-2 | Death detection → score → restart option |

Sprints archivados: [archive/SURVIVAL_MODE/](../archive/SURVIVAL_MODE/) (SV-1 ✅)

---

## Dependency chain

```
SV-1 (input) → SV-2 (binary) → SV-3 (game over)
```

## Qué ya existe (no se toca)

| Componente | Archivo | Estado |
|-----------|---------|--------|
| `InputCommand::MoveToward` | `src/sim_world.rs:87` | ✅ Definido |
| `InputCommand::CastAbility` | `src/sim_world.rs:88` | ✅ Definido |
| `WillActuator` (L7) | `src/layers/will.rs` | ✅ Funcional |
| `PlayerControlled` marker | `src/simulation/input.rs` | ✅ Definido |
| `DeathEvent` | `src/events.rs` | ✅ Emitido por `EnergyOps::drain` |
| `GameState::Playing / PostGame` | `src/simulation/states.rs` | ✅ Wired |
| `PlayState::Active / Victory` | `src/simulation/states.rs` | ✅ Wired |
| `Grimoire` (4 ability slots) | `src/layers/will.rs:213` | ✅ Definido |
| Pathfinding A* | `src/simulation/pathfinding/` | ✅ Funcional |
| Ability targeting | `src/simulation/ability_targeting.rs` | ✅ Funcional |
| `genome_to_components()` | `src/batch/bridge.rs` | ✅ Lossless |
| `load_genomes()` | `src/batch/bridge.rs` | ✅ From disk |
| Input capture (WASD) | `src/runtime_platform/input_capture/` | ✅ Existe |

## Qué se crea (nuevo, encapsulado)

| Componente | Archivo nuevo | Toca módulo existente? |
|-----------|--------------|----------------------|
| `apply_input()` impl | — | SÍ: `src/sim_world.rs` (5 LOC) |
| Survival binary | `src/bin/survival.rs` | NO — standalone |
| `SurvivalState` resource | `src/bin/survival.rs` | NO — local al binary |
| Score overlay | `src/bin/survival.rs` | NO — Bevy UI local |
| Game over screen | `src/bin/survival.rs` | NO — Bevy UI local |

**Principio:** Todo lo survival-specific vive en el binario. Zero leaking a `simulation/`, `layers/`, o `batch/`.
