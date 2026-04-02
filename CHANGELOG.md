# Changelog

## [Unreleased] — 2026-04-02

### Added — Regulatory Documentation Track (RD-1 through RD-7)
- **43 regulatory documents** + 1 master audit checklist in `docs/regulatory/` (~15,400 lines)
- **RD-1 Foundation (8 docs)**: Intended Use (IMDRF N10/N12), Safety Class A (IEC 62304), SRS (32 requirements), Dev Plan, Regulatory Strategy, Medical Device File, Maintenance Plan, Problem Resolution
- **RD-2 Risk Management (7 docs)**: ISO 14971:2019 complete — Plan, Analysis (12 hazards), Evaluation (5×5 matrix), Controls (52 measures), Residual Risk, Report, Post-Production Monitoring
- **RD-3 Traceability (4 docs)**: Traceability Matrix (32 reqs → code → tests → risks), SOUP Analysis (15 runtime deps), SBOM (NTIA-compliant), Configuration Management
- **RD-4 Validation (6 docs)**: Validation Plan, ASME V&V 40 Credibility Model (§4-8), Verification Report (3,113 tests), Validation Report (Bozic + Rosie + 4 experiments), Uncertainty Analysis, User Requirements Spec (GAMP 5)
- **RD-5 Quality System (8 docs)**: ISO 13485 QMS — Quality Manual, Policy, Document Control, Record Control, Internal Audit, Nonconforming Product, CAPA (3 real CAPAs), Competence Records
- **RD-6 Clinical (5 docs)**: Clinical Evaluation Plan/Report (IMDRF N41), Limitations Report (CAN/CANNOT/ASSUMPTIONS), Reproducibility Protocol (copy-paste commands), Reference Data Registry (7 datasets with DOIs)
- **RD-7 Release (5 docs)**: 21 CFR Part 11 Compliance (§11.10 a-k), ALCOA+ Data Integrity, Audit Trail, Cybersecurity Plan (STRIDE), Release Package (12 go/no-go criteria)
- Standards covered: IEC 62304, ISO 14971, ISO 13485, ASME V&V 40, FDA CMS Guidance 2023, 21 CFR Part 11, IMDRF SaMD, GAMP 5
- All 50 items from external audit checklist mapped to documents (42 direct, 8 distributed)
- 8 structural gaps documented honestly (signatures, CI/CD, GPG, CCB, training certs, monitoring)

### Fixed
- README.md test count: 3,095 → **3,113** (actual `cargo test` output)
- pathway_inhibitor.rs test count in SRS/traceability: 41 �� **42** (verified by `#[test]` count)
- INTENDED_USE.md: added IMDRF N12R2:2014 reference for risk categorization (was N10 only)

### Stats
- Tests: 3,070 → **3,113** (+43)
- LOC: ~110K → **~113K**
- Regulatory docs: 0 → **44** (43 documents + 1 index)

---

## [Unreleased] — 2026-03-31

### Added — Drug Design + Adaptive Therapy (PI-1–10)
- **Pathway inhibitor system**: 14 pure functions, 42 tests. Competitive/Noncompetitive/Uncompetitive inhibition. Hill dose-response. Off-target effects. Bliss independence for combinations.
- **Destructive interference**: coherence disruption via opposing frequencies. Escape frequency prediction.
- **Adaptive therapy controller**: feedback loop (profile → attack → predict escape → close → adapt). 5 decision states. Protocol output.
- **Bozic 2013 validation**: 5-arm experiment. Combo (56.5%) > double dose (53.4%). 10/10 seeds, p < 0.001.
- **Multi-seed robustness**: all experiments validated across 10 independent seeds.
- 3 new binaries: `pathway_inhibitor`, `bozic_validation`, `adaptive_therapy`.

### Added — Codebase Audit + Doc Honesty Pass
- `docs/ARCHITECTURE.md`: centralized canonical documentation.
- Derivation corrections: `SelfSustainingQeMin = DENSITY_SCALE`, `spawn_threshold = 1/3` from Axiom 2.
- Abiogenesis docs: "NOT Prigogine" explicit. Radiation pressure: "stellar analogy" disclaimer.
- 9 emergence systems marked ACTIVE, 7 marked NOT REGISTERED, 4 metabolic marked test-only.

### Fixed
- Duplicate `worldgen/materialization/` removed (identical copy).
- `pressure_frequency_alignment` → delegates to canonical `gaussian_frequency_alignment`.
- `organ_role_dimension` extracted to `metabolic_genome.rs` (was duplicated).
- `dimension_base_frequency` aligned with canonical `DIM_BASE_FREQ`.
- Worldgen hardcodes → concordance tests against `derived_thresholds`.
- Duplicate `#[test]` attribute removed. Unused imports cleaned.

### Removed
- 46 unreferenced docs from `design/` and `arquitectura/`.
- `docs/guides/`, `docs/ci/`, `docs/extraInfo/`.

### Stats
- Tests: 2,994 → **3,070** (+76)
- LOC: ~109K → **~110K**
- Binaries: 22 → **25** (+3: pathway_inhibitor, bozic_validation, adaptive_therapy)

---

### Added — Scientific Observability Pipeline (SO-1–SO-5)
- `batch/lineage.rs`: LineageId + TrackedGenome (deterministic FNV-1a ancestry, 10 tests)
- `batch/census.rs`: EntitySnapshot + PopulationCensus (per-gen capture, HOF distribution/mean, 8 tests)
- `use_cases/export.rs`: CSV/JSON stateless adapters (zero-alloc write_*_csv, 9 tests)
- `use_cases/orchestrators.rs`: ablate(closure), ensemble(), sweep(2 closures), aggregate_ensemble (5 tests)
- CSV export wired to `fermi.rs`, `cancer_therapy.rs`, `convergence.rs` (--out flag)

### Added — Exact Cache Components (zero precision loss)
- `blueprint/equations/exact_cache.rs`: kleiber_volume_factor, exact_death_tick, frequency_alignment_exact (23 tests)
- `layers/kleiber_cache.rs`: KleiberCache SparseSet (precompute radius^0.75, 8 tests)
- `layers/gompertz_cache.rs`: GompertzCache SparseSet (precompute death tick, 8 tests)
- `layers/converged.rs`: Converged<T> generic convergence flag (7 tests)

### Added — Dashboard Bridge + Visual Panels (VIS-1/2/4)
- `runtime_platform/dashboard_bridge.rs`: SimTickSummary, SimTimeSeries, RingBuffer, ViewConfig, SimSpeedConfig, DashboardBridgePlugin (11 tests)
- `runtime_platform/dashboard_panels.rs`: DashboardPanelsPlugin (3 egui systems: top_bar, controls, charts)
- bevy_egui 0.31 + egui_plot 0.29 added to Cargo.toml

### Added — Survival Mode (SV-2 + SV-3)
- `bin/survival.rs`: genome load/evolve, arena spawn, WASD control, score, death detection, game over UI, restart. Zero src/ changes.

### Added — Lab Universal + Live 2D
- `bin/lab.rs`: 8 experiments (Universe Lab, Fermi, Speciation, Cambrian, Debate, Convergence, Cancer Therapy, Live 2D Sim)
- Ablation mode (8 steps), Ensemble mode (10 seeds), CSV export button
- Live 2D: batch SimWorldFlat step-by-step with egui::Painter (nutrient heatmap + entity circles + velocity vectors)

### Fixed — Bridge Optimizer Bugs (BS-2)
- CompetitionNormBridge wired in all 5 lifecycle macros (context_fill, metrics)
- Hot reload resets BridgePhaseState to Warmup when not in Active
- shape_cache_signature_with_surface extracted from inline to equations/ (6 tests)

### Fixed — Hexagonal Boundary Cleanup
- bevy::color::Color removed from equations/field_color/ (3 files) → pure srgb_to_linear()
- bevy::prelude::Reflect removed from equations/energy_competition/{extraction,scale}.rs
- 7 DEBT markers added for remaining justified Bevy imports in equations/
- DEFAULT_FINENESS duplication eliminated (2 files → constants::FINENESS_DEFAULT)

### Added — PC-1: Particle Charge Physics (Coulomb + LJ)
- `blueprint/equations/coulomb.rs`: ChargedParticle, Coulomb force, Lennard-Jones, bond detection, molecule classification (26 tests)
- `blueprint/constants/particle_charge.rs`: all constants derived from fundamentals (COULOMB_SCALE = 1/DENSITY_SCALE, LJ_EPSILON = DISSIPATION_SOLID × 100)
- `batch/systems/particle_forces.rs`: particle_forces, detect_particle_bonds, count_molecules (9 tests)
- ForceStrategy enum (Disabled/CoulombOnly/Full, default=Disabled for backward compat)
- `use_cases/experiments/particle_lab.rs`: lightweight experiment (no batch overhead), 6 tests
- `bin/particle_lab`: emergent molecules from charged particles in 3ms

### Added — CT-1: Cancer Therapy Simulation
- `use_cases/experiments/cancer_therapy.rs`: Hill dose-response, PK ramp, stem cell quiescence, microenvironment (21 tests)
- Drug = dissipation increase (Axiom 4 pure), cancer trophic_class=4 (Warburg detritivore)
- Calibrated against Bozic 2013 (eLife): monotherapy resistance dynamics reproduced

### Fixed — Codebase Audit (2026-03-30)
- **COHERENCE_BANDWIDTH centralized**: 6 duplicate definitions (50.0 Hz) → single source in `derived_thresholds.rs`
- **Archetype dissipations derived from fundamentals**: catalog.rs magic numbers → algebraic expressions from DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}
- **Duplicate SOLAR_FLUX_BASE removed**: batch/systems/thermodynamic.rs shadowed batch/constants.rs
- **Hardcoded pack size extracted**: trophic.rs `8` → `PACK_TRAVERSE_MAX_SIZE`
- **Symbiosis rates named**: 4 inline magic numbers → `MUTUALISM_INTAKE_FRACTION`, `PARASITISM_TRANSFER_LOSS`, etc.
- **Terrain costs named**: physics.rs inline floats → `LIQUID_WATER_CHANNEL_BONUS`, `SOLID_CLIFF_PENALTY`, etc.
- **Doc comments added**: theory_of_mind, reactions constants documented bilingually

### Added — VG-1–6: Variable-Length Genome
- `VariableGenome` (4-32 genes) in `blueprint/equations/variable_genome.rs` (62 tests)
- Gene duplication/deletion (Schwefel self-adaptive). Kleiber maintenance cost.
- Epigenetic gating. Expression mapping. Serialization. GenomePhenotype cache.
- Genome side-table in `batch/arena.rs`. Reproduction propagates variable genomes.

### Added — MGN-1–7: Metabolic Genome (gene → metabolic network)
- `metabolic_genome.rs`: gene→ExergyNode, topology inference (DAG), graph from genome (80 tests)
- Node competition (Axiom 3), Hebbian rewiring, internal catalysis (Axiom 8)
- Batch wiring: `metabolic_graph_infer` + `protein_fold_infer` in pipeline
- Bevy wiring: `genome_to_metabolic_graph_system` + `genome_maintenance_drain_system`

### Added — PF-1–5: Protein Fold (lattice HP model)
- `protein_fold.rs`: polymer chain, greedy lattice fold, contact map, function inference (27 tests)
- Proto-proteins with active sites. Catalytic function from fold geometry.

### Added — PD-1–5: Proto-DNA (codon-based genome)
- `codon_genome.rs`: CodonGenome (tripletes), CodonTable (64→8 amino, evolucionable) (28 tests)
- Translation pipeline: codons → amino acids → Monomer chain
- Silent mutations + neutral drift. Code table evolves by selection.
- Batch wiring: codon side-tables, reproduction propagates codons.

### Added — MC-1–5: Multicellularity
- `multicellular.rs`: cell adhesion, colony detection (Union-Find), positional signal (27 tests)
- Differential expression: border=defense, interior=metabolism. Specialization emerges.
- Batch wiring: `multicellular_step` in pipeline. Observability: multicellular_rate.

### Added — 13 Use Cases + 14 Binaries
- fermi, speciation, cambrian, debate, versus, universe_lab, museum, fossil_record, petri_dish, ecosystem_music, mesh_export, personal_universe, convergence
- Shared CLI utilities: `use_cases/cli.rs` (parse_arg, archetype_label, resolve_preset)

### Fixed — Layer Violations
- MobaIdentity: Vec<RelationalTag> → u8 bitfield
- Grimoire: Vec<AbilitySlot> → [Option<AbilitySlot>; 8]
- AlchemicalForge: Vec<ElementId> → [ElementId; 4]

### Fixed — SV-1: Input Wiring
- `apply_input()` in `sim_world.rs`: InputCommand → WillActuator via WorldEntityId

### Changed — Observability
- GenerationStats: +gene_count_mean, metabolic_graph_rate, protein_function_rate, codon_count_mean, multicellular_rate
- `cargo run --bin evolve` shows full complexity dashboard

### Changed — Centralized Utilities
- `gaussian_frequency_alignment()` in `determinism.rs` (was duplicated 3×)
- `sanitize_unit()` in `determinism.rs` (universal NaN/Inf guard)

### Stats
- Tests: 2,834 → **2,994** (+160)
- LOC: ~87K → **~109K** (+22K)
- Binaries: 18 → **22** (+4: cancer_therapy, particle_lab, evolve_and_view, survival)
- Hardcoded constants eliminated: **13**
- Duplicate definitions removed: **7**

---

## [0.1.0] — 2026-03-25

### Added — AC-1–AC-5: Axiomatic Closure (cross-axiom dynamics)
- **AC-1**: `metabolic_interference_factor` / `apply_metabolic_interference` in `blueprint/equations/energy_competition/metabolic_interference.rs` (15 tests). `trophic_predation_attempt_system` scales assimilation by oscillatory alignment between predator and prey (Axiom 3×8)
- **AC-2**: Kuramoto entrainment in `blueprint/equations/emergence/entrainment.rs` (12 tests) + `simulation/emergence/entrainment.rs` system (4 integration tests). Distance-weighted frequency alignment via `Phase::AtomicLayer`
- **AC-3**: `frequency_imitation_affinity` + `should_imitate_with_affinity` in `blueprint/equations/emergence/culture.rs` (12 tests). `cultural_transmission_system` gates imitation by oscillatory affinity (Axiom 6×8)
- **AC-4**: `frequency_purity_at_distance` + `entrainment_coupling_at_distance` in `blueprint/equations/signal_propagation.rs` (11 tests). Exponential spatial decay for entrainment coupling (Axiom 7×8)
- **AC-5**: `cooperation_is_beneficial` + `extraction_estimate_in_group` + `defection_temptation` in `blueprint/equations/emergence/symbiosis.rs` (10 tests). `cooperation_evaluation_system` in `simulation/cooperation.rs` (3 integration tests). Nash-stable alliance detection with AC-1 interference cost
- **Events**: `AllianceProposedEvent`, `AllianceDefectEvent` in `events.rs`, registered in bootstrap
- **Constants**: `METABOLIC_INTERFERENCE_FLOOR`, `KURAMOTO_BASE_COUPLING`, `ENTRAINMENT_SCAN_RADIUS`, `COOPERATION_GROUP_BONUS`, `COOPERATION_DEFECT_THRESHOLD`, `CULTURE_COHERENCE_IMITATION_BONUS_CAP`
- **Plugins**: `AtomicPlugin` registers entrainment after spatial index; `MetabolicPlugin` registers cooperation after trophic
- **Test count**: 1721 → 2150

### Added — Sprint MG-8: Morphogenesis Integration & Demo
- **EntityBuilder**: `with_organ_manifest()`, `with_metabolic_graph_inferred()`, `with_metabolic_graph()`, `irradiance()` methods for metabolic graph composition
- **Archetypes**: `spawn_aquatic_organism`, `spawn_desert_plant`, `spawn_desert_creature`, `spawn_forest_plant` with `MorphogenesisSpawnPreset` constants
- **Map**: `assets/maps/morphogenesis_demo.ron` — 3 biomes (ocean, desert, forest) for phenotype emergence
- **Benchmark**: `benches/morphogenesis_benchmark.rs` — 100 entities, 12-node DAG, 6 MG systems
- **Tests**: 25 new tests covering builder API, archetype spawn, phenotype convergence (fineness, albedo, rugosity), legacy entity regression

### Added — Rosa Lifecycle Demo
- **Default map** optimized for single rosa simulation (Terra + Lux nuclei, cell_size=0.5)
- **demo_level.rs**: Rosa lifecycle startup with `Materialized`, `EnergyVisual`, `LifecycleStageCache`, `QuantizedPrecision`, `InferenceProfile`, camera close-up
- **Systems**: `enforce_rosa_focus_system` (hides non-rosa entities), `stabilize_rosa_energy_system` (energy floor), `stabilize_rosa_growth_system` (LOD + mesh rebuild), `pin_rosa_lod_focus_system` (LOD anchor)
- **Debug telemetry**: `debug_rosa_inference_chain_system` — prints lifecycle stage, biomass, precision, capabilities every 2s

### Removed — Legacy Demo Cleanup
- **Maps deleted**: `demo_arena.ron`, `demo_floor.ron`, `demo_minimal.ron`, `demo_strata.ron`, `demo_river_plateau.ron`, `layer_ladder.ron`, `proving_grounds.ron`, `four_flowers.ron`
- **Code deleted**: `src/world/demos/layer_ladder.rs` and all references
- **Root docs deleted**: `DESIGNING.md`, `PLANT_SIMULATION.md`, `TOPOLOGY_AND_LAYERS.md` (content lives in `docs/`)

### Changed
- **demo_level.rs**: Simplified from 4-plant sandbox to single rosa lifecycle
- **debug_plugin.rs**: Removed layer_ladder dispatch, added rosa lifecycle systems (LOD pin, visibility filter, energy stabilizer)
- **default.ron**: Replaced 3-nucleus arena with 2-nucleus rosa garden (Terra + Lux, cell_size=0.5)
- Camera controls 70% slower for flora-scale navigation
