# Design Docs — Index

Especificaciones de alto nivel del proyecto Resonance. Para contratos runtime por módulo ver `docs/arquitectura/`. Para backlog de sprints ver `docs/sprints/`.

---

## Filosofía y fundamentos

| Documento | Alcance |
|-----------|---------|
| [BLUEPRINT.md](BLUEPRINT.md) | Axioma fundacional: 14 capas ortogonales, todo es energía, plano de composición |
| [DESIGNING.md](../../DESIGNING.md) | Filosofía de capas, test de 5 preguntas para proponer nuevas, lifecycle de energía |

## Subsistemas activos (código implementado)

| Documento | Módulo `src/` | Descripción |
|-----------|---------------|-------------|
| [V7.md](V7.md) | `worldgen/` | Worldgen procedural: field_grid, nucleus, propagation, materialization |
| [V6.md](V6.md) | `runtime_platform/` | Plataforma 2D/3D: 12 sub-módulos, compat profiles, render bridge |
| [V5.md](V5.md) | `bridge/` | Determinismo y cache: FrameLocalCache, canonicalization, invalidation DAG |
| [TOPOLOGY.md](TOPOLOGY.md) | `topology/` | Terreno procedural: noise, slope, drainage, hydraulics, mutations |
| [ECO_BOUNDARIES.md](ECO_BOUNDARIES.md) | `eco/` | Zonas ecológicas: lazy evaluation, climate, boundary markers |
| [BRIDGE_OPTIMIZER.md](BRIDGE_OPTIMIZER.md) | `bridge/` | BridgeCache<B>: 11 equation kinds, quantization bands, histéresis |
| [QUANTIZED_COLOR_ENGINE.md](QUANTIZED_COLOR_ENGINE.md) | `rendering/` | GPU color quantization: precision_factor, palette LUT, LOD |
| [EMERGENT_FLORA.md](EMERGENT_FLORA.md) | `simulation/`, `geometry_flow/` | Flora emergente: fotosíntesis, nutrientes, GF1, autopoiesis |
| [TERRAIN_MESHER.md](TERRAIN_MESHER.md) | `topology/` | Mesh procedural de terreno: stateless pipeline, DoD memory layout |

## Subsistemas en diseño (blueprint / sprints activos)

| Documento | Track sprints | Descripción |
|-----------|---------------|-------------|
| [MORPHOGENESIS.md](MORPHOGENESIS.md) | `sprints/MORPHOGENESIS_INFERENCE/` | Forma inferida desde termodinámica: MetabolicGraph, EntropyLedger, solvers |
| [THERMODYNAMIC_LADDER.md](THERMODYNAMIC_LADDER.md) | `sprints/THERMODYNAMIC_LADDER/` | 5 capas de complejidad: osmosis, nutrients, growth, photosynthesis, allometry |
| [MACRO_STEPPING.md](MACRO_STEPPING.md) | `sprints/MACRO_STEPPING/` | Temporal LOD: analytical solvers O(1) para entidades lejanas |
| [GEOMETRY_DEFORMATION_ENGINE.md](GEOMETRY_DEFORMATION_ENGINE.md) | `sprints/GEOMETRY_FLOW/` (GF2) | Deformación post-branching: tensores termodinámicos, hydro response |
| [GAMEDEV_PATTERNS.md](GAMEDEV_PATTERNS.md) | `sprints/GAMEDEV_PATTERNS/` | Catálogo MOBA: cooldowns, buffs, vision, targeting → modelo energético |
| [GAMEDEV_IMPLEMENTATION.md](GAMEDEV_IMPLEMENTATION.md) | `sprints/GAMEDEV_PATTERNS/` | Mapeo mecánicas MOBA → ECS: patterns, invariants, philosophy checklist |

## Propuestas (sin código aún)

| Documento | Descripción |
|-----------|-------------|
| [LAYER15_TACTICAL_INFERENCE.md](LAYER15_TACTICAL_INFERENCE.md) | L15 propuesta: MotionIntent, BranchIntent, tactical decision inference |

## Referencia estructural

| Documento | Descripción |
|-----------|-------------|
| [FOLDER_STRUCTURE.md](FOLDER_STRUCTURE.md) | Inventario vivo de ~190 archivos, 14 módulos, migraciones M1–M5 |

---

## Eliminados (2026-03-24)

Docs obsoletos removidos por estar supersedidos:

| Archivo | Razón |
|---------|-------|
| V2.md | Iteración histórica; filosofía absorbida por BLUEPRINT.md |
| V3.md | Data-driven element system; codificado en runtime |
| V4.md | Fractal ontology (L11-L13); layers implementadas en código |
| SIM_LAYERS.md | 5-layer pipeline; supersedido por `simulation/mod.rs` Phase enum |
| CHEMICAL_REFACTOR.md | Refactoring completado; patrón en código |
| VISUAL_QUANTIZATION.md | Contenido en QUANTIZED_COLOR_ENGINE.md + arquitectura/ |
| SENSORY_ATTENTION.md | Contenido en arquitectura/blueprint_sensory_lod.md |
