# Sprints — Active Backlog

Open work tracked here. Completed tracks archived in [`archive/`](archive/).

High-level design: [`docs/design/INDEX.md`](../design/INDEX.md). Runtime contracts: [`docs/arquitectura/`](../arquitectura/).

---

## Conteo global

| Métrica | Valor |
|---------|-------|
| Sprints pendientes | **8** |
| Tracks activos | **2** |
| Oleadas restantes | **3** |
| Tracks archivados | **39** |

---

## 8 sprints pendientes

### GAMEPLAY_SYSTEMS (6 sprints pendientes)

| Sprint | Descripción | Esfuerzo | Bloqueado por | Doc |
|--------|-------------|----------|---------------|-----|
| [GS-2](GAMEPLAY_SYSTEMS/SPRINT_GS2_NETCODE_ROLLBACK.md) | Rollback + predicción | Alto | GS-1 ✅ | |
| [GS-4](GAMEPLAY_SYSTEMS/SPRINT_GS4_PACK_DYNAMICS.md) | Pack formation + threat gradient | Medio | GS-3 ✅ | |
| [GS-6](GAMEPLAY_SYSTEMS/SPRINT_GS6_MAP_ENERGY.md) | Map as energy landscape | Medio | GS-5 ✅ | |
| [GS-7](GAMEPLAY_SYSTEMS/SPRINT_GS7_VISUAL_CONTRACT.md) | Injective visual mapping | Medio | GS-5 ✅, GS-6 | |
| [GS-8](GAMEPLAY_SYSTEMS/SPRINT_GS8_ARCHETYPE_CONFIG.md) | Archetype as physics config (RON) | Medio | GS-5 ✅, GS-6 | |
| [GS-9](GAMEPLAY_SYSTEMS/SPRINT_GS9_ONBOARDING.md) | Onboarding experience sequence | Alto | GS-7, GS-8 | |

Sprints archivados del track: [archive/GAMEPLAY_SYSTEMS/](archive/GAMEPLAY_SYSTEMS/) (GS-1 ✅, GS-3 ✅, GS-5 ✅)

---

### SURVIVAL_MODE (2 sprints pendientes)

| Sprint | Descripción | Esfuerzo | Bloqueado por | Doc |
|--------|-------------|----------|---------------|-----|
| [SV-2](SURVIVAL_MODE/SPRINT_SV2_SURVIVAL_BINARY.md) | Survival binary: load genomes + spawn + play | Medio | SV-1 ✅ | |
| [SV-3](SURVIVAL_MODE/SPRINT_SV3_GAME_OVER.md) | Game over: death detection + score + restart | Bajo | SV-2 | |

Sprints archivados del track: [archive/SURVIVAL_MODE/](archive/SURVIVAL_MODE/) (SV-1 ✅)

Track README: [SURVIVAL_MODE/](SURVIVAL_MODE/)

---

### DEMO (1 sprint)

| Sprint | Descripción | Esfuerzo | Bloqueado por | Doc |
|--------|-------------|----------|---------------|-----|
| Proving Grounds | ~46 entidades, 14/14 capas, 7 funciones spawn | Alto | Oleada 3 | [Doc](DEMO_PROVING_GROUNDS.md) |

---

## Tabla de oleadas

### Oleada 1 — ✅ COMPLETA (2026-03-25)

| Sprint | Track | Descripción | Esfuerzo | Desbloquea |
|--------|-------|-------------|----------|------------|
| SF-4 | SIMULATION_FOUNDATIONS | Export CSV/JSON a disco | Bajo | SF-7 |
| SF-5 | SIMULATION_FOUNDATIONS | Checkpoint save/load | Bajo | SF-7, GS-1 |
| SF-6 | SIMULATION_FOUNDATIONS | Propagación multi-tick (WaveFront) | Bajo | SF-7 |
| GS-1 | GAMEPLAY_SYSTEMS | Lockstep determinista | Bajo | GS-2 |
| GS-3 | GAMEPLAY_SYSTEMS | Nash AI targeting equations | Bajo | GS-4 |
| GS-5 | GAMEPLAY_SYSTEMS | Victory condition física | Bajo | GS-6, GS-7, GS-8 |
| SM-8D | SIMULATION_QUALITY | God-system splits (containment 248 LOC) | Bajo | — |
| SM-8F | SIMULATION_QUALITY | Lifecycle query documentation | Bajo | — |
| SM-8G | SIMULATION_QUALITY | Input SRP (grimoire → 3 sistemas) | Bajo | — |

> Todos los sprints de Oleada 1 pueden ejecutarse en paralelo (3 tracks independientes).

---

### Oleada 2 — ⏳ Desbloqueada

| Sprint | Track | Descripción | Esfuerzo | Bloqueado por | Desbloquea |
|--------|-------|-------------|----------|---------------|------------|
| [GS-2](GAMEPLAY_SYSTEMS/SPRINT_GS2_NETCODE_ROLLBACK.md) | GAMEPLAY_SYSTEMS | Rollback + predicción | Alto | GS-1 ✅ | — |
| [GS-4](GAMEPLAY_SYSTEMS/SPRINT_GS4_PACK_DYNAMICS.md) | GAMEPLAY_SYSTEMS | Pack formation + threat gradient | Medio | GS-3 ✅ | — |
| [GS-6](GAMEPLAY_SYSTEMS/SPRINT_GS6_MAP_ENERGY.md) | GAMEPLAY_SYSTEMS | Map as energy landscape | Medio | GS-5 ✅ | GS-7, GS-8 |

> Los tres GS pueden ejecutarse en paralelo.

---

### Oleada 3 — 🔒 Requiere Oleada 2

| Sprint | Track | Descripción | Esfuerzo | Bloqueado por | Desbloquea |
|--------|-------|-------------|----------|---------------|------------|
| [GS-7](GAMEPLAY_SYSTEMS/SPRINT_GS7_VISUAL_CONTRACT.md) | GAMEPLAY_SYSTEMS | Injective visual mapping | Medio | GS-5 ✅, GS-6 | GS-9 |
| [GS-8](GAMEPLAY_SYSTEMS/SPRINT_GS8_ARCHETYPE_CONFIG.md) | GAMEPLAY_SYSTEMS | Archetype as physics config (RON) | Medio | GS-5 ✅, GS-6 | GS-9 |
| [GS-9](GAMEPLAY_SYSTEMS/SPRINT_GS9_ONBOARDING.md) | GAMEPLAY_SYSTEMS | Onboarding experience sequence | Alto | GS-7, GS-8 | DEMO |

> GS-7 y GS-8 en paralelo → GS-9 en serie.

---

### Oleada 4 — 🔒 Requiere Oleada 3

| Sprint | Track | Descripción | Esfuerzo | Bloqueado por |
|--------|-------|-------------|----------|---------------|
| [Proving Grounds](DEMO_PROVING_GROUNDS.md) | DEMO | ~46 entidades, 14/14 capas, 7 funciones spawn | Alto | GS-9 |

---

## Resumen ejecutivo

| Oleada | # Sprints | Track | Paralelos | Estado | Precondición |
|--------|-----------|-------|-----------|--------|--------------|
| **1** | 9 | SF+GS+SM | 9 | ✅ COMPLETA | — |
| **SF-7** | 1 | SF | 1 | ✅ COMPLETA | Oleada 1 ✅ |
| **ET** | 16 | ET | 16 | ✅ COMPLETA | — |
| **AC** | 5 | AC | 5 | ✅ COMPLETA | — |
| **AI** | 7 | AXIOMATIC_INFERENCE | 7 | ✅ COMPLETA | — |
| **BS** | 7 | BATCH_SIMULATOR | 7 | ✅ COMPLETA | — |
| **EM** | 4 | EMERGENT_MORPHOLOGY | 4 | ✅ COMPLETA | — |
| **AS** | 3 | ANALYTICAL_STEPPING | 3 | ✅ COMPLETA | — |
| **SV** | 3 | SURVIVAL_MODE | 1 ✅ → 1 → 1 | ⏳ SV-1 done | — (independiente) |
| **2** | 3 | GS | 3 | ⏳ Desbloqueada | Oleada 1 ✅ |
| **3** | 3 | GS | 2 → 1 | 🔒 | Oleada 2 |
| **4** | 1 | DEMO | 1 | 🔒 | Oleada 3 |
| **Total** | **55** | — | — | 47 ✅ · 8 ⏳ | |

### Tracks por estado

| Estado | Tracks | Sprints |
|--------|--------|---------|
| ✅ Archivados | 39 tracks | 47 sprints |
| ⏳ Activos | GAMEPLAY_SYSTEMS (6), SURVIVAL_MODE (2) | 8 sprints |
| 🔒 Bloqueados | DEMO (1) | 1 sprint |

---

## Archived Tracks

Implementation in `src/`, contracts in `docs/design/` and `docs/arquitectura/`. Full list in [`archive/README.md`](archive/README.md):

- **SURVIVAL_MODE (parcial)** — SV-1: apply_input() wiring (InputCommand → WillActuator via WorldEntityId lookup). 5 LOC in sim_world.rs (2026-03-28) — [archive/SURVIVAL_MODE/](archive/SURVIVAL_MODE/)
- **ANALYTICAL_STEPPING** — AS-1–AS-3: O(1) analytical equations (dissipation_n, growth_n, senescence_n, locomotion_n), convergence detection (radial_max_delta, field_converged), tick_fast pipeline. 16 tests (2026-03-28) — [archive/ANALYTICAL_STEPPING/](archive/ANALYTICAL_STEPPING/)
- **EMERGENT_MORPHOLOGY** — EM-1–EM-4: 2D radial field (16×8=128 nodes), peak detection, bilateral emergence, appendage inference, joint articulation. Gravity + climate + asteroids. 30+ tests (2026-03-28) — [archive/EMERGENT_MORPHOLOGY/](archive/EMERGENT_MORPHOLOGY/)
- **AXIOMATIC_INFERENCE** — AI-1–AI-7: derived_thresholds module (12 tests), matter state thresholds from dissipation ratios, capability thresholds from density+coherence, senescence from metabolic rate (Kleiber), basal drain+pressure from dissipation, inline extraction+duplicates, visual_calibration.rs separation. 0 DEBT, 0 hardcode. (2026-03-27) — [archive/AXIOMATIC_INFERENCE/](archive/AXIOMATIC_INFERENCE/)
- **BATCH_SIMULATOR** — BS-0–BS-6: EntitySlot+SimWorldFlat arena, 33 batch systems (Tier 1/2/3), GeneticHarness+FitnessReport, GenomeBlob bridge (Batch↔Bevy), rayon parallelism, criterion benchmarks. 156 tests, 17 files (2026-03-26) — [archive/BATCH_SIMULATOR/](archive/BATCH_SIMULATOR/)
- **AXIOMATIC_CLOSURE** — AC-1–AC-5: metabolic interference (Axiom 3×8), Kuramoto entrainment (Axiom 8), culture coherence (Axiom 6×8), frequency attenuation (Axiom 7×8), cooperation emergence (Axiom 3 game theory). 60+ tests (2026-03-25) — [archive/AXIOMATIC_CLOSURE/](archive/AXIOMATIC_CLOSURE/)
- **EMERGENCE_TIERS** — ET-1–ET-16: associative memory, theory of mind, cultural transmission, infrastructure, symbiosis, epigenetics, senescence, coalitions, niche, timescales, multiscale, continental drift, geological LOD, institutions, language, consciousness (2026-03-25)
- **GAMEPLAY_SYSTEMS (parcial)** — GS-1/3/5: lockstep, Nash targeting, victory nucleus (2026-03-25) — [archive/GAMEPLAY_SYSTEMS/](archive/GAMEPLAY_SYSTEMS/)
- **SIMULATION_FOUNDATIONS** — SF-4–SF-7: metrics export, checkpoint, wavefront propagation, integration replay 11 tests (2026-03-25)
- **SIMULATION_QUALITY** — SM-8A–G: magic numbers, change detection, inline math, god-system splits, lifecycle docs, input SRP (2026-03-25)
- **INFERRED_WORLD_GEOMETRY** — IWG-1–IWG-7: body plan bilateral, terrain mesh V7, water surface, atmosphere inference, integration demo (2026-03-25)
- **MORPHOGENESIS_INFERENCE** — MG-1–MG-8: thermodynamic equations, MetabolicGraph, DAG step, shape optimization, albedo, Writer Monad, rugosity, integration demo (2026-03-25)
- **GEOMETRY_FLOW** — GF1–GF2: stateless spine+mesh, thermodynamic deformation+cache+EnergyFieldGrid wiring (2026-03-25)
- **THERMODYNAMIC_LADDER** — TL1–TL4, TL6: osmosis, nutrient field, growth budget, photosynthesis, branching; TL5 coupling closed with GF2 (2026-03-25)
- **CODE_QUALITY** — Q2, Q3, Q5, Q8: named constants, encapsulation, plugin split, color isolation (2026-03-25)
- **STRUCTURE_MIGRATION** — SM-1–SM-7: worldgen split, sim subdirs, bridge split, archetypes, macro, constants, docs (2026-03-25)
- **SIMULATION_RELIABILITY** — R1–R9: conservation, determinism, benchmarks, calibration, sensitivity, observability, morph, surrogate, CI gates (2026-03-25)
- **MACRO_STEPPING** — M1–M6: analytics, ECS routing, normalization, LOD observer, benchmark, BridgeCache audit (2026-03-25)
- **GAMEDEV_PATTERNS** — G5–G9, G11: pathfinding, change detection, event ordering, strong IDs (2026-03-25)
- **BLUEPRINT_V7** — V7-06/07/14: materialization delta, worldgen plugin, quantized color GPU (2026-03-25)
- **ENERGY_COMPETITION** — EC-1–EC-8: pools, extraction, conservation, dynamics, scale (2026-03-25)
- **CODEBASE_AUDIT** — CA-1/2/3: build fix, DOD violations, 40 core_physics tests (2026-03-25)
- **ECOSYSTEM_AUTOPOIESIS** — EA1–EA8: spawn + stress + reproduction
- **LIVING_ORGAN_INFERENCE** — LI + EET1: organ manifest + lifecycle
- **EMERGENT_FLORA** — FL1–FL6: photosynthesis + nutrients + GF1
- **FLORA_ROSA** — RS1–RS2: demos
- **ENERGY_PARTS_INFERENCE** — EPI1–EPI4: field → vertex pipeline
- **ELEMENT_ALMANAC_CANON** — EAC1–EAC4: element tables
- **ECO_BOUNDARIES** — E1–E6: zone classification
- **TOPOLOGY** — T1–T10: terrain generation
- **MIGRATION** — M1–M5: folder structure
- **CHEMICAL_REFACTOR** — C1–C4: MatterLense, catalytic pipeline
- **BRIDGE_OPTIMIZER** — B1–B10: 11 equation kinds
- **BLUEPRINT_V4/V5/V6** — Layers L11–L13, determinism+cache, 2D/3D platform
