# Archived Sprints

Completed tracks with code in `src/`. Each folder keeps its original README as historical reference.

Live contracts: `docs/design/` (specs) and `docs/arquitectura/` (runtime blueprints per module).

## Index

| Track | Description | Closed |
|-------|-------------|--------|
| **MULTI_TELESCOPE** | MT-1–MT-5 ✅: Quantum-inspired hierarchical speculative execution (ADR-016). Collapse + re-emanation (not cascade). speculative_visibility (Englert D²+V²≤1), conservation-bounded projection (Axiom 4+5), frequency-aware decay (Axiom 8). TelescopeStack (8 levels, 16⁸ ≈ 4.3×10⁹ ticks reach). Earth Telescope Demo binary. 1 new file + 5 modified, 26 tests. | 2026-04-04 |
| **TEMPORAL_TELESCOPE** | TT-1–TT-10: Dual-timeline speculative execution (ADR-015). Ancla (ground truth) + Telescopio (analytical projection) + Calibration Bridge (feedback loop). 9 new files, 179 tests. Sliding statistics, Hurst DFA, Fisher information, projection normalizers, diff engine, cascade propagator, calibration bridge, dual pipeline, activation/dashboard wiring. Axioms 4/5/7 property-tested. 0 hardcoded values. neighbors_within_radius centralized in batch_stepping.rs. | 2026-04-04 |
| **REGULATORY_INFRASTRUCTURE** | RI-1/2/3: GitHub Actions CI (5 jobs: check, test, clippy, audit, fmt), branch protection, rust-toolchain.toml, Dependabot, document approval workflow (46 docs updated with approval fields), validation script, REVIEW_LOG.md, CCB charter, training matrix, 3 issue templates, quarterly review template. 8 ADRs in `docs/arquitectura/ADR/`. | 2026-04-02 |
| **REGULATORY_DOCUMENTATION** | RD-1 through RD-7: 43 regulatory documents + audit checklist (~15,400 lines) covering IEC 62304, ISO 14971, ISO 13485, ASME V&V 40, FDA CMS 2023, 21 CFR Part 11, IMDRF SaMD, GAMP 5. Safety Class A, IMDRF Category I. 50/50 external audit items mapped. 8 ADRs in `docs/arquitectura/ADR/`. Structural gaps → REGULATORY_INFRASTRUCTURE track. | 2026-04-02 |
| **DECOUPLING_AUDIT (parcial)** | DC-1/DC-3/DC-4: domain enum extraction (4 enums → blueprint/domain_enums.rs, 25+ imports migrados), state repatriation (WorldgenReady + transition_to_active_system), pure math boundary (terrain_blocks_vision → equations/vision.rs, AttentionGrid → contracts/, inline math extracted). Auditoría: 0.016 dt fix, dead code cleanup, element_bands.rs, DISSIPATION-derived symbiosis. 12 tests nuevos, 0 warnings. DC-2/DC-5 pendientes. | 2026-04-01 |
| **BRIDGE_STRATEGY_DECOUPLING (parcial)** | BS-2: CompetitionNormBridge wired (5 macros), hot reload phase reset, shape_cache_signature extraction. BS-3 parcial: exact_cache.rs (kleiber_volume_factor, exact_death_tick, frequency_alignment_exact), KleiberCache, GompertzCache, Converged\<T\> components. 52 tests. BS-1/BS-4–BS-7 pendientes. | 2026-03-30 |
| **SCIENTIFIC_OBSERVABILITY** | SO-1–SO-5: lineage (FNV-1a u64), census (alive_mask capture + HOF distribution/mean), CSV/JSON adapters (zero-alloc write), HOF orchestrators (ablate, ensemble, sweep), CSV wired to fermi/cancer/convergence binaries. 32 tests. | 2026-03-30 |
| **PROTO_DNA** | PD-1–5: CodonGenome, CodonTable (evolucionable), translation, silent mutations, neutral drift, batch wiring. 28 tests. | 2026-03-30 |
| **MULTICELLULARITY** | MC-1–5: cell adhesion, colony detection (Union-Find), positional signal, differential expression, batch wiring. 33 tests. | 2026-03-30 |
| **METABOLIC_GENOME** | MGN-1–7: gene→ExergyNode mapping, topology inference, graph from genome, evolution integration, node competition (Axiom 3), Hebbian rewiring (Hebb 1949), internal catalysis with frequency alignment (Axiom 8). 80 tests, 100% metabolic networks. | 2026-03-29 |
| **VARIABLE_GENOME** | VG-1–6: VariableGenome [4-32 genes], Kleiber maintenance cost, duplication/deletion mutation (Schwefel), expression mapping, epigenetic gating, GenomeBlob bridge, serialization. 62 tests. | 2026-03-29 |
| **VISUALIZATION_PIPELINE (parcial)** | VIS-1/2/4: DashboardBridgePlugin (SimTickSummary, SimTimeSeries, RingBuffer, ViewConfig, SimSpeedConfig), DashboardPanelsPlugin (3 tabs: Simulation/Parameters/Analysis, egui_plot time series, population/energy/species charts, speed control, color/camera modes). 11 tests. bevy_egui 0.31 + egui_plot 0.29. | 2026-03-30 |
| **LAB_UNIVERSAL** | Lab binary: 7 experiments (Universe Lab, Fermi, Speciation, Cambrian, Debate, Convergence, Cancer Therapy) with egui selector, shared params, domain-specific controls, result rendering (charts, grids, genomes tables). 436 LOC, zero src/ changes. | 2026-03-30 |
| **USE_CASE_BINARIES** | 14 binarios standalone: fermi, cancer_therapy, speciation, cambrian, debate, convergence, versus, fossil_record, mesh_export, museum, petri_dish, personal_universe, universe_lab, ecosystem_music. 12 experiment modules en use_cases/experiments/. Todos funcionales con CLI + CSV export. | 2026-03-30 |
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
