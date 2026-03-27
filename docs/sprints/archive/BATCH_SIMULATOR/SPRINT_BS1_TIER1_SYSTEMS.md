# Sprint BS-1 — Tier 1 Systems: 12 Systems SIMD-Friendly

**Modulo:** `src/batch/systems/thermodynamic.rs`, `atomic.rs`, `chemical.rs`
**Tipo:** Systems batch que operan en 1-2 campos por entidad, sin interaccion entre entidades.
**Onda:** BS-0 → BS-1.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post BS-0)

- `EntitySlot`, `SimWorldFlat`, `ScratchPad` — tipos core del arena.
- 3 systems funcionando: `dissipation`, `movement_integrate`, `collision`.
- `blueprint/equations/` — 50+ dominios de math pura.

---

## Objetivo

Implementar los 12 systems que operan en campos individuales de `EntitySlot` sin interaccion entre entidades. Estos son auto-vectorizables por el compilador (inner loop sobre `entities[0..64]`).

---

## Systems a implementar

| # | System | Fase | Ecuacion source | Campos leidos | Campos escritos |
|---|--------|------|-----------------|---------------|-----------------|
| 1 | `engine_processing` | Thermo | `equations::engine_intake_tick` | `engine_buffer`, `engine_max`, `input_valve`, `output_valve`, `qe` | `engine_buffer`, `qe` |
| 2 | `irradiance_update` | Thermo | `equations::photosynthesis_yield` | `position`, `frequency_hz` | Escribe en `irradiance_grid` |
| 3 | `velocity_cap` | Atomic | `equations::velocity_clamp` | `velocity`, `matter_state` | `velocity` |
| 4 | `will_to_velocity` | Atomic | `equations::will_force` | `will_intent`, `qe`, `radius`, `viscosity` | `velocity` |
| 5 | `locomotion_drain` | Atomic | `equations::locomotion_cost` | `velocity`, `radius`, `qe` | `qe` |
| 6 | `homeostasis` | Chemical | `equations::homeostasis_adapt` | `frequency_hz`, `adapt_rate_hz`, `stability_band`, `qe` | `frequency_hz`, `qe` |
| 7 | `state_transitions` | Chemical | `equations::equivalent_temperature`, `phase_transition_threshold` | `qe`, `radius`, `bond_energy`, `conductivity` | `matter_state` |
| 8 | `photosynthesis` | Chemical | `equations::photosynthesis_net_gain` | `frequency_hz`, `position`, `irradiance_grid` | `qe` |
| 9 | `nutrient_uptake` | Chemical | `equations::nutrient_extraction_rate` | `position`, `radius`, `nutrient_grid` | `qe`, `nutrient_grid` |
| 10 | `pool_distribution` | Metabolic | `equations::extract_proportional` | `qe`, `engine_buffer` | `qe` |
| 11 | `senescence` | Morpho | `equations::senescence_drain` | `qe`, `tick_id` (age proxy) | `qe` |
| 12 | `growth_inference` | Morpho | `equations::allometric_radius` | `qe`, `growth_bias`, `radius` | `radius` |

---

## Patron de implementacion

Todos los systems Tier 1 siguen el mismo patron:

```rust
pub fn system_name(world: &mut SimWorldFlat) {
    let dt = world.dt;
    for i in 0..MAX_ENTITIES {
        if world.alive_mask & (1 << i) == 0 { continue; }
        let e = &mut world.entities[i];
        // 1. Leer campos
        // 2. Llamar ecuacion pura de blueprint/equations/
        // 3. Escribir resultado con guard
        let result = equations::some_fn(e.field_a, e.field_b, dt);
        if e.target != result { e.target = result; }
    }
}
```

**Reglas:**
- Un solo loop sobre `entities[0..64]`.
- Toda math delegada a `blueprint/equations/`.
- Guard de cambio antes de write (`if old != new`).
- Sin `ScratchPad` — no hay interaccion entre entidades.

---

## Organizacion de archivos

```
src/batch/systems/
├── mod.rs              ← pub use de los 6 archivos
├── thermodynamic.rs    ← engine_processing, irradiance_update (+ dissipation de BS-0)
├── atomic.rs           ← velocity_cap, will_to_velocity, locomotion_drain, movement_integrate (+ collision de BS-0)
├── chemical.rs         ← homeostasis, state_transitions, photosynthesis, nutrient_uptake
├── metabolic.rs        ← pool_distribution (+ trophic/social en BS-2)
└── morphological.rs    ← senescence, growth_inference (+ reproduction/abiogenesis en BS-3)
```

Los systems de BS-0 (`dissipation`, `movement_integrate`, `collision`) se mueven de `pipeline.rs` a sus archivos correspondientes.

---

## NO hace

- No implementa systems con interaccion entre entidades — eso es BS-2.
- No implementa spawn/despawn — eso es BS-3.
- No usa rayon — single-threaded.
- No modifica ecuaciones existentes.

---

## Dependencias

- BS-0 — arena, pipeline, scratch, constants.
- `crate::blueprint::equations::core_physics` — drag, kinetic, locomotion.
- `crate::blueprint::equations::growth_engine` — allometric_radius.
- `crate::blueprint::equations::homeostasis` — adapt functions.
- `crate::blueprint::equations::field_color` — photosynthesis yield.
- `crate::blueprint::constants` — phase thresholds, homeostasis rates.

---

## Criterios de aceptacion

### Por system
- Cada system produce el mismo delta numerico que la ecuacion correspondiente
  llamada directamente. Test: `system(world)` vs `equation(inputs)` → identical f32.
- Entidades muertas no son afectadas (`alive_mask & (1 << i) == 0` → skip).
- Conservation: `total_qe` no aumenta tras un tick completo (salvo intake de grids).

### Integracion
- `SimWorldFlat::tick()` expande a los 15 systems (3 de BS-0 + 12 nuevos) en orden de fase.
- 100K mundos × 1000 ticks sin panic, conservation dentro de epsilon.
- `cargo test --lib` sin regresion.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` §3.3 — tiers de complejidad
- `src/blueprint/equations/mod.rs` — facade de ecuaciones
- `src/simulation/pipeline.rs` — orden canonico de fases (INV-B7)
