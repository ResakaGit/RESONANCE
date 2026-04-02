---
document_id: RD-1.1
title: Intended Use Statement
standard: IMDRF SaMD N10, IMDRF SaMD N12, IEC 62304 §4.1
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Intended Use Statement

## 1. Intended Use Statement

RESONANCE is a computational research tool for simulating emergent life dynamics, therapeutic resistance evolution, and drug interaction strategies from first principles. It is intended for use by researchers in computational biology, mathematical oncology, and pharmacology to generate, explore, and test hypotheses about how population-level therapeutic resistance emerges from energy-based interactions governed by 8 foundational axioms and 4 fundamental constants. RESONANCE operates exclusively on abstract energy units (qe) — not on molar concentrations, molecular structures, or patient-derived data — and produces qualitative predictions (e.g., "combination therapy suppresses more than monotherapy") suitable for informing preclinical research directions. It is not intended, designed, or validated for clinical decision-making, patient diagnosis, treatment selection, or any use where its output directly or indirectly influences the care of a specific patient.

**Codebase references:**
- Axioms and constants: `CLAUDE.md` §The 8 Foundational Axioms, §The 4 Fundamental Constants
- Abstract energy model: `src/layers/energy.rs` (L0 BaseEnergy — all entities are qe)
- Drug models: `src/blueprint/equations/pathway_inhibitor.rs` (11 pure functions, 42 tests), `src/use_cases/experiments/cancer_therapy.rs` (24 tests)
- Disclaimers: `README.md` lines 18–22 ("What It Is NOT")
- Paper scope: `docs/paper/resonance_arxiv.tex` §5 Limitations (lines 1258–1272)

## 2. Intended Users

### 2.1 Primary Users

| User Category | Description | Expected Competence |
|---------------|-------------|---------------------|
| Computational biology researchers | Academic or industry scientists studying emergent dynamics, evolutionary game theory, or agent-based models | Graduate-level biology or bioinformatics; proficiency with simulation tools |
| Mathematical oncology researchers | Scientists modeling tumor heterogeneity, resistance dynamics, or adaptive therapy strategies | PhD-level quantitative biology or applied mathematics |
| Pharmacology researchers (preclinical) | R&D teams generating hypotheses about drug combination strategies before in vitro/in vivo validation | Understanding of Hill pharmacokinetics, dose-response relationships, and resistance mechanisms |

### 2.2 Secondary Users

| User Category | Description | Expected Competence |
|---------------|-------------|---------------------|
| Academic institutions | University courses or labs using RESONANCE as a teaching tool for emergent systems, ECS architecture, or computational modeling | Instructor-supervised use |
| Pharmaceutical R&D teams | Preclinical hypothesis generation: exploring combinatorial drug strategies before committing to wet lab experiments | Teams with computational biology expertise; results require independent validation |

### 2.3 Explicitly Excluded Users

| Excluded Category | Reason |
|-------------------|--------|
| Clinicians (physicians, oncologists, veterinarians) | RESONANCE output is not validated against patient outcomes and must not inform individual treatment decisions |
| Patients or caregivers | No patient-facing interface; output requires expert interpretation |
| Regulatory submission preparers | RESONANCE does not generate data suitable for regulatory filings (e.g., IND, NDA, 510(k)) without independent validation |
| Clinical trial designers | Output is qualitative; quantitative parameters (dosing, timing, endpoints) require independent pharmacokinetic modeling and clinical validation |

**Codebase references:**
- README.md lines 20–22: "Not a clinical tool", "Not a drug discovery pipeline", "Not a substitute for oncology"
- `CLAUDE.md` §Drug Models — Honest scope: "NOT clinical tools"
- `src/use_cases/experiments/pathway_inhibitor_exp.rs` line 1295: "DISCLAIMER: SIMULATED. NOT VETERINARY ADVICE."

## 3. Intended Use Environment

### 3.1 Computing Environment

| Parameter | Specification |
|-----------|---------------|
| Platform | Desktop workstation or HPC cluster (Linux, macOS, Windows) |
| Runtime | Rust stable 2024 edition (MSRV 1.85), Bevy 0.15 ECS engine |
| GPU | Optional (headless mode available: `cargo run --bin headless_sim -- --ticks N --scale S --out file.ppm`) |
| Network | None required. RESONANCE operates entirely offline. No telemetry, no cloud dependencies, no network calls. |
| Patient data | None. RESONANCE does not accept, process, store, or transmit patient data of any kind. |
| Input data | Simulation parameters only: axiom constants, population sizes, drug configurations (frequency, concentration, inhibition mode). All inputs are abstract (qe, Hz), not patient-derived. |
| Output data | Simulation reports: efficiency ratios, suppression percentages, growth rates, population counts — all in abstract units. Optionally: PPM image files (headless), CSV/JSON exports (`src/use_cases/export.rs`). |
| Determinism | Bit-exact reproducible output. Hash-based RNG with no external randomness source. Same seed produces identical results on any machine. |

### 3.2 Deployment Context

RESONANCE is deployed as source code (compiled locally by the user) or as a pre-compiled binary. It is not deployed as a service, SaaS product, cloud application, or embedded device software. There is no installer, no auto-update mechanism, and no user account system.

**Codebase references:**
- Headless mode: `src/bin/headless_sim.rs`
- Determinism: `src/blueprint/equations/determinism.rs` (23 tests — `hash_f32_slice`, `next_u64`, `unit_f32`, `gaussian_f32`)
- Batch simulator (no Bevy, rayon parallel): `src/batch/` (33 systems, 156+ tests)
- Export: `src/use_cases/export.rs` (9 tests, stateless CSV/JSON adapters)
- No network code: verified by absence of `tokio`, `reqwest`, `hyper`, or any networking crate in `Cargo.toml`; `CLAUDE.md` §Stack: "Async: None — Bevy schedule only, no tokio/async-std"

## 4. Use Context

### 4.1 Research-Only Context

RESONANCE is intended for use exclusively in research settings where:

1. **No patient is in the decision loop.** Simulation output informs research hypotheses, not treatment plans.
2. **Results require independent validation.** Any finding from RESONANCE must be validated through standard scientific methods (in vitro, in vivo, or clinical trial) before influencing any healthcare decision.
3. **Users understand the abstraction.** The energy-based model (qe) is a computational abstraction, not a biophysical measurement. Frequency is a proxy for genetic/epigenetic identity, not a measured biological observable.
4. **Disclaimers are visible.** The README, paper, and in-code comments explicitly state limitations.

### 4.2 Excluded Use Contexts

| Context | Status | Rationale |
|---------|--------|-----------|
| Point-of-care | EXCLUDED | No patient data input, no clinical output format, no real-time requirements |
| Clinical decision support (CDS) | EXCLUDED | Output not validated against patient outcomes (`README.md` line 141: "Against patient outcomes: Not yet") |
| Companion diagnostic | EXCLUDED | Does not analyze patient biomarkers or genomic data |
| Drug label claims | EXCLUDED | Abstract units (qe), not pharmacokinetic parameters (AUC, Cmax, t1/2) |
| Veterinary clinical use | EXCLUDED | Canine mast cell case (Experiment 7) is a simulation exercise, not veterinary guidance (`docs/paper/resonance_arxiv.tex` line 1116–1117: "simulated from press reports, not peer-reviewed clinical data. Not veterinary advice.") |
| Regulatory submission evidence | EXCLUDED | No GxP validation, no 21 CFR Part 11 compliance, no audit trail |

### 4.3 Clinical Calibration Profiles — Scope Clarification

RESONANCE includes 4 clinical calibration profiles that map abstract simulation units to real-world units (nM, days, cell count):

| Profile | Source | File Reference |
|---------|--------|----------------|
| CML / imatinib | Bozic et al. 2013 (eLife) | `src/blueprint/equations/clinical_calibration.rs` lines 44–57 |
| Prostate / abiraterone | Gatenby et al. 2009 (Cancer Research) | `src/blueprint/equations/clinical_calibration.rs` lines 59–72 |
| NSCLC / erlotinib | Published EGFR-mutant data | `src/blueprint/equations/clinical_calibration.rs` lines 74–87 |
| Canine MCT / toceranib | London et al. 2009 + press reports | `src/blueprint/equations/clinical_calibration.rs` lines 89–108 |

These profiles exist to demonstrate that the simulation's abstract output can be contextualized against published data. They do **not** constitute clinical validation. Specifically:

- Calibration maps simulation generations to published doubling times and IC50 values.
- Calibration does **not** validate that simulation predictions match patient-level outcomes.
- The Bozic 2013 comparison (Experiment 5) validates a qualitative prediction (combination > monotherapy) across 10 independent seeds, not quantitative resistance timelines.
- The canine MCT profile uses toceranib IC50 as a pharmacological proxy for an mRNA vaccine — a fundamentally different mechanism (kinase inhibition vs. immune-mediated killing).

**Codebase references:**
- Calibration profiles: `src/blueprint/equations/clinical_calibration.rs` (21 tests)
- Bozic validation: `src/use_cases/experiments/pathway_inhibitor_exp.rs` (31 tests), `src/bin/bozic_validation.rs`
- Paper limitations: `docs/paper/resonance_arxiv.tex` lines 1258–1272

## 5. What RESONANCE Is NOT

This section establishes explicit negative scope. Each exclusion is justified by a specific technical limitation of the codebase.

### 5.1 Not a Diagnostic Device

RESONANCE does not analyze patient samples, biomarkers, imaging, or any patient-derived data. It cannot identify, detect, or classify diseases. There is no input pathway for patient data and no output format compatible with clinical diagnostic workflows.

*Technical basis:* No patient data structures exist in the codebase. All entities are abstract energy configurations (`src/layers/energy.rs`: `BaseEnergy { qe: f32 }`). Input is simulation parameters only.

### 5.2 Not a Prescription or Treatment Selection Tool

RESONANCE does not recommend, suggest, or rank specific drugs, dosages, treatment schedules, or therapeutic regimens for any patient (human or animal). Drug models operate on abstract frequency/concentration pairs, not named pharmaceuticals with dosing schedules.

*Technical basis:* Drug definitions are abstract structs — `Inhibitor { freq: f32, concentration: f32, ki: f32, mode: InhibitionMode }` (`src/blueprint/equations/pathway_inhibitor.rs` lines 40–55). No drug database, no formulary, no contraindication logic.

### 5.3 Not a Substitute for Clinical Trials

Simulation results — including the Bozic 2013 replication and adaptive therapy controller — demonstrate qualitative consistency with published predictions but do not constitute clinical evidence. Specifically:

- Bozic comparison validates suppression percentages, not absolute cell counts or time-to-resistance in weeks.
- Adaptive controller stabilizes simulated growth rate, not measured tumor volume by RECIST criteria.
- No pharmacokinetic modeling (no ADME: absorption, distribution, metabolism, excretion).

*Technical basis:* `CLAUDE.md` §Drug Models — Honest scope: "Bozic comparison is qualitative (suppression %, not absolute cell counts or time-to-resistance in weeks)." Paper (`docs/paper/resonance_arxiv.tex` lines 1266–1272): "The drug model operates in abstract energy units (qe), not molar concentrations, and does not model ADME pharmacokinetics, tumor microenvironment (vasculature, hypoxia), or adaptive immune response."

### 5.4 Not a Molecular Simulator

RESONANCE does not model individual molecules, protein structures at atomic resolution, or chemical reactions with stoichiometric accuracy. Its energy model is an abstraction:

- "Frequency" is a computational proxy for genetic/epigenetic identity, not a measured spectral property.
- "Protein folding" uses a 2D HP lattice with 8 amino acid types — not comparable to AlphaFold or molecular dynamics.
- "Molecular bonding" uses Coulomb + Lennard-Jones classical potentials with constants derived from 4 fundamentals — not quantum mechanical calculations.

*Technical basis:* Protein fold: `src/blueprint/equations/protein_fold.rs` (27 tests, 2D HP lattice). Coulomb/LJ: `src/blueprint/equations/coulomb.rs` (26 tests). Paper line 1259: "The protein folding model is a 2D HP lattice with 8 amino acid types, far simpler than real 3D protein structure prediction."

### 5.5 Not a Pharmacokinetic Model

RESONANCE does not model drug absorption, distribution, metabolism, or excretion (ADME). Drug concentration is a static parameter per experiment, not a time-varying compartmental model. There is no half-life calculation, no bioavailability parameter, no first-pass effect, and no renal/hepatic clearance.

*Technical basis:* Drug concentration is a constant field in `Inhibitor` struct (`src/blueprint/equations/pathway_inhibitor.rs`). Hill response function (`hill_response`, line 114) takes `effective_concentration` as a static input. No time-decay of drug levels exists in the codebase.

### 5.6 Not a Tumor Microenvironment Model

RESONANCE does not model vasculature, angiogenesis, hypoxia gradients, stromal interactions, immune cell infiltration, or extracellular matrix dynamics. Tumors are modeled as homogeneous populations of energy entities with frequency-based heterogeneity.

*Technical basis:* Paper line 1267–1269: "does not model ADME pharmacokinetics, tumor microenvironment (vasculature, hypoxia), or adaptive immune response." No immune system components exist in the 14 ECS layers (`src/layers/mod.rs`). Paper future work (line 1282): "Immune system: Model T-cell, NK-cell interactions as frequency-selective predation" — listed as unimplemented.

## 6. IMDRF SaMD Classification

### 6.1 Framework Application

The International Medical Device Regulators Forum (IMDRF) Software as a Medical Device (SaMD) framework classifies software based on two dimensions. The foundational definitions come from IMDRF/SaMD WG/N10:2013 (definitions), while the risk categorization matrix is specified in IMDRF/SaMD WG/N12R2:2014 (risk categories). Both are applied below:

1. **State of healthcare situation** — the seriousness of the condition the software addresses.
2. **Significance of information provided by the SaMD to the healthcare decision** — whether the software's output is used to treat, diagnose, drive clinical management, or inform clinical management.

### 6.2 Classification Analysis

#### Significance of Information to Healthcare Decision

| Level | Definition (IMDRF N10) | Applicability to RESONANCE |
|-------|------------------------|---------------------------|
| **Treat or diagnose** | Software provides treatment or diagnosis | Not applicable. RESONANCE does not treat or diagnose. |
| **Drive clinical management** | Software output drives a clinical intervention without clinician review | Not applicable. RESONANCE has no clinical output format and no integration with clinical systems. |
| **Inform clinical management** | Software output informs a clinician who then makes the decision | Not applicable *when used as intended*. Output is for research hypothesis generation, not clinical management. |
| **Inform** (non-clinical) | Software provides information that does not directly inform clinical decisions | **This level applies.** RESONANCE informs research directions — e.g., "combination therapy may be more effective than monotherapy for this resistance profile." |

**Determination:** The significance of RESONANCE's information to healthcare decisions is **"Inform"** (lowest level) — specifically, informing research planning and hypothesis generation, not clinical management.

#### State of Healthcare Situation

| State | Definition (IMDRF N10) | Applicability to RESONANCE |
|-------|------------------------|---------------------------|
| **Critical** | Situation or condition that poses imminent risk to life | Not applicable. RESONANCE does not operate in critical care contexts. |
| **Serious** | Situation or condition that requires medical/surgical intervention to prevent impairment | Not applicable. RESONANCE is not used in treatment contexts. |
| **Non-serious** | Situation or condition where failure to act is not expected to cause harm | **This level applies.** RESONANCE is used in research planning where incorrect output leads to wasted research effort, not patient harm. |

**Determination:** The state of healthcare situation is **"Non-serious"**.

### 6.3 IMDRF Risk Category

Per the IMDRF SaMD N12R2:2014 risk categorization matrix:

| | Inform | Drive | Diagnose | Treat |
|---|--------|-------|----------|-------|
| **Non-serious** | **Category I** | Category II | Category II | Category II |
| **Serious** | Category II | Category III | Category III | Category III |
| **Critical** | Category III | Category III | Category IV | Category IV |

**RESONANCE classification: Category I** (lowest risk) — when used as intended (research-only, no patient in the decision loop).

### 6.4 Misuse Reclassification Warning

If RESONANCE output were used to directly inform clinical decisions (e.g., a clinician selecting a combination therapy protocol based solely on simulation results), the classification would shift:

| Misuse Scenario | Reclassified Significance | Reclassified State | New Category |
|-----------------|--------------------------|-------------------|--------------|
| Inform oncology treatment planning | Inform clinical management | Serious | **Category II** |
| Drive drug selection without clinician review | Drive clinical management | Serious | **Category III** |
| Select veterinary treatment protocol | Inform clinical management | Serious | **Category II** |

To mitigate misuse risk, RESONANCE implements the following controls:

| Control | Location | Mechanism |
|---------|----------|-----------|
| README disclaimers | `README.md` lines 18–22 | "Not a clinical tool", "Not a drug discovery pipeline", "Not a substitute for oncology" |
| In-code disclaimers | `src/blueprint/equations/clinical_calibration.rs` line 92 | "DISCLAIMER: Calibrated from press reports [...] NOT from peer-reviewed trial data" |
| In-code disclaimers | `src/use_cases/experiments/pathway_inhibitor_exp.rs` line 1295 | "DISCLAIMER: SIMULATED. NOT VETERINARY ADVICE." |
| Paper limitations | `docs/paper/resonance_arxiv.tex` lines 1258–1272 | Full limitations paragraph: abstract units, no ADME, no TME, qualitative only |
| Paper disclaimers | `docs/paper/resonance_arxiv.tex` lines 1116–1117 | "simulated from press reports, not peer-reviewed clinical data. Not veterinary advice." |
| CLAUDE.md scope | `CLAUDE.md` §Drug Models — Honest scope | "NOT clinical tools. Bozic comparison is qualitative" |
| Validation table | `README.md` line 141 | "Against patient outcomes: Not yet" |

### 6.5 Regulatory Consequence

As a **Category I** research tool:

- RESONANCE is **not classified as a medical device** under IMDRF SaMD N10 when used within its intended use.
- No premarket review (FDA 510(k), De Novo, PMA) is required for research-only use.
- No CE marking under EU MDR is required for research-only use.
- Documentation in this regulatory track is maintained as **best practice** for transparency and credibility, not as a regulatory obligation.

Should RESONANCE's intended use evolve to include clinical decision support, a formal regulatory pathway assessment (FDA, EU MDR, or equivalent) would be required before any such use.

## 7. Codebase References

All claims in this document are traceable to specific files, tests, or outputs in the RESONANCE codebase at commit `971c7acb99decde45bf28860e6e10372718c51e2`.

| Claim | Reference | Verification |
|-------|-----------|--------------|
| 8 axioms, 4 fundamental constants | `CLAUDE.md` §The 8 Foundational Axioms, §The 4 Fundamental Constants | Read document |
| 113K LOC | `src/` directory, measured via `wc -l` on all `.rs` files | `wc -l $(find src -name '*.rs') | tail -1` → 106,002 lines (source only, excluding tests/ and docs/) |
| 3,113 automated tests (0 failures) | `cargo test` output | `cargo test` → 3,113 tests passed, 0 failures, 35.78s |
| Bit-exact determinism | `src/blueprint/equations/determinism.rs` | 23 unit tests verify hash-based RNG produces identical output across runs |
| Pathway inhibitor: 11 pure fns, 42 tests | `src/blueprint/equations/pathway_inhibitor.rs` | `#[test]` count: 42 in file |
| 3 inhibition modes | `src/blueprint/equations/pathway_inhibitor.rs` — `InhibitionMode` enum | Competitive, Noncompetitive, Uncompetitive |
| Bozic 2013 validated 10/10 seeds | `src/bin/bozic_validation.rs`, `src/use_cases/experiments/pathway_inhibitor_exp.rs` | `cargo run --release --bin bozic_validation` → 10/10 confirm combo > mono |
| 4 calibration profiles | `src/blueprint/equations/clinical_calibration.rs` | CML_IMATINIB, PROSTATE_ABIRATERONE, NSCLC_ERLOTINIB, CANINE_MAST_CELL constants |
| Clinical calibration tests | `src/blueprint/equations/clinical_calibration.rs` | 21 unit tests |
| Batch simulator: 33 systems, no Bevy | `src/batch/` (19 files) | rayon parallel; no `use bevy` in batch modules |
| Headless sim | `src/bin/headless_sim.rs` | `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm` |
| No unsafe | Codebase-wide | `grep -r "unsafe" src/ --include="*.rs"` yields zero occurrences in runtime code (per `CLAUDE.md` Hard Block #1) |
| No networking crates | `Cargo.toml` | No tokio, reqwest, hyper, or equivalent |
| Paper published | https://zenodo.org/records/19342036 | DOI: 10.5281/zenodo.19342036 |
| AGPL-3.0 license | `LICENSE` file, `Cargo.toml` | Repository root |
| Abstract qe units | `src/layers/energy.rs` — `BaseEnergy { qe: f32 }` | L0 layer definition |
| No patient data structures | All 14 layers in `src/layers/` | No PII, PHI, or patient-identifiable fields in any component |

## 8. Revision History

| Version | Date | Author | Change Description |
|---------|------|--------|--------------------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. Establishes intended use, user profiles, IMDRF Category I classification, and negative scope. All claims traced to codebase at commit `971c7ac`. |
