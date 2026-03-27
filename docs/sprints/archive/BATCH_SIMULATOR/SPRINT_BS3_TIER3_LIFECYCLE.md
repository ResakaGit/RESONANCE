# Sprint BS-3 — Tier 3 Lifecycle: Spawn, Death, Reproduction, Abiogenesis

**Modulo:** `src/batch/systems/morphological.rs` (expandir)
**Tipo:** Systems que modifican `alive_mask` — crean y destruyen entidades.
**Onda:** BS-2 → BS-3.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post BS-2)

- 28 systems funcionando (Tier 1 + Tier 2).
- `EventBuffer` con `deaths` y `reproductions` registrados pero no procesados.
- `SimWorldFlat::spawn()` y `kill()` implementados en BS-0.

---

## Objetivo

Implementar los 5 systems que modifican la poblacion del mundo: muerte por inanicion, reproduccion con herencia + mutacion, abiogenesis espontanea, adaptacion morfologica, y ciclo de vida de organos. Estos son los que hacen que la simulacion sea **evolutiva**.

---

## Systems a implementar

| # | System | Fase | Que hace | Modifica alive_mask |
|---|--------|------|----------|-------------------|
| 1 | `death_reap` | Post-tick | Procesa `events.deaths` → `kill()` | Si (clear bit) |
| 2 | `reproduction` | Morpho | Si `qe > threshold` y cooldown = 0 → spawn hijo | Si (set bit) |
| 3 | `abiogenesis` | Morpho | Si energia ambiental alta → spawn celula primitiva | Si (set bit) |
| 4 | `morpho_adaptation` | Morpho | Bergmann/Allen/Wolff → modifica `InferenceProfile` biases | No |
| 5 | `organ_lifecycle` | Morpho | Lifecycle stage: grow/mature/senesce | No |

---

## Responsabilidades detalladas

### BS-3A: death_reap

```rust
pub fn death_reap(world: &mut SimWorldFlat) {
    // 1. Marcar entidades con qe < QE_MIN_EXISTENCE
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        if world.entities[i].qe < QE_MIN_EXISTENCE {
            world.events.record_death(i as u8);
        }
    }
    // 2. Procesar deaths
    for d in 0..world.events.deaths_len {
        let idx = world.events.deaths[d] as usize;
        // Return nutrients to grid
        let pos = world.entities[idx].position;
        let cell = grid_cell_index(pos, GRID_SIDE);
        world.nutrient_grid[cell] += world.entities[idx].qe * 0.5;  // 50% return
        world.kill(idx);
    }
}
```

### BS-3B: reproduction

```rust
pub fn reproduction(world: &mut SimWorldFlat) {
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let e = &world.entities[i];
        // Guard: enough energy + not at capacity
        if e.qe < constants::REPRODUCTION_THRESHOLD { continue; }

        let Some(child_idx) = world.first_free_slot() else { continue; };

        // Inherit genome with deterministic mutation
        let parent_genome = GenomeBlob::from_slot(e);
        let rng = equations::determinism::next_u64(
            world.seed ^ world.tick_id ^ (i as u64)
        );
        let child_genome = parent_genome.mutate(rng, constants::DEFAULT_MUTATION_SIGMA);

        // Create child slot
        let mut child = EntitySlot::default();
        child.alive = true;
        child.entity_id = world.next_id;
        world.next_id += 1;
        child_genome.apply(&mut child);

        // Energy transfer: 30% parent → child
        let transfer = world.entities[i].qe * 0.3;
        child.qe = transfer;
        world.entities[i].qe -= transfer;

        // Position: near parent
        child.position = [
            e.position[0] + equations::determinism::unit_f32(rng) * e.radius * 2.0,
            e.position[1] + equations::determinism::unit_f32(
                equations::determinism::next_u64(rng)
            ) * e.radius * 2.0,
        ];

        world.entities[child_idx] = child;
        world.alive_mask |= 1 << child_idx;
        world.entity_count += 1;
        world.events.record_reproduction(i as u8, child_idx as u8);
    }
}
```

### BS-3C: abiogenesis

```rust
pub fn abiogenesis(world: &mut SimWorldFlat) {
    // Solo si hay pocos organismos y energia ambiental alta
    if world.entity_count >= constants::ABIOGENESIS_POP_CAP { return; }
    let grid_energy: f32 = world.irradiance_grid.iter().sum();
    if grid_energy < constants::ABIOGENESIS_ENERGY_THRESHOLD { return; }

    let Some(idx) = world.first_free_slot() else { return; };
    let rng = equations::determinism::next_u64(world.seed ^ world.tick_id);

    let mut cell = EntitySlot::default();
    cell.alive = true;
    cell.archetype = 3;  // cell
    cell.entity_id = world.next_id;
    world.next_id += 1;
    cell.qe = constants::ABIOGENESIS_INITIAL_QE;
    cell.radius = constants::ABIOGENESIS_INITIAL_RADIUS;
    cell.frequency_hz = equations::determinism::range_f32(rng, 200.0, 800.0);
    // Random genome
    cell.growth_bias = equations::determinism::unit_f32(rng);
    cell.resilience = 0.5;
    // Position: random on grid
    cell.position = [
        equations::determinism::range_f32(
            equations::determinism::next_u64(rng), 0.0, GRID_SIDE as f32
        ),
        equations::determinism::range_f32(
            equations::determinism::next_u64(equations::determinism::next_u64(rng)),
            0.0, GRID_SIDE as f32
        ),
    ];

    world.entities[idx] = cell;
    world.alive_mask |= 1 << idx;
    world.entity_count += 1;
}
```

### BS-3D: morpho_adaptation

```rust
pub fn morpho_adaptation(world: &mut SimWorldFlat) {
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let e = &mut world.entities[i];
        // Bergmann: cold → increase growth_bias
        let cell = grid_cell_index(e.position, GRID_SIDE);
        let temp = equations::equivalent_temperature(
            equations::density(e.qe, e.radius)
        );
        let bergmann_delta = equations::morpho_adaptation::bergmann_growth_delta(
            temp, constants::MORPHO_TARGET_TEMPERATURE, constants::MORPHO_ADAPTATION_RATE
        );
        e.growth_bias = (e.growth_bias + bergmann_delta).clamp(0.0, 1.0);
        // Wolff: moving → strengthen bonds
        let speed_sq = e.velocity[0] * e.velocity[0] + e.velocity[1] * e.velocity[1];
        let wolff_delta = equations::morpho_adaptation::wolff_bond_delta(
            speed_sq, constants::WOLFF_SEDENTARY_SPEED
        );
        e.bond_energy = (e.bond_energy + wolff_delta).max(0.0);
    }
}
```

### BS-3E: organ_lifecycle

```rust
pub fn organ_lifecycle(world: &mut SimWorldFlat) {
    // Simplified lifecycle: track age via tick_id - spawn_tick (implicit)
    // Entities that exceed a maturity threshold stop growing
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let e = &mut world.entities[i];
        // Age proxy: entity_id was assigned at spawn, lower = older
        // Growth slows as qe stabilizes (logistic curve)
        let growth_factor = equations::growth_engine::logistic_growth_rate(
            e.radius, e.growth_bias * constants::ALLOMETRIC_MAX_RADIUS
        );
        e.radius += growth_factor * world.dt;
    }
}
```

---

## Ecuaciones nuevas requeridas

| Funcion | Archivo | Status |
|---------|---------|--------|
| `determinism::unit_f32(state) -> f32` | `determinism.rs` | Nuevo — `[0,1)` from state |
| `determinism::range_f32(state, min, max) -> f32` | `determinism.rs` | Nuevo |
| `morpho_adaptation::bergmann_growth_delta` | `morpho_adaptation.rs` | Ya existe |
| `morpho_adaptation::wolff_bond_delta` | `morpho_adaptation.rs` | Ya existe |
| `growth_engine::logistic_growth_rate` | `growth_engine.rs` | Ya existe |

---

## Constantes nuevas

```rust
// batch/constants.rs — ampliar
pub const REPRODUCTION_THRESHOLD: f32 = 50.0;    // qe minimo para reproduccion
pub const ABIOGENESIS_POP_CAP: u8 = 48;          // no abiogenesis si > 48 entidades
pub const ABIOGENESIS_ENERGY_THRESHOLD: f32 = 1000.0;  // irradiance grid minima
pub const ABIOGENESIS_INITIAL_QE: f32 = 10.0;
pub const ABIOGENESIS_INITIAL_RADIUS: f32 = 0.3;
pub const ALLOMETRIC_MAX_RADIUS: f32 = 3.0;
```

---

## NO hace

- No implementa GeneticHarness — eso es BS-4.
- No implementa GenomeBlob completo — solo `from_slot`/`apply`/`mutate` basicos para reproduction.
- No usa rayon.

---

## Dependencias

- BS-2 — `EventBuffer` con deaths/reproductions.
- `crate::blueprint::equations::determinism` — RNG determinista (+nuevas: `unit_f32`, `range_f32`).
- `crate::blueprint::equations::morpho_adaptation` — Bergmann, Wolff.
- `crate::blueprint::equations::growth_engine` — logistic growth.
- `crate::blueprint::constants` — thresholds de reproduccion, abiogenesis.

---

## Criterios de aceptacion

- Mundo con 10 herbivores + nutrient grid → entidades se reproducen → population crece.
- Mundo sin comida → entidades mueren → population decrece a 0.
- Abiogenesis: mundo vacio con high irradiance → celulas emergen.
- Reproduccion hereda genome: `child.growth_bias ≈ parent.growth_bias ± sigma`.
- `alive_mask` siempre consistente con `entities[i].alive`.
- Entity count estable (no explota ni colapsa) en 10K ticks con ecosistema balanceado.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` §3.3 — Tier 3
- `src/simulation/lifecycle/` — referencia para reproduction, morpho_adaptation
- `src/simulation/abiogenesis/` — referencia para abiogenesis logic
