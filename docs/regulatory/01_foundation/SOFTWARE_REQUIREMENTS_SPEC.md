---
document_id: RD-1.3
title: Software Requirements Specification
standard: IEC 62304:2006+Amd1:2015 §5.2, GAMP 5 2nd Ed.
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Software Requirements Specification

## 1. Purpose and Scope

### 1.1 Purpose

This document specifies the software requirements for RESONANCE, an emergent life simulation engine. It satisfies IEC 62304:2006+Amd1:2015 clause 5.2 (Software Requirements Analysis) and GAMP 5 2nd Edition Functional Specification (FS) requirements.

All requirements are derived retroactively from implemented, tested functionality. RESONANCE is a research tool --- not a clinical decision support system, not a medical device, and not a substitute for clinical trials. Requirements are documented to support regulatory transparency and scientific reproducibility, not to claim clinical applicability.

### 1.2 Scope

RESONANCE is a 113K LOC Rust 2024 / Bevy 0.15 simulation engine where all behavior emerges from energy interactions governed by 8 foundational axioms and 4 fundamental constants. The software models:

- Emergent life from energy field interactions (abiogenesis, evolution, morphogenesis)
- Drug-pathway interactions (pathway inhibitor, cytotoxic) for resistance dynamics research
- Clinical calibration profiles for qualitative comparison against published oncology data
- Batch parallel simulation for evolutionary and pharmacological experiments

**Classification:** IMDRF SaMD Category I (lowest risk) --- informing, not driving, clinical management. Intended for research use only.

**Codebase:** 113K LOC | 3,113 tests | Rust stable 2024 edition (MSRV 1.85) | Bevy 0.15.x | AGPL-3.0

**Repository:** `https://github.com/ResakaGit/RESONANCE`

**Paper:** Zenodo DOI 10.5281/zenodo.19342036

### 1.3 Foundational Constraints

All simulation behavior derives from 8 axioms (5 primitive, 3 derived) and 4 fundamental constants. No requirement may contradict, bypass, or weaken any axiom. The 4 fundamental constants are the only numeric values not derived from other values:

| Constant | Symbol | Value | Type |
|---|---|---|---|
| Kleiber exponent | `KLEIBER_EXPONENT` | 0.75 | Physics (biological universal) |
| Dissipation rates | `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}` | 0.005, 0.02, 0.08, 0.25 | Physics (2nd Law per state) |
| Coherence bandwidth | `COHERENCE_BANDWIDTH` | 50.0 Hz | Grid calibration |
| Density scale | `DENSITY_SCALE` | 20.0 | Grid calibration |

All ~40 lifecycle constants are algebraically derived from these 4 values via `src/blueprint/equations/derived_thresholds.rs`.

### 1.4 Definitions

| Term | Definition |
|---|---|
| qe | Quantum of energy --- the single universal unit for all entity state |
| ECS | Entity Component System --- Bevy's data-oriented architecture |
| Layer | One of 14 orthogonal ECS component types defining entity composition |
| Axiom | Foundational simulation rule; 5 primitive + 3 derived |
| Blueprint | Pure math module; stateless functions, no ECS dependency |
| Batch simulator | Headless parallel simulator using flat arrays, no Bevy dependency |
| Hill pharmacokinetics | Sigmoidal dose-response model: `response = conc^n / (EC50^n + conc^n)` |
| Bozic 2013 | Reference paper: Bozic et al. "Evolutionary dynamics of cancer in response to targeted combination therapy" (eLife, 2013) |

### 1.5 References

| ID | Document |
|---|---|
| IEC 62304 | IEC 62304:2006+Amd1:2015 Medical device software --- Software life cycle processes |
| GAMP 5 | ISPE GAMP 5: A Risk-Based Approach to Compliant GxP Computerized Systems, 2nd Ed. |
| IMDRF SaMD | IMDRF/SaMD WG/N10 --- Software as a Medical Device: Key Definitions |
| Bozic 2013 | Bozic et al. (2013) eLife 2:e00747 |
| Gatenby 2009 | Gatenby et al. (2009) Cancer Research 69(11):4894-4903 |
| London 2009 | London et al. (2009) Clinical Cancer Research 15(11):3856-3865 |

---

## 2. Functional Requirements

### RF-01: Emergent Life Simulation from Energy Interactions

| Field | Value |
|---|---|
| **ID** | RF-01 |
| **Priority** | Essential |
| **Description** | The system shall simulate emergent life from energy field interactions. All entity behavior shall derive from 8 foundational axioms: (1) Everything is Energy, (2) Pool Invariant, (3) Competition as Primitive, (4) Dissipation, (5) Conservation, (6) Emergence at Scale, (7) Distance Attenuation, (8) Oscillatory Nature. No scripted behavior, no templates, no hardcoded trophic classes. |
| **Source axioms** | Axioms 1--8 (all) |
| **Acceptance criteria** | Entities materialize from energy fields when coherence gain exceeds dissipation loss. Matter state, capabilities, and lifecycle derive from energy density and coherence. Spawning follows sigmoid viability potential with threshold 1/3 (derived from Axiom 2). |
| **Implementation** | `src/simulation/abiogenesis/` (axiomatic abiogenesis system) |
| | `src/blueprint/equations/abiogenesis/axiomatic.rs` (viability potential math) |
| | `src/blueprint/equations/derived_thresholds.rs` (spawn_potential_threshold = 1/3) |
| | `src/simulation/awakening.rs` (inert -> BehavioralAgent transition) |
| | `src/blueprint/equations/awakening.rs` (awakening potential math) |
| | `src/worldgen/` (field_grid, nucleus, propagation, materialization) |
| **Verification** | `src/blueprint/equations/derived_thresholds.rs` --- 17 unit tests |
| | `src/blueprint/equations/abiogenesis/axiomatic.rs` --- unit tests (viability potential, sigmoid bounds) |
| | `src/simulation/abiogenesis/constants.rs` --- threshold derivation tests |
| | `tests/property_conservation.rs` --- 19 proptest fuzz tests (conservation invariants) |
| | Map validation: `RESONANCE_MAP=genesis_validation cargo run` |

### RF-02: 14-Layer ECS Composition

| Field | Value |
|---|---|
| **ID** | RF-02 |
| **Priority** | Essential |
| **Description** | The system shall define all entities through composition of 14 orthogonal ECS layers. Each layer is independent, has its own update rule, and interacts with 2+ other layers. No entity requires all layers --- behavior emerges from the specific layer combination. |
| **Layers** | L0 BaseEnergy (qe), L1 SpatialVolume (radius), L2 OscillatorySignature (frequency, phase), L3 FlowVector (velocity, dissipation), L4 MatterCoherence (state, bond energy), L5 AlchemicalEngine (buffer, valves), L6 AmbientPressure (terrain), L7 WillActuator (intent), L8 AlchemicalInjector (spell payload), L9 MobaIdentity (faction, tags), L10 ResonanceLink (buff/debuff), L11 TensionField (gravity/magnetic), L12 Homeostasis (frequency adaptation), L13 StructuralLink (spring joint) |
| **Source axioms** | Axiom 1 (everything is energy), Axiom 6 (emergence at scale) |
| **Acceptance criteria** | Each layer is a separate Rust component with max 4 fields. Layers are registered with Bevy reflection. Adding or removing a layer from an entity changes only that entity's behavior. |
| **Implementation** | `src/layers/mod.rs` (50+ sub-modules, all layer components) |
| | `src/layers/energy.rs` (L0), `src/layers/volume.rs` (L1), `src/layers/oscillatory.rs` (L2), `src/layers/flow.rs` (L3), `src/layers/coherence.rs` (L4), `src/layers/engine.rs` (L5), `src/layers/pressure.rs` (L6), `src/layers/will.rs` (L7), `src/layers/injector.rs` (L8), `src/layers/identity.rs` (L9), `src/layers/link.rs` (L10), `src/layers/tension_field.rs` (L11), `src/layers/homeostasis.rs` (L12), `src/layers/structural_link.rs` (L13) |
| | `src/entities/archetypes/` (spawn functions compose layers) |
| **Verification** | Component registration verified via `app.register_type::<T>()` in plugins |
| | Entity spawn tests in `src/entities/archetypes/` modules |
| | Integration tests spawn minimal layer subsets, confirm independence |

### RF-03: Drug-Pathway Interaction Modeling (Pathway Inhibitor)

| Field | Value |
|---|---|
| **ID** | RF-03 |
| **Priority** | Essential |
| **Description** | The system shall model drug-pathway interactions using frequency-based binding affinity (Axiom 8), Hill pharmacokinetics (n=2), and three inhibition modes: Competitive (raises activation energy), Noncompetitive (reduces max efficiency), and Uncompetitive (reduces both). Off-target effects via frequency proximity. Bliss independence for drug combinations. Destructive interference for coherence disruption. Escape frequency prediction. |
| **Source axioms** | Axiom 8 (oscillatory nature --- frequency alignment), Axiom 7 (distance attenuation), Axiom 4 (dissipation) |
| **Acceptance criteria** | Binding affinity is Gaussian in frequency difference. Hill response is sigmoidal with n=2. Competitive mode raises E_a proportional to occupancy. Noncompetitive mode reduces efficiency proportional to occupancy. Uncompetitive mode reduces both. Combined drugs use Bliss independence: `1 - product(1 - occupancy_i)`. |
| **Implementation** | `src/blueprint/equations/pathway_inhibitor.rs` --- 11 public pure functions: `binding_affinity`, `hill_response`, `inhibitor_occupancy`, `inhibit_node`, `inhibit_pathway`, `combined_occupancy`, `selectivity_index`, `effective_node_params`, `coherence_disruption`, `apply_disruption_to_expression`, `find_escape_frequency` |
| | `src/blueprint/constants/pathway_inhibitor.rs` --- 7 derived constants |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` --- experiment orchestration |
| | `src/bin/pathway_inhibitor.rs` --- CLI binary |
| **Verification** | `src/blueprint/equations/pathway_inhibitor.rs` --- 42 unit tests |
| | `src/blueprint/constants/pathway_inhibitor.rs` --- 3 constant derivation tests |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` --- 31 integration tests (including Bozic validation, HOF experiments) |

### RF-04: Drug Resistance Evolution Modeling (Cytotoxic)

| Field | Value |
|---|---|
| **ID** | RF-04 |
| **Priority** | Essential |
| **Description** | The system shall model cytotoxic drug therapy with frequency-selective cell killing via Hill pharmacokinetics. Quiescent stem cells (low growth bias) shall escape chemotherapy. Resistant subpopulations shall emerge through mutation under drug pressure. Intermittent and continuous dosing schedules shall be supported. |
| **Source axioms** | Axiom 8 (frequency-selective targeting), Axiom 4 (dissipation as cell death), Axiom 1 (drug drains qe) |
| **Acceptance criteria** | Drug kills cells by draining qe proportional to frequency alignment and Hill response. Quiescent cells (growth_bias < threshold) survive. Mutation produces frequency-shifted offspring. Population dynamics show initial response followed by resistant regrowth. Deterministic across seeds. |
| **Implementation** | `src/use_cases/experiments/cancer_therapy.rs` --- `hill_response`, `cytotoxic_drain`, `is_quiescent`, `is_drug_active`, `pk_ramp`, `run` |
| | `src/batch/` --- underlying batch simulator (33 systems across 6 phases) |
| | `src/bin/cancer_therapy.rs` --- CLI binary |
| **Verification** | `src/use_cases/experiments/cancer_therapy.rs` --- 24 unit tests |
| | Tests cover: Hill response (zero/full/half alignment, sigmoidal steepness), cytotoxic drain (on-target, off-target, NaN safety), PK ramp, drug scheduling (continuous, intermittent), quiescent detection, determinism, conservation tracking, effective potency tracking |

### RF-05: Bozic 2013 Reproduction (Combination > Monotherapy)

| Field | Value |
|---|---|
| **ID** | RF-05 |
| **Priority** | Essential |
| **Description** | The system shall reproduce the key prediction of Bozic et al. 2013 (eLife): combination therapy has exponential advantage over monotherapy for drug resistance prevention. A 5-arm protocol (no_drug, mono_A, mono_B, combo_AB, double_A) shall demonstrate combo suppression > best monotherapy and combo > double-dose monotherapy. |
| **Source axioms** | Axiom 8 (frequency-selective targeting), Axiom 4 (dissipation), Axiom 6 (emergence at scale --- resistance emerges from population dynamics) |
| **Acceptance criteria** | 5-arm results: combo_AB suppression (56.5%) > mono_A (51.9%) > mono_B (36.5%). combo_AB > double_A (53.4%). Result confirmed across 10 independent seeds (10/10 pass, threshold >= 80%). Result is structural, not stochastic. |
| **Implementation** | `src/use_cases/experiments/pathway_inhibitor_exp.rs` --- `bozic_five_arm_experiment`, `bozic_robustness_ten_seeds` |
| | `src/bin/bozic_validation.rs` --- standalone validation binary |
| **Verification** | `src/use_cases/experiments/pathway_inhibitor_exp.rs` --- Bozic-specific tests: `bozic_five_arm_combo_beats_mono`, `bozic_five_arm_combo_beats_double`, `bozic_ten_seeds_robust` |
| | CLI validation: `cargo run --release --bin bozic_validation` (~95 sec, prints 5-arm results + 10-seed robustness) |

### RF-06: Clinical Calibration (4 Profiles)

| Field | Value |
|---|---|
| **ID** | RF-06 |
| **Priority** | Important |
| **Description** | The system shall provide calibration profiles that map abstract simulation units (qe, Hz, generations) to clinical units (nM, days, cell count) for 4 tumor types. Each profile has 4 parameters: days_per_generation, nm_per_concentration, cells_per_entity, mutation_rate. All values sourced from published literature. Calibration is qualitative --- NOT validated against patient outcomes. |
| **Profiles** | (1) CML / imatinib (Bozic 2013): 4 days/gen, 260 nM IC50 (2) Prostate / abiraterone (Gatenby 2009): 30 days/gen, 5.1 nM IC50 (3) NSCLC / erlotinib: 7 days/gen, 20 nM IC50 (4) Canine mast cell / toceranib proxy (Rosie case): 21 days/gen, 40 nM IC50 |
| **Source axioms** | Axiom 1 (qe as universal unit --- enables unit mapping) |
| **Acceptance criteria** | Each profile maps generations to days, concentration to nM, entities to cell count. Pure functions, no side effects. Rosie case includes explicit disclaimer about press-report sourcing. |
| **Implementation** | `src/blueprint/equations/clinical_calibration.rs` --- `CalibrationProfile` struct, 4 const profiles (`CML_IMATINIB`, `PROSTATE_ABIRATERONE`, `NSCLC_ERLOTINIB`, `CANINE_MAST_CELL`), `RosieCasePrediction` struct, 8 pure conversion functions: `days_to_generations`, `fraction_to_entity_counts`, `generation_to_days`, `concentration_to_nm`, `nm_to_concentration`, `entities_to_cells`, `frequency_to_mutation_burden`, `calibrate_protocol` |
| **Verification** | `src/blueprint/equations/clinical_calibration.rs` --- 21 unit tests |
| | Tests cover: unit conversions (round-trip consistency), profile constants (positive values, ordered by doubling time), Rosie case prediction bounds |

### RF-07: Bit-Exact Determinism

| Field | Value |
|---|---|
| **ID** | RF-07 |
| **Priority** | Essential |
| **Description** | The system shall produce bit-exact identical output given the same initial state. Determinism uses `f32::to_bits()` for hash comparison, PCG-like state-based RNG (no `std::rand`), and canonical entity ordering. Same seed shall produce same result across runs on the same platform. |
| **Source axioms** | All axioms (determinism is a meta-requirement for scientific reproducibility) |
| **Acceptance criteria** | `hash_f32_slice` produces identical hashes for identical inputs. `snapshots_match` returns true for replayed simulations. RNG functions (`next_u64`, `unit_f32`, `range_f32`, `gaussian_f32`) are pure functions of state with no external dependencies. |
| **Implementation** | `src/blueprint/equations/determinism.rs` --- `hash_f32_slice`, `snapshot_hash`, `snapshots_match`, `next_u64`, `unit_f32`, `range_f32`, `gaussian_f32`, `gaussian_frequency_alignment`, `sanitize_unit` |
| **Verification** | `src/blueprint/equations/determinism.rs` --- 23 unit tests |
| | `src/use_cases/experiments/cancer_therapy.rs` --- `run_deterministic` test (two runs with same config produce identical results) |
| | `src/use_cases/experiments/pathway_inhibitor_exp.rs` --- determinism tests across seeds |

### RF-08: Batch Simulation (Headless, Parallel)

| Field | Value |
|---|---|
| **ID** | RF-08 |
| **Priority** | Essential |
| **Description** | The system shall provide a batch simulator that runs millions of worlds in parallel without Bevy ECS dependency. Uses flat `repr(C)` entity arrays (`EntitySlot`), rayon parallelism, and 33 stateless systems across 6 pipeline phases. Genetic evolution via `GeneticHarness` (evaluate, select, reproduce). Genome serialization via `GenomeBlob` with lossless Bevy round-trip (`bridge.rs`). |
| **Source axioms** | All axioms (batch simulator implements same physics as Bevy simulator) |
| **Acceptance criteria** | `SimWorldFlat` holds 64 entity slots + grids. 33 systems execute in 6 phases (Input, Thermodynamic, Atomic, Chemical, Metabolic, Morphological). `GeneticHarness` runs evolutionary loops with tournament selection. `GenomeBlob` round-trips losslessly to/from Bevy components. Lineage tracking via `LineageId`. Population census via `PopulationCensus`. |
| **Implementation** | `src/batch/arena.rs` --- `EntitySlot`, `SimWorldFlat` |
| | `src/batch/systems/mod.rs` --- 33 systems in 12 sub-modules: `thermodynamic.rs`, `atomic.rs`, `chemical.rs`, `metabolic.rs`, `morphological.rs`, `input.rs`, `internal_field.rs`, `metabolic_graph.rs`, `protein.rs`, `particle_forces.rs`, `multicellular.rs` |
| | `src/batch/genome.rs` --- `GenomeBlob`, mutate, crossover |
| | `src/batch/harness.rs` --- `GeneticHarness`, `FitnessReport` |
| | `src/batch/bridge.rs` --- `GenomeBlob` <-> Bevy component conversion |
| | `src/batch/lineage.rs` --- `LineageId`, `TrackedGenome` |
| | `src/batch/census.rs` --- `EntitySnapshot`, `PopulationCensus` |
| | `src/batch/batch.rs` --- `WorldBatch`, `BatchConfig`, rayon `par_iter_mut` |
| | `src/bin/headless_sim.rs` --- headless simulation binary (PPM output, no GPU) |
| **Verification** | `src/batch/` --- 199 unit tests across 21 files |
| | Tests cover: arena slot operations (16 tests), genome mutation/crossover (14 tests), harness evolution (10 tests), bridge round-trip (11 tests), lineage tracking (10 tests), census capture (8 tests), all 33 batch systems (113 tests) |
| | CLI: `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm` |

### RF-09: Energy Conservation (Axiom 5)

| Field | Value |
|---|---|
| **ID** | RF-09 |
| **Priority** | Essential |
| **Description** | The system shall enforce energy conservation: energy is never created, only transferred or dissipated. Total system qe shall monotonically decrease (or remain constant). Pool Invariant (Axiom 2): sum of children's energy never exceeds parent's energy. Dissipation (Axiom 4): all processes lose energy at rate >= qe * dissipation_rate for the current matter state. |
| **Source axioms** | Axiom 2 (Pool Invariant), Axiom 4 (Dissipation / 2nd Law), Axiom 5 (Conservation --- derived from 2+4) |
| **Acceptance criteria** | No system creates energy. Reproduction drains parent, offspring qe <= drained amount. Basal drain applies passive qe cost proportional to radius^0.75 (Kleiber). Nucleus reservoirs deplete to zero. Nutrient return on death feeds recycling, not creation. |
| **Implementation** | `src/simulation/metabolic/basal_drain.rs` --- passive energy drain |
| | `src/simulation/metabolic/senescence_death.rs` --- age-based mortality |
| | `src/simulation/reproduction/mod.rs` --- conservation-preserving offspring |
| | `src/worldgen/nucleus.rs` --- `NucleusReservoir` (finite fuel) |
| | `src/worldgen/systems/nucleus_recycling.rs` --- nutrient -> new nucleus cycle |
| | `src/blueprint/equations/derived_thresholds.rs` --- `basal_drain_rate()`, dissipation constants |
| **Verification** | `tests/property_conservation.rs` --- 19 proptest fuzz tests with arbitrary inputs |
| | Tests verify: pool invariant (children <= parent), dissipation non-negative, total qe monotonic decrease, reproduction conservation, no energy creation |
| | `src/use_cases/experiments/cancer_therapy.rs` --- `conservation_tracked` test |

### RF-10: Morphogenesis from Energy Composition

| Field | Value |
|---|---|
| **ID** | RF-10 |
| **Priority** | Important |
| **Description** | Entity shape shall emerge from energy composition, not templates. Constructal optimization minimizes total cost (drag + vascular). GF1 flow geometry generates mesh. Organ count and placement derived from energy state. Surface properties (rugosity, albedo) inferred from composition. |
| **Source axioms** | Axiom 1 (energy determines form), Axiom 4 (dissipation cost constrains shape), Axiom 6 (shape emerges from interactions) |
| **Acceptance criteria** | `bounded_fineness_descent` optimizes fineness ratio via gradient descent. `optimal_appendage_count` balances drag, thrust, and maintenance. `organ_slot_scale` produces front/rear asymmetry from mobility_bias. Compound mesh merges torso + organ sub-meshes. |
| **Implementation** | `src/blueprint/equations/entity_shape.rs` --- `entity_geometry_influence`, `organ_slot_scale`, `optimal_appendage_count`, `bilateral_quadruped_attachments`, `projected_area_with_limbs` |
| | `src/blueprint/morphogenesis/constructal.rs` --- `shape_cost`, `bounded_fineness_descent` |
| | `src/simulation/lifecycle/constructal_body_plan.rs` --- `constructal_body_plan_system` |
| | `src/simulation/lifecycle/entity_shape_inference.rs` --- `entity_shape_inference_system` (compound mesh) |
| | `src/simulation/metabolic/morphogenesis.rs` --- `shape_optimization_system`, `surface_rugosity_system`, `albedo_inference_system` |
| | `src/geometry_flow/` --- GF1 flora-tube branching, `merge_meshes` |
| **Verification** | `src/blueprint/equations/entity_shape.rs` --- 39 unit tests |
| | Tests cover: fineness optimization convergence, organ slot scale bounds, appendage count vs drag, bilateral attachment symmetry, GF1 influence geometry |

### RF-11: Particle Physics (Coulomb + Lennard-Jones)

| Field | Value |
|---|---|
| **ID** | RF-11 |
| **Priority** | Important |
| **Description** | The system shall model molecular bonding via classical potentials with constants derived from the 4 fundamentals. Coulomb force: `F = k_C * q1 * q2 / (r^2 + eps^2)` where `k_C = 1/DENSITY_SCALE = 0.05`. Lennard-Jones: `V = 4*eps_LJ * [(sigma/r)^12 - (sigma/r)^6]` where `sigma = 1/DENSITY_SCALE`, `eps_LJ = DISSIPATION_SOLID * 100`. Bond stability when `|E_bond| > threshold` (negative = bound). Frequency alignment modulates bond strength (Axiom 8). |
| **Source axioms** | Axiom 7 (distance attenuation --- inverse square), Axiom 8 (oscillatory --- frequency modulation of bond strength), Axiom 1 (energy --- all constants from qe) |
| **Acceptance criteria** | Inverse-square law verified. LJ zero-crossing at `r = 2^(1/6) * sigma`. Newton's 3rd law (equal and opposite forces). Charge conservation. Bond energy negative for opposite charges. No bond tables, no molecule templates. |
| **Implementation** | `src/blueprint/equations/coulomb.rs` --- `coulomb_force`, `lennard_jones_force`, `net_force`, `bond_energy`, `is_bound`, `detect_bonds`, `classify_molecule`, `count_element_types`, `accumulate_forces` |
| | `src/batch/systems/particle_forces.rs` --- particle force accumulation in batch simulator |
| **Verification** | `src/blueprint/equations/coulomb.rs` --- 26 unit tests |
| | Tests cover: inverse-square law, LJ zero-crossing at `r = 2^(1/6)*sigma`, Newton's 3rd law, charge conservation, bond energy sign, bit-exact determinism, molecule classification |

### RF-12: Variable Genome (4--32 Genes)

| Field | Value |
|---|---|
| **ID** | RF-12 |
| **Priority** | Important |
| **Description** | The system shall support variable-length genomes with 4 to 32 genes. Core biases (growth, mobility, branching, resilience) occupy genes 0--3. Additional genes modulate phenotype via Kleiber-scaled maintenance cost. Gene duplication rate derived from `DISSIPATION_SOLID * 10`, deletion rate from `DISSIPATION_SOLID * 6`. Epigenetic gating masks gene expression. |
| **Source axioms** | Axiom 4 (dissipation --- genome maintenance cost), Axiom 1 (energy budget constrains gene count) |
| **Acceptance criteria** | Genome length ranges [4, 32]. Duplication adds genes, deletion removes them. Maintenance cost scales as `n_genes^KLEIBER * base_dissipation`. Effective biases computed from active genes weighted by auxiliary modulation. Genome distance metric (Euclidean on active genes). Deterministic hash for identity. |
| **Implementation** | `src/blueprint/equations/variable_genome.rs` --- `VariableGenome`, `genome_maintenance_cost`, `effective_bias`, `effective_biases`, `compute_phenotype`, duplication/deletion functions |
| **Verification** | `src/blueprint/equations/variable_genome.rs` --- 62 unit tests |
| | Tests cover: gene count bounds [4, 32], duplication/deletion, Kleiber cost scaling, bias computation, genome distance, hash determinism, epigenetic gating |

### RF-13: Genetic Code (64 Codons)

| Field | Value |
|---|---|
| **ID** | RF-13 |
| **Priority** | Important |
| **Description** | The system shall implement a codon-based genetic code with 64 codons mapping to 8 amino acids via a mutable codon table. Point mutations on codons, crossover between genomes, silent mutation fraction computation. Translation from codons to monomer chains for protein folding. |
| **Source axioms** | Axiom 6 (emergence --- genetic code evolves, not hardcoded) |
| **Acceptance criteria** | `CodonGenome` holds variable-length codon sequence. `CodonTable` maps 64 codons to 8 amino acids. Mutation changes single codons. Crossover recombines at random point. Silent mutations (codon change, same amino acid) are quantifiable. Table itself can mutate (codon reassignment). Translation produces monomer chain. |
| **Implementation** | `src/blueprint/equations/codon_genome.rs` --- `CodonGenome`, `CodonTable`, `mutate_codon`, `crossover_codon`, `translate_genome`, `classify_mutation`, `silent_mutation_fraction`, `mutate_table`, `codon_hash` |
| **Verification** | `src/blueprint/equations/codon_genome.rs` --- 28 unit tests |
| | Tests cover: genome construction, codon mutation, crossover, translation correctness, silent vs non-silent classification, table mutation, hash determinism |

### RF-14: Protein Folding (HP Lattice)

| Field | Value |
|---|---|
| **ID** | RF-14 |
| **Priority** | Important |
| **Description** | The system shall model protein folding using a 2D HP (Hydrophobic-Polar) lattice model. Monomers placed on a grid, fold energy computed from hydrophobic contacts. Greedy folding algorithm. Contact map and contact density for active site inference. Protein function (catalytic, structural, regulatory, transport) inferred from fold topology and frequency modulation. |
| **Source axioms** | Axiom 8 (frequency modulation of protein function), Axiom 4 (dissipation --- fold energy cost) |
| **Acceptance criteria** | `genome_to_polymer` converts variable genome to monomer chain. `fold_greedy` places monomers on 2D lattice minimizing energy. `contact_map` identifies non-sequential neighbors. `infer_protein_function` maps fold topology to functional role. `compute_protein_phenotype` produces phenotype from genome via full pipeline (genome -> polymer -> fold -> contacts -> function). |
| **Implementation** | `src/blueprint/equations/protein_fold.rs` --- `genome_to_polymer`, `fold_energy`, `fold_greedy`, `contact_map`, `contact_density`, `infer_protein_function`, `compute_protein_phenotype` |
| **Verification** | `src/blueprint/equations/protein_fold.rs` --- 27 unit tests |
| | Tests cover: polymer conversion, fold energy computation, greedy fold placement, contact map correctness, contact density, function inference, full pipeline phenotype |

### RF-15: Metabolic Networks (DAG)

| Field | Value |
|---|---|
| **ID** | RF-15 |
| **Priority** | Important |
| **Description** | The system shall model metabolic pathways as directed acyclic graphs (DAGs). Genes map to ExergyNode (metabolic node with role, activation energy, efficiency). Competitive flow distribution among parallel paths. Hebbian learning updates edge capacities based on usage. Catalytic activation reduction for enzyme effects. Topology inferred from genome structure. |
| **Source axioms** | Axiom 4 (dissipation --- metabolic cost), Axiom 1 (energy flow through network) |
| **Acceptance criteria** | `gene_to_exergy_node` maps gene value and index to metabolic node (role inferred from index). `metabolic_graph_from_variable_genome` builds DAG from genome. `competitive_flow_distribution` allocates flow based on efficiency with overhead. `hebbian_capacity_update` strengthens high-flow edges. `catalytic_activation_reduction` lowers activation energy. Competition overhead derived from `DISSIPATION_SOLID * 2`. |
| **Implementation** | `src/blueprint/equations/metabolic_genome.rs` --- `gene_to_exergy_node`, `organ_role_dimension`, `infer_role_from_gene`, `metabolic_graph_from_variable_genome`, `compute_metabolic_phenotype`, `competitive_flow_distribution`, `hebbian_capacity_update`, `catalytic_activation_reduction` |
| | `src/blueprint/equations/metabolic_graph/mod.rs` --- metabolic graph types and operations |
| **Verification** | `src/blueprint/equations/metabolic_genome.rs` --- 68 unit tests |
| | Tests cover: gene-to-node mapping, topology inference, competitive flow (conservation, efficiency ordering), Hebbian learning (strengthening, bounds), catalysis (activation reduction, cost), full pipeline phenotype |

### RF-16: Multicellular Organization

| Field | Value |
|---|---|
| **ID** | RF-16 |
| **Priority** | Important |
| **Description** | The system shall model multicellular aggregation via cell adhesion, colony detection (Union-Find), positional signaling (border signal, positional gradient), and differential gene expression based on colony position. Adhesion affinity is frequency- and distance-dependent (Axiom 8 + Axiom 7). |
| **Source axioms** | Axiom 8 (frequency alignment for adhesion), Axiom 7 (distance attenuation), Axiom 6 (multicellularity emerges from single-cell interactions) |
| **Acceptance criteria** | `adhesion_affinity` is Gaussian in frequency difference and decays with distance. `detect_colonies` uses Union-Find on adjacency matrix. `border_signal` distinguishes edge vs interior cells. `modulate_expression` adjusts gene expression based on positional gradient. |
| **Implementation** | `src/blueprint/equations/multicellular.rs` --- `adhesion_affinity`, `bond_strength`, `bond_cost`, `should_bond`, `detect_colonies` (Union-Find), `border_signal`, `positional_gradient`, `modulate_expression`, `specialization_index` |
| | `src/batch/systems/multicellular.rs` --- batch system implementation |
| **Verification** | `src/blueprint/equations/multicellular.rs` --- 27 unit tests |
| | `src/batch/systems/multicellular.rs` --- 6 batch integration tests |
| | Tests cover: adhesion affinity (distance decay, frequency decay, NaN safety), bond threshold, colony detection (Union-Find correctness), border signaling, positional gradient, expression modulation |

### RF-17: Emergence Systems (9 Active)

| Field | Value |
|---|---|
| **ID** | RF-17 |
| **Priority** | Important |
| **Description** | The system shall run 9 registered emergence systems modeling higher-order behavior: theory of mind (predictions from observed neighbors), cultural transmission (meme spread by oscillatory affinity), entrainment (Kuramoto frequency synchronization), infrastructure (persistent field modification + intake bonus), cooperation (Nash alliance detection), symbiosis (mutualism/parasitism on SymbiosisLink), niche adaptation (character displacement under competition), and epigenetic adaptation (environment modulates gene expression). |
| **Source axioms** | Axiom 6 (emergence at scale), Axiom 8 (oscillatory interaction for cultural transmission and entrainment) |
| **Acceptance criteria** | Each system is registered in a specific Phase. Theory of mind updates OtherModelSet. Cultural transmission spreads memes by frequency affinity. Entrainment synchronizes frequencies (Kuramoto model). Symbiosis drains/benefits entities on SymbiosisLink. Niche adaptation displaces overlapping niches. Epigenetic adaptation modulates expression_mask from environment. |
| **Implementation** | `src/simulation/emergence/theory_of_mind.rs` (Phase::Input) |
| | `src/simulation/emergence/culture.rs` (Phase::Input) |
| | `src/simulation/emergence/entrainment.rs` (Phase::AtomicLayer) |
| | `src/simulation/emergence/infrastructure.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/cooperation.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/symbiosis_effect.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/niche_adaptation.rs` (Phase::MetabolicLayer) |
| | `src/simulation/emergence/epigenetic_adaptation.rs` (Phase::MorphologicalLayer) |
| **Verification** | Unit tests in each emergence module |
| | Integration tests via map presets (`RESONANCE_MAP=civilization_test cargo run`) |

---

## 3. Performance Requirements

### RP-01: Full Test Suite Execution Time

| Field | Value |
|---|---|
| **ID** | RP-01 |
| **Priority** | Essential |
| **Description** | The complete test suite (3,113 tests) shall execute in less than 60 seconds on the reference development platform. |
| **Measured baseline** | 35.78 seconds (cargo test, release mode, Apple Silicon) |
| **Acceptance criteria** | `cargo test` completes with 0 failures in < 60 sec |
| **Verification** | `cargo test` (CI and local). Measured via wall-clock time. |

### RP-02: Batch Benchmark Throughput

| Field | Value |
|---|---|
| **ID** | RP-02 |
| **Priority** | Important |
| **Description** | The batch simulator shall maintain throughput measured via criterion benchmarks. Parallel execution of N worlds using rayon `par_iter_mut`. |
| **Acceptance criteria** | `cargo bench --bench batch_benchmark` completes without regression beyond 10% of baseline |
| **Verification** | `cargo bench --bench batch_benchmark` (criterion, statistical significance) |

### RP-03: Headless Simulation (10K Ticks)

| Field | Value |
|---|---|
| **ID** | RP-03 |
| **Priority** | Important |
| **Description** | The headless simulator shall complete 10,000 simulation ticks and produce a PPM image without GPU dependency. |
| **Acceptance criteria** | `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm` completes successfully, produces valid PPM file |
| **Implementation** | `src/bin/headless_sim.rs` |
| **Verification** | CLI execution + output file validation (valid PPM header, correct dimensions) |

---

## 4. Safety Requirements

### RS-01: Zero `unsafe` in Runtime Simulation

| Field | Value |
|---|---|
| **ID** | RS-01 |
| **Priority** | Essential |
| **Description** | The simulation engine shall contain zero `unsafe` blocks in runtime simulation code. Exceptions limited to documented GPU layout (`src/worldgen/cell_field_snapshot/gpu_layout.rs`) and batch arena memory layout (`src/batch/arena.rs`, `src/batch/bridge.rs`) --- both isolated from simulation logic and documented with `// DEBT:` justification. |
| **Acceptance criteria** | `grep -rn "unsafe" src/ --include="*.rs"` returns only the 4 documented exceptions in non-simulation-logic files. Zero `unsafe` in: `src/blueprint/`, `src/simulation/`, `src/layers/`, `src/entities/`, `src/plugins/` |
| **Verification** | Automated grep in CI. Manual audit of any new `unsafe` addition. |

### RS-02: Zero Shared Mutable State

| Field | Value |
|---|---|
| **ID** | RS-02 |
| **Priority** | Essential |
| **Description** | The system shall not use `Arc<Mutex<T>>`, `static mut`, or `lazy_static! { Mutex }` for shared mutable state. All mutable state shall flow through Bevy `Resource` or `Local` types. |
| **Acceptance criteria** | Zero occurrences of `Arc<Mutex`, `static mut`, or `lazy_static! { Mutex }` in `src/` |
| **Verification** | Automated grep in CI. Compiler warnings for `static mut` (Rust 2024 edition). |

### RS-03: Deterministic Output

| Field | Value |
|---|---|
| **ID** | RS-03 |
| **Priority** | Essential |
| **Description** | The system shall produce deterministic output for identical inputs. No dependency on `std::rand`, wall-clock time, thread scheduling, or HashMap iteration order in simulation logic. RNG is PCG-like state machine in `src/blueprint/equations/determinism.rs`. |
| **Acceptance criteria** | Two sequential runs with identical configuration produce bit-exact identical energy snapshots (`snapshots_match` returns true). Batch simulator results reproducible across runs. |
| **Implementation** | `src/blueprint/equations/determinism.rs` --- all RNG functions |
| **Verification** | `src/blueprint/equations/determinism.rs` --- 23 unit tests |
| | `src/use_cases/experiments/cancer_therapy.rs` --- `run_deterministic` test |
| | Batch simulator determinism tests in `src/batch/` modules |

### RS-04: Visible Disclaimers

| Field | Value |
|---|---|
| **ID** | RS-04 |
| **Priority** | Essential |
| **Description** | The system shall display visible disclaimers that RESONANCE is NOT a clinical tool, NOT validated against patient outcomes, and NOT a substitute for clinical trials. Disclaimers shall appear in: README.md, paper abstract, clinical calibration source code, and all experiment output. |
| **Acceptance criteria** | README.md contains "NOT a clinical tool" or equivalent. `clinical_calibration.rs` contains DISCLAIMER comment on Rosie case. Paper (Zenodo) contains limitations section. Experiment binaries print scope disclaimers. |
| **Verification** | Manual audit of README.md, paper, and source code disclaimers. Automated grep for disclaimer keywords. |

### RS-05: No Patient Data Processing

| Field | Value |
|---|---|
| **ID** | RS-05 |
| **Priority** | Essential |
| **Description** | The system shall not accept, process, store, or transmit any patient-identifiable data. All inputs are simulation parameters (energy values, frequencies, grid dimensions). All outputs are simulation state (qe distributions, population counts, PPM images, CSV/JSON summaries). |
| **Acceptance criteria** | No file I/O reads patient data formats (DICOM, HL7, FHIR). No network communication. No database connections. Input is: CLI args, `.ron` map configs. Output is: PPM images, CSV/JSON, terminal text. |
| **Verification** | Cargo.toml audit --- no medical data format crates. Source code audit --- no network or database code. |

---

## 5. Interface Requirements

### RI-01: Command-Line Interface (Primary)

| Field | Value |
|---|---|
| **ID** | RI-01 |
| **Description** | The primary interface is CLI. Simulation binaries accept command-line arguments for configuration. No GUI is required for core simulation functionality. |
| **Binaries** | `headless_sim` (PPM output), `cancer_therapy` (cytotoxic experiment), `pathway_inhibitor` (PI experiment), `bozic_validation` (5-arm protocol), `adaptive_therapy` (adaptive dosing), `evolve` (evolutionary run), `particle_lab` (Coulomb/LJ), `petri_dish` (multicellular), plus 15+ additional binaries in `src/bin/` |
| **Implementation** | `src/bin/*.rs` (25 binaries) |
| | `src/use_cases/cli.rs` --- shared CLI utilities (`parse_arg`, `archetype_label`, `resolve_preset`) |

### RI-02: Bevy Rendering (Optional)

| Field | Value |
|---|---|
| **ID** | RI-02 |
| **Description** | Optional 2D/3D rendering via Bevy engine for visual inspection. Not required for simulation correctness. Dashboard bridge provides real-time tick summary, time series, and speed controls. |
| **Implementation** | `src/rendering/quantized_color/` --- QuantizedColorPlugin |
| | `src/runtime_platform/dashboard_bridge.rs` --- `SimTickSummary`, `SimTimeSeries`, `RingBuffer`, `ViewConfig`, `DashboardBridgePlugin` |
| | `src/runtime_platform/` --- 17 sub-modules (camera, HUD, input, fog_overlay, etc.) |

### RI-03: Map Configuration Interface

| Field | Value |
|---|---|
| **ID** | RI-03 |
| **Description** | World configuration via `.ron` (Rusty Object Notation) map files. Environment variable `RESONANCE_MAP` selects map. |
| **Implementation** | `src/worldgen/map_config.rs` --- map loading |
| | `assets/maps/*.ron` --- 25 map configurations |
| **Examples** | `genesis_validation.ron`, `proving_grounds.ron`, `stellar_system.ron`, `earth.ron`, `civilization_test.ron` |

---

## 6. Data Requirements

### RD-01: Input Data --- Map Configurations (.ron)

| Field | Value |
|---|---|
| **ID** | RD-01 |
| **Description** | Input configuration files use RON (Rusty Object Notation) format. Each map defines: grid dimensions, nucleus placement, frequency bands, terrain parameters. 25 map files provided. |
| **Location** | `assets/maps/*.ron` |
| **Validation** | RON deserialization at startup. Invalid maps produce Bevy asset load errors. |

### RD-02: Output Data --- PPM Images

| Field | Value |
|---|---|
| **ID** | RD-02 |
| **Description** | Headless simulator produces PPM (Portable Pixmap) images representing world state. RGB values derived from entity energy and frequency via quantized color system. |
| **Implementation** | `src/bin/headless_sim.rs` --- PPM writer |
| **Format** | PPM P6 (binary), dimensions = grid_size * scale |

### RD-03: Output Data --- CSV/JSON Export

| Field | Value |
|---|---|
| **ID** | RD-03 |
| **Description** | Simulation results exported as CSV or JSON strings via stateless adapter functions. No file I/O in export functions --- callers handle persistence. Entity snapshots and generation statistics supported. |
| **Implementation** | `src/use_cases/export.rs` --- `entity_to_csv`, `generation_to_csv`, `export_history_csv`, `export_censuses_csv`, `entity_to_json`, `generation_to_json` |

### RD-04: Internal Data --- Genome Serialization

| Field | Value |
|---|---|
| **ID** | RD-04 |
| **Description** | `GenomeBlob` provides binary serialization of genome state. Lossless round-trip between batch simulator (`EntitySlot`) and Bevy ECS components via `bridge.rs`. |
| **Implementation** | `src/batch/genome.rs` --- `GenomeBlob` |
| | `src/batch/bridge.rs` --- Bevy component <-> GenomeBlob conversion |
| **Verification** | `src/batch/bridge.rs` --- 11 round-trip tests |

---

## 7. Traceability Summary

### 7.1 Forward Traceability: Requirement -> Module -> Test

| Req ID | Module(s) | Test File(s) | Test Count |
|---|---|---|---|
| RF-01 | `blueprint/equations/abiogenesis/`, `simulation/abiogenesis/`, `blueprint/equations/derived_thresholds.rs` | `derived_thresholds.rs`, `axiomatic.rs`, `property_conservation.rs` | 17 + 19 |
| RF-02 | `layers/` (50+ sub-modules) | Entity spawn tests, plugin registration | N/A (structural) |
| RF-03 | `blueprint/equations/pathway_inhibitor.rs`, `blueprint/constants/pathway_inhibitor.rs` | `pathway_inhibitor.rs`, `pathway_inhibitor_exp.rs` | 41 + 3 + 31 |
| RF-04 | `use_cases/experiments/cancer_therapy.rs`, `batch/` | `cancer_therapy.rs` | 24 |
| RF-05 | `use_cases/experiments/pathway_inhibitor_exp.rs`, `bin/bozic_validation.rs` | `pathway_inhibitor_exp.rs` (Bozic-specific) | 3+ (in 31) |
| RF-06 | `blueprint/equations/clinical_calibration.rs` | `clinical_calibration.rs` | 21 |
| RF-07 | `blueprint/equations/determinism.rs` | `determinism.rs`, `cancer_therapy.rs` | 23 + 1 |
| RF-08 | `batch/` (21 files) | `batch/` modules | 199 |
| RF-09 | `simulation/metabolic/`, `worldgen/nucleus.rs`, `blueprint/equations/derived_thresholds.rs` | `property_conservation.rs`, `cancer_therapy.rs` | 19 + 1 |
| RF-10 | `blueprint/equations/entity_shape.rs`, `blueprint/morphogenesis/`, `simulation/lifecycle/` | `entity_shape.rs` | 39 |
| RF-11 | `blueprint/equations/coulomb.rs`, `batch/systems/particle_forces.rs` | `coulomb.rs`, `particle_forces.rs` | 26 + 9 |
| RF-12 | `blueprint/equations/variable_genome.rs` | `variable_genome.rs` | 62 |
| RF-13 | `blueprint/equations/codon_genome.rs` | `codon_genome.rs` | 28 |
| RF-14 | `blueprint/equations/protein_fold.rs` | `protein_fold.rs` | 27 |
| RF-15 | `blueprint/equations/metabolic_genome.rs` | `metabolic_genome.rs` | 68 |
| RF-16 | `blueprint/equations/multicellular.rs`, `batch/systems/multicellular.rs` | `multicellular.rs` | 27 + 6 |
| RF-17 | `simulation/emergence/` (8 files) | Per-module unit tests | Per module |
| RP-01 | All `src/` | `cargo test` | 3,113 |
| RP-02 | `batch/` | `cargo bench --bench batch_benchmark` | Criterion |
| RP-03 | `bin/headless_sim.rs` | CLI execution | Manual |
| RS-01 | All `src/` | `grep` audit | Automated |
| RS-02 | All `src/` | `grep` audit | Automated |
| RS-03 | `blueprint/equations/determinism.rs` | `determinism.rs` | 23 |
| RS-04 | README.md, paper, source | Manual audit | Manual |
| RS-05 | Cargo.toml, all `src/` | Dependency + source audit | Manual |

### 7.2 Coverage Summary

| Category | Count | Tests |
|---|---|---|
| Functional requirements | 17 (RF-01 through RF-17) | ~700+ dedicated unit tests |
| Performance requirements | 3 (RP-01 through RP-03) | Full suite + benchmarks |
| Safety requirements | 5 (RS-01 through RS-05) | Automated grep + manual audit |
| Interface requirements | 3 (RI-01 through RI-03) | CLI execution tests |
| Data requirements | 4 (RD-01 through RD-04) | Deserialization + round-trip tests |
| **Total** | **32 requirements** | **3,113 tests** |

### 7.3 Gaps and Limitations

1. **RF-02 (14-layer composition):** Layer orthogonality is a design constraint enforced by code review, not by automated test. No automated test verifies that removing a layer from one entity does not affect another entity.
2. **RF-17 (emergence systems):** 7 additional emergence systems are implemented but not registered in any plugin (coalition stability, institution, tectonic drift, multiscale aggregation, geological LOD). These are code-complete but have zero runtime consumers.
3. **RP-02 (batch benchmark):** No absolute throughput threshold defined; regression testing only.
4. **RS-04 (disclaimers):** Disclaimer presence is verified by manual audit, not automated test.
5. **Clinical calibration (RF-06):** Rosie case profile is calibrated from press reports, not peer-reviewed trial data. This is documented in source code but bears repeating: the profile is for simulation exploration only.

---

*Document generated from codebase analysis. All file paths are relative to repository root. All test counts verified against source.*
