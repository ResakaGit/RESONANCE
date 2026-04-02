---
document_id: RD-2.1
title: Risk Management Plan
standard: ISO 14971:2019 §5.1
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Risk Management Plan

## 1. Purpose

This document establishes the risk management plan for RESONANCE, an emergent life simulation engine classified as IEC 62304 Class A (RD-1.2). It defines the scope, risk acceptability criteria, analysis methods, responsibilities, and review schedule for systematic identification, evaluation, control, and monitoring of risks associated with the software throughout its lifecycle.

This plan satisfies ISO 14971:2019 §5.1 (Risk management plan) and governs the production of all subsequent risk management deliverables in this track:

| Document | ID | Standard Reference |
|----------|----|--------------------|
| Risk Analysis | RD-2.2 | ISO 14971:2019 §5.3-5.4 |
| Risk Evaluation | RD-2.3 | ISO 14971:2019 §5.5 |
| Risk Control Measures | RD-2.4 | ISO 14971:2019 §6 |
| Residual Risk Evaluation | RD-2.5 | ISO 14971:2019 §7 |
| Risk Management Report | RD-2.6 | ISO 14971:2019 §8 |

**Cross-references:**
- RD-1.1 (Intended Use Statement): Defines intended use, intended users, and excluded use contexts
- RD-1.2 (Software Safety Classification): Establishes IEC 62304 Class A classification
- RD-1.5 (Regulatory Strategy): IMDRF SaMD Category I positioning and gap analysis

## 2. Scope

### 2.1 Product Scope

RESONANCE is a computational research tool for simulating emergent life dynamics, therapeutic resistance evolution, and drug interaction strategies from first principles. The risk management scope covers:

- **Core simulation engine:** 113K LOC, Rust 2024 / Bevy 0.15, 14 orthogonal ECS layers
- **Drug models:** Level 1 cytotoxic (`src/use_cases/experiments/cancer_therapy.rs`) and Level 2 pathway inhibitor (`src/blueprint/equations/pathway_inhibitor.rs` — 11 pure functions, 42 tests, 3 inhibition modes)
- **Clinical calibration profiles:** CML/imatinib, prostate/abiraterone, NSCLC/erlotinib, canine MCT/toceranib (`src/blueprint/equations/clinical_calibration.rs`)
- **Batch simulator:** 33 stateless systems, rayon parallel, no Bevy dependency (`src/batch/`)
- **Headless mode:** PPM image output, no GPU (`src/bin/headless_sim.rs`)
- **Deterministic RNG:** Hash-based, bit-exact reproducibility (`src/blueprint/equations/determinism.rs`, 23 tests)
- **All SOUP (Software of Unknown Provenance):** 14 direct dependencies as listed in `Cargo.toml`
- **All outputs:** simulation reports, CSV/JSON exports (`src/use_cases/export.rs`), PPM images, CLI text output

### 2.2 Lifecycle Scope

This plan applies from the current release (commit `971c7acb99decde45bf28860e6e10372718c51e2`) through all future versions. Risk analysis is performed:

- At initial release (this document)
- At each major release (new drug model, new clinical calibration profile, new intended user population)
- When SOUP dependencies are updated (new Bevy version, new crate additions)
- When intended use changes (any trigger per RD-1.5 §7.2)
- Annually, regardless of changes

### 2.3 Exclusions

The following are excluded from the risk management scope:

| Exclusion | Rationale |
|-----------|-----------|
| Rendering-only code (`src/rendering/`, `Update` schedule visual systems) | No simulation output dependency; visual-only |
| Third-party crate internal defects (beyond known CVEs) | Managed via SOUP analysis (RD-3); not auditable at source level |
| Operating system and hardware failures | Out of scope for application-level risk management |
| User's downstream interpretation in contexts outside intended use | Addressed by disclaimers and labeling, not by software design |

## 3. Risk Acceptability Criteria

### 3.1 Risk Classification Framework

Risks are classified using a 5x5 probability-severity matrix. This framework applies to RESONANCE's current intended use as a research-only tool. If intended use changes to include clinical decision support, the acceptability criteria must be reassessed.

#### Probability Scale

| Level | Label | Definition | Frequency Estimate |
|-------|-------|------------|--------------------|
| P1 | Improbable | Requires multiple independent failures to occur simultaneously | < 1 in 10,000 simulation runs |
| P2 | Remote | Theoretically possible but not observed in testing or production | < 1 in 1,000 simulation runs |
| P3 | Occasional | Could occur under specific, unusual conditions | < 1 in 100 simulation runs |
| P4 | Probable | Expected to occur under certain foreseeable conditions | < 1 in 10 uses |
| P5 | Frequent | Expected to occur in normal use | > 1 in 10 uses |

#### Severity Scale

| Level | Label | Definition (Research-Only Context) |
|-------|-------|------------------------------------|
| S1 | Negligible | No impact on research conclusions; cosmetic or UI issue |
| S2 | Minor | Slightly misleading result that would be caught by standard scientific review |
| S3 | Moderate | Incorrect simulation result that could waste research effort (weeks) if not detected |
| S4 | Major | Systematically incorrect results that could lead to publication of flawed findings |
| S5 | Critical | Results misinterpreted as clinical evidence, potentially contributing to harm if used outside intended use |

### 3.2 Risk Acceptability Zones (ALARP)

The As Low As Reasonably Practicable (ALARP) framework is adopted. Three zones are defined:

| Zone | Risk Levels | Disposition |
|------|-------------|-------------|
| **Acceptable** | P1-P2 combined with S1-S3; P1 combined with S4 | Risk accepted. No additional controls required. Document and monitor. |
| **ALARP** | P3 combined with S3-S4; P2 combined with S4; P1-P3 combined with S5 | Risk tolerable only if further reduction is impracticable or disproportionate to benefit. Controls required. Justify residual risk. |
| **Inaceptable** | P4-P5 combined with S4-S5; P3-P5 combined with S5 | Risk not acceptable. Must be reduced before release. If reduction is impossible, benefit-risk analysis required per ISO 14971:2019 §7. |

### 3.3 Research-Only vs. SaMD Criteria

The acceptability criteria above apply to RESONANCE's current research-only classification (IMDRF Category I, IEC 62304 Class A). If RESONANCE were reclassified as SaMD:

- S5 severity would be redefined as "patient harm from suboptimal treatment selection based on incorrect simulation output"
- The ALARP zone boundary would shift: P2 combined with S4 would become Inaceptable
- Additional controls (clinical validation, PK/PD modeling, regulatory clearance) would be mandatory

This conditional reassessment is documented in RD-2.3 (Risk Evaluation) and RD-1.5 (Regulatory Strategy §5).

## 4. Risk Analysis Methods

### 4.1 Primary Method: Software FMEA

Failure Mode and Effects Analysis (FMEA) is the primary risk analysis method. Each identified hazard is characterized by:

- **Hazard ID:** Unique identifier (H-XX)
- **Hazard description:** What can go wrong
- **Potential harm:** Consequence to the user or downstream stakeholders
- **Cause(s):** Root cause(s) — software defect, design limitation, user error, or SOUP failure
- **Probability (P1-P5):** Likelihood of occurrence, justified by codebase evidence
- **Severity (S1-S5):** Impact severity under current intended use
- **Risk level:** Derived from probability-severity matrix
- **Existing controls:** Controls already implemented in the codebase
- **Residual risk:** Risk remaining after controls

### 4.2 Supplementary Method: Hazard Analysis of Intended Use Deviation

For hazards arising from use outside the intended use envelope (foreseeable misuse per ISO 14971:2019 §5.3), a separate analysis evaluates:

- What happens if a clinician uses simulation output to inform treatment
- What happens if calibration profiles are mistaken for clinical validation
- What happens if disclaimer text is removed or ignored

### 4.3 Evidence Sources

Risk analysis draws on the following evidence from the RESONANCE codebase:

| Evidence Type | Source | Quantity |
|---------------|--------|----------|
| Automated tests | `cargo test` | 3,113 tests (0 failures) |
| Property-based fuzzing | `tests/property_conservation.rs` | 19 proptest cases |
| Determinism tests | `src/blueprint/equations/determinism.rs` | 23 tests |
| Bozic validation | `src/bin/bozic_validation.rs` | 10 independent seeds, 10/10 confirm |
| Derived threshold tests | `src/blueprint/equations/derived_thresholds.rs` | 17 tests |
| Pathway inhibitor tests | `src/blueprint/equations/pathway_inhibitor.rs` | 42 tests |
| Coulomb/LJ tests | `src/blueprint/equations/coulomb.rs` | 26 tests |
| Clinical calibration tests | `src/blueprint/equations/clinical_calibration.rs` | 21 tests |
| Disclaimers | README.md, paper §5, CLI outputs, in-code comments | 7 distinct locations |

## 5. Review Schedule

### 5.1 Planned Reviews

| Trigger | Review Type | Scope | Responsible |
|---------|-------------|-------|-------------|
| Initial release | Full FMEA | All hazards | All roles (see §6) |
| Major release (new drug model or calibration profile) | Targeted FMEA | New/modified hazards only | Alquimista + Verificador |
| SOUP update (Bevy version bump, new dependency) | SOUP-focused review | H-05 and related | Alquimista + Observador |
| Intended use change trigger (per RD-1.5 §7.2) | Full reassessment | All hazards + acceptability criteria | All roles |
| Annual review | Full review | All hazards, controls, residual risk | All roles |
| Post-incident (if a user reports misuse or incorrect result) | Targeted analysis | Affected hazard(s) + root cause | Verificador + Observador |

### 5.2 Review Records

Each review produces:

1. Updated risk analysis (RD-2.2) with new or modified hazards
2. Updated risk evaluation (RD-2.3) if acceptability changes
3. Updated risk controls (RD-2.4) if new controls are implemented
4. Updated residual risk evaluation (RD-2.5)
5. Approval record in the risk management report (RD-2.6)

Reviews are recorded in the revision history of each document and tracked via Git commit history.

## 6. Responsibilities

ISO 14971:2019 §4.3 requires that top management assign qualified personnel to the risk management process. RESONANCE maps its 4 defined roles (per `CLAUDE.md` Roles section) to ISO 14971 responsibilities:

### 6.1 Role Mapping

| ISO 14971 Role | RESONANCE Role | Responsibilities |
|----------------|----------------|------------------|
| **Risk management process owner** | Planificador | Maintains risk management plan. Ensures all hazards are identified and documented. Schedules reviews. Approves risk acceptability determinations. |
| **Risk analyst** | Alquimista | Performs hazard identification and root cause analysis. Provides codebase evidence for probability/severity ratings. Implements design controls. |
| **Risk evaluator / reviewer** | Observador | Reviews risk analysis for completeness and accuracy. Validates probability/severity ratings against codebase evidence. Challenges assumptions. |
| **Verification authority** | Verificador | Verifies that risk controls are implemented and effective. Confirms test coverage for each control. Issues PASS/WARN/BLOCK verdicts on risk control verification. |

### 6.2 Competence Requirements

All personnel performing risk management activities must have:

- Familiarity with the RESONANCE codebase architecture (14 layers, pipeline phases, blueprint/equations)
- Understanding of the 8 foundational axioms and 4 fundamental constants
- Working knowledge of ISO 14971:2019 risk management principles
- For drug model hazards: understanding of Hill pharmacokinetics, inhibition modes, and the distinction between abstract simulation units and clinical measurements

### 6.3 Independence

The Observador and Verificador roles must be independent of the Alquimista who implemented the feature under review. For a single-developer project, this independence is achieved by temporal separation (review performed in a separate session from implementation) and documented in the review record.

**Gap acknowledged:** RESONANCE is currently developed by a small team. Full organizational independence between risk analyst and reviewer is not yet achievable. This is documented as an accepted limitation for Class A software. If reclassified to Class B/C, independent review by an external party would be required.

## 7. Risk Management File Structure

The complete risk management file consists of the following documents, all stored under `docs/regulatory/02_risk_management/`:

| Document | ID | File | Status |
|----------|----|------|--------|
| Risk Management Plan | RD-2.1 | `RISK_MANAGEMENT_PLAN.md` | This document |
| Risk Analysis | RD-2.2 | `RISK_ANALYSIS.md` | DRAFT |
| Risk Evaluation | RD-2.3 | `RISK_EVALUATION.md` | DRAFT |
| Risk Control Measures | RD-2.4 | `RISK_CONTROLS.md` | DRAFT |
| Residual Risk Evaluation | RD-2.5 | `RESIDUAL_RISK.md` | DRAFT |
| Risk Management Report | RD-2.6 | `RISK_MANAGEMENT_REPORT.md` | DRAFT |

All documents are version-controlled via Git. The authoritative version is the one at the commit referenced in each document's header.

## 8. Cross-References

| Document | ID | Relevance to Risk Management |
|----------|----|------------------------------|
| Intended Use Statement | RD-1.1 | Defines intended use envelope — basis for hazard identification and severity assessment |
| Software Safety Classification | RD-1.2 | Class A determination — governs required rigor of risk controls |
| Software Requirements Specification | RD-1.3 | Functional and safety requirements — traceability target for risk controls |
| Software Development Plan | RD-1.4 | Lifecycle processes — verification and validation activities that serve as risk controls |
| Regulatory Strategy | RD-1.5 | IMDRF classification and gap analysis — context for acceptability criteria and SaMD reclassification scenarios |
| SOUP Analysis | RD-3.2 (planned) | Third-party dependency risk — input to H-05 |
| Validation Reports | RD-4 (planned) | Verification and validation evidence — effectiveness evidence for risk controls |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial risk management plan. Scope, ALARP criteria, FMEA method, 4-role responsibility mapping, review schedule established. All cross-references to RD-1 foundation documents confirmed. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Planificador (Process Owner) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Observador (Reviewer) | _pending_ | _pending_ | _pending_ |
| Verificador (Verification) | _pending_ | _pending_ | _pending_ |
