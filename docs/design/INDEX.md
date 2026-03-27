# Design Docs — Index

Especificaciones de alto nivel del proyecto Resonance. Para contratos runtime por modulo ver `docs/arquitectura/`. Para backlog de sprints ver `docs/sprints/`.

---

## Filosofia

| Documento | Descripcion |
|-----------|-------------|
| [BLUEPRINT.md](BLUEPRINT.md) | Axioma fundacional: 14 capas ortogonales, todo es energia, plano de composicion |

## Arquitectura

| Documento | Modulo `src/` | Descripcion |
|-----------|---------------|-------------|
| [FOLDER_STRUCTURE.md](FOLDER_STRUCTURE.md) | — | Inventario de ~190 archivos, 14 modulos, migraciones M1–M5 |
| [SIMULATION_CORE_DECOUPLING.md](SIMULATION_CORE_DECOUPLING.md) | `sim_world.rs` | SimWorld boundary: tick(cmds), snapshot(), energy_hash(), 8 invariants |
| [V5.md](V5.md) | `bridge/` | Determinismo y cache: FrameLocalCache, canonicalization, invalidation DAG |
| [V6.md](V6.md) | `runtime_platform/` | Plataforma 2D/3D: 12 sub-modulos, compat profiles, render bridge |
| [V7.md](V7.md) | `worldgen/` | Worldgen procedural: field_grid, nucleus, propagation, materialization |

## Subsistemas

| Documento | Modulo `src/` | Descripcion |
|-----------|---------------|-------------|
| [TOPOLOGY.md](TOPOLOGY.md) | `topology/` | Terreno procedural: noise, slope, drainage, hydraulics, mutations |
| [BRIDGE_OPTIMIZER.md](BRIDGE_OPTIMIZER.md) | `bridge/` | BridgeCache<B>: 11 equation kinds, quantization bands, histeresis |
| [ECO_BOUNDARIES.md](ECO_BOUNDARIES.md) | `eco/` | Zonas ecologicas: lazy evaluation, climate, boundary markers |
| [QUANTIZED_COLOR_ENGINE.md](QUANTIZED_COLOR_ENGINE.md) | `rendering/` | GPU color quantization: precision_factor, palette LUT, LOD |
| [EMERGENT_FLORA.md](EMERGENT_FLORA.md) | `simulation/`, `geometry_flow/` | Flora emergente: fotosintesis, nutrientes, GF1, autopoiesis |
| [TERRAIN_MESHER.md](TERRAIN_MESHER.md) | `topology/` | Mesh procedural de terreno: stateless pipeline, DoD memory layout |
| [MORPHOGENESIS.md](MORPHOGENESIS.md) | `blueprint/`, `simulation/lifecycle/` | Forma inferida: MetabolicGraph, EntropyLedger, solvers |
| [THERMODYNAMIC_LADDER.md](THERMODYNAMIC_LADDER.md) | `simulation/thermodynamic/` | 5 capas de complejidad: osmosis, nutrients, growth, photosynthesis, allometry |
| [GEOMETRY_DEFORMATION_ENGINE.md](GEOMETRY_DEFORMATION_ENGINE.md) | `geometry_flow/` | Deformacion post-branching: tensores termodinamicos, hydro response |
| [INFERRED_WORLD_GEOMETRY.md](INFERRED_WORLD_GEOMETRY.md) | `worldgen/inference/` | Geometria del mundo inferida desde campos energeticos |
| [MACRO_STEPPING.md](MACRO_STEPPING.md) | `simulation/` | Temporal LOD: analytical solvers O(1) para entidades lejanas |
| [BATCH_SIMULATOR.md](../arquitectura/blueprint_batch_simulator.md) | `batch/` | Simulador masivo sin Bevy: SimWorldFlat, WorldBatch ×1M, GeneticHarness, evolucion real |

## Gameplay

| Documento | Descripcion |
|-----------|-------------|
| [GAMEDEV_PATTERNS.md](GAMEDEV_PATTERNS.md) | Catalogo MOBA: cooldowns, buffs, vision, targeting → modelo energetico |
| [GAMEDEV_IMPLEMENTATION.md](GAMEDEV_IMPLEMENTATION.md) | Mapeo mecanicas MOBA → ECS: patterns, invariants, philosophy checklist |
| [AXIOMATIC_CLOSURE.md](AXIOMATIC_CLOSURE.md) | 5 dinamicas cross-axiom: interference, Kuramoto, culture, purity, cooperation |
| [EMERGENCE_TIERS.md](EMERGENCE_TIERS.md) | 16 modulos de emergencia organizados en tiers |

## Propuestas

| Documento | Descripcion |
|-----------|-------------|
| [LAYER15_TACTICAL_INFERENCE.md](LAYER15_TACTICAL_INFERENCE.md) | L15 propuesta: MotionIntent, BranchIntent, tactical decision inference |

---

## Eliminados (2026-03-24)

Docs obsoletos removidos por estar supersedidos:

| Archivo | Razon |
|---------|-------|
| V2.md | Iteracion historica; filosofia absorbida por BLUEPRINT.md |
| V3.md | Data-driven element system; codificado en runtime |
| V4.md | Fractal ontology (L11-L13); layers implementadas en codigo |
| SIM_LAYERS.md | 5-layer pipeline; supersedido por `simulation/mod.rs` Phase enum |
| CHEMICAL_REFACTOR.md | Refactoring completado; patron en codigo |
| VISUAL_QUANTIZATION.md | Contenido en QUANTIZED_COLOR_ENGINE.md + arquitectura/ |
| SENSORY_ATTENTION.md | Contenido en arquitectura/blueprint_sensory_lod.md |
| ~~DESIGNING.md~~ | Filosofia absorbida por BLUEPRINT.md — 5-Test para capas, lifecycle de energia |
