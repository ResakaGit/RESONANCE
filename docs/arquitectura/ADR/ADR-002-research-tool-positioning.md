# ADR-002: Research Tool Positioning (Not SaMD)

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** IMDRF SaMD Classification and Regulatory Strategy

## Context

RESONANCE includes drug pathway modeling (cytotoxic therapy with Hill pharmacokinetics, pathway inhibitor with three inhibition modes), an adaptive therapy controller, four clinical calibration profiles (CML, prostate, NSCLC, canine MCT), and Bozic 2013 validated combination therapy predictions. These capabilities place RESONANCE in a gray zone: the software has functionality that could qualify as Software as a Medical Device (SaMD) under IMDRF N10 if the intended use were to inform clinical decisions.

The regulatory positioning decision affects every downstream document -- risk management, quality system scope, post-market obligations, and partnership eligibility.

## Decision

Position RESONANCE as a **computational research tool**, not Software as a Medical Device. Adopt voluntary compliance with medical device standards (IEC 62304, ISO 14971, ISO 13485) as best practice, not as regulatory obligation.

This means:
1. RESONANCE is **IMDRF Category I** (informing clinical management, not treating/diagnosing).
2. Even within Category I, the intended use is restricted to **research hypothesis generation**, not clinical decision support.
3. All 43 regulatory documents are voluntary -- they demonstrate rigor and credibility but do not create regulatory obligations.
4. The decision is **reversible**: RD-1.5 documents the explicit pathway to SaMD classification if intended use evolves.

## Consequences

### Positive
- Voluntary compliance provides credibility for academic partnerships, journal submissions, and grant applications without the regulatory burden of formal SaMD designation.
- No post-market surveillance obligations, no mandatory adverse event reporting, no notified body involvement.
- Documentation investment is not wasted -- it directly maps to SaMD requirements if reclassification is needed.
- Honest positioning that matches actual capability (abstract qe units, no patient data, no clinical validation).

### Negative
- Cannot market RESONANCE as a clinical tool or claim clinical utility without reclassification.
- Some potential partners or institutions may require formal SaMD status before engaging, regardless of voluntary compliance quality.

### Risks
- If a user deploys RESONANCE in a clinical context without reclassification, both the user and the developer could face regulatory scrutiny. Mitigation: disclaimers in README, paper, and all clinical-adjacent documentation.
- Voluntary compliance may create a false sense of regulatory readiness. Actual SaMD submission would require clinical validation studies, predicate device comparison, and potentially a 510(k) or De Novo pathway.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Declare as SaMD immediately | Premature -- no patient data, no clinical validation, no predicate device. Would create regulatory obligations the project cannot currently meet (post-market surveillance, complaint handling, CAPA with external reporting). |
| Ignore regulatory entirely | Risky -- limits academic credibility, blocks partnerships with regulated institutions, makes future SaMD pathway harder. The documentation effort is modest relative to the credibility gained. |
| Classify as "wellness" or "general purpose" software | Dishonest -- RESONANCE has explicit drug modeling capabilities. Pretending otherwise invites regulatory challenge if the project gains visibility. |

## References

- `docs/regulatory/01_foundation/RD-1.1` -- System Requirements Specification (Intended Use)
- `docs/regulatory/01_foundation/RD-1.5` -- SaMD Classification & Reclassification Criteria
- IMDRF/SaMD WG/N10 -- Framework for Risk Categorization
- IMDRF/SaMD WG/N12 -- Key Definitions
- FDA Guidance: "Software as a Medical Device (SaMD): Clinical Evaluation" (2017)
- `CLAUDE.md` -- "Honest scope (all levels)" section
