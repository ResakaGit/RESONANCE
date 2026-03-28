# Sprint SV-2 — Survival Binary: Load Genomes + Spawn + Play

**Modulo:** `src/bin/survival.rs` (nuevo, standalone)
**Tipo:** Bevy binary que compone módulos existentes.
**Onda:** SV-1 → SV-2.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post SV-1)

- `apply_input()` wired — InputCommand → WillActuator.
- `genome_to_components()` — GenomeBlob → Bevy components.
- `load_genomes()` — deserializar desde .bin.
- `creature_builder` — GF1 mesh desde genome.
- `evolve_and_view.rs` — referencia de cómo spawnar + renderizar.

---

## Objetivo

Un binario standalone que:
1. Carga genomes evolucionados desde archivo (o genera nuevos con preset).
2. Spawna entidades en arena Bevy (misma física que simulación principal).
3. Marca UNA entidad como `PlayerControlled`.
4. El jugador la controla con WASD.
5. Score = tick_id (cuánto sobrevivió).

---

## Responsabilidades

### SV-2A: CLI arguments

```bash
# Cargar genomes evolucionados:
cargo run --release --bin survival -- --genomes assets/evolved/seed_42.bin

# Generar mundo nuevo con preset:
cargo run --release --bin survival -- --preset earth --seed 42

# Mapa específico:
cargo run --release --bin survival -- --map demo_animal
```

### SV-2B: Startup system

```rust
fn spawn_survival_world(
    mut commands: Commands,
    genomes: Res<EvolvedGenomes>,
    // ...
) {
    // 1. Spawn ground, camera, light (como evolve_and_view)
    // 2. Spawn entities from genomes via genome_to_components()
    // 3. Mark first entity as PlayerControlled
    // 4. Initialize SurvivalState { score: 0, alive: true }
}
```

### SV-2C: Player input system

```rust
fn player_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut WillActuator, With<PlayerControlled>>,
) {
    for mut will in &mut q {
        let mut intent = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) { intent.y += 1.0; }
        if keys.pressed(KeyCode::KeyS) { intent.y -= 1.0; }
        if keys.pressed(KeyCode::KeyA) { intent.x -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { intent.x += 1.0; }
        will.movement_intent = intent.normalize_or_zero();
    }
}
```

**Nota:** Este system vive SOLO en el binario. No en `simulation/`. No en `layers/`.
No contamina ningún módulo existente.

### SV-2D: Score tracking

```rust
#[derive(Resource)]
struct SurvivalState {
    score: u64,      // tick_id al morir
    alive: bool,
}

fn score_update_system(
    time: Res<Time>,
    player: Query<&BaseEnergy, With<PlayerControlled>>,
    mut state: ResMut<SurvivalState>,
) {
    if let Ok(energy) = player.get_single() {
        if energy.qe() <= QE_MIN_EXISTENCE {
            state.alive = false;
        }
    } else {
        state.alive = false; // entity despawned
    }
}
```

### SV-2E: HUD overlay

```rust
fn score_hud_system(
    state: Res<SurvivalState>,
    mut text: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(mut t) = text.get_single_mut() {
        *t = Text::from(format!("Score: {} | qe: {:.1}",
            state.score, /* player qe */));
    }
}
```

---

## Encapsulamiento

**Todo vive en `src/bin/survival.rs`.** No se crea ningún módulo en `src/`.
El binario importa de:
- `resonance::batch::bridge` — load genomes
- `resonance::batch::genome` — GenomeBlob
- `resonance::layers` — BaseEnergy, WillActuator, PlayerControlled
- `resonance::use_cases::presets` — universe presets

**No modifica:** `simulation/`, `layers/`, `batch/`, `plugins/`, `events.rs`.

---

## NO hace

- No implementa game over UI — eso es SV-3.
- No implementa abilities/spells — futuro.
- No modifica la simulación — misma física para player y AI.
- No crea nuevos components ni systems en `src/`.

---

## Criterios de aceptacion

### Funcional
- `cargo run --release --bin survival` abre ventana Bevy con criaturas.
- WASD mueve la criatura marcada como player.
- Score incrementa cada tick mientras el player está vivo.
- Si player `qe < QE_MIN_EXISTENCE` → `SurvivalState.alive = false`.

### Encapsulamiento
- `grep -r "survival" src/ | grep -v bin/survival | grep -v test` retorna 0 resultados.
- Zero cambios en `src/simulation/`, `src/layers/`, `src/batch/`.

### Performance
- 60 FPS con 50+ entidades en arena.
- Player input responsive (< 1 frame lag).

---

## Referencias

- `src/bin/evolve_and_view.rs` — pattern de referencia (spawn + render + camera)
- `src/sim_world.rs` — `InputCommand` API
- `src/layers/will.rs` — `WillActuator`, `PlayerControlled`
- `src/batch/bridge.rs` — `genome_to_components`, `load_genomes`
