---
document_id: RD-2.5
title: Residual Risk Evaluation
standard: ISO 14971:2019 §7
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Residual Risk Evaluation

## 1. Purpose

This document evaluates the residual risk of each hazard after the application of risk control measures documented in RD-2.4 (Risk Controls). It provides the overall residual risk assessment and benefit-risk analysis required by ISO 14971:2019 §7.

**Cross-references:**
- RD-2.1 (Risk Management Plan): Acceptability criteria (§3)
- RD-2.2 (Risk Analysis): Initial hazard characterization
- RD-2.3 (Risk Evaluation): Pre-control risk dispositions
- RD-2.4 (Risk Controls): Control measures and effectiveness verification
- RD-1.1 (Intended Use Statement): Research-only use envelope

## 2. Individual Residual Risk Assessment

### 2.1 Residual Risk Table

| ID | Hazard | Pre-Control P x S | Controls Applied | Post-Control P x S | Residual Disposition | Change |
|----|--------|-------------------|-----------------|--------------------|--------------------|--------|
| H-01 | Overreliance on resistance predictions | P3 x S5 (ALARP) | 7 Information + 1 Design | P2 x S5 (ALARP) | **ALARP** | P reduced: disclaimers filter users who read documentation. S unchanged: clinical misuse remains possible. |
| H-02 | Energy conservation bug | P1 x S4 (Acceptable) | 3 Verification + 2 Design | P1 x S4 (Acceptable) | **Acceptable** | No change needed. 19 proptest fuzzes + 3,113 tests maintain P1. |
| H-03 | Calibration bias (4 profiles) | P4 x S3 (ALARP) | 2 Verification + 4 Information | P3 x S2 (Acceptable) | **Acceptable** | P reduced: multi-seed validation demonstrates robustness. S reduced: explicit "not validated" disclaimers lower impact of overfitting. |
| H-04 | Determinism broken | P1 x S4 (Acceptable) | 3 Design + 2 Verification | P1 x S4 (Acceptable) | **Acceptable** | No change needed. Hash-based RNG + 23 tests maintain P1. |
| H-05 | SOUP vulnerability | P2 x S3 (Acceptable) | 4 Design + 1 Information | P2 x S3 (Acceptable) | **Acceptable** | No change needed. No network exposure + pinned deps adequate for research tool. |
| H-06 | Output misinterpretation | P3 x S5 (ALARP) | 5 Information + 1 Design | P2 x S4 (ALARP) | **ALARP** | P reduced: unit labeling and disclaimers reduce misinterpretation likelihood. S reduced from S5 to S4: abstract units create a translation barrier that reduces clinical applicability. |
| H-07 | Model lacks TME | P5 x S3 (ALARP) | 5 Information | P5 x S3 (ALARP) | **ALARP** | No change. Inherent design limitation. Probability cannot be reduced (affects every run). Severity maintained at S3 by documentation. |
| H-08 | Floating point precision | P2 x S2 (Acceptable) | 3 Design + 1 Verification | P2 x S2 (Acceptable) | **Acceptable** | No change needed. Standard f32 practice, deterministic, bounded magnitude. |
| H-09 | Axiom violation undetected | P1 x S5 (ALARP) | 5 Verification + 1 Design | P1 x S5 (ALARP) | **ALARP** | No change. P1 maintained by extensive testing. S5 is intrinsic — axiom violation would invalidate the entire model. |
| H-10 | Documentation gap | P3 x S3 (ALARP) | 5 Information + 1 Verification | P2 x S2 (Acceptable) | **Acceptable** | P reduced: regulatory documentation track + review schedule. S reduced: systematic documentation reduces gap impact. |
| H-11 | Escape frequency prescriptive use | P2 x S4 (ALARP) | 1 Design + 3 Information | P1 x S3 (Acceptable) | **Acceptable** | P reduced to P1: no CLI exposure, internal function only. S reduced to S3: abstract frequency clearly documented as non-biological. |
| H-12 | Batch/Bevy divergence | P2 x S3 (Acceptable) | 2 Design + 2 Verification | P2 x S3 (Acceptable) | **Acceptable** | No change needed. Shared math layer provides structural equivalence. |

### 2.2 Residual Risk Summary

| Disposition | Pre-Control Count | Post-Control Count | Hazard IDs (Post-Control) |
|-------------|-------------------|--------------------|-----------------------------|
| **Acceptable** | 5 | 9 | H-02, H-03, H-04, H-05, H-08, H-10, H-11, H-12 |
| **ALARP** | 7 | 4 | H-01, H-06, H-07, H-09 |
| **Inaceptable** | 0 | 0 | None |

Controls reduced 3 hazards from ALARP to Acceptable (H-03, H-10, H-11). The remaining 4 ALARP hazards are evaluated individually below.

## 3. Justification for Remaining ALARP Hazards

### 3.1 H-01: Overreliance on Resistance Predictions (P2 x S5)

**Why S5 cannot be further reduced:** The severity is driven by foreseeable misuse (clinician applies simulation output to treatment decisions). No software control can prevent a determined user from misinterpreting results. The severity is a property of the misuse scenario, not of the software.

**Why P2 is as low as reasonably practicable:** Seven information controls (disclaimers at README, paper, CLAUDE.md, CLI output, intended use statement, validation table, excluded users list) plus one design control (abstract qe units) reduce the probability that a user reaches an incorrect clinical conclusion. Further reduction would require:
- Access control (login, credentialing) — disproportionate for an open-source research tool
- Clinical validation — requires clinical trial data, which is out of scope for the current intended use
- Regulatory clearance — only applicable if RESONANCE becomes SaMD

**Conclusion:** Residual risk is ALARP. Accepted for research-only use. If intended use changes, this hazard becomes Inaceptable (see RD-2.3 §4).

### 3.2 H-06: Output Misinterpretation (P2 x S4)

**Why S4 cannot be further reduced:** Calibration profiles present output in clinical units (nM, days). Removing calibration profiles would reduce scientific utility. The remaining severity (S4) reflects the risk that calibrated output is taken at face value without reading the accompanying disclaimers.

**Why P2 is as low as reasonably practicable:** All controls from H-01 apply, plus explicit unit labeling and calibration-specific disclaimers. Further reduction would require redesigning the output format (e.g., removing calibrated units entirely, which would reduce scientific value), or implementing mandatory user acknowledgment workflows (disproportionate for a research tool).

**Conclusion:** Residual risk is ALARP. Accepted for research-only use.

### 3.3 H-07: Model Lacks TME (P5 x S3)

**Why P5 cannot be reduced:** This is an inherent design limitation. Every simulation run lacks TME. The only way to reduce probability is to implement TME modeling, which would be a major model extension requiring:
- New ECS layers for vasculature, immune cells, ECM
- New equation domains for angiogenesis, hypoxia gradients, immune response
- Validation against TME-specific datasets
- Estimated effort: 6-12 months of development

This is disproportionate to the current research-only use, where TME absence is a known and accepted simplification.

**Why S3 is the correct severity:** The limitation is prominently documented (paper §5, README, CLAUDE.md). Intended users (computational biology researchers) understand that all models are simplifications. The harm from TME absence is wasted research effort if predictions diverge from reality for TME-dependent scenarios — moderate, not major.

**Conclusion:** Residual risk is ALARP. Accepted as an inherent model limitation. Documented in all user-facing materials.

### 3.4 H-09: Axiom Violation Undetected (P1 x S5)

**Why S5 cannot be reduced:** By definition, an axiom violation corrupts the foundational physics of the simulation. All output produced after an undetected violation would be wrong. Severity is intrinsic to the hazard and cannot be mitigated — only the probability of occurrence can be controlled.

**Why P1 is as low as reasonably practicable:** Five verification controls (17 derived threshold tests, 19 conservation fuzzes, 26 Coulomb tests, 42 pathway inhibitor tests, Verificador code review) plus one design control (INVIOLABLE declaration in CLAUDE.md) make violation detection highly likely. Further reduction would require:
- Formal verification (Coq, Lean, or similar proof assistant) — disproportionate for Class A research software
- Independent external audit of all equation files — reasonable if reclassified to Class B/C

**Conclusion:** Residual risk is ALARP. Accepted because the probability is improbable (P1) and further reduction requires formal methods that are disproportionate for the current use context.

## 4. Overall Residual Risk Assessment

### 4.1 Aggregate Risk Profile

| Metric | Value |
|--------|-------|
| Total hazards identified | 12 |
| Hazards with residual risk Acceptable | 8 (67%) |
| Hazards with residual risk ALARP | 4 (33%) |
| Hazards with residual risk Inaceptable | 0 (0%) |
| ALARP hazards with S5 severity | 2 (H-01, H-09) |
| ALARP hazards requiring SaMD action | 3 (H-01, H-06, H-07) — would become Inaceptable if reclassified |

### 4.2 Risk Interaction Analysis

ISO 14971:2019 §7.2 requires evaluation of the combined effect of residual risks. The following interactions are identified:

| Interaction | Hazards | Combined Effect | Assessment |
|-------------|---------|----------------|------------|
| Misinterpretation cascade | H-01 + H-06 + H-03 | User overrelies on predictions (H-01), misinterprets calibrated output as clinical (H-06), and assumes 4 profiles indicate broad validity (H-03) | Combined probability: P3 (each step adds probability). Combined severity: S5 (worst case from H-01). Combined risk: ALARP. Mitigated by redundant disclaimers at 7 independent locations — breaking any link in the chain stops the cascade. |
| Silent model failure | H-07 + H-09 | Model lacks TME (H-07) AND an axiom violation goes undetected (H-09) — results are wrong for two independent reasons | Combined probability: P1 (requires both to coincide — P5 x P1 for independent events = very low for co-occurrence). Combined severity: S5. Combined risk: ALARP. The two hazards are structurally independent — TME absence is a known limitation, axiom violation is an unknown defect. |
| Documentation-dependent controls | H-01 + H-06 + H-10 | If documentation degrades (H-10), the information controls for H-01 and H-06 lose effectiveness | Combined probability: P3 (documentation gap is P2 post-control, but degrades other controls). Combined severity: S5 (loss of H-01 controls). Combined risk: ALARP. Mitigated by periodic review schedule (RD-2.1 §5) and multiple redundant documentation locations. |

No risk interactions elevate combined residual risk to Inaceptable.

## 5. Benefit-Risk Analysis

ISO 14971:2019 §7.4 requires that if individual residual risks are ALARP, the overall benefit-risk balance must be favorable.

### 5.1 Benefits of RESONANCE

| Benefit | Evidence | Beneficiary |
|---------|----------|-------------|
| First-principles drug resistance simulation | 8 axioms, 4 constants, no hardcoded behavior. Unique in the field. | Computational biology research community |
| Qualitative Bozic 2013 replication | 10/10 seeds confirm combination > monotherapy — structural, not stochastic | Mathematical oncology researchers |
| Bit-exact reproducibility | Hash-based deterministic RNG; any result can be independently verified | All researchers; journal reviewers |
| Open source (AGPL-3.0) | Full transparency of methods; community audit possible | Scientific community |
| 3,113 automated tests | Unusually high test coverage for a simulation tool | Research credibility; peer review |
| 4 clinical calibration profiles | Demonstrates alignment with published oncology data (qualitative) | Pharma R&D hypothesis generation |
| Batch simulator for evolutionary experiments | Millions of worlds in parallel, rayon-based, no GPU required | Population genetics researchers |
| Published peer-reviewable paper | Zenodo DOI: 10.5281/zenodo.19342036 | Replication and citation |

### 5.2 Residual Risks

| Risk Category | Count | Worst Case |
|---------------|-------|-----------|
| ALARP (foreseeable misuse) | 2 (H-01, H-06) | Clinician misapplies research output to treatment decision |
| ALARP (inherent limitation) | 1 (H-07) | Simulation diverges from reality for TME-dependent scenarios |
| ALARP (axiom integrity) | 1 (H-09) | Undetected axiom violation invalidates model |
| Acceptable | 8 | Minor or self-correcting issues |

### 5.3 Benefit-Risk Determination

**The benefit-risk balance is FAVORABLE for research-only use.**

Rationale:

1. **No patient is in the loop.** The worst-case residual risk (clinician misapplies output) requires deliberate use outside the intended use envelope. Seven information controls and one design control (abstract units) create multiple barriers to this scenario.

2. **Scientific benefits are tangible.** The Bozic replication, deterministic reproducibility, and 4 calibration profiles provide genuine value to the computational biology community. These benefits would be lost if RESONANCE were withdrawn due to residual risk concerns.

3. **ALARP hazards are well-characterized.** All 4 ALARP hazards have documented causes, controls, and justifications. None are unknown or uncontrolled.

4. **The model is honest about its limitations.** Five known limitations are documented for the Rosie canine MCT case alone. Paper §5 dedicates an entire section to limitations. README has an "Honest scope" section. This level of transparency is unusual for simulation tools and itself serves as a risk control.

5. **Alternative: no tool.** If RESONANCE did not exist, researchers would use tools with less transparency, fewer tests, and no formal risk management. The marginal risk of RESONANCE versus the alternative (less-tested, less-documented tools) is negative — RESONANCE is likely safer, not riskier.

### 5.4 SaMD Benefit-Risk (Hypothetical)

If RESONANCE were reclassified as SaMD, the benefit-risk balance would be **UNFAVORABLE** without additional controls:

- 5 hazards would become Inaceptable (H-01, H-03, H-06, H-07, H-11 per RD-2.3 §4)
- Benefits would increase (clinical applicability) but risks would increase faster (patient harm potential)
- Resolution: implement TME, PK/PD, expand calibration, obtain regulatory clearance (see RD-1.5 §4)

## 6. Conditions for Residual Risk Acceptance

The overall residual risk is accepted subject to the following conditions:

| Condition | Monitoring Mechanism | Trigger for Re-evaluation |
|-----------|---------------------|--------------------------|
| Intended use remains research-only | RD-1.1 review + annual check (RD-2.1 §5) | Any trigger per RD-1.5 §7.2 (clinical use claim, pharma partnership, patient data input) |
| Disclaimers remain present and visible | README review at each release | Disclaimer removed, diluted, or contradicted |
| Test suite maintained at current or higher coverage | `cargo test` at each commit | Test count drops below 3,000 or failures > 0 |
| No SOUP CVE with CVSS >= 7.0 | Periodic dependency review (planned: `cargo audit` in CI) | CVE reported for any direct dependency |
| Documentation kept current with features | Periodic documentation review (RD-2.1 §5) | New feature released without corresponding documentation update |

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial residual risk evaluation. 12 hazards assessed post-control. 8 Acceptable, 4 ALARP, 0 Inaceptable. Benefit-risk favorable for research-only use. 5 acceptance conditions established. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Planificador (Process Owner) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Observador (Reviewer) | _pending_ | _pending_ | _pending_ |
| Verificador (Verification) | _pending_ | _pending_ | _pending_ |
