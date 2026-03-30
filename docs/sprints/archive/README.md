# Archived Sprints

Completed tracks with code in `src/`. Each folder keeps its original README as historical reference.

Live contracts: `docs/design/` (specs) and `docs/arquitectura/` (runtime blueprints per module).

## Index

| Track | Description | Closed |
|-------|-------------|--------|
| **BRIDGE_STRATEGY_DECOUPLING (parcial)** | BS-2: CompetitionNormBridge wired (5 macros), hot reload phase reset, shape_cache_signature extraction. BS-3 parcial: exact_cache.rs (kleiber_volume_factor, exact_death_tick, frequency_alignment_exact), KleiberCache, GompertzCache, Converged\<T\> components. 52 tests. BS-1/BS-4–BS-7 pendientes. | 2026-03-30 |
| **SCIENTIFIC_OBSERVABILITY** | SO-1–SO-5: lineage (FNV-1a u64), census (alive_mask capture + HOF distribution/mean), CSV/JSON adapters (zero-alloc write), HOF orchestrators (ablate, ensemble, sweep), CSV wired to fermi/cancer/convergence binaries. 32 tests. | 2026-03-30 |
| **PROTO_DNA** | PD-1–5: CodonGenome, CodonTable (evolucionable), translation, silent mutations, neutral drift, batch wiring. 28 tests. | 2026-03-30 |
| **MULTICELLULARITY** | MC-1–5: cell adhesion, colony detection (Union-Find), positional signal, differential expression, batch wiring. 33 tests. | 2026-03-30 |
| **METABOLIC_GENOME** | MGN-1–7: gene→ExergyNode mapping, topology inference, graph from genome, evolution integration, node competition (Axiom 3), Hebbian rewiring (Hebb 1949), internal catalysis with frequency alignment (Axiom 8). 80 tests, 100% metabolic networks. | 2026-03-29 |
| **VARIABLE_GENOME** | VG-1–6: VariableGenome [4-32 genes], Kleiber maintenance cost, duplication/deletion mutation (Schwefel), expression mapping, epigenetic gating, GenomeBlob bridge, serialization. 62 tests. | 2026-03-29 |
| **SURVIVAL_MODE** | SV-1–SV-3: input wiring (5 LOC), survival binary (WASD, genome load, arena, score, HUD, dashboard bridge), game over (DeathEvent + qe fallback, overlay, restart). Zero src/ changes except sim_world.rs. | 2026-03-30 |
| **ANALYTICAL_STEPPING** | AS-1–AS-3: O(1) analytical stepping (dissipation, growth, senescence), convergence detection, tick_fast pipeline (16 tests) | 2026-03-28 |
| **EMERGENT_MORPHOLOGY** | EM-1–EM-4: 2D radial field (16×8=128 nodes), peak detection, bilateral emergence, appendage inference, joints, gravity+climate+asteroids (30+ tests) | 2026-03-28 |
| **BATCH_SIMULATOR** | BS-0–BS-6: batch arena (EntitySlot, SimWorldFlat), 33 systems (Tier 1/2/3), GeneticHarness, GenomeBlob bridge, rayon parallelism, criterion benchmarks (156 tests, 17 files) | 2026-03-26 |
| **EMERGENCE_TIERS** | ET-1–ET-16: associative memory, theory of mind, cultural transmission, infrastructure, obligate symbiosis, epigenetics, senescence, coalitions, multidimensional niche, timescales, multiscale info, continental drift, geological LOD, institutions, language, consciousness (127 unit tests) | 2026-03-25 |
| **GAMEPLAY_SYSTEMS (parcial)** | GS-1 lockstep determinista, GS-3 Nash targeting, GS-5 victory nucleus (Onda 0 completa) | 2026-03-25 |
| **SIMULATION_FOUNDATIONS** | SF-4–SF-7: metrics export CSV/JSON, checkpoint save/load, wavefront propagation, integration replay (11 SF-7 tests) | 2026-03-25 |
| **SIMULATION_QUALITY** | SM-8A–G: magic numbers, change detection guards, inline math, god-system splits (containment+input), lifecycle docs, input SRP (grimoire → 3 sistemas) | 2026-03-25 |
| **CODE_QUALITY** | Q2 (named constants), Q3 (encapsulation), Q5 (plugin split), Q8 (color isolation) | 2026-03-25 |
| **STRUCTURE_MIGRATION** | SM-1–SM-7: worldgen split, sim subdirs, bridge split, archetypes split, bridge macro, constants consolidation, docs cleanup | 2026-03-25 |
| **SIMULATION_RELIABILITY** | R1–R9: units+conservation, determinism+replay, benchmarks, calibration, sensitivity, observability, morph robustness, surrogate error, CI gates | 2026-03-25 |
| **MACRO_STEPPING** | M1–M6: analytics equations, ECS routing, normalization, LOD observer, benchmark, BridgeCache audit | 2026-03-25 |
| **GAMEDEV_PATTERNS** | G5 (pathfinding), G8 (change detection), G9 (event ordering), G11 (strong IDs) | 2026-03-25 |
| **BLUEPRINT_V7** | V7-06 (materialization incremental), V7-07 (WorldgenPlugin), V7-14 (quantized color GPU) | 2026-03-25 |
| **ENERGY_COMPETITION** | EC-1–EC-8: pool equations → pool components → extraction registry → distribution → competition dynamics → conservation ledger → scale composition → integration demo | 2026-03-25 |
| **CODEBASE_AUDIT** | CA-1/2/3: build fix, DOD violations (Vec→bitmask, String→Cow, expect→let-else), 40 core_physics tests | 2026-03-25 |
| **ECOSYSTEM_AUTOPOIESIS** | EA1–EA8: spawn, stress, reproduction, phenology | — |
| **LIVING_ORGAN_INFERENCE** | LI + EET1: organ manifest, 12 roles, lifecycle cache | — |
| **EMERGENT_FLORA** | FL1–FL6: photosynthesis, nutrients, GF1 branching | — |
| **FLORA_ROSA** | RS1–RS2: rosa lifecycle demo | — |
| **ENERGY_PARTS_INFERENCE** | EPI1–EPI4: field → vertex pipeline | — |
| **ELEMENT_ALMANAC_CANON** | EAC1–EAC4: element tables, `find_stable_band*`, hz_identity_weight | — |
| **ECO_BOUNDARIES** | E1–E6: zone classification, climate biomes | — |
| **TOPOLOGY** | T1–T10: terrain generation (noise, slope, drainage, hydraulics, classifier, mutations) | — |
| **MIGRATION** | M1–M5: folder structure reorganization | — |
| **CHEMICAL_REFACTOR** | C1–C4: MatterLense, catalytic pipeline | — |
| **BRIDGE_OPTIMIZER** | B1–B10: BridgeCache<B>, 11 equation kinds | — |
| **BLUEPRINT_V6** | 2D/3D platform (runtime_platform/, 17 sub-modules) | — |
| **BLUEPRINT_V5** | Determinism + BridgeCache | — |
| **AXIOMATIC_CLOSURE** | AC-1–AC-5: metabolic interference (Axiom 3×8), Kuramoto entrainment (Axiom 8), culture coherence (Axiom 6×8), frequency attenuation (Axiom 7×8), cooperation emergence (Axiom 3 game theory). 60+ unit tests, 7 integration tests | 2026-03-25 |
| **BLUEPRINT_V4** | Layers L11–L13 (TensionField, Homeostasis, StructuralLink) | — |
