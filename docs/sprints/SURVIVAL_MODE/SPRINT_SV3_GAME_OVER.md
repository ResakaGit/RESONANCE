# Sprint SV-3 — Game Over: Death Detection + Score + Restart

**Modulo:** `src/bin/survival.rs` (extensión del binario SV-2)
**Tipo:** UI overlay + state transitions.
**Onda:** SV-2 → SV-3.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post SV-2)

- `SurvivalState { score, alive }` — resource local al binario.
- `DeathEvent { entity, cause }` — emitido por `EnergyOps::drain` cuando `qe < QE_MIN_EXISTENCE`.
- `GameState::PostGame` — estado Bevy existente.
- `PlayState::Victory` — sub-estado existente (reutilizable como "game ended").
- Score HUD overlay — texto actualizado cada tick.
- Player entity marcada con `PlayerControlled`.

---

## Objetivo

Detectar muerte del player → congelar simulación → mostrar pantalla game over con score final → opción de restart.

---

## Responsabilidades

### SV-3A: Death detection via DeathEvent

```rust
fn detect_player_death(
    mut death_events: EventReader<DeathEvent>,
    player: Query<Entity, With<PlayerControlled>>,
    mut state: ResMut<SurvivalState>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    let Ok(player_entity) = player.get_single() else { return; };
    for ev in death_events.read() {
        if ev.entity == player_entity {
            state.alive = false;
            next_game.set(GameState::PostGame);
        }
    }
}
```

**Nota:** Usa `DeathEvent` existente. No crea nuevos eventos. No modifica `events.rs`.

### SV-3B: Game over overlay

```rust
#[derive(Component)]
#[component(storage = "SparseSet")]
struct GameOverScreen;

fn spawn_game_over_ui(
    mut commands: Commands,
    state: Res<SurvivalState>,
) {
    commands.spawn((
        GameOverScreen,
        StateScoped(GameState::PostGame),
        // Node + Text children:
        // "GAME OVER"
        // "Score: {state.score}"
        // "Press R to restart"
    ));
}
```

**Usa `StateScoped`:** la UI se limpia automáticamente al salir de `PostGame`.

### SV-3C: Restart system

```rust
fn restart_on_r(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut state: ResMut<SurvivalState>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        state.score = 0;
        state.alive = true;
        next_game.set(GameState::Playing);
        // Re-trigger startup system via state transition
    }
}
```

**Nota:** El restart re-entra `GameState::Playing` lo cual reactiva `PlayState::Warmup` → spawna nuevo mundo.

### SV-3D: Score freeze on death

```rust
fn score_freeze_guard(
    state: Res<SurvivalState>,
    // score_update_system solo corre si alive
) {
    // Implementado como run_if(|s: Res<SurvivalState>| s.alive)
    // en el system registration de score_update_system
}
```

---

## Encapsulamiento

**Todo vive en `src/bin/survival.rs`.** No se crea ningún módulo en `src/`.
El binario lee de:
- `resonance::events::DeathEvent` — detección de muerte (evento existente).
- `resonance::simulation::states::{GameState, PlayState}` — transiciones (estados existentes).

**No modifica:** `simulation/`, `layers/`, `events.rs`, `plugins/`.

---

## NO hace

- No implementa abilities/spells — futuro.
- No modifica la simulación — misma física.
- No crea leaderboard persistente — futuro.
- No modifica ningún system en `src/`.
- No crea nuevos components ni events en `src/`.

---

## Criterios de aceptacion

### Funcional
- Player muere (qe < min) → `GameState::PostGame` activado.
- Pantalla game over muestra "GAME OVER" + score final.
- Press R → nuevo mundo, score reseteado, simulación corriendo.
- Score congelado durante PostGame (no incrementa).

### Encapsulamiento
- `grep -r "game_over\|GameOverScreen\|restart" src/ | grep -v bin/survival | grep -v test` retorna 0 resultados.
- Zero cambios en `src/simulation/`, `src/layers/`, `src/events.rs`.

### Performance
- Transición PostGame → Playing < 500ms (respawn completo).
- Game over overlay: 0 allocation per frame (static text).

---

## Referencias

- `src/events.rs:191` — `DeathEvent` (producer: `EnergyOps::drain`)
- `src/simulation/states.rs` — `GameState::PostGame`, `PlayState::Victory`
- `src/bin/survival.rs` — SV-2 binary (score, HUD, spawn)
