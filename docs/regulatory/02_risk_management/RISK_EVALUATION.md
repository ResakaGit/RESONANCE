---
document_id: RD-2.3
title: Risk Evaluation
standard: ISO 14971:2019 §5.5
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Risk Evaluation

## 1. Purpose

This document evaluates each hazard identified in RD-2.2 (Risk Analysis) against the acceptability criteria defined in RD-2.1 (Risk Management Plan §3). Each hazard receives a disposition: Acceptable, ALARP, or Inaceptable. The evaluation is performed for two contexts:

1. **Current context:** Research-only tool (IMDRF Category I, IEC 62304 Class A)
2. **Hypothetical SaMD context:** Software as a Medical Device informing clinical decisions (for forward-looking risk awareness)

**Cross-references:**
- RD-2.1 (Risk Management Plan): §3 Risk Acceptability Criteria
- RD-2.2 (Risk Analysis): Hazard register with probability/severity ratings
- RD-1.1 (Intended Use Statement): Defines the research-only use envelope
- RD-1.2 (Software Safety Classification): Class A determination

## 2. Risk Matrix

### 2.1 Probability vs. Severity Grid (Research-Only Context)

```
              S1            S2            S3            S4            S5
          Negligible      Minor       Moderate       Major        Critical
     +-------------+-------------+-------------+-------------+-------------+
P5   |             |             |   H-07      |             |             |
Freq.|  Acceptable |  Acceptable |   ALARP     | Inaceptable | Inaceptable |
     +-------------+-------------+-------------+-------------+-------------+
P4   |             |             |   H-03      |             |             |
Prob.|  Acceptable |  Acceptable |   ALARP     | Inaceptable | Inaceptable |
     +-------------+-------------+-------------+-------------+-------------+
P3   |             |             |   H-10      |             | H-01, H-06  |
Occ. |  Acceptable |  Acceptable |   ALARP     |   ALARP     |   ALARP     |
     +-------------+-------------+-------------+-------------+-------------+
P2   |             |   H-08      | H-05, H-12  |  H-04,H-11  |             |
Rem. |  Acceptable |  Acceptable |  Acceptable |   ALARP     |   ALARP     |
     +-------------+-------------+-------------+-------------+-------------+
P1   |             |             |             |   H-02      |   H-09      |
Imp. |  Acceptable |  Acceptable |  Acceptable |  Acceptable |   ALARP     |
     +-------------+-------------+-------------+-------------+-------------+
```

### 2.2 Zone Legend

| Zone | Color | Criteria | Action Required |
|------|-------|----------|-----------------|
| **Acceptable** | Green | Low probability AND low-moderate severity | Document. Monitor. No additional controls required. |
| **ALARP** | Yellow | Medium risk — tolerable if further reduction is impracticable or disproportionate | Implement controls. Document justification for residual risk. Review at each release. |
| **Inaceptable** | Red | High probability AND high severity | Must be reduced before release. If irreducible, formal benefit-risk analysis per ISO 14971 §7. |

## 3. Hazard Evaluation (Research-Only Context)

### 3.1 Evaluation Table

| ID | Hazard | P | S | Risk Level | Disposition | Justification |
|----|--------|---|---|------------|-------------|---------------|
| H-01 | Overreliance on resistance predictions | P3 | S5 | ALARP | **ALARP — Controls required** | S5 severity derives from foreseeable misuse (clinical application), not intended use. Under intended use (research only), severity would be S3. Controls: disclaimers at 7 locations (see RD-2.4). Residual risk acceptable because intended users are trained researchers. |
| H-02 | Energy conservation bug | P1 | S4 | Acceptable | **Acceptable** | P1 probability justified by 19 proptest conservation fuzzes + 3,113 total tests. S4 impact would be major if it occurred, but the probability is sufficiently low. Existing controls: automated tests + CI enforcement. |
| H-03 | Calibration bias (4 profiles) | P4 | S3 | ALARP | **ALARP — Controls required** | 4 profiles is a minimal validation set. Probability of overfitting is high (P4) but severity is moderate (S3) because research users are expected to validate independently. Controls: explicit "NOT validated against patient outcomes" disclaimer + multi-seed robustness. |
| H-04 | Determinism broken | P1 | S4 | Acceptable | **Acceptable** | 23 dedicated determinism tests + `to_bits()` hashing + no shared mutable state (Hard Block #4) make probability improbable. If it occurred, severity would be major (non-reproducible results). Probability rating alone places this in Acceptable zone. |
| H-05 | SOUP vulnerability | P2 | S3 | Acceptable | **Acceptable** | No network exposure, no untrusted input, local execution only. Widely-used dependencies. RESONANCE does not run in clinical or production environments. Risk is within normal software operation tolerance. |
| H-06 | Output misinterpretation | P3 | S5 | ALARP | **ALARP — Controls required** | S5 severity derives from the same foreseeable misuse as H-01. Calibrated output in clinical units (nM, days) exacerbates the risk. Controls: disclaimers + explicit unit labeling in output. Residual risk acceptable if controls are maintained. |
| H-07 | Model lacks TME | P5 | S3 | ALARP | **ALARP — Inherent limitation, documented** | This is a fundamental design limitation, not a defect. Every simulation run is affected (P5), but severity is moderate (S3) because the limitation is well-documented and expected by the intended user population. Controls: paper §5 limitations, README honest scope. Cannot be reduced without redesigning the model. |
| H-08 | Floating point precision | P2 | S2 | Acceptable | **Acceptable** | f32 is used consistently (no truncation), errors are deterministic and small relative to simulation dynamics. Standard numerical computing practice. No controls required beyond existing architecture. |
| H-09 | Axiom violation undetected | P1 | S5 | ALARP | **ALARP — Controls required** | S5 because axiom violation would invalidate the entire model. P1 because of extensive test coverage (17 derived threshold tests, 19 conservation fuzzes, code review protocol). Controls: test suite + Verificador review role. Residual risk acceptable at P1. |
| H-10 | Documentation gap | P3 | S3 | ALARP | **ALARP — Controls required** | Documentation is maintained manually, creating drift risk. Controls: this regulatory documentation track + periodic review schedule (RD-2.1 §5). Residual risk acceptable because documentation gaps affect understanding, not simulation correctness. |
| H-11 | Escape frequency prescriptive use | P2 | S4 | ALARP | **ALARP — Controls required** | Internal function with no CLI endpoint reduces probability. Severity is major because the output looks precise but is grounded in abstract frequency, not molecular biology. Controls: function-level documentation + module-level disclaimers. |
| H-12 | Batch/Bevy simulator divergence | P2 | S3 | Acceptable | **Acceptable** | Both simulators call the same pure functions. Round-trip bridge tests exist. Divergence would be confusing but not harmful. No clinical decision depends on batch/Bevy agreement. |

### 3.2 Summary by Disposition

| Disposition | Count | Hazard IDs |
|-------------|-------|-----------|
| **Acceptable** | 5 | H-02, H-04, H-05, H-08, H-12 |
| **ALARP** | 7 | H-01, H-03, H-06, H-07, H-09, H-10, H-11 |
| **Inaceptable** | 0 | None |

**Conclusion for research-only context:** No hazards are Inaceptable. All 7 ALARP hazards have existing or planned controls that reduce residual risk to acceptable levels. See RD-2.4 for control details and RD-2.5 for residual risk assessment.

## 4. Hypothetical SaMD Evaluation

If RESONANCE were reclassified as SaMD (informing clinical management, per RD-1.5 §4), the risk evaluation changes materially. This section documents the shift for forward-looking awareness.

### 4.1 SaMD Risk Matrix (Changes Only)

| ID | Hazard | Research P | Research S | SaMD P | SaMD S | SaMD Disposition |
|----|--------|-----------|-----------|--------|--------|-----------------|
| H-01 | Overreliance on predictions | P3 | S5 | P4 | S5 | **Inaceptable** |
| H-03 | Calibration bias | P4 | S3 | P4 | S4 | **Inaceptable** |
| H-06 | Output misinterpretation | P3 | S5 | P4 | S5 | **Inaceptable** |
| H-07 | Model lacks TME | P5 | S3 | P5 | S4 | **Inaceptable** |
| H-09 | Axiom violation | P1 | S5 | P1 | S5 | ALARP (unchanged) |
| H-11 | Escape frequency prescriptive | P2 | S4 | P3 | S5 | **Inaceptable** |

### 4.2 SaMD Inaceptable Hazards

Under SaMD classification, 5 hazards would be Inaceptable:

| ID | Why Inaceptable | Required Controls (Not Currently Implemented) |
|----|-----------------|-----------------------------------------------|
| H-01 | Clinical decisions informed by unvalidated predictions | Clinical validation against patient outcomes, PK/PD integration, regulatory clearance |
| H-03 | 4 profiles insufficient for clinical generalization | Minimum 20+ profiles across diverse cancer types, prospective validation study |
| H-06 | Clinical units in output without clinical validation | Output format redesign, mandatory clinical context warnings, user authentication |
| H-07 | TME absence unacceptable for clinical predictions | TME modeling (vasculature, hypoxia, immune) — fundamental model extension |
| H-11 | Escape frequencies could influence drug design | Molecular target mapping, wet-lab validation of frequency-target correspondence |

### 4.3 SaMD Gap Summary

Transitioning from research tool to SaMD would require addressing all 5 Inaceptable hazards. This would involve:

- Fundamental model extensions (TME, PK/PD, immune system)
- Clinical validation studies (prospective, multi-site)
- Regulatory clearance (FDA De Novo, EU MDR Class IIa)
- Formal quality management system (ISO 13485)

These requirements are documented in RD-1.5 (Regulatory Strategy) §4-5 and would be addressed in RD-6 (Clinical Evaluation) if triggered.

## 5. Overall Risk Acceptability Determination

### 5.1 Research-Only Context (Current)

**Determination: Overall risk is ACCEPTABLE.**

Rationale:
- No hazards are Inaceptable under research-only intended use
- All 7 ALARP hazards have existing controls (disclaimers, tests, documentation) or planned controls (this regulatory track)
- The primary risk vector (foreseeable misuse by clinicians) is mitigated by pervasive disclaimers at 7 codebase locations
- The intended user population (graduate-level researchers) is competent to evaluate simulation limitations
- No patient is in the causal chain between software output and clinical action

### 5.2 SaMD Context (Hypothetical)

**Determination: Overall risk would be UNACCEPTABLE without major model and process changes.**

5 of 12 hazards would be Inaceptable. The model's fundamental limitations (no TME, no PK/PD, 4 calibration profiles, abstract units) cannot be addressed by documentation or disclaimers alone — they require engineering work that is not currently planned or scoped.

## 6. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial risk evaluation. 12 hazards evaluated against ALARP criteria for research-only and hypothetical SaMD contexts. Overall risk acceptable for research-only use. 5 hazards Inaceptable under SaMD. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Planificador (Evaluator) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Observador (Reviewer) | _pending_ | _pending_ | _pending_ |
