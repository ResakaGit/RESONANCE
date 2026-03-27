# Sprint BS-2 — Tier 2 Systems: 13 Systems Per-World Interaction

**Modulo:** `src/batch/systems/input.rs`, `metabolic.rs` (expandir), `src/batch/events.rs`
**Tipo:** Systems con interaccion N² entre entidades del mismo mundo.
**Onda:** BS-1 → BS-2.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post BS-1)

- 15 systems Tier 1 funcionando (BS-0 + BS-1).
- `ScratchPad` con buffers para pairs, neighbors, deaths.

---

## Objetivo

Implementar los 13 systems que requieren comparar entidades entre si dentro del mismo mundo: colision (ya existe de BS-0, se refina), social, trophic, behavior, cooperacion, culture. Estos usan `ScratchPad` para buffers de pares/vecinos.

---

## Systems a implementar

| # | System | Fase | Patron | Usa ScratchPad |
|---|--------|------|--------|---------------|
| 1 | `behavior_assess` | Input | Scan neighbors por threats/food | `neighbors` |
| 2 | `behavior_decide` | Input | Utility scoring → intent | — |
| 3 | `resonance_link_scan` | Thermo | N² scan, apply buff/debuff | `pairs` |
| 4 | `entrainment` | Atomic | Kuramoto phase sync (N² nearby) | `pairs` |
| 5 | `trophic_forage` | Metabolic | Herbivores extract from grid | — (grid access) |
| 6 | `trophic_predation` | Metabolic | Carnivore vs prey (N² nearby) | `pairs` |
| 7 | `social_pack` | Metabolic | Same-faction grouping | `neighbors` |
| 8 | `cooperation_eval` | Metabolic | Nash-stable alliance check | `pairs` |
| 9 | `culture_transmission` | Metabolic | Meme imitation (N² nearby) | `meme_candidates` |
| 10 | `ecology_census` | Metabolic | Count species by freq band | — (per-world scan) |
| 11 | `competitive_exclusion` | Chemical | Niche displacement | `neighbors` |
| 12 | `containment_check` | Thermo | Overlap → drag/thermal | `pairs` |
| 13 | `tension_field_apply` | Atomic | L11 gravity/magnetic force | `pairs` |

---

## EventBuffer — eventos intra-tick

```rust
// src/batch/events.rs

/// Ring buffer por mundo para eventos intra-tick.
/// Capacidad fija. Se limpia al inicio de cada tick.
pub struct EventBuffer {
    pub deaths:       [u8; MAX_ENTITIES],
    pub deaths_len:   usize,
    pub reproductions: [(u8, u8); 32],  // (parent_idx, child_slot)
    pub repro_len:    usize,
    pub hunger:       [u8; MAX_ENTITIES],
    pub hunger_len:   usize,
}

impl EventBuffer {
    pub fn clear(&mut self) { ... }
    pub fn record_death(&mut self, idx: u8) { ... }
    pub fn record_reproduction(&mut self, parent: u8, child: u8) { ... }
}
```

Se integra en `SimWorldFlat` como campo (`pub events: EventBuffer`).

---

## Patron de implementacion Tier 2

```rust
pub fn system_with_pairs(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    // 1. Collect interacting pairs
    scratch.pairs_len = 0;
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        for j in (i+1)..MAX_ENTITIES {
            if world.alive_mask & (1 << j) == 0 { continue; }
            if within_range(&world.entities[i], &world.entities[j], RANGE) {
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }
    // 2. Process pairs calling equations
    for p in 0..scratch.pairs_len {
        let (i, j) = (scratch.pairs[p].0 as usize, scratch.pairs[p].1 as usize);
        let result = equations::some_interaction(
            world.entities[i].field_a,
            world.entities[j].field_a,
        );
        // 3. Apply result
        world.entities[i].field_b += result;
        world.entities[j].field_b -= result;
    }
}
```

**Complejidad:** N² = C(64,2) = 2016 pares max. Con range filter, tipicamente ~100-300 pares activos.

---

## NO hace

- No implementa spawn/despawn — los events de death/reproduction se registran pero se procesan en BS-3.
- No usa rayon — single-threaded.
- No modifica ecuaciones existentes — solo las invoca.

---

## Dependencias

- BS-1 — 15 systems Tier 1 como base.
- `crate::blueprint::equations::behavior` — utility scores, threat evaluation.
- `crate::blueprint::equations::trophic` — predation, assimilation.
- `crate::blueprint::equations::social_communication` — pack cohesion.
- `crate::blueprint::equations::energy_competition` — competitive extraction.
- `crate::blueprint::equations::emergence` — entrainment, culture affinity.
- `crate::blueprint::constants::cooperation_ac5` — scan radius, defect threshold.
- `crate::blueprint::constants::entrainment_ac2` — Kuramoto coupling.
- `crate::blueprint::constants::culture` — imitation radius, coherence bonus.

---

## Criterios de aceptacion

### Por system
- `behavior_assess` + `behavior_decide`: entidades con `trophic_class = Herbivore` eligen forage cuando hungry.
- `trophic_predation`: carnivoro drena qe de presa en rango; presa pierde qe exactamente igual.
- `social_pack`: entidades same-faction se agrupan (velocities converge).
- `entrainment`: entidades nearby con freq similar convergen en phase.
- `culture_transmission`: memes se propagan entre entidades con alta oscillatory affinity.
- `cooperation_eval`: pares Nash-estables se identifican.

### Integracion
- Trophic chain sobrevive 1000 ticks: herbivores comen grid, carnivores comen herbivores, ambos coexisten.
- No NaN ni Inf en ningun campo despues de 10K ticks.
- Conservation: `total_qe` dentro de epsilon por tick.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` §3.3 — Tier 2 systems
- `src/simulation/behavior.rs` — referencia para behavior AI logic
- `src/simulation/metabolic/trophic.rs` — referencia para trophic chains
- `src/simulation/emergence/culture.rs` — referencia para culture transmission
