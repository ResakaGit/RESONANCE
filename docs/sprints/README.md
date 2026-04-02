# Sprints — Active Backlog

Open work tracked here. Completed tracks archived in [`archive/`](archive/).

High-level design: [`docs/design/INDEX.md`](../design/INDEX.md). Runtime contracts: [`docs/arquitectura/`](../arquitectura/).

---

## Conteo global

| Métrica | Valor |
|---------|-------|
| Sprints pendientes | **42** (39 prev + 3 RI) |
| Tracks activos | **9** |
| Oleadas restantes | **4** |
| Tracks archivados | **50** (49 prev + RD ✅) |
| Tests | **3,113** |
| LOC | **113K** |
| Binarios | **25** |

---

## 42 sprints pendientes

### REGULATORY_INFRASTRUCTURE (3 sprints — cierra gaps estructurales) — NEW

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| [RI-1](REGULATORY_INFRASTRUCTURE/SPRINT_RI1_CICD_PIPELINE.md) | CI/CD Pipeline + Branch Protection | Medio | — |
| [RI-2](REGULATORY_INFRASTRUCTURE/SPRINT_RI2_SIGNING_IDENTITY.md) | Commit Signing + Document Approval | Medio | RI-1 |
| [RI-3](REGULATORY_INFRASTRUCTURE/SPRINT_RI3_GOVERNANCE_ACTIVATION.md) | Governance + Monitoring Activation | Bajo | RI-1 |

Oleada: **0** RI-1 → **1** RI-2 + RI-3 (paralelos)

ADRs: [`docs/arquitectura/ADR/`](../arquitectura/ADR/) — 8 decisiones documentadas

---

### REGULATORY_DOCUMENTATION ✅ COMPLETADO (2026-04-02)

43 documentos + 1 índice en `docs/regulatory/` (~15,400 líneas). 50/50 ítems del checklist externo cubiertos. 8 gaps estructurales → track RI.

Track README: [REGULATORY_DOCUMENTATION/](REGULATORY_DOCUMENTATION/) — Sprint docs: RD-1 through RD-7 ✅

---

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

### SURVIVAL_MODE ✅ COMPLETADO

Sprints archivados: SV-1 ✅ (input wiring), SV-2 ✅ (survival binary), SV-3 ✅ (game over + restart).

`cargo run --release --bin survival -- --genomes assets/evolved/seed_42.bin`

Track README: [SURVIVAL_MODE/](SURVIVAL_MODE/) — [archive/SURVIVAL_MODE/](archive/SURVIVAL_MODE/)

---

### PARTICLE_CHARGE (7 sprints — infraestructura + átomos)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| PC-0 | Entity Scale (64→1024, bitset) | 2 sem | — |
| PC-1 | Spatial Acceleration (Barnes-Hut O(N log N)) | 2 sem | — |
| PC-2 | Continuous Forces (force accumulator) | 1 sem | — |
| PC-3 | Charge Layer (L-1: charge + mass) | 1 sem | PC-0,1,2 |
| PC-4 | Coulomb Force + Lennard-Jones | 1 sem | PC-3 |
| PC-5 | Emergent Bonding (stable pairs) | 2 sem | PC-4 |
| PC-6 | Element Emergence (observability) | 2 sem | PC-5 |

Track README: [PARTICLE_CHARGE/](PARTICLE_CHARGE/)

---

### NERVOUS_SYSTEM (4 sprints)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| NS-1 | Signal Propagation (freq pulse por links) | 1 sem | MC ✅ |
| NS-2 | Activation Threshold (fire or not) | 1 sem | NS-1 |
| NS-3 | Reflex Arc (sensory → signal → motor) | 1 sem | NS-2 |
| NS-4 | Batch Integration + observability | 1 sem | NS-3 |

Track README: [NERVOUS_SYSTEM/](NERVOUS_SYSTEM/)

---

### EMERGENT_INTELLIGENCE (3 sprints)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| EI-1 | Prediction Quality (SelfModel accuracy) | 1 sem | NS ✅ |
| EI-2 | Planning Horizon (action selection) | 1 sem | EI-1 |
| EI-3 | Batch Integration + intelligence_score | 1 sem | EI-2 |

Track README: [EMERGENT_INTELLIGENCE/](EMERGENT_INTELLIGENCE/)

---

### TOOL_USE (4 sprints — herramientas + agricultura)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| TU-1 | Modify Intent (change without consume) | 1 sem | EI ✅ |
| TU-2 | Tool Crafting (modify bond_energy) | 1 sem | TU-1 |
| TU-3 | Farming (harvest without killing) | 1 sem | TU-1 |
| TU-4 | Batch Integration + tool_use_rate | 1 sem | TU-2, TU-3 |

Track README: [TOOL_USE/](TOOL_USE/)

---

### EMERGENT_LANGUAGE (4 sprints)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| EL-1 | Signal Emission (freq pulse + label) | 1 sem | NS ✅ |
| EL-2 | Signal-Event Association (learning) | 1 sem | EL-1 |
| EL-3 | Compositionality (combine 2 signals) | 1 sem | EL-2 |
| EL-4 | Batch Integration + vocab_size_mean | 1 sem | EL-3 |

Track README: [EMERGENT_LANGUAGE/](EMERGENT_LANGUAGE/)

---

### CIVILIZATION (4 sprints)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| CV-1 | Persistent Structures (buildings) | 1 sem | TU ✅ |
| CV-2 | Access Rules (coalition-based) | 1 sem | CV-1, EL ✅ |
| CV-3 | Resource Trade (qe exchange) | 1 sem | CV-2 |
| CV-4 | Batch Integration + civilization_score | 1 sem | CV-3 |

Track README: [CIVILIZATION/](CIVILIZATION/)

---

### BRIDGE_STRATEGY_DECOUPLING (5 sprints pendientes)

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| BS-1 | NormStrategy enum + desacople normalización | Medio | — |
| BS-4 | 6 bridges nuevos (basal, senescence, awakening, rad, shape, epi) | Alto | BS-1 |
| BS-5 | TDD: tests unitarios + integración tier 1 | Alto | BS-4 |
| BS-6 | HOF composition (NormPipeline) | Medio | BS-1 |
| BS-7 | RON presets para estrategias + validación keys | Bajo | BS-6 |

Sprints archivados del track: BS-2 ✅ (bug fixes), BS-3 parcial ✅ (exact cache components)

Track README: [BRIDGE_STRATEGY_DECOUPLING/](BRIDGE_STRATEGY_DECOUPLING/)

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
| **SV** | 3 | SURVIVAL_MODE | 3 ✅ | ✅ ARCHIVADA | — |
| **VG** | 6 | VARIABLE_GENOME | 6 serie | ✅ COMPLETA | — |
| **MGN** | 7 | METABOLIC_GENOME | 4 serie → 3 paralelo | ✅ COMPLETA | VG ✅ |
| **2** | 3 | GS | 3 | ⏳ Desbloqueada | Oleada 1 ✅ |
| **3** | 3 | GS | 2 → 1 | 🔒 | Oleada 2 |
| **4** | 1 | DEMO | 1 | 🔒 | Oleada 3 |
| **PD** | 5 | PROTO_DNA | 5 serie | ✅ COMPLETA | VG ✅ |
| **MC** | 5 | MULTICELLULARITY | 5 serie | ✅ COMPLETA | — |
| **PC** | 7 | PARTICLE_CHARGE | 3 infra → 4 core | ⏳ Diseñado | — |
| **NS** | 4 | NERVOUS_SYSTEM | 4 serie | ⏳ Diseñado | MC ✅ |
| **EI** | 3 | EMERGENT_INTELLIGENCE | 3 serie | ⏳ Diseñado | NS |
| **TU** | 4 | TOOL_USE | 2 serie + 2 paralelo | ⏳ Diseñado | EI |
| **EL** | 4 | EMERGENT_LANGUAGE | 4 serie | ⏳ Diseñado | NS |
| **CV** | 4 | CIVILIZATION | 4 serie | ⏳ Diseñado | TU + EL |
| **BSD** | 7 | BRIDGE_STRATEGY_DECOUPLING | 2 ✅ → 5 pendiente | ⏳ BS-2/3 done | — (independiente) |
| **SO** | 5 | SCIENTIFIC_OBSERVABILITY | 5 ✅ | ✅ ARCHIVADA | — |
| **RI** | 3 | REGULATORY_INFRASTRUCTURE | 1 → 2 paralelo | ⏳ Diseñado | RD ✅ |
| **RD** | 7 | REGULATORY_DOCUMENTATION | 4 oleadas | ✅ COMPLETA | — |
| **Total** | **115** | — | — | 77 ✅ · 40 ⏳ · 1 🔒 | |

### Tracks por estado

| Estado | Tracks | Sprints |
|--------|--------|---------|
| ✅ Archivados | 50 tracks (incl. RD ✅) | 85 sprints |
| ⏳ Activos | GS(6), PC(7), NS(4), EI(3), TU(4), EL(4), CV(4), BSD(5), RI(3) | 40 sprints |
| 🔒 Bloqueados | DEMO (1) | 1 sprint |

---

## Archived Tracks

Implementation in `src/`, contracts in `docs/design/` and `docs/arquitectura/`. Full list in [`archive/README.md`](archive/README.md):

- **BRIDGE_STRATEGY_DECOUPLING (parcial)** — BS-2 ✅: CompetitionNormBridge wired + hot reload fix. BS-3 parcial ✅: exact_cache (KleiberCache, GompertzCache, Converged\<T\>), shape_cache_signature extraction. 52 tests (2026-03-30) — [BRIDGE_STRATEGY_DECOUPLING/](BRIDGE_STRATEGY_DECOUPLING/)
- **SCIENTIFIC_OBSERVABILITY** — SO-1–SO-5 ✅: lineage, census, CSV/JSON export, HOF orchestrators, CSV in binaries. 32 tests (2026-03-30) — [archive/SCIENTIFIC_OBSERVABILITY/](archive/SCIENTIFIC_OBSERVABILITY/)
- **PROTO_DNA** — PD-1–PD-5: CodonGenome (tripletes), CodonTable (64→8 amino, evolucionable), translate_genome, silent mutations, neutral drift, batch wiring. 28 tests (2026-03-30) — [archive/PROTO_DNA/](archive/PROTO_DNA/)
- **MULTICELLULARITY** — MC-1–MC-5: cell adhesion (freq×distance), colony detection (Union-Find), positional signaling, differential expression (borde=defensa, interior=metabolismo), batch wiring. 33 tests (2026-03-30) — [archive/MULTICELLULARITY/](archive/MULTICELLULARITY/)
- **METABOLIC_GENOME** — MGN-1–MGN-7: gene→ExergyNode, topology inference, graph from genome, evolution integration, node competition, Hebbian rewiring, internal catalysis. 80 tests, 100% metabolic networks. (2026-03-29) — [archive/METABOLIC_GENOME/](archive/METABOLIC_GENOME/)
- **VARIABLE_GENOME** — VG-1–VG-6: VariableGenome (4-32 genes), maintenance cost (Kleiber), duplication/deletion mutation, expression mapping, epigenetic gating, bridge/serialization. 62 tests. (2026-03-29) — [archive/VARIABLE_GENOME/](archive/VARIABLE_GENOME/)
- **SURVIVAL_MODE** — SV-1–SV-3 ✅: input wiring, survival binary (WASD + genome load + arena + score + death + game over + restart). Zero src/ changes. (2026-03-30) — [archive/SURVIVAL_MODE/](archive/SURVIVAL_MODE/)
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
