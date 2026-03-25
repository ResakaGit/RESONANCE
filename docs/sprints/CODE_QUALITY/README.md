# Sprints — Code Quality (track completo)

Índice maestro: [`../README.md`](../README.md).

**Cerrados:** Q1, Q4, Q6, Q7 — sus `SPRINT_*.md` se eliminaron; el criterio quedó aplicado en `src/`.

## Cerrados

| Sprint | Implementación | Estado |
|--------|---------------|--------|
| **Q2** | 11 constantes nombradas en `energy_competition_ec.rs` + `organ_inference_li3.rs`; `dynamics.rs`/`scale.rs` wired | ✅ |
| **Q3** | `PoolConservationLedger` fields privados + `new()` + getters; todos los call sites actualizados | ✅ |
| **Q5** | `SimulationPlugin` → 6 domain plugins: `ThermodynamicPlugin`, `AtomicPlugin`, `ChemicalPlugin`, `InputPlugin`, `MetabolicPlugin`, `MorphologicalPlugin`; `pipeline.rs` 554→126 LOC | ✅ |
| **Q8** | `vertex_along_flow_color` → `blueprint/equations/field_color/`; inline shading en `build_petal_fan` extraído | ✅ |

## Cerrados (referencia)

**Q1, Q4, Q6, Q7** — ver mensajes de cierre en cada `SPRINT_Q*.md` y código en `src/layers/`, `simulation/reactions`, etc.

## Referencias

- `docs/design/FOLDER_STRUCTURE.md`
- `docs/arquitectura/blueprint_gamedev_patterns.md`
