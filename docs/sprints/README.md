# Sprints — backlog activo

Índice de trabajo **no cerrado** en código. Tracks completados archivados en [`archive/`](archive/).

Para diseño de alto nivel ver [`docs/design/INDEX.md`](../design/INDEX.md). Para contratos runtime ver [`docs/arquitectura/`](../arquitectura/).

---

## Worldgen / render

| Track | Pendiente | Doc |
|------|-----------|-----|
| **BLUEPRINT_V7** | Sprint **06** (materialización incremental), **07** (`worldgen_plugin`), **14** (color cuantizado GPU/WGSL) | [BLUEPRINT_V7/](BLUEPRINT_V7/) |

## Tracks cerrados recientes

- **STRUCTURE_MIGRATION** — SM-1–SM-7: todos cerrados (2026-03-25). Ver [STRUCTURE_MIGRATION/](STRUCTURE_MIGRATION/)
- **CODE_QUALITY** — Q2, Q3, Q5, Q8: todos cerrados (2026-03-25). Ver [CODE_QUALITY/](CODE_QUALITY/)

## Patrones MOBA

| Track | Pendiente | Doc |
|------|-----------|-----|
| **GAMEDEV_PATTERNS** | **G5** (pathfinding: flowfield/avoidance), **G8** (change detection audit), **G9** (event ordering table), **G11** (strong IDs e2e). G10 (minimap) y G12 (fog) ya implementados | [GAMEDEV_PATTERNS/](GAMEDEV_PATTERNS/) |

## Confiabilidad del simulador

| Track | Pendiente | Doc |
|------|-----------|-----|
| **SIMULATION_RELIABILITY** | **R1–R9** (unidades, determinismo, benchmarks, calibración, sensibilidad, observabilidad, morfología, surrogate, CI) | [SIMULATION_RELIABILITY/](SIMULATION_RELIABILITY/) |

## Fundamentos de simulación

| Track | Pendiente | Doc |
|------|-----------|-----|
| **SIMULATION_FOUNDATIONS** | **SF-1–SF-7**: observabilidad externa (CSV/JSON), serialización/checkpoint/replay, latencia de señal (propagación multi-tick). SF-1/2/3 paralelos → SF-4/5/6 paralelos → SF-7 integración | [SIMULATION_FOUNDATIONS/](SIMULATION_FOUNDATIONS/) |

## Morfogénesis inferida

| Track | Pendiente | Doc |
|------|-----------|-----|
| **MORPHOGENESIS_INFERENCE** | **MG-8** pendiente; MG-1–MG-7 cerrados | [MORPHOGENESIS_INFERENCE/](MORPHOGENESIS_INFERENCE/) |

## Escalera termodinámica / temporal LOD / geometría

| Track | Pendiente | Doc |
|------|-----------|-----|
| **THERMODYNAMIC_LADDER** | TL1–TL6 en código; acoplamiento **TL5-GF2** abierto | [THERMODYNAMIC_LADDER/](THERMODYNAMIC_LADDER/) |
| **MACRO_STEPPING** | **M1–M6** (analytics, ECS routing, normalización, LOD observer, benchmark, bridge macro) | [MACRO_STEPPING/](MACRO_STEPPING/) |
| **GEOMETRY_FLOW** | **GF2** deformación post-branching (GF1 cerrado) | [GEOMETRY_FLOW/](GEOMETRY_FLOW/) |

---

## Tracks cerrados (archivados)

Implementación en `src/`, contratos en `docs/design/` y `docs/arquitectura/`. READMEs residuales en [`archive/`](archive/):

- **CODE_QUALITY** — Q2, Q3, Q5, Q8: constantes nombradas, encapsulación, plugin split, color isolation (2026-03-25)
- **STRUCTURE_MIGRATION** — SM-1–SM-7: split worldgen/sim/bridge/archetypes, macro bridge, constants, docs (2026-03-25)
- **ENERGY_COMPETITION** — EC-1–EC-8: pools jerárquicos, extracción competitiva, conservación, escala-invariante (2026-03-25)
- **BLUEPRINT_V4** — Capas L11–L13 (TensionField, Homeostasis, StructuralLink)
- **BLUEPRINT_V5** — Determinismo y cache (BridgeCache)
- **BLUEPRINT_V6** — Plataforma 2D/3D (runtime_platform/)
- **BRIDGE_OPTIMIZER** — B1–B10, 11 equation kinds
- **CHEMICAL_REFACTOR** — C1–C4, MatterLense, catalytic pipeline
- **ECO_BOUNDARIES** — E1–E6, zone classification
- **ELEMENT_ALMANAC_CANON** — EAC1–EAC4, element tables
- **EMERGENT_FLORA** — FL1–FL6, photosynthesis + nutrients + GF1
- **ENERGY_PARTS_INFERENCE** — EPI1–EPI4, field → vertex pipeline
- **FLORA_ROSA** — RS1–RS2, demos
- **LIVING_ORGAN_INFERENCE** — LI + EET1, organ manifest + lifecycle
- **ECOSYSTEM_AUTOPOIESIS** — EA1–EA8, spawn + stress + reproduction
- **MIGRATION** — M1–M5, folder structure
- **TOPOLOGY** — T1–T10, terrain generation
