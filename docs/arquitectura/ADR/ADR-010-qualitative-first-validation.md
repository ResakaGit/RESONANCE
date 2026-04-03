# ADR-010: Qualitative-First Validation Strategy

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** Paper validation track (PV-1 through PV-5)

## Context

RESONANCE operates in abstract energy units (qe), not molar concentrations or cell counts. Published papers report results in clinical units (months, nM, cells/mL). Direct quantitative comparison is impossible without a calibration layer that introduces its own assumptions.

The question is: validate quantitatively (requiring calibration) or qualitatively (structural patterns)?

## Decision

Validate structural predictions first, not absolute values. Each experiment targets a qualitative truth that must hold regardless of unit system:

| Paper | Qualitative target (unit-independent) |
|-------|--------------------------------------|
| Zhang 2022 | Adaptive TTP > continuous TTP |
| Sharma 2010 | Small fraction survives, recovers sensitivity after drug removal |
| GDSC/CCLE | RESONANCE's n=2 falls within empirical distribution |
| Foo & Michor 2009 | Optimal dose exists (non-monotonic resistance curve) |
| Michor 2005 | Biphasic decline with stem cell survival |

Acceptance thresholds are ranges (e.g., "slope ratio 3-15×") not exact values (e.g., "slope ratio = 7.2×").

## Consequences

### Positive
- Does not require calibration constants that could be wrong
- Structural truths are stronger evidence than fitted curves (harder to fake)
- A structural match across 5 independent papers is more convincing than 1 quantitative fit
- Honest about what abstract units can and cannot prove

### Negative
- Cannot claim "RESONANCE predicts TTP = 33.5 months" — only "adaptive > continuous"
- Reviewers expecting quantitative validation may find this insufficient
- Does not close the gap to clinical utility (still needs calibration for real use)

### Risks
- If all 5 qualitative validations pass, it may create false confidence that the model is quantitatively correct

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Quantitative calibration per paper (map qe→molar, gen→days) | Introduces free parameters that could overfit; calibration itself needs validation |
| Skip validation entirely, rely on Bozic alone | 1 comparator is insufficient for credibility |
| Validate only against computational models (not experimental data) | Model-vs-model comparison doesn't prove biological relevance |

## References

- `docs/regulatory/06_clinical/LIMITATIONS_REPORT.md` (RD-6.3) — CAN/CANNOT scope
- `docs/regulatory/04_validation/CREDIBILITY_MODEL.md` (RD-4.2) — ASME V&V 40 §8 Applicability
- ADR-004 (abstract energy units) — foundational decision that constrains validation approach
