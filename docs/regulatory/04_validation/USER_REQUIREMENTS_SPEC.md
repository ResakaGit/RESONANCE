---
document_id: RD-4.6
title: User Requirements Specification
standard: GAMP 5 2nd Ed.
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# User Requirements Specification

## 1. Purpose

This document specifies the user requirements for RESONANCE, mapping user needs to system capabilities. It satisfies GAMP 5 2nd Edition User Requirements Specification (URS) requirements and provides the highest-level traceability anchor: user needs drive functional requirements (RD-1.3), which drive implementation and verification.

The URS is distinct from the Software Requirements Specification (RD-1.3):
- **URS (this document):** What users need the system to do, expressed in user-domain language.
- **SRS (RD-1.3):** What the system shall do, expressed in technical terms with implementation references.

All user requirements in this document are derived retroactively from implemented, tested functionality. RESONANCE is a research tool --- these requirements reflect research needs, not clinical needs.

**Cross-references:**
- RD-1.1 `docs/regulatory/01_foundation/INTENDED_USE.md` --- Intended users, use environment, excluded users
- RD-1.3 `docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md` --- Functional/performance/safety requirements
- RD-3.1 `docs/regulatory/03_traceability/TRACEABILITY_MATRIX.md` --- Full traceability chain
- RD-4.1 `docs/regulatory/04_validation/VALIDATION_PLAN.md` --- Validation strategy and acceptance criteria
- RD-4.4 `docs/regulatory/04_validation/VALIDATION_REPORT.md` --- Validation results

## 2. Intended Users

Per RD-1.1 §2, the intended users of RESONANCE are:

### 2.1 Primary Users

| User Category | Description | Key Need |
|---------------|-------------|----------|
| Computational biology researchers | Scientists studying emergent dynamics, evolutionary game theory, agent-based models | Simulate emergent life from first principles without scripted behavior |
| Mathematical oncology researchers | Scientists modeling tumor heterogeneity, resistance dynamics, adaptive therapy | Model drug-pathway interactions and resistance evolution |
| Pharmacology researchers (preclinical) | R&D teams generating hypotheses about drug combination strategies | Validate combination therapy advantage over monotherapy |

### 2.2 Secondary Users

| User Category | Description | Key Need |
|---------------|-------------|----------|
| Academic institutions | University courses/labs using RESONANCE for teaching | Accessible, well-documented simulation with reproducible output |
| Pharmaceutical R&D teams | Preclinical hypothesis generation | Scalable batch experiments with statistical robustness |

### 2.3 Explicitly Excluded Users

Clinicians, patients, caregivers, regulatory submission preparers, and clinical trial designers are **excluded** from the intended user population (RD-1.1 §2.3). No user requirement in this document addresses clinical use.

## 3. User Requirements

### UN-01: Simulate Emergent Life from First Principles

| Field | Value |
|-------|-------|
| **ID** | UN-01 |
| **User Need** | As a computational biology researcher, I need to simulate emergent life dynamics where all behavior arises from energy interactions --- not from scripted behaviors, templates, or hardcoded trophic classes --- so that I can study how complexity emerges from simple rules. |
| **Acceptance Criteria** | (1) Entities materialize from energy fields without templates. (2) Matter state, capabilities, and lifecycle derive from energy density. (3) No entity has hardcoded behavior --- all behavior is compositional from orthogonal layers. (4) The system operates on 8 foundational axioms and 4 fundamental constants. |
| **Maps to Requirements** | RF-01 (Emergent life simulation), RF-02 (14-layer ECS composition), RF-09 (Energy conservation) |
| **Validation Evidence** | 17 derived threshold tests, viability potential tests, map validation (`genesis_validation`). See RD-4.3 §3, RD-4.4 §3. |

### UN-02: Model Drug-Pathway Interactions

| Field | Value |
|-------|-------|
| **ID** | UN-02 |
| **User Need** | As a mathematical oncology researcher, I need to model how drugs interact with cellular pathways --- including competitive, noncompetitive, and uncompetitive inhibition --- so that I can study how different inhibition modes affect resistance evolution. |
| **Acceptance Criteria** | (1) Three inhibition modes available: Competitive (raises activation energy), Noncompetitive (reduces max efficiency), Uncompetitive (reduces both). (2) Drug binding is frequency-selective (Gaussian affinity). (3) Hill pharmacokinetics (n=2) governs dose-response. (4) Drug combinations use Bliss independence. (5) Off-target effects via frequency proximity. |
| **Maps to Requirements** | RF-03 (Pathway inhibitor), RF-04 (Cytotoxic drug model) |
| **Validation Evidence** | 42 pathway inhibitor tests, 24 cancer therapy tests. See RD-4.3 §3.3. |

### UN-03: Validate Against Published Data

| Field | Value |
|-------|-------|
| **ID** | UN-03 |
| **User Need** | As a pharmacology researcher, I need to validate that the simulation's qualitative predictions are consistent with published oncology data --- specifically, that combination therapy outperforms monotherapy for resistance prevention --- so that I can trust the simulation for hypothesis generation. |
| **Acceptance Criteria** | (1) Bozic 2013 (eLife) key prediction reproduced: combination therapy suppression > best monotherapy. (2) Result confirmed across 10 independent seeds (>=80% threshold). (3) Result is structural, not stochastic. (4) Validation is qualitative (suppression percentages, not absolute cell counts or time-to-resistance in weeks). |
| **Maps to Requirements** | RF-05 (Bozic 2013 reproduction), RF-06 (Clinical calibration profiles) |
| **Validation Evidence** | Bozic 5-arm: combo 56.5% > mono_A 51.9% > mono_B 36.5%. 10/10 seeds confirm. `cargo run --release --bin bozic_validation`. See RD-4.4 §3.1. |

### UN-04: Reproducible Results

| Field | Value |
|-------|-------|
| **ID** | UN-04 |
| **User Need** | As a researcher publishing simulation results, I need bit-exact reproducible output --- the same configuration and seed must produce identical results across runs --- so that my results can be independently verified and my experiments are scientifically valid. |
| **Acceptance Criteria** | (1) Same seed + same configuration = bit-exact identical output. (2) No dependency on wall-clock time, thread scheduling, or HashMap iteration order. (3) RNG is deterministic (hash-based, no external randomness). (4) Reproducibility holds within a platform (cross-platform not guaranteed due to f32 operations). |
| **Maps to Requirements** | RF-07 (Bit-exact determinism), RS-03 (Deterministic output) |
| **Validation Evidence** | 23 determinism tests, `run_deterministic` cross-experiment check, `snapshots_match` API. See RD-4.3 §3.5. |

### UN-05: Scalable Batch Experiments

| Field | Value |
|-------|-------|
| **ID** | UN-05 |
| **User Need** | As a researcher running evolutionary or pharmacological experiments, I need to simulate millions of worlds in parallel without GPU dependency --- so that I can perform statistical analyses, evolutionary sweeps, and parameter sensitivity studies at scale. |
| **Acceptance Criteria** | (1) Batch simulator runs without Bevy ECS (flat arrays, rayon parallel). (2) Genetic evolution via tournament selection, mutation, crossover. (3) Genome round-trips losslessly between batch and Bevy representations. (4) Lineage tracking for ancestry analysis. (5) Population census for per-generation statistics. |
| **Maps to Requirements** | RF-08 (Batch simulation) |
| **Validation Evidence** | 199 batch tests, bridge round-trip tests, harness evolution tests, census capture tests. `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm`. See RD-4.3 §3.4. |

### UN-06: Visual and Headless Output

| Field | Value |
|-------|-------|
| **ID** | UN-06 |
| **User Need** | As a researcher, I need both visual (real-time rendering) and headless (no GPU) output modes --- visual for exploration and understanding, headless for batch experiments on servers without displays. |
| **Acceptance Criteria** | (1) Real-time Bevy rendering with quantized color system. (2) Headless PPM image output via `headless_sim` binary. (3) Map configuration via `.ron` files in `assets/maps/`. (4) CLI interface for experiment binaries with configurable parameters. |
| **Maps to Requirements** | RI-01 (Command-line interface), RI-02 (Bevy rendering), RI-03 (Map configuration) |
| **Validation Evidence** | Headless sim produces valid PPM file. Map presets load and execute. See RD-4.3 §3.6. |

### UN-07: Extensible Biological Hierarchy

| Field | Value |
|-------|-------|
| **ID** | UN-07 |
| **User Need** | As a computational biology researcher, I need the simulation to model biological organization at multiple scales --- from molecular (protein folding, metabolic networks) through cellular (multicellular organization) to population (emergence, evolution) --- so that I can study how higher-level behavior emerges from lower-level interactions. |
| **Acceptance Criteria** | (1) Morphogenesis: shapes emerge from energy composition via constructal optimization. (2) Particle physics: Coulomb + LJ bonding from 4 fundamental constants. (3) Variable genome: 4--32 genes with duplication/deletion. (4) Genetic code: 64 codons mapping to 8 amino acids. (5) Protein folding: 2D HP lattice model. (6) Metabolic networks: DAG with competitive flow. (7) Multicellular: adhesion, colony detection (Union-Find), differential expression. (8) Emergence: 9 registered systems (theory of mind, culture, entrainment, infrastructure, cooperation, symbiosis, niche adaptation, epigenetics). |
| **Maps to Requirements** | RF-10 (Morphogenesis), RF-11 (Particle physics), RF-12 (Variable genome), RF-13 (Genetic code), RF-14 (Protein folding), RF-15 (Metabolic networks), RF-16 (Multicellular), RF-17 (Emergence systems) |
| **Validation Evidence** | Per-module test suites: entity_shape (39), coulomb (26), variable_genome (62), codon_genome (28), protein_fold (27), metabolic_genome (68), multicellular (27+6), emergence modules. See RD-4.3 §3 for complete verification evidence. |

## 4. Traceability Matrix: User Needs to System Requirements

| User Need | Functional Requirements | Performance Requirements | Safety Requirements | Interface Requirements |
|-----------|------------------------|--------------------------|---------------------|----------------------|
| UN-01 | RF-01, RF-02, RF-09 | --- | RS-01, RS-02, RS-03 | --- |
| UN-02 | RF-03, RF-04 | --- | RS-04 | RI-01 |
| UN-03 | RF-05, RF-06 | --- | RS-04 | RI-01 |
| UN-04 | RF-07 | RP-01 | RS-03 | --- |
| UN-05 | RF-08 | RP-02, RP-03 | --- | RI-01 |
| UN-06 | --- | RP-03 | --- | RI-01, RI-02, RI-03 |
| UN-07 | RF-10 through RF-17 | --- | RS-01, RS-02 | --- |

### 4.1 Requirement ID Reference

For convenience, the full requirement inventory from RD-1.3:

**Functional (RF):**
- RF-01: Emergent life simulation
- RF-02: 14-layer ECS composition
- RF-03: Pathway inhibitor model
- RF-04: Cytotoxic drug model
- RF-05: Bozic 2013 reproduction
- RF-06: Clinical calibration (4 profiles)
- RF-07: Bit-exact determinism
- RF-08: Batch simulation
- RF-09: Energy conservation
- RF-10: Morphogenesis
- RF-11: Particle physics (Coulomb + LJ)
- RF-12: Variable genome (4--32 genes)
- RF-13: Genetic code (64 codons)
- RF-14: Protein folding (HP lattice)
- RF-15: Metabolic networks (DAG)
- RF-16: Multicellular organization
- RF-17: Emergence systems (9 active)

**Performance (RP):** RP-01 (test suite <60s), RP-02 (batch throughput), RP-03 (headless 10K ticks)

**Safety (RS):** RS-01 (zero `unsafe`), RS-02 (zero shared mutable state), RS-03 (deterministic output), RS-04 (visible disclaimers), RS-05 (abstract units)

**Interface (RI):** RI-01 (CLI), RI-02 (Bevy rendering), RI-03 (map configuration)

## 5. Traceability: User Needs to Validation

| User Need | Validation Activity | Validation Document | Result |
|-----------|-------------------|-------------------|--------|
| UN-01 | Map validation: `RESONANCE_MAP=genesis_validation cargo run`. Entities materialize from energy fields. Conservation proptest. | RD-4.4 §3 | Pass --- entities emerge without templates |
| UN-02 | Pathway inhibitor test suite (42 tests). All three inhibition modes verified. Bliss independence verified. | RD-4.3 §3.3 | Pass --- 42/42 tests |
| UN-03 | Bozic 5-arm experiment + 10-seed robustness. `cargo run --release --bin bozic_validation`. | RD-4.4 §3.1 | Pass --- 10/10 seeds, combo > mono |
| UN-04 | Determinism test suite (23 tests). `run_deterministic` cross-experiment check. | RD-4.3 §3.5 | Pass --- bit-exact across runs |
| UN-05 | Batch test suite (199 tests). Headless sim binary. Bridge round-trip. | RD-4.3 §3.4 | Pass --- 199/199 tests |
| UN-06 | Headless sim produces valid PPM. Bevy rendering launches. Map presets load. | RD-4.3 §3.6 | Pass --- visual and headless confirmed |
| UN-07 | Per-module test suites (morphogenesis through emergence). See §3 above for test counts. | RD-4.3 §3 | Pass --- all modules tested |

## 6. User Needs Not Currently Addressed

The following user needs have been identified but are **not yet implemented**. They are listed here for completeness and future planning.

| ID | User Need | Status | Notes |
|----|-----------|--------|-------|
| UN-08 | Immune system modeling (T-cell, NK-cell interactions as frequency-selective predation) | Not implemented | Listed in paper future work (line 1282) |
| UN-09 | Tumor microenvironment (vasculature, hypoxia, stromal interactions) | Not implemented | Paper limitation (line 1267--1269) |
| UN-10 | Pharmacokinetic modeling (ADME: absorption, distribution, metabolism, excretion) | Not implemented | Paper limitation (line 1266) |
| UN-11 | Cross-platform determinism (identical output across Windows/macOS/Linux) | Not guaranteed | f32 operations may differ across platforms. Within-platform determinism is guaranteed (RF-07). |
| UN-12 | Clinical validation against patient-level outcome data | Not implemented | README.md line 141: "Against patient outcomes: Not yet" |

These unaddressed needs represent the honest scope boundary of RESONANCE. They are not failures --- they are documented limitations of a research tool at version 0.1.0.

## 7. Codebase References

| Claim | Reference | Verification |
|-------|-----------|--------------|
| 7 user needs mapped to 17+ requirements | This document §3--4 | Cross-reference with RD-1.3 |
| Bozic 10/10 seeds | `src/bin/bozic_validation.rs` | `cargo run --release --bin bozic_validation` |
| 3,113 tests | `cargo test` output | Measured at commit `971c7ac` |
| 42 pathway inhibitor tests | `src/blueprint/equations/pathway_inhibitor.rs` | `#[test]` count |
| 199 batch tests | `src/batch/` modules | `#[test]` count across 21 files |
| 23 determinism tests | `src/blueprint/equations/determinism.rs` | `#[test]` count |
| 4 calibration profiles | `src/blueprint/equations/clinical_calibration.rs` | CML, Prostate, NSCLC, Canine MCT |

## 8. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial URS. 7 user needs (UN-01 through UN-07) mapped to 17 functional + 3 performance + 5 safety + 3 interface requirements. 5 unaddressed needs (UN-08 through UN-12) documented as future scope. |
