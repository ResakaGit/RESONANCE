# Design Docs — Index

Especificaciones de alto nivel del proyecto Resonance. Para contratos runtime por modulo ver `docs/arquitectura/`. Para backlog de sprints ver `docs/sprints/`.

---

## Filosofia

| Documento | Descripcion |
|-----------|-------------|
| [BLUEPRINT.md](BLUEPRINT.md) | Axioma fundacional: 14 capas ortogonales, todo es energia, plano de composicion |

## Fundamentos Axiomáticos

| Concepto | Descripcion |
|----------|-------------|
| **8 Axiomas** | Reglas del universo (Everything is Energy, Pool Invariant, Competition, Dissipation, Conservation, Emergence, Distance Attenuation, Oscillatory Nature) |
| **4 Constantes Fundamentales** | Parámetros irreducibles: `KLEIBER_EXPONENT` (0.75), `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}` (0.005–0.25), `COHERENCE_BANDWIDTH` (50 Hz), `DENSITY_SCALE` (20.0) |
| **Derivación** | `blueprint/equations/derived_thresholds.rs` — computa todas las constantes de lifecycle desde los 4 fundamentales (12 tests). Cero hardcode en production. |
| **Sprint** | ✅ ARCHIVED `docs/sprints/archive/AXIOMATIC_INFERENCE/` — 7/7 sprints completed. `visual_calibration.rs` for rendering tuning. |

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
| [EVOLUTION_GROUP_BEHAVIOR.md](EVOLUTION_GROUP_BEHAVIOR.md) | Evolucion, seleccion natural, dinamicas grupales |

## Ciclo de Energia

| Documento | Descripcion |
|-----------|-------------|
| CLAUDE.md §Energy Cycle | Closed loop: nucleus → field → entities → death → nutrients → new nucleus |
| `blueprint/constants/nucleus_lifecycle.rs` | Reservoir, depletion, pressure, recycling constants |
| `blueprint/constants/senescence.rs` | Age/death constants differentiated by entity type |
| `blueprint/equations/radiation_pressure.rs` | Non-linear outward push when qe > threshold |

## Bevy Decoupling

| Area | Estado |
|------|--------|
| `math_types.rs` (glam re-exports) | ✅ 34 files migrated to `crate::math_types` |
| `blueprint/equations/` (178 files) | ✅ 100% bevy::math free |
| `blueprint/constants/` | ✅ 100% bevy-free |
| `topology/`, `geometry_flow/`, `eco/`, `bridge/` | ✅ Pure math decoupled |
| `layers/`, `simulation/`, `plugins/` | Pending — #[derive(Component)] coupled |
| Headless runner (`src/bin/headless_sim.rs`) | ✅ Full sim → PPM without GPU |

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
