# Sprint BS-0 — Arena Prototype: EntitySlot + SimWorldFlat + 3 Systems

**Modulo:** `src/batch/arena.rs`, `src/batch/pipeline.rs`, `src/batch/scratch.rs`, `src/batch/constants.rs`
**Tipo:** Tipos core + pipeline minimo + benchmark.
**Onda:** Fundacion — sin bloqueantes.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SI existe:**

- `src/blueprint/equations/core_physics.rs` — `dissipation_loss_dt`, `drag_force`, `kinetic_energy`.
- `src/blueprint/equations/energy_competition/pool_equations.rs` — extraction, conservation.
- `src/blueprint/constants/` — 70+ shards de tuning reutilizables.
- `src/sim_world.rs` — contrato `SimWorld` con `tick()`, `snapshot()`, invariantes INV-4/7/8.
- `src/blueprint/equations/conservation.rs` — `is_valid_qe`, `global_conservation_error`.

**Lo que NO existe:**

1. **EntitySlot.** No hay struct plano `repr(C)` para entidad sin Bevy.
2. **SimWorldFlat.** No hay mundo completo sin `bevy::App`.
3. **ScratchPad.** No hay buffers thread-local pre-allocados.
4. **Pipeline batch.** No hay `tick()` como secuencia de fn calls.
5. **Benchmark a 1M mundos.** No hay prueba de escala.

---

## Objetivo

Crear el nucleo minimo del simulador batch: tipos de datos (`EntitySlot`, `SimWorldFlat`, `ScratchPad`), 3 systems representativos (dissipation, movement, collision), y validar que 1M mundos corren 100 ticks sin panic ni violacion de conservacion.

---

## Responsabilidades

### BS-0A: EntitySlot

```rust
// src/batch/arena.rs

/// Entidad como struct plano. Sin heap. Sin Option. Sin Vec.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C, align(64))]
pub struct EntitySlot {
    pub alive:        bool,
    pub archetype:    u8,
    pub entity_id:    u32,
    pub qe:           f32,
    pub radius:       f32,
    pub frequency_hz: f32,
    pub phase:        f32,
    pub velocity:     [f32; 2],
    pub dissipation:  f32,
    pub matter_state: u8,
    pub bond_energy:  f32,
    pub conductivity: f32,
    pub engine_buffer: f32,
    pub engine_max:   f32,
    pub input_valve:  f32,
    pub output_valve: f32,
    pub pressure_dqe: f32,
    pub viscosity:    f32,
    pub will_intent:  [f32; 2],
    pub channeling:   bool,
    pub faction:      u8,
    pub adapt_rate_hz: f32,
    pub stability_band: f32,
    pub position:     [f32; 2],
    pub growth_bias:  f32,
    pub mobility_bias: f32,
    pub branching_bias: f32,
    pub resilience:   f32,
    pub trophic_class: u8,
    pub satiation:    f32,
    pub expression_mask: [f32; 4],
    pub _pad:         [u8; 3],
}
```

**Invariantes:**
- `size_of::<EntitySlot>()` multiplo de 64 (cache line aligned).
- `Copy` obligatorio — sin heap.
- `Default` produce slot muerto (`alive = false`, todo cero).

### BS-0B: SimWorldFlat

```rust
// src/batch/arena.rs

pub const MAX_ENTITIES: usize = 64;
pub const GRID_CELLS: usize = 256;  // 16×16

#[derive(Clone)]
#[repr(C)]
pub struct SimWorldFlat {
    pub tick_id:         u64,
    pub seed:            u64,
    pub dt:              f32,
    pub entity_count:    u8,
    pub alive_mask:      u64,
    pub next_id:         u32,
    pub entities:        [EntitySlot; MAX_ENTITIES],
    pub total_qe:        f32,
    pub nutrient_grid:   [f32; GRID_CELLS],
    pub irradiance_grid: [f32; GRID_CELLS],
}

impl SimWorldFlat {
    /// Inicializa mundo vacio con seed.
    pub fn new(seed: u64, dt: f32) -> Self { ... }

    /// Spawn entidad en primer slot libre. Retorna indice o None si lleno.
    pub fn spawn(&mut self, slot: EntitySlot) -> Option<usize> { ... }

    /// Mata entidad en indice. Limpia slot, actualiza alive_mask.
    pub fn kill(&mut self, idx: usize) { ... }

    /// Primer slot libre (alive_mask trailing zeros).
    pub fn first_free_slot(&self) -> Option<usize> { ... }

    /// Limpia entidades muertas (qe < QE_MIN_EXISTENCE).
    pub fn reap_dead(&mut self) { ... }

    /// Actualiza total_qe desde suma de entidades vivas.
    pub fn update_total_qe(&mut self) { ... }

    /// Debug: assert INV-B2 conservation.
    pub fn assert_conservation(&self) { ... }
}
```

### BS-0C: ScratchPad

```rust
// src/batch/scratch.rs

pub struct ScratchPad {
    pub pairs:        [(u8, u8); 2048],
    pub pairs_len:    usize,
    pub neighbors:    [u8; MAX_ENTITIES],
    pub neighbors_len: usize,
    pub deaths:       [u8; MAX_ENTITIES],
    pub deaths_len:   usize,
    pub meme_candidates: [(u8, u32, f32); 64],
    pub meme_len:     usize,
}

impl ScratchPad {
    pub fn new() -> Self { ... }
    pub fn clear(&mut self) { ... }
}
```

### BS-0D: Tres systems minimos

```rust
// src/batch/pipeline.rs

/// Dissipation: L3 flow → L0 energy drain.
/// Calls equations::dissipation_loss_dt(qe, dissipation, dt).
pub fn dissipation(world: &mut SimWorldFlat) {
    let dt = world.dt;
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let e = &mut world.entities[i];
        let loss = equations::dissipation_loss_dt(e.qe, e.dissipation, dt);
        e.qe = (e.qe - loss).max(0.0);
    }
}

/// Movement: L3 velocity → position integration.
pub fn movement_integrate(world: &mut SimWorldFlat) {
    let dt = world.dt;
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let e = &mut world.entities[i];
        e.position[0] += e.velocity[0] * dt;
        e.position[1] += e.velocity[1] * dt;
    }
}

/// Collision: N² brute force, radius overlap → energy exchange.
pub fn collision(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    scratch.pairs_len = 0;
    // Collect pairs
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        for j in (i+1)..MAX_ENTITIES {
            if world.alive_mask & (1 << j) == 0 { continue; }
            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            let dist_sq = dx * dx + dy * dy;
            let r_sum = world.entities[i].radius + world.entities[j].radius;
            if dist_sq < r_sum * r_sum {
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }
    // Resolve pairs: energy exchange via equations::interference()
    for p in 0..scratch.pairs_len {
        let (i, j) = scratch.pairs[p];
        let (i, j) = (i as usize, j as usize);
        let transfer = equations::interference(
            world.entities[i].frequency_hz,
            world.entities[j].frequency_hz,
            world.entities[i].phase,
            world.entities[j].phase,
        );
        let amount = transfer.abs().min(world.entities[i].qe).min(world.entities[j].qe) * 0.01;
        if transfer > 0.0 {
            world.entities[i].qe -= amount;
            world.entities[j].qe += amount;
        } else {
            world.entities[j].qe -= amount;
            world.entities[i].qe += amount;
        }
    }
}
```

### BS-0E: tick() y benchmark

```rust
impl SimWorldFlat {
    pub fn tick(&mut self, scratch: &mut ScratchPad) {
        scratch.clear();
        self.tick_id += 1;

        dissipation(self);
        movement_integrate(self);
        collision(self, scratch);

        self.reap_dead();
        self.update_total_qe();

        #[cfg(debug_assertions)]
        self.assert_conservation();
    }
}
```

### BS-0F: Constantes

```rust
// src/batch/constants.rs

pub const MAX_ENTITIES: usize = 64;
pub const GRID_CELLS: usize = 256;
pub const GRID_SIDE: usize = 16;

// Re-exports de blueprint
pub use crate::blueprint::constants::QE_MIN_EXISTENCE;
pub use crate::blueprint::constants::DISSIPATION_RATE_DEFAULT;
```

---

## Tacticas

- **sizeof assertion.** `const _: () = assert!(size_of::<EntitySlot>() % 64 == 0);` en arena.rs.
- **alive_mask bitops.** `first_free_slot` = `(!alive_mask).trailing_zeros()`. `kill(i)` = `alive_mask &= !(1 << i)`.
- **No Bevy en batch/.** Ningun `use bevy::` en ningun archivo del modulo.
- **Benchmark con criterion.** `benches/batch_benchmark.rs` con 1M mundos × 1 tick.

---

## NO hace

- No implementa systems Tier 2 ni Tier 3 — solo dissipation, movement, collision.
- No implementa GeneticHarness ni GenomeBlob — eso es BS-4.
- No usa rayon — eso es BS-6. Este sprint es single-threaded.
- No implementa EventBuffer — eso es BS-2.
- No toca ningun archivo existente fuera de `src/batch/` y `src/lib.rs` (pub mod batch).

---

## Dependencias

- `crate::blueprint::equations::core_physics` — `dissipation_loss_dt`.
- `crate::blueprint::equations::core_physics` — `interference`.
- `crate::blueprint::constants` — `QE_MIN_EXISTENCE`, `DISSIPATION_RATE_DEFAULT`.

---

## Criterios de aceptacion

### BS-0A (EntitySlot)
- `size_of::<EntitySlot>() % 64 == 0`.
- `EntitySlot::default().alive == false`.
- Todos los campos `f32` default = `0.0`.

### BS-0B (SimWorldFlat)
- `SimWorldFlat::new(42, 0.05).tick_id == 0`.
- `spawn` → `alive_mask` bit seteado. `kill` → bit limpiado.
- `first_free_slot` retorna `None` cuando lleno (64 entidades).
- `reap_dead` elimina entidades con `qe < QE_MIN_EXISTENCE`.

### BS-0C (ScratchPad)
- `ScratchPad::new()` — todos los `_len = 0`.
- `clear()` resetea todos los contadores.

### BS-0D (Systems)
- `dissipation` reduce `qe` de entidades vivas. No toca muertas.
- `movement_integrate` desplaza `position` por `velocity * dt`.
- `collision` detecta pares con overlap y transfiere energia.

### BS-0E (Benchmark)
- 1M mundos × 100 ticks sin panic.
- `total_qe` nunca negativo en ningun mundo.
- Conservation: `|total_qe_after - total_qe_before| < 1.0` por tick por mundo.

### General
- `cargo test --lib` sin regresion.
- Zero `use bevy::` en `src/batch/`.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` — blueprint completa
- `src/blueprint/equations/core_physics.rs` — `dissipation_loss_dt`, `interference`
- `src/blueprint/equations/conservation.rs` — `is_valid_qe`, `global_conservation_error`
- `src/sim_world.rs` — contrato INV-B1/B2/B3 que este sprint replica
