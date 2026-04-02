---
document_id: RD-3.1
title: Requirements Traceability Matrix
standard: IEC 62304:2006+Amd1:2015 §5.7.4, GAMP 5 2nd Ed.
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
---

# Requirements Traceability Matrix

## 1. Purpose

This document provides the complete bidirectional traceability matrix linking all 32 software requirements (from RD-1.3, Software Requirements Specification) to their architectural design elements, implementation modules, verification tests, and associated risk hazards. It satisfies IEC 62304:2006+Amd1:2015 §5.7.4 (Software system test record traceability) and GAMP 5 2nd Edition traceability requirements.

**Traceability chain:**

```
Requirement (RD-1.3 SRS)
  -> Design (docs/ARCHITECTURE.md, docs/design/, CLAUDE.md)
    -> Implementation (src/ modules)
      -> Verification (tests, grep audits, CLI validation)
        -> Risk (ISO 14971 hazards from RD-2.2)
```

**Cross-references:**
- RD-1.3: Software Requirements Specification (source of all 32 requirements)
- RD-1.4: Software Development Plan (verification strategy, §7)
- RD-2.1: Risk Management Plan (hazard identification framework)
- RD-2.2: Risk Analysis (hazard definitions H-01 through H-10)

## 2. Conventions

| Symbol | Meaning |
|--------|---------|
| RF-XX | Functional requirement |
| RP-XX | Performance requirement |
| RS-XX | Safety requirement |
| RI-XX | Interface requirement |
| RD-XX | Data requirement |
| H-XX | Hazard identifier (ISO 14971) |
| -> | Forward traceability (requirement to test) |
| <- | Backward traceability (test to requirement) |

All file paths are relative to the repository root. Test counts are verified against the codebase at the commit referenced in the document header.

## 3. Hazard Register (Summary)

The following hazards are referenced in the traceability matrix. Full analysis is in RD-2.2.

| ID | Hazard | Severity |
|----|--------|----------|
| H-01 | Overconfidence in resistance prediction leading to suboptimal therapy if used clinically | S5 Critical |
| H-02 | Energy conservation bug producing incorrect simulation results | S3 Moderate |
| H-03 | Calibration bias from limited profile set (4 profiles) | S3 Moderate |
| H-04 | Determinism failure producing non-reproducible results | S2 Minor |
| H-05 | SOUP dependency with unpatched CVE | S3 Moderate |
| H-06 | User misinterprets output as clinical prediction | S5 Critical |
| H-07 | Model lacks tumor microenvironment, results diverge from biology | S3 Moderate |
| H-08 | Incorrect drug inhibition math produces wrong suppression ratios | S4 Major |
| H-09 | Cross-platform floating-point drift breaks reproducibility | S2 Minor |
| H-10 | Build environment divergence produces non-equivalent binaries | S2 Minor |

---

## 4. Forward Traceability Matrix (Requirement -> Test)

### 4.1 Functional Requirements (RF-01 through RF-17)

#### RF-01: Emergent Life Simulation from Energy Interactions

| Aspect | Reference |
|--------|-----------|
| **Requirement** | All entity behavior derives from 8 axioms. Entities materialize when coherence gain > dissipation loss. Spawn threshold = 1/3 (Axiom 2). |
| **Design** | `docs/ARCHITECTURE.md` §Energy Cycle, §Axiomatic Abiogenesis; `CLAUDE.md` §The 8 Foundational Axioms |
| **Implementation** | `src/simulation/abiogenesis/` (axiomatic abiogenesis system) |
| | `src/blueprint/equations/abiogenesis/axiomatic.rs` (viability potential math) |
| | `src/blueprint/equations/derived_thresholds.rs` (spawn_potential_threshold = 1/3) |
| | `src/simulation/awakening.rs` (inert -> BehavioralAgent transition) |
| | `src/blueprint/equations/awakening.rs` (awakening potential math) |
| | `src/worldgen/` (field_grid, nucleus, propagation, materialization) |
| **Verification** | `src/blueprint/equations/derived_thresholds.rs` -- 17 unit tests |
| | `src/blueprint/equations/abiogenesis/axiomatic.rs` -- unit tests (viability potential, sigmoid bounds) |
| | `src/simulation/abiogenesis/constants.rs` -- threshold derivation tests |
| | `tests/property_conservation.rs` -- 19 proptest fuzz tests (conservation invariants) |
| | Map validation: `RESONANCE_MAP=genesis_validation cargo run` |
| **Risks** | H-02 (conservation bug), H-07 (model limitations) |

#### RF-02: 14-Layer ECS Composition

| Aspect | Reference |
|--------|-----------|
| **Requirement** | All entities defined by composition of 14 orthogonal layers. Each layer independent with its own update rule. Max 4 fields per component. |
| **Design** | `docs/ARCHITECTURE.md` §The 14 Orthogonal Layers; `CLAUDE.md` §The 14 Orthogonal Layers |
| **Implementation** | `src/layers/mod.rs` (50+ sub-modules, all layer components) |
| | `src/layers/energy.rs` (L0), `src/layers/volume.rs` (L1), `src/layers/oscillatory.rs` (L2), `src/layers/flow.rs` (L3), `src/layers/coherence.rs` (L4), `src/layers/engine.rs` (L5), `src/layers/pressure.rs` (L6), `src/layers/will.rs` (L7), `src/layers/injector.rs` (L8), `src/layers/identity.rs` (L9), `src/layers/link.rs` (L10), `src/layers/tension_field.rs` (L11), `src/layers/homeostasis.rs` (L12), `src/layers/structural_link.rs` (L13) |
| | `src/entities/archetypes/` (spawn functions compose layers) |
| **Verification** | Component registration verified via `app.register_type::<T>()` in plugins |
| | Entity spawn tests in `src/entities/archetypes/` modules |
| | Integration tests spawn minimal layer subsets, confirm independence |
| **Risks** | H-02 (incorrect layer interaction) |
| **Gap** | Layer orthogonality enforced by code review, not automated test |

#### RF-03: Drug-Pathway Interaction Modeling (Pathway Inhibitor)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Frequency-based binding (Axiom 8), Hill pharmacokinetics (n=2), 3 inhibition modes (Competitive, Noncompetitive, Uncompetitive). Off-target via frequency proximity. Bliss independence for combinations. |
| **Design** | `docs/ARCHITECTURE.md` §Drug Models Level 2; `CLAUDE.md` §Drug Models |
| **Implementation** | `src/blueprint/equations/pathway_inhibitor.rs` -- 11 public pure functions: `binding_affinity`, `hill_response`, `inhibitor_occupancy`, `inhibit_node`, `inhibit_pathway`, `combined_occupancy`, `selectivity_index`, `effective_node_params`, `coherence_disruption`, `apply_disruption_to_expression`, `find_escape_frequency` |
| | `src/blueprint/constants/pathway_inhibitor.rs` -- 7 derived constants |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` -- experiment orchestration |
| | `src/bin/pathway_inhibitor.rs` -- CLI binary |
| **Verification** | `src/blueprint/equations/pathway_inhibitor.rs` -- 42 unit tests |
| | `src/blueprint/constants/pathway_inhibitor.rs` -- 3 constant derivation tests |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` -- 31 integration tests |
| **Risks** | H-01 (overconfidence), H-03 (calibration bias), H-06 (misinterpretation), H-08 (math error) |

#### RF-04: Drug Resistance Evolution Modeling (Cytotoxic)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Cytotoxic drug therapy with frequency-selective cell killing. Quiescent stem cells escape chemo. Resistant subpopulations emerge through mutation. |
| **Design** | `docs/ARCHITECTURE.md` §Drug Models Level 1; `CLAUDE.md` §Drug Models |
| **Implementation** | `src/use_cases/experiments/cancer_therapy.rs` -- `hill_response`, `cytotoxic_drain`, `is_quiescent`, `is_drug_active`, `pk_ramp`, `run` |
| | `src/batch/` -- underlying batch simulator (33 systems, 6 phases) |
| | `src/bin/cancer_therapy.rs` -- CLI binary |
| **Verification** | `src/use_cases/experiments/cancer_therapy.rs` -- 24 unit tests |
| | Tests cover: Hill response (zero/full/half alignment, sigmoidal steepness), cytotoxic drain, PK ramp, drug scheduling, quiescent detection, determinism, conservation |
| **Risks** | H-01 (overconfidence), H-06 (misinterpretation), H-07 (no TME), H-08 (math error) |

#### RF-05: Bozic 2013 Reproduction (Combination > Monotherapy)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | 5-arm protocol reproducing Bozic et al. 2013 prediction. Combo > mono and combo > double-dose. 10/10 seeds confirm. |
| **Design** | `docs/paper/resonance_arxiv.tex` §Experiment 5; `docs/ARCHITECTURE.md` §Bozic Validation |
| **Implementation** | `src/use_cases/experiments/pathway_inhibitor_exp.rs` -- `bozic_five_arm_experiment`, `bozic_robustness_ten_seeds` |
| | `src/bin/bozic_validation.rs` -- standalone validation binary |
| **Verification** | `src/use_cases/experiments/pathway_inhibitor_exp.rs` -- Bozic-specific tests: `bozic_five_arm_combo_beats_mono`, `bozic_five_arm_combo_beats_double`, `bozic_ten_seeds_robust` |
| | CLI: `cargo run --release --bin bozic_validation` (~95 sec) |
| **Risks** | H-01 (overconfidence), H-03 (limited validation) |

#### RF-06: Clinical Calibration (4 Profiles)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | 4 calibration profiles mapping qe/Hz/gen to nM/days/cells. Qualitative only -- NOT validated against patient outcomes. |
| **Design** | `docs/ARCHITECTURE.md` §Drug Models; `CLAUDE.md` §Drug Models: Honest scope |
| **Implementation** | `src/blueprint/equations/clinical_calibration.rs` -- `CalibrationProfile`, 4 const profiles (`CML_IMATINIB`, `PROSTATE_ABIRATERONE`, `NSCLC_ERLOTINIB`, `CANINE_MAST_CELL`), 8 pure conversion functions |
| **Verification** | `src/blueprint/equations/clinical_calibration.rs` -- 21 unit tests |
| | Tests cover: unit conversion round-trips, profile constants positive, Rosie case prediction bounds |
| **Risks** | H-03 (calibration bias), H-06 (misinterpretation as clinical validation) |

#### RF-07: Bit-Exact Determinism

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Bit-exact identical output for identical initial state. Hash-based RNG (no `std::rand`). Canonical entity ordering. |
| **Design** | `docs/ARCHITECTURE.md` §Determinism; `CLAUDE.md` §Hard Blocks HB-3/HB-5 |
| **Implementation** | `src/blueprint/equations/determinism.rs` -- `hash_f32_slice`, `snapshot_hash`, `snapshots_match`, `next_u64`, `unit_f32`, `range_f32`, `gaussian_f32`, `gaussian_frequency_alignment`, `sanitize_unit` |
| **Verification** | `src/blueprint/equations/determinism.rs` -- 23 unit tests |
| | `src/use_cases/experiments/cancer_therapy.rs` -- `run_deterministic` test |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` -- determinism tests across seeds |
| **Risks** | H-04 (determinism failure), H-09 (cross-platform f32 drift) |

#### RF-08: Batch Simulation (Headless, Parallel)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Batch simulator with `repr(C)` flat arrays, rayon parallelism, 33 systems, 6 phases. GeneticHarness for evolution. GenomeBlob lossless round-trip. |
| **Design** | `docs/ARCHITECTURE.md` §Batch Simulator; `docs/arquitectura/blueprint_batch_simulator.md` |
| **Implementation** | `src/batch/arena.rs` -- `EntitySlot`, `SimWorldFlat` |
| | `src/batch/systems/mod.rs` -- 33 systems in 12 sub-modules |
| | `src/batch/genome.rs` -- `GenomeBlob`, mutate, crossover |
| | `src/batch/harness.rs` -- `GeneticHarness`, `FitnessReport` |
| | `src/batch/bridge.rs` -- `GenomeBlob` <-> Bevy component conversion |
| | `src/batch/lineage.rs` -- `LineageId`, `TrackedGenome` |
| | `src/batch/census.rs` -- `EntitySnapshot`, `PopulationCensus` |
| | `src/batch/batch.rs` -- `WorldBatch`, `BatchConfig`, rayon `par_iter_mut` |
| | `src/bin/headless_sim.rs` -- headless simulation binary |
| **Verification** | `src/batch/` -- 199 unit tests across 21 files |
| | Tests cover: arena slots (16), genome (14), harness (10), bridge round-trip (11), lineage (10), census (8), 33 batch systems (113) |
| | CLI: `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm` |
| **Risks** | H-02 (conservation), H-04 (determinism), H-05 (rayon SOUP) |

#### RF-09: Energy Conservation (Axiom 5)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Energy never created, only transferred/dissipated. Total qe monotonically decreases. Pool Invariant: children <= parent. Dissipation >= qe * rate. |
| **Design** | `CLAUDE.md` §Axioms 2, 4, 5; `docs/ARCHITECTURE.md` §Energy Cycle |
| **Implementation** | `src/simulation/metabolic/basal_drain.rs` -- passive energy drain |
| | `src/simulation/metabolic/senescence_death.rs` -- age-based mortality |
| | `src/simulation/reproduction/mod.rs` -- conservation-preserving offspring |
| | `src/worldgen/nucleus.rs` -- `NucleusReservoir` (finite fuel) |
| | `src/worldgen/systems/nucleus_recycling.rs` -- nutrient -> new nucleus cycle |
| | `src/blueprint/equations/derived_thresholds.rs` -- `basal_drain_rate()`, dissipation constants |
| **Verification** | `tests/property_conservation.rs` -- 19 proptest fuzz tests with arbitrary inputs |
| | Tests verify: pool invariant, dissipation non-negative, total qe monotonic decrease, reproduction conservation |
| | `src/use_cases/experiments/cancer_therapy.rs` -- `conservation_tracked` test |
| **Risks** | H-02 (conservation violation is direct failure of this requirement) |

#### RF-10: Morphogenesis from Energy Composition

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Shape emerges from energy, not templates. Constructal optimization. GF1 flow geometry. Organ count from energy state. |
| **Design** | `docs/ARCHITECTURE.md` §Morphogenesis Pipeline; `docs/design/TERRAIN_MESHER.md` |
| **Implementation** | `src/blueprint/equations/entity_shape.rs` -- `entity_geometry_influence`, `organ_slot_scale`, `optimal_appendage_count` |
| | `src/blueprint/morphogenesis/constructal.rs` -- `shape_cost`, `bounded_fineness_descent` |
| | `src/simulation/lifecycle/constructal_body_plan.rs` -- `constructal_body_plan_system` |
| | `src/simulation/lifecycle/entity_shape_inference.rs` -- compound mesh |
| | `src/geometry_flow/` -- GF1 flora-tube branching, `merge_meshes` |
| **Verification** | `src/blueprint/equations/entity_shape.rs` -- 39 unit tests |
| | Tests cover: fineness optimization convergence, organ slot bounds, appendage count vs drag, bilateral symmetry |
| **Risks** | H-02 (incorrect energy-shape derivation) |

#### RF-11: Particle Physics (Coulomb + Lennard-Jones)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Classical potentials with constants from 4 fundamentals. No bond tables. Frequency alignment modulates bond strength (Axiom 8). |
| **Design** | `docs/ARCHITECTURE.md` §Particle Physics; `CLAUDE.md` §Particle Physics |
| **Implementation** | `src/blueprint/equations/coulomb.rs` -- `coulomb_force`, `lennard_jones_force`, `net_force`, `bond_energy`, `is_bound`, `detect_bonds`, `classify_molecule`, `accumulate_forces` |
| | `src/batch/systems/particle_forces.rs` -- batch particle force accumulation |
| **Verification** | `src/blueprint/equations/coulomb.rs` -- 26 unit tests |
| | Tests cover: inverse-square law, LJ zero-crossing at `r = 2^(1/6)*sigma`, Newton 3rd law, charge conservation, bond energy sign, bit-exact determinism |
| **Risks** | H-02 (physics error), H-08 (math error in force computation) |

#### RF-12: Variable Genome (4--32 Genes)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Variable-length genomes [4,32]. Kleiber-scaled maintenance cost. Gene duplication/deletion from DISSIPATION_SOLID. Epigenetic gating. |
| **Design** | `docs/ARCHITECTURE.md` §Evolution & Emergence Pipeline |
| **Implementation** | `src/blueprint/equations/variable_genome.rs` -- `VariableGenome`, `genome_maintenance_cost`, `effective_bias`, `effective_biases`, `compute_phenotype`, duplication/deletion |
| **Verification** | `src/blueprint/equations/variable_genome.rs` -- 62 unit tests |
| | Tests cover: gene count bounds [4,32], duplication/deletion, Kleiber cost scaling, bias computation, genome distance, hash determinism, epigenetic gating |
| **Risks** | H-02 (incorrect cost scaling) |

#### RF-13: Genetic Code (64 Codons)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | 64 codons mapping to 8 amino acids. Point mutations. Silent mutation fraction. Table mutation. |
| **Design** | `docs/ARCHITECTURE.md` §Evolution & Emergence Pipeline |
| **Implementation** | `src/blueprint/equations/codon_genome.rs` -- `CodonGenome`, `CodonTable`, `mutate_codon`, `crossover_codon`, `translate_genome`, `classify_mutation`, `silent_mutation_fraction`, `mutate_table` |
| **Verification** | `src/blueprint/equations/codon_genome.rs` -- 28 unit tests |
| | Tests cover: genome construction, codon mutation, crossover, translation, silent vs non-silent, table mutation, hash determinism |
| **Risks** | H-02 (translation error) |

#### RF-14: Protein Folding (HP Lattice)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | 2D HP lattice. Greedy folding. Contact map. Function inference from topology and frequency. |
| **Design** | `docs/ARCHITECTURE.md` §Evolution & Emergence Pipeline |
| **Implementation** | `src/blueprint/equations/protein_fold.rs` -- `genome_to_polymer`, `fold_energy`, `fold_greedy`, `contact_map`, `contact_density`, `infer_protein_function`, `compute_protein_phenotype` |
| **Verification** | `src/blueprint/equations/protein_fold.rs` -- 27 unit tests |
| | Tests cover: polymer conversion, fold energy, greedy fold, contact map, contact density, function inference, full pipeline phenotype |
| **Risks** | H-07 (model abstraction: 2D HP vs real 3D protein folding) |

#### RF-15: Metabolic Networks (DAG)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Genes map to ExergyNode. Competitive flow distribution. Hebbian learning. Catalytic activation. |
| **Design** | `docs/ARCHITECTURE.md` §Evolution & Emergence Pipeline |
| **Implementation** | `src/blueprint/equations/metabolic_genome.rs` -- `gene_to_exergy_node`, `metabolic_graph_from_variable_genome`, `competitive_flow_distribution`, `hebbian_capacity_update`, `catalytic_activation_reduction` |
| | `src/blueprint/equations/metabolic_graph/mod.rs` -- metabolic graph types |
| **Verification** | `src/blueprint/equations/metabolic_genome.rs` -- 68 unit tests |
| | Tests cover: gene-to-node mapping, topology inference, competitive flow (conservation, efficiency ordering), Hebbian learning, catalysis, full pipeline phenotype |
| **Risks** | H-02 (flow conservation error), H-07 (model abstraction) |

#### RF-16: Multicellular Organization

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Cell adhesion via frequency/distance. Colony detection (Union-Find). Positional signaling. Differential expression. |
| **Design** | `docs/ARCHITECTURE.md` §Evolution & Emergence Pipeline |
| **Implementation** | `src/blueprint/equations/multicellular.rs` -- `adhesion_affinity`, `bond_strength`, `bond_cost`, `should_bond`, `detect_colonies`, `border_signal`, `positional_gradient`, `modulate_expression` |
| | `src/batch/systems/multicellular.rs` -- batch system |
| **Verification** | `src/blueprint/equations/multicellular.rs` -- 27 unit tests |
| | `src/batch/systems/multicellular.rs` -- 6 batch integration tests |
| | Tests cover: adhesion (distance decay, frequency decay, NaN safety), bond threshold, Union-Find correctness, border signal, positional gradient |
| **Risks** | H-02 (incorrect colony detection), H-07 (model abstraction) |

#### RF-17: Emergence Systems (9 Active)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | 9 registered emergence systems: theory of mind, cultural transmission, entrainment, infrastructure (2 systems), cooperation, symbiosis, niche adaptation, epigenetic adaptation. Each assigned to a specific Phase. |
| **Design** | `docs/ARCHITECTURE.md` §Evolution & Emergence Pipeline; `CLAUDE.md` §Emergence Systems |
| **Implementation** | `src/simulation/emergence/theory_of_mind.rs` (Phase::Input) |
| | `src/simulation/emergence/culture.rs` (Phase::Input) |
| | `src/simulation/emergence/entrainment.rs` (Phase::AtomicLayer) |
| | `src/simulation/emergence/infrastructure.rs` (Phase::MetabolicLayer, 2 systems) |
| | `src/simulation/emergence/cooperation.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/symbiosis_effect.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/niche_adaptation.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/epigenetic_adaptation.rs` (Phase::MorphologicalLayer) |
| **Verification** | Unit tests in each emergence module |
| | Integration tests via map presets (`RESONANCE_MAP=civilization_test cargo run`) |
| **Risks** | H-07 (emergence models are simplified abstractions) |
| **Gap** | 7 additional emergence systems implemented but not registered (coalition, institution, tectonic, multiscale, geological). Code-complete with zero runtime consumers. |

### 4.2 Performance Requirements (RP-01 through RP-03)

#### RP-01: Full Test Suite Execution Time

| Aspect | Reference |
|--------|-----------|
| **Requirement** | 3,113 tests in < 60 seconds |
| **Design** | `CLAUDE.md` §Testing; RD-1.4 §7.9 |
| **Implementation** | All `src/` test modules |
| **Verification** | `cargo test` -- measured 35.78s (0 failures, 1 ignored) |
| **Risks** | None (performance target, not safety) |

#### RP-02: Batch Benchmark Throughput

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Batch throughput maintained; no regression > 10% |
| **Design** | RD-1.4 §7.7 |
| **Implementation** | `src/batch/` (33 systems, rayon parallel) |
| **Verification** | `cargo bench --bench batch_benchmark` (criterion statistical analysis) |
| **Risks** | None (performance target) |
| **Gap** | No absolute throughput threshold defined; regression testing only |

#### RP-03: Headless Simulation (10K Ticks)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Complete 10,000 ticks, produce valid PPM, no GPU |
| **Design** | `docs/ARCHITECTURE.md` §Headless Simulation |
| **Implementation** | `src/bin/headless_sim.rs` |
| **Verification** | CLI: `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm` |
| **Risks** | H-10 (build environment divergence) |

### 4.3 Safety Requirements (RS-01 through RS-05)

#### RS-01: Zero `unsafe` in Runtime Simulation

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Zero `unsafe` blocks in `src/blueprint/`, `src/simulation/`, `src/layers/`, `src/entities/`, `src/plugins/` |
| **Design** | `CLAUDE.md` §Hard Blocks HB-1: "NO `unsafe` -- zero tolerance" |
| **Implementation** | Entire simulation codebase. 4 documented exceptions in non-simulation files (GPU layout, batch arena). |
| **Verification** | `grep -rn "unsafe" src/ --include="*.rs"` -- automated audit |
| **Risks** | H-02 (memory safety violation -> incorrect results) |

#### RS-02: Zero Shared Mutable State

| Aspect | Reference |
|--------|-----------|
| **Requirement** | No `Arc<Mutex<T>>`, `static mut`, `lazy_static! { Mutex }`. State via Bevy `Resource`/`Local` only. |
| **Design** | `CLAUDE.md` §Hard Blocks HB-4, HB-5 |
| **Implementation** | All `src/` modules |
| **Verification** | Automated grep in CI. Rust 2024 edition compiler warnings for `static mut`. |
| **Risks** | H-04 (non-determinism from race condition) |

#### RS-03: Deterministic Output

| Aspect | Reference |
|--------|-----------|
| **Requirement** | No `std::rand`, no wall-clock time, no HashMap iteration order in simulation logic. PCG-like RNG. |
| **Design** | `CLAUDE.md` §Hard Blocks HB-3; `docs/ARCHITECTURE.md` §Determinism |
| **Implementation** | `src/blueprint/equations/determinism.rs` -- all RNG functions |
| **Verification** | `src/blueprint/equations/determinism.rs` -- 23 unit tests |
| | `src/use_cases/experiments/cancer_therapy.rs` -- `run_deterministic` test |
| | Batch simulator determinism tests in `src/batch/` modules |
| **Risks** | H-04 (determinism failure), H-09 (cross-platform f32 drift) |

#### RS-04: Visible Disclaimers

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Disclaimers in README.md, paper, source code, experiment output stating NOT a clinical tool. |
| **Design** | RD-1.1 §4.2, §6.4 (misuse mitigation controls) |
| **Implementation** | `README.md` lines 18-22 ("What It Is NOT") |
| | `CLAUDE.md` §Drug Models: Honest scope |
| | `docs/paper/resonance_arxiv.tex` §5 Limitations |
| | `src/blueprint/equations/clinical_calibration.rs` (DISCLAIMER on Rosie case) |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` ("DISCLAIMER: SIMULATED. NOT VETERINARY ADVICE.") |
| **Verification** | Manual audit of 7 disclaimer locations. Automated grep for "NOT" + "clinical" keywords. |
| **Risks** | H-01 (overconfidence), H-06 (misinterpretation) |
| **Gap** | Disclaimer presence verified by manual audit, not automated CI gate |

#### RS-05: No Patient Data Processing

| Aspect | Reference |
|--------|-----------|
| **Requirement** | No patient data accepted, processed, stored, or transmitted. No DICOM, HL7, FHIR. No network. |
| **Design** | RD-1.1 §3 (Intended Use Environment); `CLAUDE.md` §Stack: "Async: None" |
| **Implementation** | Entire codebase. No medical data format crates. No network crates (no tokio, reqwest, hyper). |
| **Verification** | `Cargo.toml` audit -- no medical data format or network crates |
| | Source audit -- no network or database code |
| **Risks** | H-01 (boundary violation if patient data added), H-06 (misinterpretation) |

### 4.4 Interface Requirements (RI-01 through RI-03)

#### RI-01: Command-Line Interface (Primary)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | CLI is primary interface. 25 binaries. Shared CLI utilities. |
| **Implementation** | `src/bin/*.rs` (25 binaries); `src/use_cases/cli.rs` (shared utilities) |
| **Verification** | Binary compilation via `cargo build`. CLI execution of key binaries (headless_sim, bozic_validation, cancer_therapy). |
| **Risks** | None (interface, not safety) |

#### RI-02: Bevy Rendering (Optional)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Optional 2D/3D rendering. Dashboard bridge for real-time monitoring. |
| **Implementation** | `src/rendering/quantized_color/` -- QuantizedColorPlugin |
| | `src/runtime_platform/dashboard_bridge.rs` -- DashboardBridgePlugin |
| | `src/runtime_platform/` -- 17 sub-modules |
| **Verification** | Visual inspection via `cargo run` with map presets |
| **Risks** | None (rendering is non-simulation; excluded from risk scope per RD-2.1 §2.3) |

#### RI-03: Map Configuration Interface

| Aspect | Reference |
|--------|-----------|
| **Requirement** | `.ron` map files. `RESONANCE_MAP` env var selects map. |
| **Implementation** | `src/worldgen/map_config.rs`; `assets/maps/*.ron` (25 maps) |
| **Verification** | RON deserialization at startup. Invalid maps produce load errors. |
| **Risks** | H-10 (configuration drift between environments) |

### 4.5 Data Requirements (RD-01 through RD-04)

#### RD-01: Input Data -- Map Configurations (.ron)

| Aspect | Reference |
|--------|-----------|
| **Requirement** | RON format. 25 map files. Validated at startup. |
| **Implementation** | `assets/maps/*.ron` |
| **Verification** | RON deserialization. Invalid maps produce Bevy asset load errors. |
| **Risks** | H-10 (map configuration not version-controlled properly) |

#### RD-02: Output Data -- PPM Images

| Aspect | Reference |
|--------|-----------|
| **Requirement** | PPM P6 binary format. Dimensions = grid_size * scale. |
| **Implementation** | `src/bin/headless_sim.rs` -- PPM writer |
| **Verification** | CLI execution + output file header validation |
| **Risks** | None (output format, not safety) |

#### RD-03: Output Data -- CSV/JSON Export

| Aspect | Reference |
|--------|-----------|
| **Requirement** | Stateless adapter functions. No file I/O in export functions. |
| **Implementation** | `src/use_cases/export.rs` -- 6 export functions |
| **Verification** | `src/use_cases/export.rs` -- 9 unit tests |
| **Risks** | None (output format) |

#### RD-04: Internal Data -- Genome Serialization

| Aspect | Reference |
|--------|-----------|
| **Requirement** | `GenomeBlob` binary serialization. Lossless round-trip batch <-> Bevy. |
| **Implementation** | `src/batch/genome.rs`; `src/batch/bridge.rs` |
| **Verification** | `src/batch/bridge.rs` -- 11 round-trip tests |
| **Risks** | H-04 (serialization error breaks determinism) |

---

## 5. Backward Traceability Matrix (Test -> Requirement)

This section maps key test files back to the requirements they verify, demonstrating that every significant test cluster traces to at least one requirement.

| Test File / Suite | Test Count | Requirements Verified |
|-------------------|------------|----------------------|
| `src/blueprint/equations/derived_thresholds.rs` | 17 | RF-01, RF-09 |
| `src/blueprint/equations/pathway_inhibitor.rs` | 41 | RF-03, RF-05 |
| `src/blueprint/constants/pathway_inhibitor.rs` | 3 | RF-03 |
| `src/use_cases/experiments/pathway_inhibitor_exp.rs` | 31 | RF-03, RF-04, RF-05 |
| `src/use_cases/experiments/cancer_therapy.rs` | 24 | RF-04, RF-07, RF-09, RS-03 |
| `src/blueprint/equations/determinism.rs` | 23 | RF-07, RS-03 |
| `src/blueprint/equations/clinical_calibration.rs` | 21 | RF-06 |
| `src/blueprint/equations/entity_shape.rs` | 39 | RF-10 |
| `src/blueprint/equations/coulomb.rs` | 26 | RF-11 |
| `src/blueprint/equations/variable_genome.rs` | 62 | RF-12 |
| `src/blueprint/equations/codon_genome.rs` | 28 | RF-13 |
| `src/blueprint/equations/protein_fold.rs` | 27 | RF-14 |
| `src/blueprint/equations/metabolic_genome.rs` | 68 | RF-15 |
| `src/blueprint/equations/multicellular.rs` | 27 | RF-16 |
| `src/batch/systems/multicellular.rs` | 6 | RF-16 |
| `tests/property_conservation.rs` | 19 | RF-01, RF-09, RS-03 |
| `src/batch/` (all modules) | 199 | RF-08, RF-09 |
| `src/batch/bridge.rs` | 11 | RD-04 |
| `src/use_cases/export.rs` | 9 | RD-03 |
| `src/blueprint/equations/abiogenesis/axiomatic.rs` | (module tests) | RF-01 |
| `src/simulation/emergence/` (all modules) | (per-module tests) | RF-17 |
| `cargo test` (full suite) | 3,113 | RP-01 |
| `cargo bench --bench batch_benchmark` | (criterion) | RP-02 |
| `grep` audits (unsafe, Arc<Mutex>, static mut) | (automated) | RS-01, RS-02 |
| Manual disclaimer audit | (7 locations) | RS-04 |
| `Cargo.toml` dependency audit | (structural) | RS-05 |

---

## 6. Risk-to-Requirement Backward Traceability

This section maps each hazard to the requirements whose verification mitigates it.

| Hazard | Mitigating Requirements | Key Controls |
|--------|------------------------|--------------|
| H-01: Overconfidence in resistance prediction | RS-04, RS-05, RF-06 | Disclaimers (7 locations), abstract qe units, no patient data |
| H-02: Energy conservation bug | RF-09, RS-01, RF-01 | 19 proptest fuzz tests, zero `unsafe`, 17 derived threshold tests |
| H-03: Calibration bias | RF-05, RF-06 | Bozic 10-seed validation, 4 independent calibration profiles, honest scope documentation |
| H-04: Determinism failure | RF-07, RS-02, RS-03 | 23 determinism tests, zero shared mutable state, hash-based RNG |
| H-05: SOUP vulnerability | RS-05 | `Cargo.lock` pins, SOUP analysis (RD-3.2), `cargo audit` monitoring |
| H-06: Output misinterpretation | RS-04, RF-06, RS-05 | Disclaimers, abstract units, no clinical output format |
| H-07: Model lacks TME | RF-06, RS-04 | Paper §5 Limitations, README honest scope, calibration caveats |
| H-08: Incorrect drug math | RF-03, RF-04, RF-05 | 41 pathway inhibitor tests, 24 cancer therapy tests, Bozic 5-arm validation |
| H-09: Cross-platform f32 drift | RF-07, RS-03 | `f32::to_bits()` hashing, determinism test suite, `sanitize_unit` |
| H-10: Build environment divergence | RP-03, RD-01 | `Cargo.lock` deterministic builds, version-controlled maps, headless CLI validation |

---

## 7. Coverage Summary

### 7.1 Requirements Coverage

| Category | Count | All Traced to Tests? | All Traced to Design? | All Traced to Risks? |
|----------|-------|----------------------|-----------------------|----------------------|
| Functional (RF-01 -- RF-17) | 17 | Yes (700+ dedicated tests) | Yes | Yes |
| Performance (RP-01 -- RP-03) | 3 | Yes (suite + benchmarks) | Yes | Partial (RP-01/02 no risk) |
| Safety (RS-01 -- RS-05) | 5 | Yes (automated + manual) | Yes | Yes |
| Interface (RI-01 -- RI-03) | 3 | Partial (RI-02 visual only) | Yes | Partial (no safety risk) |
| Data (RD-01 -- RD-04) | 4 | Yes (deserialization + round-trip) | Yes | Partial |
| **Total** | **32** | **32/32 verified** | **32/32 traced** | **26/32 risk-linked** |

### 7.2 Unlinked Requirements (No Direct Risk)

The following 6 requirements have no direct hazard linkage because they address performance, interface, or output format concerns that do not contribute to hazardous situations under the current intended use:

- RP-01 (test execution time)
- RP-02 (batch benchmark)
- RI-01 (CLI interface)
- RI-02 (Bevy rendering)
- RD-02 (PPM output)
- RD-03 (CSV/JSON export)

### 7.3 Identified Gaps

| Gap | Severity | Requirement | Mitigation |
|-----|----------|-------------|------------|
| Layer orthogonality not automated | Low | RF-02 | Enforced by code review; structural guarantee from ECS architecture |
| 7 emergence systems unregistered | Low | RF-17 | Documented as implemented-not-wired; zero runtime consumers |
| No absolute batch throughput threshold | Low | RP-02 | Regression testing only; absolute threshold deferred |
| Disclaimer presence not CI-gated | Medium | RS-04 | Manual audit; recommended to add `grep` CI check |
| Cross-platform determinism untested | Medium | RF-07, RS-03 | Same-platform determinism verified; cross-platform testing deferred to RD-4 |

---

## 8. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial traceability matrix. All 32 requirements from RD-1.3 traced forward (req -> design -> impl -> test -> risk) and backward (test -> req, risk -> req). 5 gaps documented. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Planificador | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Verificador | _pending_ | _pending_ | _pending_ |
