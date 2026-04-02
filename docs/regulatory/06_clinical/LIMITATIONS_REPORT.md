---
document_id: RD-6.3
title: Limitations and Scope Report
standard: ASME V&V 40, Scientific Ethics
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Limitations and Scope Report

## 1. Purpose

This document provides a comprehensive, honest accounting of what RESONANCE can do, what it cannot do, and the assumptions underlying its computational model. It is written for three audiences:

1. **Researchers** evaluating whether RESONANCE is appropriate for their study
2. **Reviewers** assessing the credibility of RESONANCE-derived findings
3. **Future developers** deciding whether to extend the model into clinical territory

This document follows the ASME V&V 40 framework (Verification and Validation in Computational Modeling of Medical Devices) for context of use assessment, and adheres to scientific ethics principles requiring transparent disclosure of model limitations.

**Cross-references:**

- RD-6.1 (Clinical Evaluation Plan): Defines claims and evidence gaps
- RD-6.2 (Clinical Evaluation Report): Quantitative evidence and benefit-risk assessment
- RD-1.1 (Intended Use Statement): Research-only scope
- `CLAUDE.md` Drug Models: Honest scope section

## 2. Scope Matrix

### 2.1 CAN Do (Validated Capabilities)

| Capability | Evidence | Confidence | Source files |
|-----------|----------|------------|-------------|
| Simulate emergent life from 8 axioms and 4 constants | 3,113 tests, axiom compliance verified | High | `src/blueprint/equations/derived_thresholds.rs` (17 tests) |
| Deterministic bit-exact reproduction | Hash-based RNG, 23 tests, run-twice verification | High | `src/blueprint/equations/determinism.rs` |
| Conserve energy (no creation, only transfer/dissipation) | Property-based fuzzing with arbitrary inputs, 0 violations | High | `tests/property_conservation.rs` |
| Model pathway inhibition with 3 modes (Competitive, Noncompetitive, Uncompetitive) | 32+ tests, dose-response monotonic in 10/10 seeds | High | `src/blueprint/equations/pathway_inhibitor.rs` |
| Demonstrate combo > mono qualitative ordering | 10/10 seeds, structural result | High | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Stabilize growth via adaptive feedback controller | 7/10 seeds achieve zero growth rate | Moderate | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Map abstract units to clinical parameters via calibration | 4 profiles, 21 tests, published DOIs | High | `src/blueprint/equations/clinical_calibration.rs` |
| Produce monotonic dose-response curves | Hill pharmacokinetics n=2, verified across doses | High | `src/blueprint/equations/pathway_inhibitor.rs` |
| Run headless (no GPU) with PPM image output | Binary verified | High | `src/bin/headless_sim.rs` |
| Export results as CSV/JSON | Stateless adapters, 9 tests | High | `src/use_cases/export.rs` |

### 2.2 CANNOT Do (Explicit Exclusions)

| Capability gap | Consequence | Mitigation | Severity |
|---------------|-------------|------------|----------|
| Cannot model molecular targets (EGFR, BCR-ABL, PD-L1) | Cannot predict target-specific drug responses | Use frequency as abstract proxy only; do not claim molecular specificity | High |
| Cannot model pharmacokinetics (ADME) | Drug concentration is static; no half-life, no bioavailability, no drug-drug interaction via metabolism | Document as limitation; do not interpret results as PK predictions | High |
| Cannot model tumor microenvironment (vasculature, hypoxia, ECM) | Results invalid when TME dynamics dominate response | Limit claims to population-level dynamics; flag TME-dependent scenarios | High |
| Cannot model immune system (T-cells, NK cells, macrophages) | Cannot predict immunotherapy response; Rosie case mechanism mismatch | Use kinase inhibitor as proxy with documented caveats | High |
| Cannot predict patient-level outcomes | Cannot generate data for clinical decision-making | Research-only intended use; pervasive disclaimers | Critical |
| Cannot produce RECIST-compatible tumor measurements | Efficiency ratio is not tumor volume | Do not equate efficiency reduction with tumor shrinkage | Medium |
| Cannot model spatial tumor heterogeneity | Drug experiments use well-mixed population, not spatially structured tumor | Acknowledge homogeneous mixing assumption | Medium |
| Cannot model resistance via point mutations explicitly | Uses frequency escape (metabolic compensation) instead | Document mechanism difference when comparing with mutation-based models | Medium |
| Cannot model multi-drug pharmacokinetic interactions | Bliss independence for combos; no metabolic drug-drug interaction | Document Bliss assumption; do not claim PK interaction modeling | Medium |
| Cannot model quiescent stem cell dynamics beyond Level 1 | Level 1 (cytotoxic) has quiescent escape; Level 2 (pathway inhibitor) does not | Document per-level scope | Low |
| Cannot generate regulatory-grade evidence (IND, NDA, 510(k)) | Not GxP validated, no 21 CFR Part 11, no audit trail beyond Git | Research-only output; regulatory submission requires independent validation | High |

### 2.3 Assumptions

| Assumption | Consequence if violated | Testability | Source |
|-----------|------------------------|-------------|--------|
| Everything is energy (Axiom 1) | If real biological systems require non-energy state variables (e.g., spatial configuration, epigenetic marks independent of energy), model output diverges from biology | Not directly testable; philosophical assumption | `CLAUDE.md` Axiom 1 |
| Frequency is a valid proxy for genetic/epigenetic identity | If two biologically distinct cell types happen to have the same "frequency," model cannot distinguish them | Tested indirectly via calibration profiles; acknowledged as limitation | `src/blueprint/equations/clinical_calibration.rs` |
| Drug binding is determined by frequency alignment (Gaussian) | If real binding affinity depends on structural complementarity rather than a single scalar, model is too simple | Not testable within RESONANCE; requires molecular validation | `src/blueprint/equations/pathway_inhibitor.rs` |
| Hill coefficient n=2 for all drugs | Real drugs have variable Hill coefficients (n=1 to n=4+); fixed n=2 may under- or over-estimate cooperativity | Could be parameterized per drug in future versions | `src/blueprint/equations/pathway_inhibitor.rs` |
| Bliss independence for drug combinations | If drugs interact synergistically or antagonistically via shared pathways, Bliss assumption underestimates or overestimates combo effect | Known pharmacology limitation; Bliss is a standard default | `CLAUDE.md` Drug Models Level 2 |
| Well-mixed population (no spatial structure in drug experiments) | If drug penetration gradients or spatial heterogeneity drive resistance, model misses these dynamics | Not modeled; would require spatial drug diffusion | Batch simulator assumption |
| Conservation is absolute (no energy creation) | If real biological systems involve energy sources not captured in the model (e.g., external feeding during treatment), conservation overestimates depletion | Model includes nutrient fields; drug experiments use finite nutrient | `tests/property_conservation.rs` |
| Dissipation ratios 1:4:16:50 are physically motivated | If real dissipation scaling differs, matter state transitions and derived constants shift | Ratios are calibration parameters, not measured | `src/blueprint/equations/derived_thresholds.rs` |
| Kleiber exponent 0.75 applies to abstract energy entities | If allometric scaling does not apply to the model's abstraction level, metabolic calculations are incorrect | Well-validated for biological organisms; extrapolation to abstract entities is assumed | Kleiber 1947, West et al. 1997 |

## 3. Model Assumptions — Detailed Analysis

### 3.1 The Everything-is-Energy Assumption

RESONANCE's foundational assumption (Axiom 1) is that all entities are characterized by a single scalar: energy (qe). This is a radical simplification of biology, where cells have spatial structure, thousands of protein species, post-translational modifications, and epigenetic states that cannot be reduced to a single number.

**Consequence:** The model cannot distinguish two cell types that differ in ways not captured by energy level, frequency, or phase. For example, two cells with identical qe and frequency but different protein expression profiles are indistinguishable to RESONANCE.

**When this matters:** When the outcome of interest depends on molecular details (e.g., specific mutation conferring drug resistance, protein conformation change, receptor density). When it does not matter: When the outcome depends on population-level energy dynamics (e.g., "does the population survive or collapse?").

**Mitigation:** RESONANCE uses frequency (Axiom 8) as a second identity axis and phase as a third. The 14-layer composition provides additional differentiation. But these are still abstract proxies, not molecular descriptors.

### 3.2 Frequency as Proxy for Genetic Identity

In RESONANCE, an entity's oscillatory frequency serves as a proxy for its genetic/epigenetic identity. Entities at similar frequencies interact more strongly (via Gaussian frequency alignment). Drug binding affinity is determined by the Gaussian overlap between drug frequency and target frequency.

**Consequence:** This is a one-dimensional projection of a high-dimensional genetic space. Two cells with different mutations but similar "frequency" would appear identical. Conversely, a single mutation might shift "frequency" by an amount that does not correspond to the actual change in drug sensitivity.

**When this matters:** When the resistance mechanism involves a specific mutation (e.g., T790M in EGFR) that changes drug binding but might not proportionally shift the abstract frequency.

**Mitigation:** The frequency-based model is acknowledged as abstract in all publications. Calibration profiles map frequency space to biological observables (IC50, doubling time) but do not validate the frequency-identity mapping itself.

### 3.3 Static Drug Concentration

Drug concentration in RESONANCE is a fixed parameter per experiment arm. There is no time-varying compartmental pharmacokinetic model. No half-life, no absorption, no metabolism, no excretion.

**Consequence:** The model cannot capture time-dependent drug exposure, peak-trough variation, or drug accumulation. It implicitly assumes continuous steady-state exposure at the specified concentration.

**When this matters:** When PK dynamics significantly influence outcome (e.g., pulsed dosing strategies, drugs with short half-lives, prodrugs requiring metabolic activation).

**Mitigation:** The adaptive therapy controller (Experiment 6) adds and removes drugs between generations, providing a coarse approximation of treatment holidays. But within a generation, concentration is static.

## 4. Failure Conditions

The following conditions identify scenarios where RESONANCE output should not be trusted:

### 4.1 Tumor Microenvironment Dominates

**Scenario:** The treatment response is primarily determined by vasculature (angiogenesis), hypoxia gradients, or stromal interactions rather than intrinsic tumor cell dynamics.

**Examples:** Anti-angiogenic therapy (bevacizumab), hypoxia-activated prodrugs, CAR-T cell therapy requiring tumor infiltration.

**RESONANCE response:** Output will be unreliable because the model does not represent the dominant driving mechanism.

**Indicator:** If the literature for the clinical scenario emphasizes TME as the primary determinant of response, RESONANCE should not be used.

### 4.2 Immune Response is the Primary Mechanism

**Scenario:** Treatment efficacy depends on immune cell recruitment, activation, or evasion (checkpoint inhibitors, CAR-T, cancer vaccines including mRNA vaccines).

**Examples:** Pembrolizumab (PD-1), ipilimumab (CTLA-4), the Rosie case (mRNA vaccine).

**RESONANCE response:** Output reflects metabolic suppression dynamics, not immune-mediated killing. The model cannot capture T-cell exhaustion, antigen presentation, or immune memory.

**Indicator:** Experiment 7 (Rosie case) explicitly falls in this category. The results are presented as a simulation exercise with a kinase inhibitor proxy, not as a model of immune-mediated response.

### 4.3 Pharmacokinetics Matter

**Scenario:** The outcome depends on drug exposure dynamics (peak concentration, trough levels, drug-drug metabolic interactions via CYP450, renal clearance).

**Examples:** Narrow therapeutic window drugs, prodrugs, drugs with significant first-pass metabolism.

**RESONANCE response:** Output assumes steady-state concentration. If PK dynamics are the differentiator between treatment outcomes, the model's predictions are not informative.

### 4.4 Molecular Specificity Required

**Scenario:** The research question requires distinguishing specific molecular targets (e.g., "does mutant X confer resistance to drug Y?").

**RESONANCE response:** The model operates on abstract frequencies, not molecular targets. It cannot answer target-specific questions. It can answer questions of the form "does frequency-distant subpopulation escape treatment?" — which is an abstract analog, not a molecular-resolution answer.

### 4.5 Quantitative Prediction Needed

**Scenario:** The research requires quantitative predictions in clinical units (days to progression, absolute cell counts, tumor volume in cm3).

**RESONANCE response:** Calibration profiles provide approximate mapping, but the mapping is qualitative. The model operates in abstract units (qe, Hz, generations) and cannot produce validated quantitative predictions in clinical units.

## 5. Per-Module Scope

### 5.1 Level 1: Cytotoxic Drug Model

**File:** `src/use_cases/experiments/cancer_therapy.rs`

| CAN do | CANNOT do |
|--------|-----------|
| Model frequency-selective cell killing via Hill kinetics (n=2) | Model specific drug mechanisms (alkylation, topoisomerase inhibition, etc.) |
| Model quiescent stem cell escape from chemotherapy | Model PK-dependent drug exposure or drug schedule optimization |
| Demonstrate that heterogeneous populations develop resistance | Predict time-to-resistance in calendar units |
| Show that frequency-distant cells survive treatment | Model acquired resistance via specific mutations (e.g., MDR1 upregulation) |

### 5.2 Level 2: Pathway Inhibitor Drug Model

**File:** `src/blueprint/equations/pathway_inhibitor.rs` (11 pure functions, 32+ tests)

| CAN do | CANNOT do |
|--------|-----------|
| Model 3 inhibition modes (Competitive, Noncompetitive, Uncompetitive) | Model allosteric specificity or binding kinetics beyond Hill |
| Demonstrate dose-response (monotonic, Hill-derived) | Predict IC50 for a novel drug (requires experimental data) |
| Show combination advantage over monotherapy (Bozic replication) | Model drug-drug pharmacokinetic interactions (CYP450, etc.) |
| Model off-target effects via frequency proximity | Model off-target effects via structural similarity |
| Predict escape frequency from binding affinity landscape | Predict specific resistance mutations |
| Apply Bliss independence for drug combinations | Model synergistic or antagonistic drug interactions beyond Bliss |
| Feed into adaptive therapy controller | Optimize dosing schedules with PK constraints |

### 5.3 Clinical Calibration Module

**File:** `src/blueprint/equations/clinical_calibration.rs` (21 tests)

| CAN do | CANNOT do |
|--------|-----------|
| Map generations to days (via published doubling times) | Validate that the mapping produces clinically accurate predictions |
| Map concentration to nM (via published IC50 values) | Account for PK variability (patient weight, renal function, etc.) |
| Map entity count to cell count | Validate against actual tumor cell counts (not measurable in patients) |
| Provide research context for abstract results | Serve as a clinical dosing calculator |
| Include 4 tumor type profiles (CML, prostate, NSCLC, canine MCT) | Cover all tumor types or drug combinations |

### 5.4 Adaptive Therapy Controller

**File:** `src/use_cases/experiments/pathway_inhibitor_exp.rs` (function `run_adaptive`)

| CAN do | CANNOT do |
|--------|-----------|
| Demonstrate feedback-based growth stabilization (7/10 seeds) | Guarantee stabilization in all cases |
| Show dynamic drug addition/removal based on growth trajectory | Model clinical biomarker lag (PSA delay, imaging intervals) |
| Predict that adaptive strategy can delay resistance | Predict optimal treatment holiday duration in calendar days |
| Operate on perfect information (exact cell efficiency) | Model the imperfect information available to clinicians |

## 6. Limitation-Consequence-Mitigation Registry

| ID | Limitation | Consequence | Mitigation | Severity |
|----|-----------|-------------|------------|----------|
| L-1 | Abstract qe units, not molar concentrations | Output not directly interpretable in clinical terms | Calibration profiles with published DOIs (21 tests); disclaimers | High |
| L-2 | No molecular targets (no EGFR, BCR-ABL, PD-L1) | Cannot predict target-specific drug sensitivity | Frequency used as abstract proxy; do not claim molecular resolution | High |
| L-3 | No tumor microenvironment | Invalid when TME dominates response | Document as failure condition (Section 4.1); limit claims to population dynamics | High |
| L-4 | Not validated against patient-level outcomes | Cannot support clinical decisions | Research-only intended use (RD-1.1); pervasive disclaimers | Critical |
| L-5 | Bozic comparison qualitative (suppression %, not time-to-resistance) | Cannot predict resistance timeline | Document in all Bozic-related outputs | Medium |
| L-6 | Efficiency reduction is not tumor volume reduction | Metabolic suppression and cell death are different phenomena | Clearly distinguish "efficiency" from "volume" in all outputs | Medium |
| L-7 | Frequency is computational proxy, not biological observable | Cannot be measured or validated in a biological experiment | Acknowledge in all frequency-related claims | Medium |
| L-8 | No immune system modeled | Cannot model immunotherapy or vaccine-mediated responses | Critical limitation for Rosie case; documented in Exp 7 caveats | High |
| L-9 | Calibration is qualitative fit, not quantitative prediction | Calibrated values contextualize results but do not validate them | Include "calibrated, not validated" disclaimer with every calibrated output | Medium |
| L-10 | Hill coefficient fixed at n=2 | May under/overestimate cooperativity for specific drugs | Could be parameterized per drug; documented as assumption | Low |
| L-11 | Bliss independence for drug combinations | May miss synergistic or antagonistic interactions | Standard pharmacological default; documented as assumption | Low |
| L-12 | No spatial drug penetration gradients | Misses resistance pockets due to poor drug access | Well-mixed assumption documented; spatial extension possible | Medium |
| L-13 | Static drug concentration (no PK time decay) | Cannot model pulsed dosing, drug holidays with PK fidelity | Adaptive controller adds/removes drugs between generations (coarse) | Medium |
| L-14 | No stochastic immune response | Cannot model variable immune engagement across patients | Not modeled; acknowledged as gap | Medium |
| L-15 | Deterministic RNG (not truly random) | Identical seeds produce identical outcomes; no Monte Carlo sampling of intrinsic stochasticity | Multi-seed runs (10 seeds standard) provide ensemble averaging | Low |
| L-16 | Rosie case uses press reports, not peer-reviewed data | Clinical observation data not independently verified | Disclosed in code comments and all documentation | Medium |

## 7. Honest Statement of Scope

RESONANCE is a computational research tool that demonstrates emergent therapeutic resistance dynamics from first principles. It produces qualitative insights ("combination therapy suppresses more than monotherapy") that are consistent with established predictions in mathematical oncology.

It is **not** a clinical tool. It does **not** predict patient outcomes. It does **not** model molecular biology at resolution sufficient for drug design. It does **not** replace wet-lab experiments, clinical trials, or clinical judgment.

Its value is in hypothesis generation: providing a principled, reproducible, open-source platform for exploring "what if" questions about therapeutic strategies before committing resources to experimental validation.

Every claim made by RESONANCE comes with explicit limitations. Every limitation documented in this report is a real constraint on the model's applicability. No limitation has been hidden or minimized.

## 8. Codebase References

| Reference | File |
|-----------|------|
| Pathway inhibitor (11 pure functions, 32+ tests) | `src/blueprint/equations/pathway_inhibitor.rs` |
| Pathway inhibitor experiment (18+ tests) | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Clinical calibration (4 profiles, 21 tests) | `src/blueprint/equations/clinical_calibration.rs` |
| Derived thresholds (4 constants, 17 tests) | `src/blueprint/equations/derived_thresholds.rs` |
| Determinism (hash-based RNG, 23 tests) | `src/blueprint/equations/determinism.rs` |
| Conservation fuzz (proptest, ~20 tests) | `tests/property_conservation.rs` |
| Cancer therapy Level 1 | `src/use_cases/experiments/cancer_therapy.rs` |
| Drug models honest scope | `CLAUDE.md` Drug Models section |
| Paper limitations | `docs/paper/resonance_arxiv.tex` Section 5 |
| Intended use exclusions | `docs/regulatory/01_foundation/INTENDED_USE.md` |

## 9. Revision History

| Version | Date | Author | Change Description |
|---------|------|--------|--------------------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial limitations report. 10 validated capabilities, 11 explicit exclusions, 9 documented assumptions, 5 failure conditions, 16 limitation-consequence-mitigation entries. All claims traced to commit `971c7ac`. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
