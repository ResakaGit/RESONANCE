# Sprint ET-1 — Associative Memory: Historia Energética como Ventaja Competitiva

**Módulo:** `src/layers/memory.rs` (nuevo), `src/blueprint/equations/emergence/memory.rs` (nuevo)
**Tipo:** Nueva capa (L14) + ecuaciones puras.
**Tier:** T1-1 — Individual Adaptation. **Onda:** 0.
**BridgeKind:** `AssociativeDecayBridge` — cache Small(64), clave `(entity_id, stimulus_hash)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: que ya existe

- `layers/inference.rs::ThreatMemory` — memoria binaria de amenazas con decay fijo. Caso degenerado de AssociativeMemory.
- `simulation/behavior.rs::SensoryAwareness` — detecta estímulos pero no recuerda outcomes.
- `blueprint/equations/determinism.rs::hash_f32_slice` — para calcular stimulus_hash.
- `simulation/time_compat.rs::SimulationClock` — tick_id como reloj canónico.

**Lo que NO existe:**
1. Encoding de outcomes energéticos (positivos/negativos) por estímulo.
2. Decay exponencial de asociaciones en función del tiempo transcurrido.
3. Costo metabólico de mantener memorias (drena qe).
4. Poda automática de memorias no rentables (`net_value < 0`).

---

## Objetivo

Un organismo que recuerda qué estímulos produjeron ganancia/pérdida de qe en el pasado gasta menos energía buscando alimento y evita mejor las amenazas. La memoria tiene un costo metabólico — es una inversión, no una propiedad gratuita.

```
association_strength(stimulus, t) = Σ_i outcome_i × e^(-decay × (t - t_i))
net_value(assoc) = E[future_benefit] - retrieval_cost - maintenance_cost
prune if: net_value < 0
```

---

## Responsabilidades

### ET-1A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/memory.rs

/// Fuerza de una asociación: decae exponencialmente con el tiempo.
/// outcome_qe: qe ganado (>0) o perdido (<0). elapsed: ticks desde la experiencia.
pub fn association_strength(outcome_qe: f32, elapsed_ticks: u64, decay_rate: f32) -> f32 {
    outcome_qe * (-decay_rate * elapsed_ticks as f32).exp()
}

/// Valor esperado de un estímulo dado historial de asociaciones.
/// entries: slice de (outcome_qe, tick_occurred). current_tick para calcular elapsed.
pub fn expected_stimulus_value(
    entries: &[(f32, u64)],   // (outcome_qe, tick_occurred)
    current_tick: u64,
    decay_rate: f32,
) -> f32 {
    entries.iter()
        .map(|&(outcome, t)| {
            let elapsed = current_tick.saturating_sub(t);
            association_strength(outcome, elapsed, decay_rate)
        })
        .sum()
}

/// Costo de mantenimiento de N memorias activas. Drena BaseEnergy.qe.
pub fn memory_maintenance_cost(entry_count: u8, cost_per_entry: f32) -> f32 {
    entry_count as f32 * cost_per_entry
}

/// ¿Vale la pena mantener esta asociación?
/// benefit_estimate: E[uso futuro × ganancia por uso].
pub fn is_memory_profitable(
    strength: f32,
    benefit_estimate: f32,
    maintenance_cost: f32,
) -> bool {
    strength.abs() * benefit_estimate > maintenance_cost
}

/// hash determinista de estímulo: frecuencia + posición relativa cuantizada.
pub fn stimulus_hash(freq_hz: f32, rel_pos_x: f32, rel_pos_y: f32) -> u32 {
    let f = (freq_hz * 10.0) as u32;
    let x = (rel_pos_x.signum() as i32 + 1) as u32;  // -1→0, 0→1, +1→2
    let y = (rel_pos_y.signum() as i32 + 1) as u32;
    f.wrapping_mul(31).wrapping_add(x.wrapping_mul(17)).wrapping_add(y)
}
```

### ET-1B: Componente

```rust
// src/layers/memory.rs

/// Entrada de memoria: un outcome energético asociado a un estímulo.
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct MemoryEntry {
    pub stimulus_hash: u32,   // hash del estímulo (freq + dirección)
    pub outcome_qe: f32,      // qe ganado (+) o perdido (-)
    pub tick_occurred: u64,   // cuándo ocurrió (tick_id canónico)
    pub strength: f32,        // fuerza actual (pre-calculada en update)
}

/// Capa T1-1: AssociativeMemory — historial energético indexado por estímulo.
/// Array fijo — sin Vec, sin heap. Max 4 campos.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct AssociativeMemory {
    pub entries:      [MemoryEntry; MAX_MEMORY_ENTRIES],   // historial fijo
    pub entry_count:  u8,                                   // entradas activas [0, MAX]
    pub decay_rate:   f32,                                  // τ de decaimiento
    pub cost_per_entry: f32,                                // qe/tick por entrada activa
}

pub const MAX_MEMORY_ENTRIES: usize = 8;

impl AssociativeMemory {
    pub fn active_entries(&self) -> &[MemoryEntry] {
        &self.entries[..self.entry_count as usize]
    }
}
```

### ET-1C: Sistema

```rust
// src/simulation/emergence/memory.rs

/// Actualiza AssociativeMemory: recalcula strengths, poda no-rentables, drena qe.
/// Phase::Input, in_set(EmergenceTier1Set), after BehaviorSet::Decide.
pub fn associative_memory_update_system(
    mut agents: Query<(&mut AssociativeMemory, &mut BaseEnergy), With<BehavioralAgent>>,
    clock: Res<SimulationClock>,
) {
    for (mut memory, mut energy) in &mut agents {
        // 1. Recalcular strength de cada entrada
        for i in 0..memory.entry_count as usize {
            let e = &memory.entries[i];
            let elapsed = clock.tick_id.saturating_sub(e.tick_occurred);
            memory.entries[i].strength =
                memory_eq::association_strength(e.outcome_qe, elapsed, memory.decay_rate);
        }

        // 2. Podar entradas no rentables (strength ≈ 0)
        let mut count = memory.entry_count as usize;
        let mut i = 0;
        while i < count {
            if memory.entries[i].strength.abs() < MEMORY_PRUNE_THRESHOLD {
                memory.entries[i] = memory.entries[count - 1];
                memory.entries[count - 1] = MemoryEntry::default();
                count -= 1;
            } else {
                i += 1;
            }
        }
        memory.entry_count = count as u8;

        // 3. Costo metabólico
        let cost = memory_eq::memory_maintenance_cost(
            memory.entry_count, memory.cost_per_entry,
        );
        let new_qe = (energy.qe() - cost).max(0.0);
        if energy.qe() != new_qe { energy.set_qe(new_qe); }
    }
}

/// Registra un outcome en AssociativeMemory cuando ocurre una experiencia.
/// Llamado desde sistemas de catalysis/predation que producen ΔE.
pub fn record_memory_outcome(
    memory: &mut AssociativeMemory,
    stimulus_hash: u32,
    outcome_qe: f32,
    tick_id: u64,
) {
    if memory.entry_count as usize < MAX_MEMORY_ENTRIES {
        let idx = memory.entry_count as usize;
        memory.entries[idx] = MemoryEntry {
            stimulus_hash,
            outcome_qe,
            tick_occurred: tick_id,
            strength: outcome_qe,  // strength inicial = outcome
        };
        memory.entry_count += 1;
    } else {
        // Reemplazar la entrada más débil (LRU por strength)
        let weakest = memory.entries[..MAX_MEMORY_ENTRIES]
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.strength.abs().partial_cmp(&b.strength.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        memory.entries[weakest] = MemoryEntry {
            stimulus_hash, outcome_qe, tick_occurred: tick_id, strength: outcome_qe,
        };
    }
}
```

### ET-1D: Constantes y BridgeKind

```rust
// src/bridge/config.rs — agregar:
pub struct AssociativeDecayBridge;
impl BridgeKind for AssociativeDecayBridge {}

// src/blueprint/constants/emergence/memory.rs
pub const MEMORY_PRUNE_THRESHOLD: f32 = 0.01;   // strength mínima para mantener
pub const MEMORY_DEFAULT_DECAY_RATE: f32 = 0.005; // τ por tick (~200 ticks halflife)
pub const MEMORY_DEFAULT_COST_PER_ENTRY: f32 = 0.1; // qe/tick por entrada
```

---

## Tacticas

- **Array fijo, no Vec.** `[MemoryEntry; 8]` — cero allocations en hot path. Reemplaza LRU por strength.
- **BridgeCache para `expected_stimulus_value`.** Clave `(entity_id, stimulus_hash)` — misma entidad rara vez recibe el mismo estímulo dos veces en el mismo tick, pero el sistema de comportamiento puede consultarlo múltiples veces.
- **ThreatMemory como caso especial.** `ThreatMemory` existente es `AssociativeMemory` con `outcome_qe = -qe_amenaza`. No se elimina — se reutiliza o se deja coexistir para compatibilidad retroactiva.
- **record_memory_outcome es función libre.** La llaman sistemas externos (catalysis, trophic) via `&mut AssociativeMemory` — no toca ECS World.

---

## NO hace

- No reemplaza ThreatMemory existente — coexisten.
- No implementa memoria episódica (secuencias de eventos) — sólo asociaciones estímulo→outcome.
- No modifica BehaviorMode directamente — el behavior system consulta `expected_stimulus_value` para tomar decisiones.

---

## Dependencias

- `layers/base_energy.rs::BaseEnergy` — drain de qe por maintenance_cost.
- `simulation/behavior.rs::BehavioralAgent`, `BehaviorSet::Decide` — inserción en pipeline.
- `simulation/time_compat.rs::SimulationClock` — tick_id canónico.
- `src/bridge/config.rs` — `AssociativeDecayBridge` (nuevo kind).

---

## Criterios de Aceptación

### ET-1A (Ecuaciones)
- `association_strength(10.0, 0, 0.005)` → `10.0`.
- `association_strength(10.0, 200, 0.005)` → `≈ 3.68` (e^-1).
- `association_strength(10.0, 1000, 0.005)` → `< 0.01` (prácticamente muerto).
- `expected_stimulus_value(&[(10.0, 100), (-5.0, 50)], 150, 0.005)` → suma de los dos strengths calculados.
- `memory_maintenance_cost(4, 0.1)` → `0.4`.
- `stimulus_hash(440.0, 1.0, 0.0)` — determinista: misma entrada → mismo hash.
- Test determinismo: dos runs con mismas entradas → resultados byte-idénticos.

### ET-1C (Sistema)
- Test (MinimalPlugins): entidad con 8 entries, todas con strength < threshold → `entry_count == 0` después de 1 update.
- Test: `record_memory_outcome` con array lleno → reemplaza la entrada más débil.
- Test: maintenance_cost drena `BaseEnergy.qe` en cada tick.
- Test: `BaseEnergy.qe == 0` → no entra en negativo.

### General
- `cargo test --lib` sin regresión.
- Cero allocations en `associative_memory_update_system` (verificable con miri o profiler).
- Sin `Vec`, sin `Box`, sin `String` en el componente.

---

## Referencias

- `src/layers/inference.rs::ThreatMemory` — caso degenerado existente
- `src/bridge/config.rs` — patrón BridgeKind
- Blueprint §T1-1: "Associative Memory", ecuaciones de association_strength
- `docs/arquitectura/blueprint_emergence_tiers.md` — contrato del tier
