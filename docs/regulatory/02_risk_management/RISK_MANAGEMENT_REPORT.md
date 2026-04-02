---
document_id: RD-2.6
title: Risk Management Report
standard: ISO 14971:2019 §8
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Risk Management Report

## 1. Purpose

This document serves as the concluding summary of the risk management process for RESONANCE, satisfying ISO 14971:2019 §8. It confirms that the risk management plan (RD-2.1) has been executed, all identified hazards have been analyzed, evaluated, and controlled, and the overall residual risk is acceptable for the current intended use.

**Cross-references:**

| Document | ID | Status |
|----------|----|--------|
| Risk Management Plan | RD-2.1 | Complete |
| Risk Analysis | RD-2.2 | Complete |
| Risk Evaluation | RD-2.3 | Complete |
| Risk Control Measures | RD-2.4 | Complete |
| Residual Risk Evaluation | RD-2.5 | Complete |
| Intended Use Statement | RD-1.1 | Complete |
| Software Safety Classification | RD-1.2 | Complete |
| Software Requirements Specification | RD-1.3 | Complete |
| Software Development Plan | RD-1.4 | Complete |
| Regulatory Strategy | RD-1.5 | Complete |

## 2. Risk Management Plan Execution Confirmation

### 2.1 Plan Completeness

The risk management plan (RD-2.1) specified the following activities. Each has been executed:

| Planned Activity | ISO 14971 Clause | Delivered In | Status |
|-----------------|-------------------|-------------|--------|
| Define scope | §5.1 | RD-2.1 §2 | Complete. Scope covers simulation engine, drug models, clinical calibration, batch simulator, all SOUP, and all outputs. |
| Establish acceptability criteria (ALARP) | §5.1 | RD-2.1 §3 | Complete. 5x5 P-S matrix with 3 zones (Acceptable, ALARP, Inaceptable) defined for research-only context. |
| Select analysis method (FMEA) | §5.1 | RD-2.1 §4 | Complete. Software FMEA selected as primary method, supplemented by intended use deviation analysis. |
| Assign responsibilities | §4.3 | RD-2.1 §6 | Complete. 4 roles (Planificador, Alquimista, Observador, Verificador) mapped to ISO 14971 responsibilities. |
| Define review schedule | §5.1 | RD-2.1 §5 | Complete. Reviews at each major release, SOUP update, intended use change, and annually. |
| Identify hazards | §5.3 | RD-2.2 §3 | Complete. 12 hazards identified via 5 analysis methods. |
| Estimate risk | §5.4 | RD-2.2 §3 | Complete. Each hazard rated P1-P5, S1-S5 with codebase evidence justification. |
| Evaluate risk | §5.5 | RD-2.3 §3 | Complete. Each hazard dispositioned as Acceptable, ALARP, or Inaceptable. |
| Implement risk controls | §6 | RD-2.4 §3 | Complete. 52 total controls across 12 hazards. |
| Verify control effectiveness | §6.3 | RD-2.4 §3 (V-xx.x entries) | Complete. Each control has documented verification evidence. |
| Evaluate new hazards from controls | §6.4 | RD-2.4 §5 | Complete. 4 potential new hazards evaluated; none significant. |
| Evaluate residual risk | §7 | RD-2.5 §2-3 | Complete. 8 Acceptable, 4 ALARP, 0 Inaceptable. |
| Benefit-risk analysis | §7.4 | RD-2.5 §5 | Complete. Benefit-risk favorable for research-only use. |

### 2.2 Deviations from Plan

No deviations from the risk management plan occurred during execution.

## 3. Summary of Results

### 3.1 Hazard Inventory

| Metric | Value |
|--------|-------|
| Total hazards identified | 12 |
| Foreseeable misuse scenarios identified | 5 |
| Hazard identification methods used | 5 (intended use deviation, software failure mode, SOUP vulnerability, information deficit, model limitation) |

### 3.2 Risk Classification (Pre-Control)

| Disposition | Count | Percentage | Hazard IDs |
|-------------|-------|-----------|-----------|
| Acceptable | 5 | 42% | H-02, H-04, H-05, H-08, H-12 |
| ALARP | 7 | 58% | H-01, H-03, H-06, H-07, H-09, H-10, H-11 |
| Inaceptable | 0 | 0% | None |

### 3.3 Risk Controls Implemented

| Control Type | Count | Examples |
|-------------|-------|---------|
| Design (inherent safety) | 18 | Abstract qe units, hash-based RNG, Cargo.lock pinning, no network dependencies, no shared mutable state, shared math layer |
| Verification (automated testing) | 17 | 19 proptest conservation fuzzes, 23 determinism tests, 17 derived threshold tests, 42 pathway inhibitor tests, 26 Coulomb tests, Bozic 10-seed validation |
| Information (documentation/labeling) | 17 | README disclaimers (7 locations), paper §5 limitations, CLAUDE.md honest scope, in-code disclaimers, regulatory documentation track |
| **Total** | **52** | |

### 3.4 Risk Classification (Post-Control)

| Disposition | Count | Percentage | Hazard IDs |
|-------------|-------|-----------|-----------|
| Acceptable | 8 | 67% | H-02, H-03, H-04, H-05, H-08, H-10, H-11, H-12 |
| ALARP | 4 | 33% | H-01, H-06, H-07, H-09 |
| Inaceptable | 0 | 0% | None |

Controls reduced 3 hazards from ALARP to Acceptable (H-03: calibration bias, H-10: documentation gap, H-11: escape frequency). The 4 remaining ALARP hazards have documented justifications for acceptance (RD-2.5 §3).

### 3.5 Residual ALARP Hazards

| ID | Hazard | Residual P x S | Why ALARP Accepted |
|----|--------|----------------|-------------------|
| H-01 | Overreliance on predictions | P2 x S5 | S5 from foreseeable misuse. 7 information controls reduce P. Further reduction requires clinical validation (disproportionate for research tool). |
| H-06 | Output misinterpretation | P2 x S4 | Calibrated output in clinical units. Disclaimers reduce P. Removing calibration would reduce scientific utility. |
| H-07 | Model lacks TME | P5 x S3 | Inherent design limitation. Documented in paper, README, CLAUDE.md. TME implementation would be 6-12 months (disproportionate). |
| H-09 | Axiom violation undetected | P1 x S5 | S5 intrinsic. 5 verification controls maintain P1. Formal verification disproportionate for Class A. |

## 4. Overall Residual Risk Determination

### 4.1 Determination

**The overall residual risk of RESONANCE is ACCEPTABLE for its intended use as a research-only simulation tool.**

This determination is based on:

1. **No Inaceptable hazards** exist in either the pre-control or post-control assessment.
2. **All ALARP hazards** have documented justifications for acceptance, with identified controls and clear boundaries for what would trigger re-evaluation.
3. **Benefit-risk analysis** (RD-2.5 §5) concludes the balance is favorable: RESONANCE provides unique scientific capabilities (first-principles drug resistance simulation, bit-exact reproducibility, Bozic replication) with risks that are well-characterized and manageable through existing controls.
4. **No patient is in the decision loop.** The causal chain from software output to patient harm requires deliberate use outside the intended use envelope, which is mitigated by pervasive disclaimers.

### 4.2 Conditions for Acceptance

This determination is conditional on the following (per RD-2.5 §6):

| # | Condition | Monitoring |
|---|-----------|-----------|
| 1 | Intended use remains research-only | Annual review per RD-2.1 §5 |
| 2 | Disclaimers remain present and visible | README review at each release |
| 3 | Test suite maintained at >= 3,000 tests with 0 failures | `cargo test` at each commit |
| 4 | No SOUP CVE with CVSS >= 7.0 unaddressed | Periodic dependency review |
| 5 | Documentation kept current with features | Periodic review per RD-2.1 §5 |

If any condition is violated, the risk management process must be re-entered starting from risk analysis (RD-2.2).

## 5. SaMD Contingency Assessment

### 5.1 If Reclassified as SaMD

Per RD-2.3 §4, reclassification as SaMD would make 5 hazards Inaceptable:

| ID | Hazard | SaMD Disposition | Required Action |
|----|--------|-----------------|-----------------|
| H-01 | Overreliance on predictions | Inaceptable | Clinical validation, PK/PD integration, regulatory clearance |
| H-03 | Calibration bias | Inaceptable | 20+ calibration profiles, prospective validation study |
| H-06 | Output misinterpretation | Inaceptable | Output format redesign, mandatory warnings, user authentication |
| H-07 | Model lacks TME | Inaceptable | TME modeling implementation (vasculature, immune, ECM) |
| H-11 | Escape frequency prescriptive | Inaceptable | Molecular target mapping, wet-lab validation |

### 5.2 SaMD Readiness Gap

The gap between current state and SaMD readiness is substantial:

| Dimension | Current State | SaMD Requirement | Gap |
|-----------|--------------|-----------------|-----|
| Clinical validation | 0 patient datasets | Prospective multi-site study | Large |
| PK/PD modeling | None (static concentration) | Compartmental PK/PD | Large |
| TME modeling | None | Vasculature + hypoxia + immune | Large |
| Calibration profiles | 4 (qualitative) | 20+ (quantitative, diverse) | Large |
| Regulatory clearance | None | FDA De Novo or EU MDR IIa | Large |
| QMS | CLAUDE.md coding rules | ISO 13485 certified | Medium |
| Risk management | This file (RD-2) | Full ISO 14971 with clinical hazards | Medium |
| Post-market surveillance | None | PMS plan + PSUR | Large |

**Conclusion:** SaMD reclassification is not recommended at this time. The gap is large and would require 12-24 months of development and $100K-400K in regulatory costs (per RD-1.5 §4).

## 6. Identified Gaps in Risk Management Process

### 6.1 Process Gaps

| Gap | Severity | Planned Resolution | Sprint |
|-----|----------|-------------------|--------|
| No `cargo audit` for automated CVE scanning | Medium | Add to CI pipeline | RD-3 |
| No end-to-end Bevy/batch equivalence test | Low | Engineering backlog item | Future |
| No automated documentation consistency check | Low | Consider linting tool | RD-5 |
| Single-developer review independence | Medium | Temporal separation documented; external review if Class B/C | Ongoing |
| No post-production monitoring process (ISO 14971 §9) | Low | Not required for research-only Class A; planned for SaMD transition | RD-6 |
| No formal SOUP analysis document | Medium | Planned as RD-3.2 | RD-3 |
| Axiom 6 not testable by unit tests | Low | Accepted — code review is the primary control | N/A |

### 6.2 Gap Impact Assessment

None of the identified gaps change the overall residual risk determination. All gaps are:
- Documented (in this report and in RD-2.4 §6)
- Scheduled for resolution (specific sprints identified)
- Assessed as non-critical for Class A research-only software

If any gap were to become blocking (e.g., a high-severity SOUP CVE discovered before `cargo audit` is implemented), the gap would be escalated to a risk management review per RD-2.1 §5.

## 7. Risk Management Process Effectiveness

### 7.1 Assessment

The risk management process, as defined in RD-2.1 and executed across RD-2.2 through RD-2.5, is assessed as **EFFECTIVE** for the following reasons:

1. **Comprehensive hazard identification.** 12 hazards were identified across 5 categories (misuse, software defect, SOUP, information, model limitation). No known risk category was omitted.
2. **Evidence-based ratings.** Every probability and severity rating is justified by specific codebase evidence (test counts, file references, disclaimers locations). No ratings are unsupported assertions.
3. **Verifiable controls.** All 52 controls have documented verification evidence (V-xx.x entries in RD-2.4). Controls are traceable to specific files, tests, or documents.
4. **Honest gap reporting.** 7 gaps are explicitly documented, each with a planned resolution. The process does not claim perfection.
5. **Conditional acceptance.** Residual risk acceptance is conditional on 5 monitored conditions, not unconditional.

### 7.2 Limitations

1. **Retroactive application.** This risk management process was applied retroactively to an existing 113K LOC codebase. Hazards may exist that were not identified because they are embedded in code written before risk management was formalized.
2. **Single-team assessment.** The same team that developed RESONANCE performed the risk analysis. Independent external review would increase confidence, particularly for S5-rated hazards.
3. **No user feedback.** The hazard identification includes no input from actual users (no user surveys, no incident reports, no field data). All foreseeable misuse scenarios are theoretical.
4. **Static analysis.** This assessment is a point-in-time snapshot at commit `971c7acb99decde45bf28860e6e10372718c51e2`. Future changes may introduce new hazards or change existing risk levels.

## 8. Conclusion

The risk management process for RESONANCE has been executed per the plan (RD-2.1). The results are:

- **12 hazards identified** through systematic analysis
- **52 risk controls implemented** (18 design, 17 verification, 17 information)
- **0 Inaceptable hazards** in the research-only context
- **4 ALARP hazards** with documented acceptance justifications
- **8 Acceptable hazards** with adequate existing controls
- **Benefit-risk balance: FAVORABLE** for research-only use
- **7 gaps identified** with planned resolutions

**The overall residual risk is ACCEPTABLE for RESONANCE's intended use as a research-only simulation tool, subject to 5 monitored conditions.**

The risk management file will be reviewed per the schedule in RD-2.1 §5 (each major release, each SOUP update, each intended use change, and annually).

## 9. Document Traceability

### 9.1 Risk Management File Index

| Document | ID | File Path | Version | Date |
|----------|----|-----------|---------|------|
| Risk Management Plan | RD-2.1 | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md` | 1.0 | 2026-04-02 |
| Risk Analysis | RD-2.2 | `docs/regulatory/02_risk_management/RISK_ANALYSIS.md` | 1.0 | 2026-04-02 |
| Risk Evaluation | RD-2.3 | `docs/regulatory/02_risk_management/RISK_EVALUATION.md` | 1.0 | 2026-04-02 |
| Risk Control Measures | RD-2.4 | `docs/regulatory/02_risk_management/RISK_CONTROLS.md` | 1.0 | 2026-04-02 |
| Residual Risk Evaluation | RD-2.5 | `docs/regulatory/02_risk_management/RESIDUAL_RISK.md` | 1.0 | 2026-04-02 |
| Risk Management Report | RD-2.6 | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_REPORT.md` | 1.0 | 2026-04-02 |

### 9.2 Foundation Documents Referenced

| Document | ID | File Path |
|----------|----|-----------|
| Intended Use Statement | RD-1.1 | `docs/regulatory/01_foundation/INTENDED_USE.md` |
| Software Safety Classification | RD-1.2 | `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` |
| Software Requirements Specification | RD-1.3 | `docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md` |
| Software Development Plan | RD-1.4 | `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` |
| Regulatory Strategy | RD-1.5 | `docs/regulatory/01_foundation/REGULATORY_STRATEGY.md` |

### 9.3 Codebase Evidence Referenced

| Evidence Category | Key Files | Total Tests |
|-------------------|-----------|-------------|
| Conservation | `tests/property_conservation.rs` | 19 proptest |
| Determinism | `src/blueprint/equations/determinism.rs` | 23 |
| Derived thresholds | `src/blueprint/equations/derived_thresholds.rs` | 17 |
| Pathway inhibitor | `src/blueprint/equations/pathway_inhibitor.rs` | 42 |
| Pathway inhibitor constants | `src/blueprint/constants/pathway_inhibitor.rs` | 3 |
| Pathway inhibitor experiment | `src/use_cases/experiments/pathway_inhibitor_exp.rs` | 31 |
| Coulomb/LJ | `src/blueprint/equations/coulomb.rs` | 26 |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` | 21 |
| Bozic validation | `src/bin/bozic_validation.rs` | 10 seeds |
| **Full suite** | `cargo test` | **3,113** |

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial risk management report. Confirms plan execution, 12 hazards analyzed, 52 controls, residual risk acceptable, benefit-risk favorable. All at commit `971c7ac`. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Planificador (Process Owner) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Alquimista (Analyst) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Observador (Reviewer) | _pending_ | _pending_ | _pending_ |
| Verificador (Verification) | _pending_ | _pending_ | _pending_ |
