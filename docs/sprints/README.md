# Sprints — Active Backlog

Open work tracked here. Completed tracks archived in [`archive/`](archive/).

High-level design: [`docs/design/INDEX.md`](../design/INDEX.md). Runtime contracts: [`docs/arquitectura/`](../arquitectura/).

---

## Morphogenesis

| Track | Pending | Doc |
|-------|---------|-----|
| **MORPHOGENESIS_INFERENCE** | MG-8 (integration demo + EntityBuilder wiring); MG-1–MG-7 closed | [MORPHOGENESIS_INFERENCE/](MORPHOGENESIS_INFERENCE/) |

## Geometry / Rendering

| Track | Pending | Doc |
|-------|---------|-----|
| **GEOMETRY_FLOW** | GF2 (post-branching deformation) | [GEOMETRY_FLOW/](GEOMETRY_FLOW/) |
| **THERMODYNAMIC_LADDER** | TL5–GF2 coupling open; TL1–TL4, TL6 closed | [THERMODYNAMIC_LADDER/](THERMODYNAMIC_LADDER/) |
| **INFERRED_WORLD_GEOMETRY** | IWG-1–IWG-7: body plan, terrain mesh, water surface, atmosphere, demo | [INFERRED_WORLD_GEOMETRY/](INFERRED_WORLD_GEOMETRY/) |

## Simulation

| Track | Pending | Doc |
|-------|---------|-----|
| **SIMULATION_FOUNDATIONS** | SF-1–SF-7: observability CSV/JSON, serialization+replay, signal latency propagation | [SIMULATION_FOUNDATIONS/](SIMULATION_FOUNDATIONS/) |
| **SIMULATION_QUALITY** | SM-8: god-systems in `thermodynamic/`, magic numbers, SRP violations in `reactions.rs` | [SIMULATION_QUALITY/](SIMULATION_QUALITY/) |

---

## Archived Tracks

Implementation in `src/`, contracts in `docs/design/` and `docs/arquitectura/`. Full list in [`archive/README.md`](archive/README.md):

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
