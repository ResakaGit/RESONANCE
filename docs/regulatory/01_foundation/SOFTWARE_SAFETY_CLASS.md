---
document_id: RD-1.2
title: Software Safety Classification
standard: IEC 62304:2006+Amd1:2015, §4.3
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Software Safety Classification

## 1. Purpose

IEC 62304:2006+Amd1:2015, §4.3 requires that medical device software be assigned a safety classification before development activities commence. This classification determines the rigor of software lifecycle processes — including documentation, verification, testing, and configuration management — required throughout the entire product lifecycle.

For RESONANCE, the classification is being applied retroactively to an existing codebase (113K LOC, 3,113 automated tests) as part of a regulatory documentation initiative. The classification assigned in this document governs the documentation depth required for all subsequent deliverables in the regulatory track (sprints RD-2 through RD-7; see `docs/sprints/REGULATORY_DOCUMENTATION/README.md`).

This document must be read in conjunction with:

- **RD-1.1 INTENDED_USE.md** — defines intended use, intended users, and use environment
- **RD-1.5 REGULATORY_STRATEGY.md** — positions RESONANCE within the IMDRF SaMD framework
- **IEC 62304:2006+Amd1:2015** — the governing standard for medical device software lifecycle processes

## 2. IEC 62304 Safety Classes Defined

IEC 62304 §4.3 defines three software safety classes based on the severity of injury that could result from a hazardous situation to which the software can contribute:

### Class A — No injury or damage to health is possible

The software system cannot contribute to a hazardous situation. Failure or malfunction of the software, or latent design flaws in the software, cannot result in any injury to the patient, operator, or bystander. Class A software requires the least rigorous lifecycle processes under IEC 62304.

### Class B — Non-serious injury is possible

The software system can contribute to a hazardous situation that results in non-serious injury. "Non-serious" is defined per ISO 14971 as injury that is reversible or minor, and does not require significant medical intervention. Class B software requires intermediate lifecycle process rigor.

### Class C — Serious injury or death is possible

The software system can contribute to a hazardous situation that results in serious injury or death. "Serious injury" includes irreversible impairment of a body function or structure, or injury requiring medical or surgical intervention to prevent permanent impairment. Class C software requires the most rigorous lifecycle processes, including detailed architectural and unit-level documentation, comprehensive risk controls, and formal verification at all decomposition levels.

### Classification decision rule (IEC 62304 §4.3, Amendment 1)

Per Amendment 1 (2015), a software system may be classified as Class A if:

> The software system cannot contribute to a hazardous situation, **or** the software system can contribute to a hazardous situation which does not result in unacceptable risk after consideration of risk control measures external to the software system.

This amendment permits a lower classification when external risk controls (e.g., disclaimers, intended use restrictions, absence of clinical integration) adequately mitigate the contribution of the software to hazardous situations.

## 3. Classification Analysis

### 3.1 Current intended use

RESONANCE is a research simulation tool. Per the intended use declaration (RD-1.1):

- **What it does:** Simulates emergent life, evolution, and therapeutic strategies from 8 axioms and 4 fundamental constants using a Rust/Bevy ECS architecture.
- **What it is NOT:** "Not a clinical tool", "Not a drug discovery pipeline", "Not a substitute for oncology" (README.md, lines 20-22).
- **Intended users:** Researchers in computational biology, systems pharmacology, and theoretical ecology. Not clinicians, pharmacists, or patients.
- **Use environment:** Research workstation or server. No point-of-care deployment. No integration with clinical systems (EHR, LIMS, CDSS).
- **Output:** Simulation results in abstract energy units (qe) or calibrated units with explicit disclaimers. All outputs carry the caveat that they are not validated against patient outcomes (README.md, line 141).

### 3.2 Can RESONANCE contribute to a hazardous situation?

To contribute to a hazardous situation under IEC 62304, software must be part of a causal chain from software failure to patient harm. This requires:

1. **A patient in the loop** — someone whose health could be affected by the software output.
2. **A clinical decision informed by the software** — a prescription, dosing, diagnosis, or treatment modification.
3. **A plausible failure mode** — software produces incorrect output that misleads the decision-maker.

Analysis of each criterion:

| Criterion | RESONANCE Status | Evidence |
|-----------|-----------------|----------|
| Patient in the loop | **No.** No patient data is processed. No patient is exposed to software output. | README.md lines 20-22; no HIPAA/GDPR data handling in codebase |
| Clinical decision informed | **No.** Software output is research hypotheses and simulation results. No clinical workflow integration exists. | No EHR/CDSS interfaces in `src/`; no output format compatible with clinical decision systems |
| Plausible failure mode leading to harm | **No.** Even if simulation output were incorrect, it cannot reach a patient without deliberate misuse outside the intended use envelope. | Drug models explicitly disclaimed: "Abstract qe units (not molar concentrations). No molecular targets. NOT clinical tools." (CLAUDE.md, Drug Models section) |

### 3.3 Hazardous situation assessment

**Is there a hazardous situation when RESONANCE is used as intended?**

No. When used as intended (research exploration of emergent therapeutic strategies), RESONANCE operates entirely within a research context. Its outputs:

- Are consumed by researchers, not clinicians making bedside decisions.
- Are expressed in abstract simulation units or calibrated units with explicit "NOT validated against patient outcomes" disclaimers.
- Do not enter any clinical workflow, electronic health record, or clinical decision support system.
- Cannot directly or indirectly control, monitor, or influence any medical device, drug delivery system, or patient-facing technology.

### 3.4 Classification determination

**RESONANCE is classified as IEC 62304 Class A.**

The software system cannot contribute to a hazardous situation when used within its intended use envelope. No injury or damage to health is possible from the software's operation, failure, or latent defects, because no patient is in the causal chain between software output and clinical action.

## 4. Justification

The Class A classification rests on the following independent lines of evidence:

### 4.1 Software output is research hypotheses, not clinical recommendations

RESONANCE produces simulation results that represent theoretical explorations of emergent dynamics. The drug models (cytotoxic Level 1 in `src/use_cases/experiments/cancer_therapy.rs`; pathway inhibitor Level 2 in `src/blueprint/equations/pathway_inhibitor.rs`) are explicitly described as:

> "Theoretical models for exploring resistance dynamics. Abstract qe units (not molar concentrations). No molecular targets (no EGFR/BCR-ABL). No tumor microenvironment. Not validated against patient-level data. NOT clinical tools."
>
> -- CLAUDE.md, Drug Models: Honest scope

The Bozic 2013 comparison (in `src/use_cases/experiments/pathway_inhibitor_exp.rs` and `src/bin/bozic_validation.rs`) is explicitly characterized as qualitative: "suppression %, not absolute cell counts or time-to-resistance in weeks."

### 4.2 Explicit disclaimers in all public-facing materials

| Location | Disclaimer text | Reference |
|----------|----------------|-----------|
| README.md, line 20 | "Not a clinical tool -- not validated against patient outcomes" | `README.md` |
| README.md, line 21 | "Not a drug discovery pipeline -- does not design molecules" | `README.md` |
| README.md, line 22 | "Not a substitute for oncology -- a simulator for exploring therapeutic strategies" | `README.md` |
| README.md, line 141 | "Against patient outcomes: **Not yet** -- calibrated but not validated against longitudinal patient data" | `README.md` |
| Zenodo paper (DOI: 10.5281/zenodo.19342036) | Limitations section acknowledges abstract units, no clinical validation | `docs/paper/resonance_arxiv.tex` |
| CLAUDE.md, Drug Models section | "NOT clinical tools"; "Bozic comparison is qualitative" | `CLAUDE.md` |

### 4.3 No FDA clearance claimed or implied

RESONANCE has not been submitted to any regulatory authority (FDA, EMA, PMDA, TGA, Health Canada) for clearance, approval, or registration as a medical device or SaMD. No marketing materials, publications, or repository documentation claim or imply regulatory clearance. The AGPL-3.0 license does not constitute a medical device listing.

### 4.4 No patient data processing

The codebase contains no mechanisms for:

- Ingesting patient data (no DICOM, HL7 FHIR, CSV patient import, or PHI handling)
- Storing patient identifiers (no PII/PHI fields in any component or resource)
- Outputting patient-specific recommendations (all output is population-level simulation)

**Gap acknowledged:** There is no formal data classification policy. This is documented as a gap for RD-7 (Data Integrity). For the purposes of safety classification, the absence of any patient data handling infrastructure is sufficient evidence that no patient data is processed.

### 4.5 Abstract units prevent direct clinical interpretation

RESONANCE operates in abstract energy units (qe). While calibration profiles exist that map simulation output to clinically interpretable units (nM, days, cell count — see README.md line 24), these mappings are:

- Research calibrations, not validated clinical biomarkers.
- Accompanied by the disclaimer "calibrated but not validated against longitudinal patient data" (README.md, line 141).
- Not presented in a format compatible with clinical decision-making workflows.

A researcher could misinterpret calibrated output as clinical guidance. This risk is mitigated by disclaimers (Section 4.2) and intended use restrictions (RD-1.1). Per IEC 62304 Amendment 1, external risk controls (disclaimers + use restrictions) may be considered when classifying software. Even so, the residual risk of misinterpretation does not constitute a "contribution to a hazardous situation" by the software itself — it would require deliberate use outside the stated intended use.

### 4.6 Identified gaps in the justification

Transparency requires documenting weaknesses in the classification argument:

| Gap | Severity | Mitigation | Resolution sprint |
|-----|----------|------------|-------------------|
| No formal risk management file yet (ISO 14971) | Medium | Disclaimers serve as informal risk controls; formal file planned | RD-2 |
| No formal intended use document yet (RD-1.1) | Medium | README disclaimers + CLAUDE.md scope statements serve as interim; formal document planned | RD-1 |
| Calibration profiles could be mistaken for clinical validation | Low | Explicit "NOT validated" disclaimer present in README.md line 141 | RD-6 |
| No labeling review (IEC 62304 §4.1 note) | Low | README serves as de facto labeling; formal labeling review not yet performed | RD-5 |
| No post-market surveillance plan | Low | Not required for Class A research tool; would be required if reclassified | RD-6 |

## 5. Conditional Reclassification

If RESONANCE's intended use were to change — specifically, if it were positioned as Software as a Medical Device (SaMD) per IMDRF SaMD N10 — the safety classification would change as follows.

### 5.1 Scenario: SaMD that informs clinical decisions (Class B minimum)

If RESONANCE output were used to **inform** treatment decisions (e.g., "this simulation suggests combination therapy may outperform monotherapy for this tumor profile"), the software would contribute to a clinical decision. A software failure producing incorrect output could lead to suboptimal treatment selection.

- **IMDRF risk category:** Inform clinical management, non-serious condition = Category II; serious condition = Category III.
- **IEC 62304 classification:** **Class B** (non-serious injury possible from suboptimal but non-dangerous treatment selection) or **Class C** (serious injury possible if incorrect output leads to withholding effective treatment for a life-threatening condition).

### 5.2 Scenario: SaMD that drives clinical decisions (Class C)

If RESONANCE output were used to **drive** treatment decisions (e.g., automated dose selection, treatment protocol generation without clinician override), the software would directly influence patient outcomes. A software failure could lead to serious injury or death.

- **IMDRF risk category:** Drive clinical management, serious condition = Category IV.
- **IEC 62304 classification:** **Class C**.

### 5.3 Documentation burden comparison

The following table summarizes the difference in required documentation rigor across safety classes. This illustrates why correct classification is critical — a Class C designation would require approximately 3-4x the documentation effort of Class A.

| Documentation area | Class A | Class B | Class C |
|-------------------|---------|---------|---------|
| Software development plan | Required | Required | Required |
| Software requirements | Required | Required | Required |
| Software architecture | Not required | Required | Required |
| Detailed design (unit level) | Not required | Not required | Required |
| Software unit verification | Not required | Required | Required |
| Software integration testing | Not required | Required | Required |
| Software system testing | Required | Required | Required |
| Risk management per ISO 14971 | Required | Required | Required |
| Traceability (req → test) | Not required | Required | Required |
| Traceability (req → arch → code → test) | Not required | Not required | Required |
| Configuration management | Required | Required | Required |
| Problem resolution | Required | Required | Required |
| Formal code reviews | Not required | Required | Required |
| Regression analysis | Not required | Required | Required |

**Current state note:** Despite being Class A, RESONANCE already satisfies several Class B/C requirements due to existing engineering practices: 3,113 automated tests (unit + integration), architectural documentation (`docs/ARCHITECTURE.md`, `docs/design/`), bit-exact determinism (`blueprint/equations/determinism.rs`), and sprint-based traceability (`docs/sprints/`). These are engineering best practices voluntarily adopted, not regulatory obligations at Class A.

## 6. IEC 62304 Requirements by Class

The following table maps IEC 62304 clauses to requirements per safety class. "R" = required, "—" = not required. The "RESONANCE Status" column indicates current compliance for the Class A classification.

| IEC 62304 Clause | Description | Class A | Class B | Class C | RESONANCE Status (Class A) |
|-------------------|-------------|---------|---------|---------|---------------------------|
| §4.1 | Quality management system | R | R | R | Partial — CLAUDE.md coding rules + sprint process; formal QMS planned (RD-5) |
| §4.2 | Risk management | R | R | R | Gap — informal disclaimers only; formal ISO 14971 file planned (RD-2) |
| §4.3 | Software safety classification | R | R | R | **This document** |
| §5.1 | Software development planning | R | R | R | Partial — sprint methodology documented; formal plan planned (RD-1.4) |
| §5.2 | Software requirements analysis | R | R | R | Partial — requirements inferable from CLAUDE.md axioms + tests; formal SRS planned (RD-1.3) |
| §5.3 | Software architectural design | — | R | R | Exists voluntarily — `docs/ARCHITECTURE.md`, `docs/design/`, CLAUDE.md Module Map |
| §5.4 | Software detailed design | — | — | R | Exists partially — `docs/arquitectura/` module contracts |
| §5.5 | Software unit implementation & verification | R | R | R | Satisfied — 3,113 tests, zero `unsafe`, Rust compiler as static verifier |
| §5.6 | Software integration and integration testing | — | R | R | Exists voluntarily — integration tests in `tests/` directory |
| §5.7 | Software system testing | R | R | R | Satisfied — `cargo test` exercises full system; Bozic validation exercises drug pipeline end-to-end |
| §5.8 | Software release | R | R | R | Partial — Cargo.lock pins dependencies; no formal release procedure |
| §6.1 | Software maintenance plan | R | R | R | Gap — no formal maintenance plan |
| §7.1 | Software configuration management | R | R | R | Partial — Git + Cargo.lock; no formal CM plan |
| §7.2 | Configuration item identification | R | R | R | Partial — Cargo.toml + Cargo.lock; no formal CI register |
| §7.3 | Change control | R | R | R | Partial — Git branches + PR process; no formal change control board |
| §8.1 | Software problem resolution | R | R | R | Partial — GitHub issues; no formal problem resolution procedure |
| §9.1 | Software configuration management of SOUP | R | R | R | Partial — Cargo.lock pins SOUP versions; formal SOUP analysis planned (RD-3) |

### Summary for Class A

| Category | Required clauses | Status |
|----------|-----------------|--------|
| Fully satisfied | §4.3, §5.5, §5.7 | 3 of 14 |
| Partially satisfied | §4.1, §5.1, §5.2, §5.8, §7.1, §7.2, §7.3, §8.1, §9.1 | 9 of 14 |
| Gap | §4.2, §6.1 | 2 of 14 |
| Not required (Class A) | §5.3, §5.4, §5.6 | 3 clauses (satisfied voluntarily) |

The two gaps (§4.2 Risk Management, §6.1 Maintenance Plan) are addressed in sprints RD-2 and RD-5 respectively.

## 7. Codebase References

### 7.1 Disclaimer locations

| File | Content | Lines |
|------|---------|-------|
| `README.md` | "Not a clinical tool", "Not a drug discovery pipeline", "Not a substitute for oncology" | 20-22 |
| `README.md` | "calibrated but not validated against longitudinal patient data" | 141 |
| `CLAUDE.md` | "NOT clinical tools"; "Bozic comparison is qualitative" | Drug Models section |
| `CLAUDE.md` | "Abstract qe units (not molar concentrations). No molecular targets." | Drug Models: Honest scope |
| `docs/paper/resonance_arxiv.tex` | Limitations section | §5 |

### 7.2 Test infrastructure

| Metric | Value | Source |
|--------|-------|--------|
| Total automated tests | 3,113 | `cargo test` (measured 2026-04-02) |
| Test execution time | ~38 seconds | `cargo test` full suite |
| Test failures | 0 | `cargo test` (0 failed, 1 ignored) |
| Lines of code | ~113K | `wc -l src/**/*.rs src/*.rs` |
| External crate dependencies | Pinned via `Cargo.lock` | `Cargo.lock` in repository root |
| Determinism verification | Bit-exact | `src/blueprint/equations/determinism.rs` |
| Safety: `unsafe` blocks in runtime | 0 | CLAUDE.md Hard Block #1: "NO `unsafe` -- zero tolerance" |

### 7.3 Drug model files (classification-relevant)

| File | Purpose | Test count |
|------|---------|-----------|
| `src/blueprint/equations/pathway_inhibitor.rs` | Pathway inhibitor pure math (11 functions) | 32 tests |
| `src/blueprint/constants/pathway_inhibitor.rs` | Pathway inhibitor constants (7 derived) | 3 tests |
| `src/use_cases/experiments/pathway_inhibitor_exp.rs` | Pathway inhibitor experiment + Bozic validation | 18 tests |
| `src/use_cases/experiments/cancer_therapy.rs` | Cytotoxic drug model (Hill pharmacokinetics) | See module tests |
| `src/bin/bozic_validation.rs` | Standalone Bozic 2013 validation binary | 10-seed robustness |
| `src/blueprint/equations/derived_thresholds.rs` | All lifecycle constants from 4 fundamentals | 17 tests |

## 8. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial classification. Class A assigned based on research-only intended use. Conditional reclassification scenarios documented. |

---

**Classification:** IEC 62304 **Class A** — no contribution to a hazardous situation.

**Next review trigger:** Any change to intended use, any integration with clinical systems, any regulatory submission, or any use in a clinical trial context. Any such change requires immediate re-evaluation of this classification per IEC 62304 §4.3.

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft — not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
