# Sprint GS-2 — Netcode Rollback: Predicción y Corrección Retroactiva

**Modulo:** `src/simulation/netcode/rollback.rs` (nuevo), `src/blueprint/equations/rollback.rs` (nuevo)
**Tipo:** Ecuaciones puras + tipos + sistema de corrección.
**Onda:** A — Requiere GS-1 (lockstep) + SF-5 (checkpoint).
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `src/sim_world.rs` — `SimWorld::tick()` es pura: estado T + inputs → estado T+1. Rollback legal.
- `src/simulation/netcode/lockstep.rs` (GS-1) — `InputBuffer`, `ChecksumLog`, `LockstepConfig`.
- `src/blueprint/equations/determinism.rs` — `hash_f32_slice`, `snapshot_hash`.
- SF-5 (Checkpoint) — `SimWorld` save/restore. Prerrequisito funcional.
- `src/blueprint/equations/netcode.rs` (GS-1) — `correction_cost_ticks`.

**Lo que NO existe:**

1. **Predicción de input.** No hay mecanismo para predecir el input de un jugador ausente y simular de igual forma.
2. **Rollback buffer.** No hay historial de snapshots para retroceder N ticks.
3. **Re-simulación.** No hay sistema que replaye ticks desde un snapshot pasado con inputs corregidos.
4. **Confirmación de input.** No hay protocolo para marcar un tick como "input confirmado de todos".
5. **Corrección visual suave.** No hay blend entre estado predicho y estado corregido.

---

## Objetivo

Implementar client-side prediction con corrección retroactiva: el cliente avanza usando inputs predichos (repetir último input), y cuando llegan los inputs reales, retroactivamente re-simula desde el divergence point. Invisible al jugador si la latencia es menor al input delay.

```
Tick T: predecir input de jugador remoto → avanzar
Tick T+delay: recibir input real → ¿igual a predicción? → nada
                                  → diferente → rollback(T) → re-simular
```

---

## Responsabilidades

### GS-2A: Ecuaciones de rollback

```rust
// src/blueprint/equations/rollback.rs

/// Cuántos ticks hay que re-simular dado el punto de divergencia.
/// Igual a correction_cost_ticks de GS-1 pero con nombre semántico.
pub fn resimulation_ticks(current_tick: u64, divergence_tick: u64) -> u32 {
    current_tick.saturating_sub(divergence_tick) as u32
}

/// ¿Conviene rollback o sincronización completa?
/// Si el costo supera el umbral, full resync es más barato.
pub fn prefer_resync(resim_ticks: u32, max_rollback_window: u32) -> bool {
    resim_ticks > max_rollback_window
}

/// Blend de posición entre estado predicho y corregido para suavizar pop visual.
/// alpha ∈ [0,1]: 0 = totalmente predicho, 1 = totalmente corregido.
pub fn correction_blend(predicted: f32, corrected: f32, alpha: f32) -> f32 {
    predicted + (corrected - predicted) * alpha.clamp(0.0, 1.0)
}

/// Velocidad de blend por tick para llegar a corregido en N ticks.
pub fn blend_alpha_per_tick(blend_ticks: u32) -> f32 {
    if blend_ticks == 0 { return 1.0; }
    1.0 / blend_ticks as f32
}
```

### GS-2B: Tipos del buffer de rollback

```rust
// src/simulation/netcode/rollback.rs

/// Frame guardado para posible rollback. Contiene snapshot + inputs usados.
#[derive(Debug, Clone)]
pub struct RollbackFrame {
    pub tick_id: u64,
    pub snapshot_bytes: Vec<u8>,          // SF-5 checkpoint serializado
    pub inputs: Vec<(u8, Vec<InputCommand>)>,  // (player_id, commands) — ordenado
}

/// Buffer de frames para rollback. FIFO con capacidad fija.
/// Vec circular — no heap allocation por tick (usa swap).
#[derive(Resource, Default, Debug)]
pub struct RollbackBuffer {
    pub frames: Vec<RollbackFrame>,       // capacidad = MAX_ROLLBACK_WINDOW
    pub head: usize,                      // índice del frame más reciente
}

impl RollbackBuffer {
    pub fn push(&mut self, frame: RollbackFrame) {
        if self.frames.len() < MAX_ROLLBACK_WINDOW {
            self.frames.push(frame);
        } else {
            self.frames[self.head] = frame;
        }
        self.head = (self.head + 1) % MAX_ROLLBACK_WINDOW;
    }

    pub fn frame_at(&self, tick_id: u64) -> Option<&RollbackFrame> {
        self.frames.iter().find(|f| f.tick_id == tick_id)
    }
}

/// Input predicho: última acción conocida del jugador, usada mientras no llega el real.
#[derive(Debug, Clone, Copy)]
pub struct PredictedInput {
    pub player_id: u8,
    pub tick_id: u64,
    pub is_confirmed: bool,
}

/// Resource de estado del rollback. Un único estado activo.
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct RollbackState {
    pub divergence_tick: Option<u64>,
    pub correction_in_progress: bool,
    pub blend_alpha: f32,   // progreso del blend visual post-corrección
}
```

### GS-2C: Sistemas de rollback

```rust
/// Guarda snapshot del tick actual para posible rollback futuro.
/// Phase::Input, antes de lockstep_input_gate_system.
/// Sólo guarda si LockstepRunCondition::can_advance.
pub fn rollback_frame_save_system(
    sim: Res<SimWorldHandle>,
    clock: Res<SimulationClock>,
    buffer: Res<InputBuffer>,
    mut rollback: ResMut<RollbackBuffer>,
    run_cond: Res<LockstepRunCondition>,
) {
    if !run_cond.can_advance { return; }
    let snapshot_bytes = sim.checkpoint_bytes();  // SF-5
    let inputs = buffer.entries.iter()
        .filter(|e| e.tick_id == clock.tick_id)
        .map(|e| (e.player_id, e.commands.clone()))
        .collect();
    rollback.push(RollbackFrame {
        tick_id: clock.tick_id,
        snapshot_bytes,
        inputs,
    });
}

/// Detecta divergencia comparando checksum recibido vs local.
/// PostUpdate, después de lockstep_checksum_record_system.
pub fn rollback_detect_system(
    log: Res<ChecksumLog>,
    mut rollback_state: ResMut<RollbackState>,
    config: Res<LockstepConfig>,
) {
    // Buscar ticks con >1 checksum (local + remote)
    // Si difieren → establecer divergence_tick
    // Si resim_ticks > max → solicitar full resync en vez de rollback
    let _ = (log, rollback_state, config);
}

/// Aplica corrección retroactiva si hay divergencia.
/// PostUpdate, después de rollback_detect_system.
pub fn rollback_apply_system(
    mut sim: ResMut<SimWorldHandle>,
    mut rollback_state: ResMut<RollbackState>,
    rollback: Res<RollbackBuffer>,
    buffer: Res<InputBuffer>,
    clock: Res<SimulationClock>,
) {
    let Some(div_tick) = rollback_state.divergence_tick else { return; };
    let resim = rollback_eq::resimulation_ticks(clock.tick_id, div_tick);
    if rollback_eq::prefer_resync(resim, MAX_ROLLBACK_WINDOW as u32) {
        // solicitar resync completo — fuera del scope de GS-2
        rollback_state.divergence_tick = None;
        return;
    }
    let Some(frame) = rollback.frame_at(div_tick) else { return; };
    sim.restore_checkpoint(&frame.snapshot_bytes);  // SF-5
    // Re-simular desde div_tick hasta tick actual con inputs corregidos
    for t in div_tick..clock.tick_id {
        let cmds = buffer.entries.iter()
            .filter(|e| e.tick_id == t)
            .flat_map(|e| e.commands.iter().cloned())
            .collect::<Vec<_>>();
        sim.tick(&cmds);
    }
    rollback_state.divergence_tick = None;
    rollback_state.correction_in_progress = true;
    rollback_state.blend_alpha = 0.0;
}
```

### GS-2D: Constantes

```rust
// src/blueprint/constants/netcode.rs — ampliar GS-1

/// Ventana máxima de rollback en ticks (5 segundos a 20Hz).
pub const MAX_ROLLBACK_WINDOW: usize = 100;
/// Ticks para suavizar la corrección visual post-rollback.
pub const CORRECTION_BLEND_TICKS: u32 = 4;
/// Umbral para preferir resync completo sobre rollback.
pub const RESYNC_THRESHOLD_TICKS: u32 = 60;
```

---

## Tacticas

- **SimWorld como función pura.** `tick()` es determinista → re-simular es válido. El rollback es un artefacto de la pureza del contrato.
- **Buffer circular sin heap.** `RollbackBuffer` usa Vec pre-allocado con índice circular. Cero allocations en hot path.
- **Blend visual en Transform.** `rollback_apply` recalcula física; el blend suaviza sólo la traducción visual (Update layer), no el estado de simulación.
- **Input prediction = repeat-last.** Modelo más simple que evita predicción compleja. Correcto en 90% de casos (jugadores mantienen dirección).

---

## NO hace

- No implementa transporte de red — eso es responsabilidad de `resonance-app`.
- No implementa interpolación de entidades remotas — sólo corrección de estado propio.
- No serializa el mundo entero — delega a SF-5 (checkpoint).
- No predice habilidades (spells) — sólo inputs de movimiento base.

---

## Dependencias

- GS-1 — `InputBuffer`, `ChecksumLog`, `LockstepConfig`, `LockstepRunCondition`.
- SF-5 — `SimWorld::checkpoint_bytes()` / `restore_checkpoint()`.
- `src/blueprint/equations/rollback.rs` — ecuaciones puras (nuevo).
- `src/blueprint/constants/netcode.rs` — constantes ampliadas.

---

## Criterios de aceptacion

### GS-2A (Ecuaciones)
- `resimulation_ticks(100, 95)` → `5`.
- `prefer_resync(50, 100)` → `false`. `prefer_resync(110, 100)` → `true`.
- `correction_blend(10.0, 20.0, 0.0)` → `10.0`.
- `correction_blend(10.0, 20.0, 1.0)` → `20.0`.
- `blend_alpha_per_tick(4)` → `0.25`.

### GS-2B (Buffer)
- `RollbackBuffer::default()` — frames vacío.
- `push` × `MAX_ROLLBACK_WINDOW+1` → no pánico, descarta frame más viejo.
- `frame_at(tick)` — retorna el frame correcto.

### GS-2C (Sistemas)
- Test (MinimalPlugins + GS-1): divergencia detectada → `rollback_state.divergence_tick` seteado.
- Test: rollback aplicado → estado vuelve a ser consistente con inputs corregidos.
- Test: resim > threshold → `divergence_tick` se limpia sin rollback.

### General
- `cargo test --lib` sin regresión.
- Cero allocations por tick en hot path (buffer pre-allocado).

---

## Referencias

- `src/sim_world.rs` — `SimWorld::tick()` (contrato puro que hace rollback posible)
- `src/simulation/netcode/lockstep.rs` — GS-1 tipos
- `src/blueprint/equations/determinism.rs` — `hash_f32_slice`
- SF-5 Checkpoint — `SimWorld::checkpoint_bytes()` / `restore_checkpoint()`
- Artículo: "GGPO: Rollback Networking" — referencia de implementación
