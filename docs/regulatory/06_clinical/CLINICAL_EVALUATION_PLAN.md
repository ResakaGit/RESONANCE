---
document_id: RD-6.1
title: Clinical Evaluation Plan
standard: IMDRF SaMD N41, EU MDR Annex XIV
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Clinical Evaluation Plan

## 1. Purpose

This document defines the plan for evaluating RESONANCE's clinical evidence in the context of its classification as an IEC 62304 Class A research tool (RD-1.2) and IMDRF SaMD Category I (RD-1.1). Although RESONANCE is not a medical device and clinical evaluation is not a regulatory obligation at this classification level, the team voluntarily applies the IMDRF SaMD N41 (Clinical Evaluation of Software as a Medical Device) framework to establish credibility, identify evidence gaps, and prepare for a potential future regulatory pathway.

This plan governs the production of:

| Document | ID | Purpose |
|----------|-----|---------|
| Clinical Evaluation Report | RD-6.2 | Synthesizes all evidence against this plan |
| Limitations and Scope Report | RD-6.3 | Defines what RESONANCE can and cannot do |
| Reproducibility Protocol | RD-6.4 | Enables independent verification of all claims |
| Reference Data Registry | RD-6.5 | Catalogs all external data sources |

**Cross-references:**

- RD-1.1 (Intended Use Statement): Defines research-only scope, excluded users, IMDRF Category I
- RD-1.2 (Software Safety Classification): IEC 62304 Class A
- RD-1.5 (Regulatory Strategy): Voluntary compliance rationale
- RD-2.1 (Risk Management Plan): Hazard identification and risk acceptability

## 2. Scope

### 2.1 Product Under Evaluation

RESONANCE v1.0, commit `971c7acb99decde45bf28860e6e10372718c51e2`.

| Parameter | Value |
|-----------|-------|
| Lines of code | ~113K |
| Automated tests | 3,113 (0 failures) |
| Language / Engine | Rust 2024 edition / Bevy 0.15 |
| License | AGPL-3.0 |
| Paper | https://zenodo.org/records/19342036 |
| Safety class | IEC 62304 Class A |
| IMDRF category | Category I (research tool) |

### 2.2 Claims Under Evaluation

The clinical evaluation addresses three categories of claims made by RESONANCE:

| Claim ID | Claim | Source |
|----------|-------|--------|
| C-1 | The simulation engine produces deterministic, conservation-correct results from 8 axioms and 4 fundamental constants | `CLAUDE.md`, `docs/paper/resonance_arxiv.tex` |
| C-2 | Combination therapy suppresses metabolic efficiency more than monotherapy or doubled monotherapy (qualitative Bozic 2013 replication) | `src/use_cases/experiments/pathway_inhibitor_exp.rs`, `src/bin/bozic_validation.rs` |
| C-3 | Pathway inhibition produces monotonic dose-response behavior | `src/use_cases/experiments/pathway_inhibitor_exp.rs` (Experiment 4) |
| C-4 | An adaptive therapy controller can stabilize tumor growth at zero | `src/use_cases/experiments/pathway_inhibitor_exp.rs` (Experiment 6) |
| C-5 | Calibration profiles can contextualize abstract simulation output against published clinical parameters | `src/blueprint/equations/clinical_calibration.rs` |
| C-6 | Canine mast cell simulation predicts partial response consistent with observed Rosie case | `src/blueprint/equations/clinical_calibration.rs` (Experiment 7) |

### 2.3 Claims NOT Made

The following are explicitly excluded from the evaluation scope because RESONANCE does not make these claims:

- Quantitative prediction of tumor volume reduction (RECIST criteria)
- Time-to-resistance in calendar units (weeks, months)
- Patient-level outcome prediction
- Drug dosing recommendations in pharmacokinetic units (mg/kg, AUC, Cmax)
- Molecular target identification (no EGFR, BCR-ABL, PD-L1)
- Immune system response prediction
- Tumor microenvironment modeling

## 3. Evidence Types

### 3.1 Type 1: Analytical Evidence (Valid Scientific Evidence — Verification)

Analytical evidence demonstrates that the software performs its intended computations correctly. For RESONANCE, this means verifying that the simulation engine correctly implements the 8 axioms and 4 fundamental constants without numerical errors, conservation violations, or nondeterminism.

#### 3.1.1 Automated Test Suite

| Metric | Value | Verification command |
|--------|-------|---------------------|
| Total tests | 3,113 | `cargo test` |
| Failures | 0 | `cargo test` output: "0 failed" |
| Execution time | ~36 seconds | `cargo test` wall time |
| Coverage domains | Blueprint equations, batch systems, layers, simulation, use cases, worldgen | `src/` directory structure |

Key test categories relevant to clinical evaluation:

| Category | Test count (approx.) | Files |
|----------|---------------------|-------|
| Pathway inhibitor pure math | 32+ | `src/blueprint/equations/pathway_inhibitor.rs` |
| Pathway inhibitor experiment | 18+ | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Derived thresholds (4 constants) | 17 | `src/blueprint/equations/derived_thresholds.rs` |
| Clinical calibration | 21 | `src/blueprint/equations/clinical_calibration.rs` |
| Determinism (RNG, hashing) | 23 | `src/blueprint/equations/determinism.rs` |
| Conservation (property-based) | ~20 | `tests/property_conservation.rs` |

#### 3.1.2 Axiom Compliance

Each of the 8 axioms is verified by dedicated tests. The axiom compliance matrix is maintained in `CLAUDE.md` and enforced by the test suite. Specifically:

- **Axiom 1 (Everything is Energy):** All entities use `BaseEnergy { qe: f32 }` — verified by type system.
- **Axiom 2 (Pool Invariant):** `conservation_error` and `global_conservation_error` fuzzed by proptest in `tests/property_conservation.rs`.
- **Axiom 4 (Dissipation):** Dissipation rates tested in `src/blueprint/equations/derived_thresholds.rs`.
- **Axiom 5 (Conservation):** Total energy monotonically decreases — property-tested with arbitrary inputs, no violations found.
- **Axiom 7 (Distance Attenuation):** Monotonic decrease verified in spatial interaction tests.
- **Axiom 8 (Oscillatory Nature):** Frequency alignment via `gaussian_frequency_alignment` tested in `src/blueprint/equations/determinism.rs`.

#### 3.1.3 Conservation Fuzz Testing

Property-based tests (proptest) in `tests/property_conservation.rs` generate arbitrary valid inputs and verify:

- `is_valid_qe` returns true for all finite non-negative f32
- `has_invalid_values` detects NaN/Inf
- `global_conservation_error` remains within tolerance for all extraction strategies
- Pool equations maintain `sum(children) <= parent` invariant
- Extraction functions (`extract_proportional`, `extract_competitive`, `extract_greedy`, `extract_regulated`, `extract_aggressive`) never produce negative energy

No conservation violations have been found in arbitrary input fuzzing.

#### 3.1.4 Determinism Verification

Hash-based RNG (`src/blueprint/equations/determinism.rs`) produces bit-exact output across runs. Verification: run any experiment binary twice and compare output byte-for-byte. The protocol is specified in RD-6.4 (Reproducibility Protocol).

### 3.2 Type 2: Performance Evidence (Valid Scientific Evidence — Validation)

Performance evidence demonstrates that RESONANCE's output is consistent with established scientific predictions in the domain of therapeutic resistance dynamics. This is the primary evidence type for clinical evaluation.

#### 3.2.1 Experiment 4: Pathway Inhibition Dose-Response

| Parameter | Value |
|-----------|-------|
| Design | Single-arm dose escalation |
| Drug model | Level 2 pathway inhibitor (Competitive mode) |
| Metric | Metabolic efficiency ratio (treatment/control) |
| Seeds | 10 independent |
| Prediction | Monotonic dose-response (higher dose = lower efficiency) |
| Source file | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |

Expected results:

| Condition | Efficiency | Suppression |
|-----------|-----------|-------------|
| Control (no drug) | 1.000 | 0% |
| Concentration 0.4 | ~0.488 | ~51.2% |
| Concentration 0.8 | ~0.471 | ~52.9% |

Gold standard comparator: General pharmacological dose-response theory (Hill equation). The test verifies that RESONANCE's pathway inhibitor produces monotonically increasing suppression with concentration, consistent with Hill pharmacokinetics (n=2).

#### 3.2.2 Experiment 5: Bozic 2013 Replication (5-Arm Protocol)

| Parameter | Value |
|-----------|-------|
| Design | 5-arm parallel (no_drug, mono_A, mono_B, combo_AB, double_A) |
| Drug model | Level 2 pathway inhibitor, two drugs with different target frequencies |
| Metric | Final metabolic efficiency ratio |
| Seeds | 10 independent (10/10 confirm) |
| Prediction | combo_AB < mono_A < no_drug; combo_AB < double_A |
| Binary | `cargo run --release --bin bozic_validation` |
| Source file | `src/use_cases/experiments/pathway_inhibitor_exp.rs` (function `run_bozic_validation`) |

Expected results:

| Arm | Efficiency | Suppression | Interpretation |
|-----|-----------|-------------|----------------|
| no_drug | 1.000 | 0% | Baseline |
| mono_A (400 Hz) | 0.481 | 51.9% | Single-target suppression |
| mono_B (300 Hz) | 0.635 | 36.5% | Single-target suppression (different frequency) |
| combo_AB | 0.435 | 56.5% | Combination > either monotherapy |
| double_A (2x conc) | 0.466 | 53.4% | Doubling dose < combination |

Gold standard comparator: Bozic et al. 2013 (eLife 2:e00747, DOI: 10.7554/eLife.00747). The published prediction is that combination therapy targeting independent resistance mechanisms provides an exponential advantage over monotherapy. RESONANCE replicates the qualitative ordering (combo > mono, combo > double dose) in 10/10 independent seeds. This is a structural result — it holds regardless of seed choice.

**Limitation:** The comparison is qualitative. RESONANCE reports suppression percentages, not absolute cell counts, resistance mutation rates, or time-to-resistance in weeks. The Bozic model uses explicit resistance mutation probability; RESONANCE derives resistance from frequency-based metabolic compensation.

#### 3.2.3 Experiment 6: Adaptive Therapy Controller

| Parameter | Value |
|-----------|-------|
| Design | Closed-loop controller: profile, attack, predict escape, close, adapt |
| Drug model | Level 2 pathway inhibitor, dynamically added/removed drugs |
| Metric | Growth rate stabilization at zero |
| Seeds | 10 independent (7/10 achieve stability) |
| Binary | `cargo run --release --bin adaptive_therapy` |
| Source file | `src/use_cases/experiments/pathway_inhibitor_exp.rs` (function `run_adaptive`) |

Gold standard comparator: Gatenby et al. 2009 (Cancer Research 69:4894, DOI: 10.1158/0008-5472.CAN-08-3658). Adaptive therapy concept: modulate drug pressure to maintain sensitive population competing with resistant cells. RESONANCE demonstrates that a feedback controller can stabilize growth rate, consistent with the adaptive therapy hypothesis.

**Limitation:** 7/10 seeds achieve stability (not 10/10). The controller operates on abstract efficiency metrics, not tumor volume by RECIST criteria. No pharmacokinetic time delay is modeled.

#### 3.2.4 Experiment 7: Canine Mast Cell Tumor (Rosie Case)

| Parameter | Value |
|-----------|-------|
| Design | Single-target treatment of heterogeneous tumor |
| Drug model | Level 2 pathway inhibitor as proxy for mRNA vaccine |
| Metric | Tumor suppression (responsive fraction), resistant fraction persistence |
| Seeds | 5 independent |
| Calibration | `CANINE_MAST_CELL` profile (21-day doubling, 40 nM toceranib IC50) |
| Source file | `src/blueprint/equations/clinical_calibration.rs` |

Expected results:

| Observation | Simulation | Published/Reported |
|-------------|-----------|-------------------|
| Mono-target suppression | 50-70% | ~75% tumor reduction (press reports, March 2026) |
| Resistant fraction persists | Yes | Yes (incomplete response) |
| Combo predicted more effective | Yes | Not yet tested clinically |

Gold standard comparator: London et al. 2003 (JAAHA 39:489), London et al. 2009 (Vet Comp Oncology 7:31). Toceranib IC50 and mast cell doubling time from peer-reviewed veterinary oncology literature. Observed outcome from press reports (not peer-reviewed).

**Limitation:** The mRNA vaccine mechanism (immune-mediated killing) is fundamentally different from kinase inhibition (toceranib). RESONANCE uses toceranib IC50 as a pharmacological proxy. The simulation models metabolic suppression, not immune-mediated tumor clearance. "Press reports" are not peer-reviewed data. This experiment is a simulation exercise, not veterinary validation.

### 3.3 Type 3: Clinical Evidence (Literature-Based Contextualization)

Clinical evidence in the IMDRF SaMD N41 sense refers to evidence that the SaMD's output produces clinically meaningful outcomes. For a Category I research tool, this evidence type is not required but is included for completeness and credibility.

RESONANCE does not have prospective clinical evidence. Instead, it provides retrospective contextualization through calibration profiles that map abstract simulation output to published clinical parameters.

#### 3.3.1 Calibration Profiles

Four calibration profiles bridge abstract simulation units to clinical reality:

| Profile | Doubling Time | IC50 | Source DOI | Source file lines |
|---------|--------------|------|------------|-------------------|
| CML / imatinib | 4 days | 260 nM | 10.7554/eLife.00747 | `clinical_calibration.rs:51-57` |
| Prostate / abiraterone | 30 days | 5.1 nM | 10.1158/0008-5472.CAN-08-3658 | `clinical_calibration.rs:66-72` |
| NSCLC / erlotinib | 7 days | 20 nM | Standard EGFR-mutant literature | `clinical_calibration.rs:81-87` |
| Canine MCT / toceranib | 21 days | 40 nM | London 2003 (JAAHA), London 2009 (Vet Comp Oncol) | `clinical_calibration.rs:102-108` |

These profiles demonstrate that RESONANCE's abstract output can be placed in clinical context. They do **not** constitute clinical validation. Specifically:

- Calibration maps simulation generations to published doubling times and IC50 values.
- Calibration does **not** validate that simulation predictions match patient-level outcomes.
- Each profile includes 21 unit tests verifying conversion correctness (e.g., "CML generation 10 = day 40").

#### 3.3.2 Biological Scaling

The Kleiber exponent (0.75) used throughout RESONANCE is grounded in:

- Kleiber M. (1947). Body size and metabolic rate. *Physiological Reviews* 27:511-541.
- West GB, Brown JH, Enquist BJ. (1997). A general model for the origin of allometric scaling laws in biology. *Science* 276:122-126. DOI: 10.1126/science.276.5309.122.

This is a biological universal validated across 27 orders of magnitude (bacteria to whales). Its use in RESONANCE is well-founded.

## 4. Gold Standard Comparators

| Comparator | Used for | Type | Limitation |
|------------|----------|------|------------|
| Bozic et al. 2013 | Experiment 5 (combo vs mono) | Published computational model (eLife) | Qualitative comparison only; different underlying mechanism (explicit mutation probability vs. frequency compensation) |
| Gatenby et al. 2009 | Experiment 6 (adaptive therapy) | Published clinical concept (Cancer Research) | RESONANCE tests the concept abstractly; Gatenby tested with real patients |
| Hill equation (general pharmacology) | Experiment 4 (dose-response) | Established pharmacological theory | RESONANCE uses Hill with n=2 for all drugs; real drugs have variable Hill coefficients |
| London et al. 2003, 2009 | Experiment 7 (canine MCT) | Published veterinary oncology data | Toceranib used as proxy for mRNA vaccine; different mechanism of action |
| Kleiber 1947, West et al. 1997 | Biological scaling (Kleiber exponent) | Established biological universal | Applied to abstract energy entities, not biological organisms |

## 5. Evidence Gaps

### 5.1 No Patient-Level Outcome Data

RESONANCE has not been validated against any patient-level clinical dataset. All comparisons are against published aggregate predictions (Bozic: qualitative ordering; Gatenby: conceptual framework). This is the single largest evidence gap.

**Impact:** Cannot claim clinical predictive validity.

**Mitigation:** Intended use explicitly excludes clinical decision-making (RD-1.1). All outputs carry disclaimers.

**Resolution path:** Would require collaboration with clinical trial teams to compare simulation predictions against longitudinal patient data. Not planned for the current version.

### 5.2 No Prospective Studies

All evidence is retrospective (comparing against published results) or theoretical (axiom verification). No prospective predictions have been made and subsequently validated.

**Impact:** Cannot claim predictive power beyond retrospective consistency.

**Mitigation:** The Rosie case (Experiment 7) represents a near-prospective exercise (simulation conducted shortly after press reports), but the outcome was already known.

### 5.3 No Independent Replication

All experiments have been conducted by the development team. No independent laboratory or research group has replicated any RESONANCE experiment.

**Impact:** Cannot rule out systematic bias in experiment design or interpretation.

**Mitigation:** RD-6.4 (Reproducibility Protocol) provides exact commands for independent replication. The codebase is open source (AGPL-3.0). Deterministic output means any researcher running the same commands will get bit-identical results.

### 5.4 No Pharmacokinetic Modeling

RESONANCE does not model drug absorption, distribution, metabolism, or excretion (ADME). Drug concentration is a static parameter, not a time-varying compartmental model.

**Impact:** Cannot model drug half-life, bioavailability, or time-varying drug exposure.

**Mitigation:** Explicitly documented as a limitation in `CLAUDE.md`, paper, and RD-6.3.

### 5.5 No Tumor Microenvironment

RESONANCE does not model vasculature, hypoxia, immune infiltration, or stromal interactions.

**Impact:** Results may not hold when TME dynamics dominate treatment response.

**Mitigation:** Explicitly documented. See RD-6.3 for detailed failure condition analysis.

### 5.6 No Immune System

RESONANCE does not model T-cells, NK cells, macrophages, or any immune component. This is particularly relevant to Experiment 7 (Rosie case), where the actual treatment mechanism is immune-mediated (mRNA vaccine).

**Impact:** The Rosie case simulation uses kinase inhibition as a proxy for immune-mediated killing — a fundamentally different mechanism.

**Mitigation:** Documented as a known limitation in `src/blueprint/equations/clinical_calibration.rs` line 98 and RD-6.3.

## 6. Literature Search Strategy

### 6.1 Databases Searched

| Database | Search date | Query terms |
|----------|------------|-------------|
| PubMed | 2026-03-31 | "combination therapy resistance" AND ("computational model" OR "mathematical model") |
| Google Scholar | 2026-03-31 | "Bozic 2013" AND "combination therapy" AND "resistance mutation" |
| eLife | 2026-03-31 | Bozic et al. 2013 — direct access |
| Cancer Research (AACR) | 2026-03-31 | Gatenby 2009 adaptive therapy |
| JAAHA / Vet Comp Oncology | 2026-03-31 | London mast cell toceranib |
| Science | 2026-03-31 | West Brown Enquist 1997 allometric scaling |

### 6.2 Inclusion Criteria

- Published in peer-reviewed journal
- Contains quantitative data on therapeutic resistance dynamics, dose-response, or combination therapy advantage
- Parameters extractable for calibration (doubling time, IC50, mutation rate)

### 6.3 Exclusion Criteria

- Molecular dynamics or atomic-resolution simulations (different abstraction level)
- Patient cohort studies without extractable parameters
- Studies published after simulation experiments were conducted (to avoid post-hoc selection bias)

### 6.4 Literature Appraisal Method

Each reference is assessed using the IMDRF SaMD N41 relevance criteria:

1. **Applicability:** Does the reference address the same clinical question as the RESONANCE experiment?
2. **Quality:** Is the reference peer-reviewed? What is the journal impact factor and citation count?
3. **Consistency:** Are the reference's findings consistent across independent studies?
4. **Directness:** Does the reference provide data directly comparable to RESONANCE output?

Results of this appraisal are documented in RD-6.5 (Reference Data Registry).

## 7. Evaluation Schedule

| Phase | Activity | Deliverable | Status |
|-------|----------|-------------|--------|
| Phase 1 | Analytical evidence compilation | RD-6.2 §3 | This sprint |
| Phase 2 | Performance evidence compilation | RD-6.2 §4 | This sprint |
| Phase 3 | Clinical evidence compilation | RD-6.2 §5 | This sprint |
| Phase 4 | Limitations assessment | RD-6.3 | This sprint |
| Phase 5 | Reproducibility protocol | RD-6.4 | This sprint |
| Phase 6 | Reference data registry | RD-6.5 | This sprint |
| Phase 7 | Benefit-risk assessment | RD-6.2 §6 | This sprint |
| Phase 8 | Gap closure plan | RD-6.2 §8 | Future sprint (if needed) |

## 8. Acceptance Criteria

The clinical evaluation is considered complete when:

1. All 6 claims (C-1 through C-6) have been evaluated against available evidence.
2. Each evidence type (analytical, performance, clinical) has been documented with quantitative results.
3. All evidence gaps have been identified, assessed for impact, and assigned mitigation measures.
4. The benefit-risk assessment is favorable for the intended use (research tool).
5. The reproducibility protocol enables independent verification of all quantitative claims.
6. All external data sources are cataloged with DOI, license, and integrity assessment.

## 9. Codebase References

| Claim | File | Verification |
|-------|------|--------------|
| 3,113 tests | `cargo test` output | Run `cargo test` and count "test result: ok. N passed" |
| Bozic validation 10/10 | `src/bin/bozic_validation.rs` | `cargo run --release --bin bozic_validation` |
| Adaptive therapy 7/10 | `src/use_cases/experiments/pathway_inhibitor_exp.rs` | `cargo run --release --bin adaptive_therapy` |
| Clinical calibration 21 tests | `src/blueprint/equations/clinical_calibration.rs` | `cargo test clinical_calibration` |
| Conservation fuzz | `tests/property_conservation.rs` | `cargo test property_conservation` |
| Determinism | `src/blueprint/equations/determinism.rs` | Run any binary twice, diff output |
| Pathway inhibitor 32 tests | `src/blueprint/equations/pathway_inhibitor.rs` | `cargo test pathway_inhibitor` |
| Derived thresholds 17 tests | `src/blueprint/equations/derived_thresholds.rs` | `cargo test derived_thresholds` |

## 10. Revision History

| Version | Date | Author | Change Description |
|---------|------|--------|--------------------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial clinical evaluation plan. Three evidence types defined, 6 claims identified, 6 evidence gaps documented, literature search strategy established. All claims traced to codebase at commit `971c7ac`. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
