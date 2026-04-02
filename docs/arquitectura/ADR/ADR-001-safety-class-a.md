# ADR-001: IEC 62304 Safety Class A Classification

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** IEC 62304 Software Safety Classification

## Context

IEC 62304 Section 4.3 requires a formal safety classification for all medical device software before development activities can proceed. This classification determines the rigor of documentation, verification, and lifecycle processes required throughout the entire software lifecycle. The classification must be based on the severity of hazards that the software could contribute to if it fails or produces incorrect output.

RESONANCE contains drug pathway modeling (cytotoxic + pathway inhibitor), an adaptive therapy controller, clinical calibration profiles (CML, prostate, NSCLC, canine MCT), and Bozic 2013 validation -- capabilities that superficially resemble clinical decision support. The classification decision therefore has outsized impact on all downstream regulatory documentation.

## Decision

Classify RESONANCE as **IEC 62304 Safety Class A**: no contribution to a hazardous situation.

The classification is based on:

1. **Intended use is research-only.** RESONANCE generates hypotheses about emergent behavior, not clinical recommendations. Output is abstract energy quanta (qe), not dosing schedules or diagnostic conclusions.
2. **No patient in the loop.** No patient data is ingested, no clinical workflow depends on RESONANCE output, no treatment decision is informed by the software.
3. **Explicit disclaimers.** README.md, CLAUDE.md, and the Zenodo paper all state: "NOT clinical tools," "Abstract qe units (not molar concentrations)," "Not validated against patient-level data."
4. **Conditional reclassification.** If intended use changes to inform clinical decisions, reclassification to Class B or C will be triggered per the reclassification criteria documented in RD-1.5.

## Consequences

### Positive
- Proportionate documentation burden -- Class A requires software development planning and verification but not the full traceability matrix and detailed design documentation required by Class B/C.
- Enables a single developer to maintain regulatory compliance without dedicated quality engineering staff.
- Honest classification that accurately reflects the current risk profile, rather than aspirational over-classification.

### Negative
- If a downstream user misuses RESONANCE output for clinical decisions, the Class A classification provides no regulatory defense beyond the disclaimers.
- Reclassification to Class B/C would require significant retroactive documentation effort (detailed design, unit-level traceability, architectural decomposition per IEC 62304 Section 5.3-5.4).

### Risks
- A regulatory body could challenge the Class A classification if RESONANCE is used in a clinical research context, even without direct patient impact. Mitigation: RD-1.5 documents explicit reclassification triggers and the upgrade pathway.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Class B (software that can inform clinical decisions, non-serious injury) | RESONANCE does not currently inform clinical decisions. Over-classifying would impose disproportionate documentation burden without matching risk. |
| Class C (software that can drive clinical decisions, serious injury or death) | No clinical workflow depends on RESONANCE. Class C documentation burden is incompatible with single-developer research project. |
| Defer classification | IEC 62304 Section 4.3 requires classification before development proceeds. Deferral is non-compliant. |

## References

- `docs/regulatory/01_foundation/RD-1.2` -- IEC 62304 SDP (Safety Classification)
- `docs/regulatory/01_foundation/RD-1.1` -- System Requirements Specification (Intended Use)
- `docs/regulatory/01_foundation/RD-1.5` -- SaMD Classification & Reclassification Criteria
- `README.md` -- Disclaimers section
- `CLAUDE.md` -- "NOT clinical tools" (Drug Models section)
- IEC 62304:2006+A1:2015, Section 4.3
