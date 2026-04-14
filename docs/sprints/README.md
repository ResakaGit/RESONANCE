# Sprints — Active Backlog

Open work tracked here. Completed tracks archived in [`archive/`](archive/).

High-level design: [`docs/design/INDEX.md`](../design/INDEX.md). Runtime contracts: [`docs/arquitectura/`](../arquitectura/).

---

## Conteo global

| Métrica | Valor |
|---------|-------|
| Sprints pendientes | **48** (39 prev + 9 PP) |
| Tracks activos | **10** (9 prev + PLANT_PHYSIOLOGY) |
| Oleadas restantes | **4** (+PP independiente) |
| Tracks archivados | **57** (56 prev + BSD ✅) |
| Tests | **3,324** |
| LOC | **~117K** |
| Binarios | **26** |

---

## 42 sprints pendientes

### PAPER_VALIDATION (5 sprints — validación contra literatura peer-reviewed) — NEW

| Sprint | Paper | Dato cuantitativo | Esfuerzo | Bloqueado por |
|--------|-------|-------------------|----------|---------------|
| [PV-1](PAPER_VALIDATION/SPRINT_PV1_ZHANG_ADAPTIVE.md) | Zhang 2022 (eLife) | TTP 33.5 vs 14.3 meses, adaptive therapy | Medio | — |
| [PV-2](PAPER_VALIDATION/SPRINT_PV2_SHARMA_PERSISTERS.md) | Sharma 2010 (Cell) | 0.3% persisters, 100× resistencia, 9 doublings recovery | Medio | — |
| [PV-3](PAPER_VALIDATION/SPRINT_PV3_HILL_CALIBRATION.md) | GDSC/CCLE | Distribución real Hill slopes vs n=2 | Bajo | — |
| [PV-4](PAPER_VALIDATION/SPRINT_PV4_FOO_PULSED.md) | Foo & Michor 2009 (PLoS) | P(res) = 1-exp(-uB), continuous vs pulsed | Medio | — |
| [PV-5](PAPER_VALIDATION/SPRINT_PV5_MICHOR_BIPHASIC.md) | Michor 2005 (Nature) | Biphasic CML decline, slope ratio 6-10× | Medio-Alto | — |

Todos paralelos. Zero acoplamiento: cada PV es 1 archivo nuevo en `use_cases/experiments/paper_*.rs`. Track README: [PAPER_VALIDATION/](PAPER_VALIDATION/)

---

### TEMPORAL_TELESCOPE ✅ COMPLETADO (2026-04-04)

10 sprints: TT-1 (sliding statistics), TT-2 (Hurst DFA), TT-3 (projection normalizers), TT-4 (diff engine), TT-5 (telescope state), TT-6 (projection engine), TT-7 (calibration bridge), TT-8 (cascade propagator), TT-9 (dual pipeline), TT-10 (activation + dashboard). 9 archivos nuevos, 179 tests. ADR-015.

Track README: [archive/TEMPORAL_TELESCOPE/](archive/TEMPORAL_TELESCOPE/)

---

### MULTI_TELESCOPE ✅ COMPLETADO (2026-04-04)

5 sprints: MT-1 (quantum equations: speculative visibility, conservation-bounded projection, frequency-aware decay), MT-2 (conservation projection), MT-3 (telescope stack: collapse + re-emanation), MT-4 (stack pipeline), MT-5 (activation + metrics). 1 archivo nuevo + 5 modificados, 26 tests. ADR-016. Earth Telescope Demo binary.

Track README: [archive/MULTI_TELESCOPE/](archive/MULTI_TELESCOPE/)

---

### REGULATORY_INFRASTRUCTURE ✅ COMPLETADO (2026-04-02) — ARCHIVADO

3 sprints: RI-1 (CI/CD + branch protection), RI-2 (approval workflow + validation script), RI-3 (CCB + training matrix + issue templates + quarterly review). 8 ADRs en [`docs/arquitectura/ADR/`](../arquitectura/ADR/).

Track README: [archive/REGULATORY_INFRASTRUCTURE/](archive/REGULATORY_INFRASTRUCTURE/)

---

### REGULATORY_DOCUMENTATION ✅ COMPLETADO (2026-04-02) — ARCHIVADO

43 documentos + 1 índice en `docs/regulatory/` (~15,400 líneas). 50/50 ítems del checklist externo cubiertos. 8 gaps estructurales → track RI.

Track README: [archive/REGULATORY_DOCUMENTATION/](archive/REGULATORY_DOCUMENTATION/) — Sprint docs: RD-1 through RD-7 ✅

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

### PLANT_PHYSIOLOGY (9 sprints — propiedades materiales emergentes de flujos de energía) — NEW

| Sprint | Descripción | Esfuerzo | Bloqueado por |
|--------|-------------|----------|---------------|
| [PP-0](PLANT_PHYSIOLOGY/SPRINT_PP0_ORGAN_SUBPOOLS.md) | Organ Sub-Pools (`[f32; 12]`, pool invariant) | 1 sem | — |
| [PP-1](PLANT_PHYSIOLOGY/SPRINT_PP1_SPECTRAL_PIGMENT.md) | Spectral Pigmentation (color = freq reflejada) | 1 sem | PP-0 |
| [PP-2](PLANT_PHYSIOLOGY/SPRINT_PP2_PHOTOTROPISM.md) | Phototropism (spine sigue irradiancia) | 1 sem | — |
| [PP-3](PLANT_PHYSIOLOGY/SPRINT_PP3_PHENOLOGY.md) | Phenology Wiring (floración estacional) | 0.5 sem | — |
| [PP-4](PLANT_PHYSIOLOGY/SPRINT_PP4_TISSUE_CURVATURE.md) | Tissue Curvature (crecimiento diferencial) | 1.5 sem | PP-0 |
| [PP-5](PLANT_PHYSIOLOGY/SPRINT_PP5_ORGAN_SENESCENCE.md) | Organ Senescence (Gompertz per-organ) | 1 sem | PP-0 |
| [PP-6](PLANT_PHYSIOLOGY/SPRINT_PP6_VOLATILE_EMISSION.md) | Volatile Emission (fragancia → field grid) | 1 sem | PP-0 |
| [PP-7](PLANT_PHYSIOLOGY/SPRINT_PP7_ROOT_DIFFERENTIATION.md) | Root Differentiation (constructal underground) | 1 sem | PP-0 |
| [PP-8](PLANT_PHYSIOLOGY/SPRINT_PP8_POLLINATION.md) | Cross-Pollination (flora↔fauna reproduction) | 2 sem | PP-6 |

PP-2 y PP-3 independientes (paralelos con todo). PP-0 es fundación. ADRs: [ADR-033](../arquitectura/ADR/ADR-033-organ-sub-pools.md), [ADR-034](../arquitectura/ADR/ADR-034-spectral-absorption-model.md), [ADR-035](../arquitectura/ADR/ADR-035-volatile-field-protocol.md). Track README: [PLANT_PHYSIOLOGY/](PLANT_PHYSIOLOGY/)

---

### BRIDGE_STRATEGY_DECOUPLING ✅ COMPLETADO (2026-04-13) — ARCHIVADO

BS-2/3/5 ✅, BS-1/4/6/7 cancelados (ADR-017). 5 tests de integración BS-5.

Track README: [archive/BRIDGE_STRATEGY_DECOUPLING/](archive/BRIDGE_STRATEGY_DECOUPLING/)

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
| **PP** | 9 | PLANT_PHYSIOLOGY | 2 indep + 1 fundación → 6 paralelo | ⏳ Diseñado | — (independiente) |
| **TT** | 10 | TEMPORAL_TELESCOPE | 4 paralelo → 6 serie | ✅ COMPLETA | — (independiente) |
| **MT** | 5 | MULTI_TELESCOPE | 5 serie | ✅ COMPLETA | TT ✅ |
| **BSD** | 7 | BRIDGE_STRATEGY_DECOUPLING | 3 ✅ + 4 cancelados | ✅ COMPLETA | — (independiente) |
| **LR** | 4 | LAB_UI_REFACTOR | 4 ✅ | ✅ COMPLETA | — (independiente) |
| **SO** | 5 | SCIENTIFIC_OBSERVABILITY | 5 ✅ | ✅ ARCHIVADA | — |
| **RI** | 3 | REGULATORY_INFRASTRUCTURE | 1 → 2 paralelo | ✅ ARCHIVADA | RD ✅ |
| **RD** | 7 | REGULATORY_DOCUMENTATION | 4 oleadas | ✅ ARCHIVADA | — |
| **Total** | **119** | — | — | 81 ✅ · 40 ⏳ · 1 🔒 | |

### Tracks por estado

| Estado | Tracks | Sprints |
|--------|--------|---------|
| ✅ Archivados | 57 tracks (incl. RD ✅, RI ✅, TT ✅, MT ✅, LR ✅, BSD ✅) | 118 sprints |
| ⏳ Activos | GS(6), PC(7), NS(4), EI(3), TU(4), EL(4), CV(4), PV(5), MD(~10) | 47+ sprints |
| 🔒 Bloqueados | DEMO (1) | 1 sprint |

---

## Archived Tracks

Implementation in `src/`, contracts in `docs/design/` and `docs/arquitectura/`. Full list in [`archive/README.md`](archive/README.md):

- **LAB_UI_REFACTOR** — LR-1–LR-4 ✅: State machine, per-experiment controls (15 experiments, 4 categorías), Live 2D controls (pause, speed 0.25x–4x, reset, map selector con 25 mapas). ADR-018, ADR-019. (2026-04-12) — [archive/LAB_UI_REFACTOR/](archive/LAB_UI_REFACTOR/)
- **TEMPORAL_TELESCOPE** — TT-1–TT-10 ✅: Dual-timeline speculative execution (ADR-015). Ancla (ground truth tick-a-tick) + Telescopio (proyección analítica) + Puente de Calibración (feedback loop). 9 archivos, 179 tests, 0 hardcoded values. Axiomas 4/5/7 verificados con property tests. sliding_variance, Hurst DFA, Fisher information, RegimeMetrics, NormalizerWeights, DiffReport, cascade propagator, calibration bridge, dual pipeline sync. (2026-04-04) — [archive/TEMPORAL_TELESCOPE/](archive/TEMPORAL_TELESCOPE/)
- **BRIDGE_STRATEGY_DECOUPLING** — BS-2/3/5 ✅ (exact_cache, KleiberCache, GompertzCache, Converged\<T\>, 5 integration tests). BS-1/4/6/7 cancelados por ADR-017. 57 tests. (2026-04-13) — [archive/BRIDGE_STRATEGY_DECOUPLING/](archive/BRIDGE_STRATEGY_DECOUPLING/)
- **MOLECULAR_DYNAMICS** — MD-0–MD-19 ✅: Velocity Verlet, Langevin thermostat, PBC, neighbor lists, LJ fluid, bonded potentials, topology, 3D/f64, cutoff, peptide vacuum, TIP3P water, SHAKE, Ewald, FF loader, solvated peptide, Go model, REMD, folding validation, analysis suite, rayon-parallel forces. (2026-04-13) — [archive/MOLECULAR_DYNAMICS/](archive/MOLECULAR_DYNAMICS/)
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
- **REGULATORY_DOCUMENTATION** — RD-1–RD-7 ✅: 43 documentos regulatorios + índice maestro en `docs/regulatory/` (~15,400 líneas). IEC 62304, ISO 14971, ASME V&V 40, FDA CMS. 50/50 checklist items. (2026-04-02) — [archive/REGULATORY_DOCUMENTATION/](archive/REGULATORY_DOCUMENTATION/)
- **REGULATORY_INFRASTRUCTURE** — RI-1–RI-3 ✅: CI/CD pipeline, approval workflow, CCB charter, training matrix, issue templates, quarterly review. 8 ADRs. (2026-04-02) — [archive/REGULATORY_INFRASTRUCTURE/](archive/REGULATORY_INFRASTRUCTURE/)
- **BRIDGE_OPTIMIZER** — B1–B10: 11 equation kinds
- **BLUEPRINT_V4/V5/V6** — Layers L11–L13, determinism+cache, 2D/3D platform
