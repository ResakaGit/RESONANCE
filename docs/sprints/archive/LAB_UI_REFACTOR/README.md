# Track: LAB_UI_REFACTOR — ✅ COMPLETADO (2026-04-12)

**Objetivo:** Refactorizar el lab para composición por experiment, controles contextuales, Live 2D controls.

**Estado:** ✅ COMPLETADO
**ADRs:** ADR-018 (Lab Universal), ADR-019 (Live Controls)

---

## Sprints completados

| Sprint | Entregable | Estado |
|--------|-----------|--------|
| **LR-1** | LabMode state machine (Batch/Live) | ✅ Pre-existente |
| **LR-2** | Live 2D controls: Pause, Speed, Reset, Map Selector | ✅ 2026-04-12 |
| **LR-3** | Per-experiment controls (15 experiments contextuales) | ✅ 2026-04-12 |
| **LR-4** | Composition pattern (match dispatch + 4 categorías) | ✅ 2026-04-12 |

## Entregables adicionales (fuera del sprint original)

| Entregable | Detalle |
|-----------|---------|
| 8 experiments nuevos | Personal, PathwayInhibitor, Zhang, Sharma, FooMichor, Michor, UnifiedAxioms, ParticleLab |
| 4 categorías UI | Core Simulation, Drug & Therapy, Paper Validation, Physics |
| 12 CSV exports | 4 arreglados (Speciation, Cambrian, Debate, Convergence) + 8 nuevos |
| Pause/Resume | `GameState::Playing ↔ Paused` (estado Bevy nativo) |
| Speed 0.25x—4x | Modifica `Time<Fixed>` timestep |
| Reset World | Exclusive system: limpia entidades, resetea grids, re-warmup |
| Map Selector | Dropdown con 25 mapas `.ron`, carga en caliente |

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `src/bin/lab.rs` | 15 experiments, categorías, controles Live, reset system (~1550 LOC) |
| `src/worldgen/map_config.rs` | +`load_map_config_from_slug()` |
| `src/worldgen/field_grid.rs` | +`reset_cells()` |
| `src/worldgen/nutrient_field.rs` | +`reset_cells()` |
| `src/worldgen/systems/startup.rs` | Extraído `run_warmup_loop()` |
| `src/worldgen/mod.rs` | Re-exports |
