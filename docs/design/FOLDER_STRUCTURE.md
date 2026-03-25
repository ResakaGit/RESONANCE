# Blueprint — Folder Structure & Architecture

**Version:** 2.1 (alineado a código)  
**Date:** 2026-03-24  
**Snapshot:** ~190+ archivos `.rs` bajo `src/`, **14** módulos top-level en `lib.rs`, tests lib **~1040** (`cargo test --lib`, revisar al cambiar de toolchain).

**Contrato:** Este documento describe el **árbol real** y conserva secciones históricas (migración, constantes) donde siguen siendo útiles. Narrativa por módulo runtime: `docs/arquitectura/README.md`. Instrucciones agente: `CLAUDE.md`.

---

## 1. Objective

Definir la estructura canónica de `resonance/src/`, registrar el estado tras migraciones M1–M5 y enlazar principios de organización (constantes por contexto, worldgen cerca de datos).

---

## 2. Árbol actual (`src/`) — canónico

**Raíz:** `lib.rs` exporta `blueprint`, `bridge`, `eco`, `events`, `geometry_flow`, `plugins`, `rendering`, `runtime_platform`, `entities`, `layers`, `simulation`, `topology`, `world`, `worldgen` (14). Sin alias `v6`. `events.rs` en raíz (no `eventos`).

**Pipeline:** fases en `simulation/mod.rs` — `Phase::Input`, `ThermodynamicLayer`, `AtomicLayer`, `ChemicalLayer`, `MetabolicLayer`, `MorphologicalLayer` (+ `SimulationClockSet`). Orden en `simulation/pipeline.rs`.

```
src/
├── lib.rs, main.rs, events.rs
├── blueprint/
│   ├── mod.rs, abilities, almanac/ (loader, catalog, element_def, …), almanac_contract, constants/, element_id,
│   │   equations/ (dominios: core_physics, field_body, morphogenesis_shape, … + mod.rs), morphogenesis.rs, ids, recipes,
│   │   spell_compiler, validator
│   └── constants/            # tuning por dominio: mod.rs reexporta `pub use …::*` + submódulo `morphogenesis` (MG track)
│       └── *.rs              # ~44 shards (p.ej. numeric_math, thermal_transfer, ecosystem_abiogenesis, organ_role_visual_li6)
├── bridge/
│   ├── cache, config, constants, context_fill, decorator, metrics, normalize, presets, bridged_*, benchmark_*, mod.rs
├── eco/
│   ├── boundary_*, climate, constants, context_lookup, contracts, systems, zone_classifier, mod.rs
├── entities/
│   ├── archetypes, builder, composition, lifecycle_observers, mod.rs
├── geometry_flow/
│   ├── branching.rs, mod.rs
├── layers/                    # 14 capas + auxiliares (p.ej. growth, inference, irradiance, nutrient, vision_fog, markers)
├── plugins/
│   ├── debug_plugin, layers_plugin, simulation_plugin, mod.rs
├── rendering/
│   └── quantized_color/       # plugin, registry, systems, palette, …
├── runtime_platform/          # 18 pub mod (ver runtime_platform/mod.rs)
│   ├── camera_controller_3d, click_to_move, collision_backend_3d, compat_2d3d, contracts,
│   │   core_math_agnostic, debug_observability, fog_overlay, hud, input_capture,
│   │   intent_projection_3d, kinematics_3d_adapter, parry_nav_collider, render_bridge_3d,
│   │   scenario_isolation, simulation_tick, spatial_index_backend
├── simulation/
│   ├── mod.rs, pipeline.rs, bootstrap.rs, time_compat.rs, states.rs
│   ├── input, pre_physics, physics, reactions, post, containment, structural_runtime, element_layer2
│   ├── ability_targeting, atomic, fog_of_war, grimoire_enqueue, growth_budget, inference_growth
│   ├── nutrient_uptake, observers, osmosis, photosynthesis, player_controlled, sensory
│   ├── allometric_growth, pathfinding/, …
│   └── tests: eco_e5_*, event_ordering_tests, regression, verify_wave_gate (según cfg)
├── topology/
│   ├── config, constants, contracts, functions, mod.rs, terrain_field.rs, terrain_mesher.rs, mutations.rs
│   └── generators/ (noise, slope, drainage, classifier, hydraulics)
├── world/
│   ├── demo_level, demo_clouds, fog_of_war, grimoire_presets, marker, perception, space, mod.rs
└── worldgen/
    ├── archetypes, constants, contracts, field_grid, lod, map_config, materialization_rules
    ├── nucleus, propagation, nutrient_field, shape_inference, visual_derivation, mod.rs
    └── systems/
        ├── mod.rs, startup, prephysics, propagation, materialization, terrain, visual, performance
```

**Mapas:** `RESONANCE_MAP` → `assets/maps/{nombre}.ron` (`map_config.rs`). No hay `world/demo_arena.rs` ni `spawn_proving_grounds` como archivos dedicados: el escenario lo define el RON + worldgen.

---

## 3. Diagnosed Issues (estado 2026-03)

### 3.1 — Resuelto / obsoleto

- **`events.rs`:** raíz en inglés; no es orphan (módulo coherente con `lib.rs`).
- **Alias `v6`:** eliminado; usar `runtime_platform`.
- **Worldgen en `simulation/`:** movido a `worldgen/systems/*`.
- **`eco_boundaries`:** vive en `eco/systems.rs` (consumido desde pipeline/prephysics).
- **`bridge/constants.rs`:** existe.
- **`blueprint/constants/` (oleada 2026-03):** el monolito `blueprint/constants.rs` se reemplazó por `constants/mod.rs` + shards por dominio; la API pública sigue siendo `crate::blueprint::constants::{…}` y `crate::blueprint::constants::morphogenesis::*` (más reexport plano de MG para `use constants::*`).

### 3.2 — Abierto / residual

- **`simulation/`** sigue siendo un módulo grande (gameplay + pathfinding + fog + crecimiento + …). Mejora incremental posible; no bloquea.
- **`plugins/simulation_plugin.rs`:** wiring concentrado; split Q5 sigue siendo opcional si crece el arranque.

### 3.3 — Constants in non-constant files (severity: low-medium) — aún vigente como guía

Revisar periódicamente constantes embebidas en implementación vs `{module}/constants.rs` (reglas en §5). Rutas de archivos antiguos (`simulation/worldgen_*`) ya no aplican.

### 3.4 — `topology/mutations.rs` (T10)

**Implementado:** `src/topology/mutations.rs` (mutaciones runtime: crater, uplift, erosión puntual, flatten, etc.).

### 3.5 — `plugins/simulation_plugin.rs`

Sigue concentrando Startup; opcional dividir si el arranque crece (Q5).

---

## 4. Target Architecture (to-be / mayormente aplicado)

El **estado en disco** está descrito en **§2**. El bloque §4.2 conserva el diseño de migración; diferencias actuales respecto al texto histórico: **`geometry_flow/`**, **`rendering/`**, **`worldgen/shape_inference.rs`**, **`nutrient_field.rs`**, **`topology/mutations.rs`**, **`terrain_mesher.rs`**, **`runtime_platform` con 17 sub-módulos** (incl. `fog_overlay`, `hud`, `parry_nav_collider`), **`simulation/` ampliado** (pathfinding, fog, crecimiento, fotosíntesis, etc.), **`world/`** sin `demo_arena.rs` (mapas RON).

### 4.1 Principles

1. **One concept, one directory.** Each top-level module owns its types, constants, pure functions, AND systems. No cross-concept dumping.
2. **Centralized constants per context.** Cada módulo con tuning tiene **un punto de entrada**: `constants.rs` **o** `constants/mod.rs` que agrega submódulos por dominio (patrón actual en `blueprint/constants/`). Los shards no sustituyen el contrato: el consumidor importa desde `crate::{module}::constants::…`. Constantes privadas de algoritmo (offsets, epsilons locales) pueden quedar in-file; el resto va al árbol `constants` del módulo.
3. **Systems live near their data.** Worldgen ECS systems live in `worldgen/`, not `simulation/`. Eco systems live in `eco/`. `simulation/` retains only pipeline orchestration + game-loop systems.
4. **English identifiers.** Per linter convention, all module and file names in English.
5. **No orphans.** Every `.rs` file belongs to a `mod` hierarchy.

### 4.2 Proposed Structure (referencia histórica; cotejar con §2)

```
src/
├── lib.rs                              # Module tree root (sin alias v6)
├── main.rs
│
├── blueprint/                          # Core types + equations
│   ├── mod.rs
│   ├── abilities.rs
│   ├── almanac/ …
│   ├── constants/                      # mod.rs + shards por dominio (histórico: monolito único)
│   ├── element_id.rs
│   ├── equations/ …                    # mod.rs + submódulos por dominio (math pura)
│   ├── morphogenesis.rs
│   ├── recipes.rs
│   ├── spell_compiler.rs
│   └── validator.rs
│
├── bridge/                             # Cache optimizer (MOSTLY UNCHANGED)
│   ├── mod.rs
│   ├── constants.rs                    # NEW — consolidate from normalize, bridged_ops, decorator
│   ├── benchmark_harness.rs
│   ├── bridged_ops.rs
│   ├── bridged_physics.rs
│   ├── cache.rs
│   ├── config.rs
│   ├── context_fill.rs
│   ├── decorator.rs
│   ├── metrics.rs
│   ├── normalize.rs
│   └── presets.rs
│
├── eco/                                # Eco-boundaries + climate (UNCHANGED)
│   ├── mod.rs
│   ├── boundary_detector.rs
│   ├── boundary_field.rs
│   ├── climate.rs
│   ├── constants.rs                    # Zone thresholds (13)
│   ├── context_lookup.rs
│   ├── contracts.rs
│   ├── zone_classifier.rs
│   └── systems.rs                      # NEW — eco_boundaries_system moved from simulation/
│
├── entities/                           # Entity archetypes (UNCHANGED)
│   ├── mod.rs
│   ├── archetypes.rs
│   ├── builder.rs
│   └── composition.rs
│
├── events.rs                           # RENAMED from eventos.rs (8 event structs)
│
├── layers/                             # 14 ECS layers (UNCHANGED)
│   └── ... (18 files)
│
├── plugins/                            # Bevy plugin wiring
│   ├── mod.rs
│   ├── debug_plugin.rs
│   ├── layers_plugin.rs
│   └── simulation_plugin.rs            # Future: Q5 split
│
├── runtime_platform/                   # Platform adapters (hoy 17 sub-módulos; ver §2)
│   └── ...
│
├── simulation/                         # RESTRUCTURED — game-loop orchestration only
│   ├── mod.rs                          # Phase, InputChannelSet, re-exports
│   ├── pipeline.rs                     # System scheduling + ordering
│   ├── bootstrap.rs                    # init_simulation_bootstrap (renamed)
│   ├── time_compat.rs
│   │
│   │ # ── Core game-loop systems ──────────
│   ├── input.rs
│   ├── pre_physics.rs
│   ├── physics.rs
│   ├── reactions.rs
│   ├── post.rs
│   │
│   │ # ── Subsystems that depend on game-loop ──
│   ├── containment.rs                  # Contact detection
│   ├── element_layer2.rs               # Freq modulation
│   ├── structural_runtime.rs           # StructuralLink mgmt
│   │
│   │ # ── Tests ───────────────────────────
│   ├── regression.rs                   # #[cfg(test)]
│   └── verify_wave_gate.rs             # #[cfg(test)]
│
├── topology/                           # Terrain generation (UNCHANGED + future T10)
│   ├── mod.rs
│   ├── config.rs
│   ├── constants.rs                    # Altitude, drainage, slope (10)
│   ├── contracts.rs
│   ├── functions.rs
│   ├── terrain_field.rs
│   └── generators/
│       ├── mod.rs
│       ├── classifier.rs
│       ├── drainage.rs
│       ├── hydraulics.rs
│       ├── noise.rs
│       └── slope.rs
│
├── world/                              # Maps + spatial (hoy: demo_level, nubes, fog, presets; ver §2)
│   ├── mod.rs
│   ├── demo_level.rs
│   ├── marker.rs
│   ├── perception.rs
│   └── space.rs
│
└── worldgen/                           # V7 data model + systems (EXPANDED)
    ├── mod.rs
    ├── archetypes.rs
    ├── constants.rs                    # Materialization + visual (39 + consolidated)
    ├── contracts.rs
    ├── field_grid.rs
    ├── lod.rs
    ├── map_config.rs
    ├── materialization_rules.rs
    ├── nucleus.rs
    ├── propagation.rs
    ├── visual_derivation.rs
    │
    │ # ── Systems (moved from simulation/) ──────
    ├── systems/
    │   ├── mod.rs
    │   ├── startup.rs                  # was simulation/worldgen_startup.rs
    │   ├── propagation.rs              # was simulation/worldgen_propagation.rs
    │   ├── materialization.rs          # was simulation/worldgen_materialization.rs
    │   ├── visual.rs                   # was simulation/worldgen_visual.rs
    │   ├── terrain.rs                  # was simulation/worldgen_terrain.rs
    │   ├── prephysics.rs               # was simulation/worldgen_prephysics.rs
    │   └── performance.rs              # was simulation/worldgen_performance.rs
    │
    └── tests/                          # Worldgen-specific integration tests
        └── eco_e5_simulation_tests.rs  # moved from simulation/
```

### 4.3 Changes Summary

| Change | Files affected | Risk | Impact |
|--------|---------------|------|--------|
| Rename `eventos.rs` → `events.rs` | 1 file + 12 imports | Low | Consistency |
| Remove `v6` alias from `lib.rs` | 1 file + ~5 imports | Low | Clarity |
| Move 7 `worldgen_*` systems → `worldgen/systems/` | 7 files + mod.rs + pipeline.rs imports | Medium | Concept cohesion |
| Move `eco_boundaries_system.rs` → `eco/systems.rs` | 1 file + pipeline imports | Low | Concept cohesion |
| Move `eco_e5_simulation_tests.rs` → `worldgen/tests/` | 1 file | Low | Test locality |
| Create `bridge/constants.rs` consolidating 9 scattered constants | 3 files | Low | Centralization |
| Consolidate worldgen embedded constants → `worldgen/constants.rs` | 5 files | Low | Centralization |
| Rename `simulation_bootstrap.rs` → `bootstrap.rs` | 1 file + 1 import | Low | Brevity |

---

## 5. Constants Centralization Plan

### 5.1 Current State (6 dedicated trees/files + ~15 files with embedded constants)

| Context | Dedicated file | Embedded in | Total |
|---------|---------------|-------------|-------|
| Blueprint (physics) | `blueprint/constants/mod.rs` + ~44 dominios `*.rs` (misma cuenta de símbolos que el monolito; API plana vía `pub use`) | — | (ver shards) |
| Worldgen (V7) | `worldgen/constants.rs` (39) | lod, field_grid, map_config, propagation, archetypes, visual_derivation, materialization (7) | 46 |
| Topology | `topology/constants.rs` (10) | generators/drainage (3) | 13 |
| Eco | `eco/constants.rs` (13) | boundary_detector (1), boundary_field (1) | 15 |
| Bridge | — | normalize (5), bridged_ops (2), decorator (2), benchmark_harness (8) | 17 |
| Runtime | — | contracts (2), core_math_agnostic (1), scenario_isolation (3) | 6 |
| Plugins | — | debug_plugin (7) | 7 |

#### 5.1.1 `blueprint/constants/` — checklist de PR (oleada 2)

- **Nuevo tuning:** colocar el shard cuyo dominio coincida con el comentario de sección histórica (capas, ecosistema, inferencia LI\*, MG, fog, …); si no encaja, nuevo archivo `snake_case` en inglés en `constants/` y `mod` + `pub use …::*` en `constants/mod.rs`.
- **Colisiones:** dos `pub const` con el mismo nombre en shards distintos **no compilan** (E0252 al aplanar); elegir nombre único o no reexportar uno de los dos.
- **Morfogénesis inferida (MG):** definir const en `morphogenesis_track.rs` → submódulo `morphogenesis`. Si debe vivir también en la raíz `constants::` para `use crate::blueprint::constants::*`, **actualizar** la lista explícita `pub use morphogenesis::{ … }` en `constants/mod.rs` (si no, basta con `constants::morphogenesis::NUEVA`).
- **Código nuevo del track MG:** preferir `use crate::blueprint::constants::morphogenesis as mg` (o path completo al submódulo) para no depender del reexport plano.
- **Nombres de archivo:** preferir dominio descriptivo en inglés; sufijos tipo `_li6`, `_mg2`, `_g12` son trazabilidad a sprint — aceptados, no obligatorios para archivos nuevos.

### 5.2 Migration Rules

**Move to `constants.rs` / shard bajo `{module}/constants/`:**
- Any constant that configures behavior tuning (thresholds, multipliers, rates, limits)
- Any constant used by more than one function in the module
- Constants that a designer/balancer might want to tweak

**Keep in-file:**
- Array literals that define algorithmic structure (D8_DX/DY, NEIGHBOR_OFFSETS, CANONICAL_DIRECTIONS)
- Constants private to a single function and tightly coupled to its algorithm (FNV_OFFSET/PRIME in hashing)
- Constants in `impl` blocks that are inherently scoped
- Test-only constants

### 5.3 New/Updated `constants.rs` Files

**`bridge/constants.rs`** (NEW):
```
INTERFERENCE_TIME_QUANT_S, INTERFERENCE_PHASE_SECTORS (from bridged_ops)
VEC2_STATIC_SECTOR, VEC2_DIRECTION_ZERO_EPS_SQ (from normalize)
BENCHMARK_SCENARIO_SEED, BLUEPRINT_HIT_RATE_* (from benchmark_harness — these are test blueprints, keep if useful for runtime)
```

**`worldgen/constants.rs`** (UPDATE — add):
```
LOD_NEAR_MAX, LOD_MID_MAX (from lod.rs)
FIELD_GRID_CHUNK_SIZE (from field_grid.rs)
SEASON_TRANSITION_TICKS (from worldgen_materialization.rs)
```

**`plugins/constants.rs`** or keep in-file:
```
COMPOUND_RING_*, LABEL_FONT_*, LABEL_TEXT_SCALE, LABEL_Z_OFFSET (from debug_plugin.rs)
```
Decision: keep in-file — these are purely visual debug constants, not game balance. No other file references them.

**`runtime_platform/constants.rs`** (OPTIONAL — NEW):
```
V6_CONTRACTS_REV, BUTTON_PRIMARY_ACTION (from contracts/)
DEFAULT_SIM_STANDING_Y (from core_math_agnostic/)
V6_DEMO_FLOOR_* (from scenario_isolation/)
```
Decision: optional — these are platform-specific and rarely tuned. Can stay in-file unless runtime_platform grows.

---

## 6. Implementation Status vs Blueprints

### 6.1 V7 — Worldgen (11 modules promised)

| # | Module | Status | Notes |
|---|--------|--------|-------|
| 01 | worldgen/contracts | DONE | |
| 02 | worldgen/propagation | DONE | |
| 03 | worldgen/field_grid | DONE | |
| 04 | worldgen/materialization_rules | DONE | |
| 05 | worldgen/visual_derivation | DONE | |
| 06 | worldgen/archetypes | DONE | ~28 variants |
| 07 | worldgen/map_config | DONE | RON-driven |
| 08 | worldgen/systems/propagation | DONE | antes `simulation/worldgen_propagation.rs` |
| 09 | worldgen/systems/materialization | DONE | antes `simulation/worldgen_materialization.rs` |
| 10 | worldgen/systems/visual | DONE | antes `simulation/worldgen_visual.rs` |
| 11 | plugins wiring | DONE | `SimulationPlugin`, `pipeline`, `startup` |

**V7: 11/11 COMPLETE** (rutas actualizadas a `worldgen/systems/*`)

### 6.2 TOPOLOGY — Terrain (12 modules promised)

| # | Module | Status | Notes |
|---|--------|--------|-------|
| T1 | topology/contracts | DONE | TerrainType, DrainageClass, TerrainSample |
| T2 | topology/generators/noise | DONE | FBM + normalize |
| T3 | topology/generators/slope | DONE | Horn kernel + aspect |
| T4 | topology/generators/drainage | DONE | D8 + flow accumulation |
| T5 | topology/generators/classifier | DONE | classify_terrain + classify_all |
| T6 | topology/functions (modulation) | DONE | modulate_emission/diffusion/decay |
| T7 | topology/generators/hydraulics | DONE | erode_hydraulic + fill_pits |
| T8 | topology/terrain_field | DONE | Resource |
| T9 | topology/config | DONE | RON config + loader system |
| T10 | topology/mutations | DONE | Runtime deformations (crater, uplift, …) |
| T11 | worldgen/systems/terrain | DONE | System wiring (antes `simulation/worldgen_terrain.rs`) |
| T12 | assets/terrain_config.ron | DONE | Existe; E2E según cobertura de tests |

**Topology: 12/12 COMPLETE**

### 6.3 BRIDGE_OPTIMIZER (8 modules promised)

| # | Module | Status |
|---|--------|--------|
| 01 | bridge/config | DONE |
| 02 | bridge/normalize | DONE |
| 03 | bridge/cache | DONE |
| 04 | bridge/decorator | DONE |
| 05 | bridge/bridged_ops | DONE |
| 06 | bridge/context_fill | DONE |
| 07 | bridge/metrics | DONE |
| 08 | bridge/presets | DONE |

**Bridge: 8/8 COMPLETE**

### 6.4 ECO_BOUNDARIES (7 modules promised)

| # | Module | Status |
|---|--------|--------|
| 01 | eco/contracts | DONE |
| 02 | eco/zone_classifier | DONE |
| 03 | eco/boundary_detector | DONE |
| 04 | eco/boundary_field | DONE |
| 05 | eco/context_lookup | DONE |
| 06 | eco/climate | DONE |
| 07 | eco/systems | DONE |

**Eco: 7/7 COMPLETE**

### 6.5 All 14 ECS Layers

All 14 layers (L0-L13) are implemented with structs, Reflect, and integration in layers_plugin.rs.

### 6.6 Overall

**Mayoría de blueprints base cubiertos.** Abiertos opcionales: Q5 (split `simulation_plugin`), LOD sigue minimal, percepción/fog evolucionan en `simulation` + `world`.

---

## 7. Dependency Graph (clean, acyclic)

```
                         blueprint
                   /      |      \
                  /       |       \
            layers    events    entities
             / | \       |         |
        bridge |  \      |         |
             worldgen — topology   |
                |    \    |        |
                eco    \  |        |
                    simulation ←───┘
                         |
        geometry_flow ←──┘ (worldgen/shape_inference, GF1)
                         |
              runtime_platform
              rendering (quantized_color; plugins en main)
                         |
                       world
                         |
                      plugins
```

Key dependency rules:
- **blueprint** — base (foundation).
- **layers** — principalmente blueprint.
- **worldgen** — blueprint, layers, eco, topology; usa **geometry_flow** (`shape_inference`).
- **eco** — blueprint, layers, worldgen, topology.
- **simulation** — orquestador amplio (importa casi todo lo gameplay).
- **runtime_platform** — tick, input, 3D, HUD.
- **rendering** — color cuantizado; registrado desde `main.rs` / plugins, no es hijo lógico de `runtime_platform`.
- **plugins** — compone app (top-level).
- Evitar dependencias circulares entre módulos de dominio.

---

## 8. Migration Sprint (histórico)

Las fases 1–3 y buena parte de 4–5 **ya se aplicaron** (`events`, sin `v6`, `worldgen/systems`, `eco/systems`, `bootstrap.rs`, `bridge/constants.rs`). Lo siguiente documenta el plan original por trazabilidad.

### Phase 1 — Renames (zero-risk, no logic changes) — HECHO

1. Rename `src/eventos.rs` → `src/events.rs`
2. Update `lib.rs`: `pub mod eventos;` → `pub mod events;`
3. Find-replace `crate::eventos::` → `crate::events::` in all files
4. Remove `pub use runtime_platform as v6;` from `lib.rs`
5. Update `use crate::v6::` → `use crate::runtime_platform::` in all files

### Phase 2 — Worldgen systems extraction (medium-risk) — HECHO

1. Create `src/worldgen/systems/mod.rs`
2. Move 7 files from `simulation/worldgen_*.rs` → `worldgen/systems/`:
   - `worldgen_startup.rs` → `startup.rs`
   - `worldgen_propagation.rs` → `propagation.rs`
   - `worldgen_materialization.rs` → `materialization.rs`
   - `worldgen_visual.rs` → `visual.rs`
   - `worldgen_terrain.rs` → `terrain.rs`
   - `worldgen_prephysics.rs` → `prephysics.rs`
   - `worldgen_performance.rs` → `performance.rs`
3. Update `simulation/mod.rs` — remove worldgen mods, update re-exports
4. Update `worldgen/mod.rs` — add `pub mod systems;`
5. Update `simulation/pipeline.rs` — change imports
6. Update `plugins/simulation_plugin.rs` — change imports
7. Run `cargo test` — suite completa debe pasar

### Phase 3 — Eco system extraction (low-risk) — HECHO

1. Move `simulation/eco_boundaries_system.rs` → `eco/systems.rs`
2. Move `simulation/eco_e5_simulation_tests.rs` → `eco/` or `worldgen/tests/`
3. Update imports in pipeline.rs

### Phase 4 — Constants consolidation (low-risk) — PARCIAL / iterativo

1. `bridge/constants.rs` — existe; seguir reglas §5 para constantes sueltas en otros archivos
2. Move `LOD_NEAR_MAX`, `LOD_MID_MAX` → `worldgen/constants.rs`
3. Move `FIELD_GRID_CHUNK_SIZE` → `worldgen/constants.rs`
4. Move `SEASON_TRANSITION_TICKS` → `worldgen/constants.rs`
5. Keep algorithmic constants (D8_DX, FNV_*, NEIGHBOR_OFFSETS) in-file

### Phase 5 — Minor cleanup

1. Rename `simulation_bootstrap.rs` → `bootstrap.rs` — **HECHO** (`bootstrap.rs`).
2. Future: Q5 — split `simulation_plugin.rs` into WorldgenPlugin + GameSimPlugin (opcional).

---

## 9. File count (indicativo, marzo 2026)

Recuento `find src/<mod> -name '*.rs'` (incluye subcarpetas):

| Module | `.rs` files |
|--------|-------------|
| blueprint | 10 |
| bridge | 12 |
| eco | 9 |
| entities | 5 |
| geometry_flow | 2 |
| layers | 24 |
| plugins | 4 |
| rendering | 9 |
| runtime_platform | ~27 |
| simulation | 35 |
| topology | 14 |
| world | 8 |
| worldgen | 21 |
| root (`lib`, `main`, `events`) | 3 |
| **Total bajo `src/`** | **~190+** (recontar tras splits `blueprint/constants/`, `blueprint/equations/`) |

La migración original redujo worldgen en `simulation/`; el módulo `simulation/` volvió a crecer por features (pathfinding, metabolismo, fog, etc.).

---

## 10. Validation Checklist

Post-migración (mantener en CI / antes de release):

- [x] `cargo check` passes
- [x] `cargo test --lib` — ~1040+ tests (suite completa según configuración del repo)
- [ ] `cargo clippy` — sin warnings nuevos regresivos
- [x] No `crate::eventos::` / sin alias `v6` en `lib.rs`
- [x] No `crate::simulation::worldgen_*` (worldgen bajo `worldgen/systems`)
- [x] Blueprint: tuning centralizado en `blueprint/constants/` (`mod.rs` + dominios); otros módulos: `constants.rs` o equivalente (§5)
- [x] Raíz `src/`: solo `events.rs`, `lib.rs`, `main.rs`
- [x] `RESONANCE_MAP=demo_arena cargo run` / `cargo run` según mapas en `assets/maps/`

---

## 11. References

- `CLAUDE.md` — instrucciones agente + mapa de módulos
- `docs/arquitectura/README.md` — blueprints por carpeta alineados a runtime
- `docs/design/V7.md` — V7 worldgen architecture
- `docs/design/TOPOLOGY.md` — Terrain substrate
- `docs/design/BRIDGE_OPTIMIZER.md` — Cache optimizer
- `docs/design/ECO_BOUNDARIES.md` — Eco-boundaries
- `docs/sprints/CODE_QUALITY/` — Q1-Q7 quality sprints
- `docs/sprints/MIGRATION/` — M1–M5
- `DESIGNING.md` — 5-Test for layer validation
