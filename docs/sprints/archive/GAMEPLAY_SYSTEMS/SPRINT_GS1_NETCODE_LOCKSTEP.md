# Sprint GS-1 — Netcode Lockstep Determinista

**Modulo:** `src/simulation/netcode/lockstep.rs` (nuevo), `src/blueprint/equations/netcode.rs` (nuevo), `src/blueprint/constants/netcode.rs` (nuevo)
**Tipo:** Ecuaciones puras + tipos + plugin headless.
**Onda:** 0 — Bloquea GS-2 (rollback).
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `src/sim_world.rs` — `SimWorld::tick(inputs)` ya recibe `&[InputCommand]`. El contrato está definido.
- `src/blueprint/equations/determinism.rs` — `hash_f32_slice`, `snapshot_hash`, `snapshots_match`. Funciones de verificación determinista.
- `src/simulation/time_compat.rs` — Reloj de simulación determinista.
- SF-5 (Checkpoint) — Save/load de SimWorld. **Prerrequisito no bloqueante para Onda 0, sí para Onda A.**
- `serde` + `ron` + `serde_json` en Cargo.toml.

**Lo que NO existe:**

1. **Protocolo de input.** No hay empaquetado de inputs por tick con `tick_id`.
2. **Checksum por tick.** No hay hash del estado completo para verificación de desync.
3. **Input delay buffer.** No hay mecanismo para acumular inputs de todos los jugadores antes de avanzar.
4. **Desync detection.** Sin comparación de checksums entre clientes.
5. `NetcodePlugin` — cero abstracción de red en el codebase.

---

## Objetivo

Implementar el modelo lockstep determinista: el universo avanza sólo cuando se conocen los inputs de todos los jugadores para el tick T. El protocolo garantiza universos byte-idénticos en todos los clientes sin transmitir world state.

```
SimWorld(T+1) = tick(SimWorld(T), inputs(all_players, T))
Invariante: SimWorld_A(T) ≡ SimWorld_B(T)   ∀ T
```

---

## Responsabilidades

### GS-1A: Ecuaciones puras de netcode

```rust
// src/blueprint/equations/netcode.rs

/// Delay de input requerido para absorber la latencia más alta del grupo.
/// Resultado en ticks. Mínimo 1.
pub fn input_delay_ticks(max_rtt_ms: f32, tick_rate_hz: f32) -> u32 {
    ((max_rtt_ms * 0.5 / 1000.0) * tick_rate_hz).ceil().max(1.0) as u32
}

/// Costo de corrección en ticks (para rollback planning).
pub fn correction_cost_ticks(ticks_since_divergence: u32) -> u32 {
    ticks_since_divergence  // 1:1 — cada tick divergido requiere un re-tick
}

/// ¿Es el delay aceptable para juego competitivo?
/// Umbrales: ≤3 ticks = imperceptible, ≤6 = aceptable, >8 = degradado.
pub fn is_delay_acceptable(delay_ticks: u32) -> bool {
    delay_ticks <= 6
}

/// Hash rapido del estado para verificación de desync. No criptográfico.
/// Delega a determinism::hash_f32_slice pero con firma semántica.
pub fn tick_checksum(energy_snapshot: &[f32]) -> u64 {
    crate::blueprint::equations::determinism::hash_f32_slice(energy_snapshot)
}
```

### GS-1B: Tipos del protocolo lockstep

```rust
// src/simulation/netcode/lockstep.rs

/// Input de un jugador para el tick T. Transmitido sobre red.
/// 4 campos máximo (DOD). Sin String — player_id es u8.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]  // transient por tick
pub struct InputPacket {
    pub tick_id: u64,
    pub player_id: u8,
    pub command_count: u8,  // 0..=MAX_COMMANDS_PER_TICK
    pub checksum: u64,      // hash del estado local en tick_id-1
}

/// Resultado de verificación de desync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesyncResult {
    Synchronized,
    Desynced { player_a: u8, player_b: u8, tick_id: u64 },
}

/// Config del lockstep session. Resource.
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct LockstepConfig {
    pub player_count: u8,
    pub input_delay_ticks: u32,
    pub tick_rate_hz: f32,
}

impl Default for LockstepConfig {
    fn default() -> Self {
        Self { player_count: 2, input_delay_ticks: 3, tick_rate_hz: 20.0 }
    }
}

/// Buffer de inputs pendientes por tick_id. Resource.
/// Vec<(tick_id, player_id, commands)> ordenado por tick_id.
/// NO HashMap — orden determinista requerido (Hard Block 5).
#[derive(Resource, Default, Debug)]
pub struct InputBuffer {
    pub entries: Vec<BufferedInput>,
}

#[derive(Debug, Clone)]
pub struct BufferedInput {
    pub tick_id: u64,
    pub player_id: u8,
    pub commands: Vec<crate::sim_world::InputCommand>,
}

/// Checksums por tick para detección de desync.
#[derive(Resource, Default, Debug)]
pub struct ChecksumLog {
    /// (tick_id, player_id, checksum) — Vec ordenado, sin HashMap.
    pub entries: Vec<(u64, u8, u64)>,
}
```

### GS-1C: Sistema lockstep (Phase::Input, antes de PlatformWill)

```rust
/// Verifica que todos los inputs del tick T están disponibles antes de avanzar.
/// Si faltan inputs, el tick NO avanza (el sistema retorna sin modificar SimulationClock).
pub fn lockstep_input_gate_system(
    config: Res<LockstepConfig>,
    buffer: Res<InputBuffer>,
    clock: Res<SimulationClock>,
    mut run_condition: ResMut<LockstepRunCondition>,
) {
    let target_tick = clock.tick_id + config.input_delay_ticks as u64;
    let received = buffer.entries.iter()
        .filter(|e| e.tick_id == target_tick)
        .count();
    run_condition.can_advance = received >= config.player_count as usize;
}

/// Colecta checksum del estado actual y lo registra.
/// Corre después del tick, en PostUpdate.
pub fn lockstep_checksum_record_system(
    query: Query<(&WorldEntityId, &BaseEnergy)>,
    clock: Res<SimulationClock>,
    config: Res<LockstepConfig>,
    mut log: ResMut<ChecksumLog>,
) {
    let mut energies: Vec<(u32, f32)> = query
        .iter()
        .map(|(id, e)| (id.0, e.qe()))
        .collect();
    energies.sort_unstable_by_key(|(id, _)| *id);
    let values: Vec<f32> = energies.into_iter().map(|(_, qe)| qe).collect();
    let checksum = netcode_eq::tick_checksum(&values);
    // player_id=255 = local player sentinel
    log.entries.push((clock.tick_id, 255, checksum));
    // Prune entries > 120 ticks old (6 second window)
    let cutoff = clock.tick_id.saturating_sub(120);
    log.entries.retain(|(tick, _, _)| *tick >= cutoff);
}

/// Verifica checksums recibidos de peers. Emite DesyncEvent si divergen.
pub fn lockstep_desync_check_system(
    log: Res<ChecksumLog>,
    clock: Res<SimulationClock>,
    mut events: EventWriter<DesyncEvent>,
) {
    // Para cada tick con >1 checksum registrado, verificar que son iguales.
    // Implementación: agrupar por tick_id, comparar pares.
}
```

### GS-1D: Constantes

```rust
// src/blueprint/constants/netcode.rs

/// Delay máximo aceptable para juego competitivo (ticks a 20Hz = 300ms).
pub const LOCKSTEP_MAX_ACCEPTABLE_DELAY_TICKS: u32 = 6;
/// Delay imperceptible (ticks a 20Hz = 150ms).
pub const LOCKSTEP_IMPERCEPTIBLE_DELAY_TICKS: u32 = 3;
/// Tamaño máximo del checksum log antes de purga.
pub const CHECKSUM_LOG_MAX_ENTRIES: usize = 240;
/// Comandos máximos por jugador por tick.
pub const MAX_COMMANDS_PER_TICK: u8 = 8;
```

---

## Tacticas

- **Stateless-first.** `lockstep_input_gate_system` sólo lee — la decisión de avanzar o no es un Resource booleano (`LockstepRunCondition`), no lógica en sistemas de física.
- **Checksums light.** Usar `hash_f32_slice` de determinism existente. No SHA, no MD5.
- **Vec ordenado sobre HashMap.** `InputBuffer.entries` y `ChecksumLog.entries` usan Vec con sort determinista. No HashMap.
- **Input delay configurable.** `LockstepConfig` es un Resource RON-loadable. No constante hard-coded.

---

## NO hace

- No implementa transporte de red (TCP/UDP). La red es responsabilidad de `resonance-app`.
- No implementa rollback — eso es GS-2.
- No serializa SimWorld — eso es SF-5 (prerrequisito).
- No afecta lógica de física existente.

---

## Dependencias

- `src/sim_world.rs` — `InputCommand` (tipo de input ya definido).
- `src/blueprint/equations/determinism.rs` — `hash_f32_slice` (reutilizado en checksum).
- `src/blueprint/ids/types.rs` — `WorldEntityId` (para canonical ordering en checksum).
- SF-5 (Checkpoint) — Para snapshot de estado inicial (sync inicial y recovery).
- `src/simulation/time_compat.rs` — `SimulationClock` (tick_id como reloj canónico).

---

## Criterios de aceptacion

### GS-1A (Ecuaciones)
- `input_delay_ticks(100.0, 20.0)` → `1` (50ms / 20Hz = 1 tick).
- `input_delay_ticks(300.0, 20.0)` → `3` (150ms / 20Hz = 3 ticks).
- `is_delay_acceptable(6)` → `true`. `is_delay_acceptable(9)` → `false`.
- `tick_checksum(&[1.0, 2.0]) == tick_checksum(&[1.0, 2.0])` → determinista.
- `tick_checksum(&[1.0, 2.0]) != tick_checksum(&[2.0, 1.0])` → orden importa.

### GS-1B (Tipos)
- `InputBuffer::default()` — zero entries.
- `ChecksumLog::default()` — zero entries.
- `LockstepConfig::default()` — `player_count=2`, `input_delay_ticks=3`.

### GS-1C (Sistemas)
- Test MinimalPlugins: gate sin inputs → `can_advance=false`.
- Test MinimalPlugins: gate con 2 inputs del tick correcto → `can_advance=true`.
- Test: checksum con 0 entidades → hash estable entre runs.
- Test: checksum con N entidades → diferente si qe difiere.

### General
- `cargo test --lib` sin regresión.
- Sin `unsafe`, sin `HashMap` en hot paths.

---

## Referencias

- `src/sim_world.rs` — `InputCommand`, `TickId`, `SimWorld`
- `src/blueprint/equations/determinism.rs` — `hash_f32_slice`
- `docs/design/SIMULATION_CORE_DECOUPLING.md` — INV-4 (determinismo), INV-8 (tick_id como reloj)
- Whitepaper: "1500 Archers on a 28.8: Network Programming in Age of Empires" (lockstep reference)
