# ADR-011: Multi-Comparator Validation Strategy

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** Paper validation track (PV-1 through PV-5)

## Context

RESONANCE had 1 strong validation (Bozic 2013, 10/10 seeds). One comparator is insufficient for credibility — it could be coincidence, overfitting, or cherry-picking. The question is whether to deepen the Bozic validation (quantitative fit) or broaden to multiple independent papers.

## Decision

Breadth over depth. Validate against 5 independent papers from different research groups, testing different phenomena, before deepening any single comparison.

| Paper | Group | Phenomenon | Independence from Bozic |
|-------|-------|-----------|------------------------|
| Zhang 2022 | Gatenby/Moffitt | Adaptive therapy dynamics | Different mechanism (on/off protocol vs combo) |
| Sharma 2010 | Settleman/MGH | Drug-tolerant persisters | Different scale (single cells vs populations) |
| GDSC/CCLE | Sanger/Broad | Hill pharmacology | Different domain (dose-response shape vs resistance) |
| Foo & Michor 2009 | Michor/Harvard | Pulsed vs continuous | Different optimization (dosing schedule vs drug selection) |
| Michor 2005 | Michor/MSKCC | Biphasic CML decline | Different observable (decline kinetics vs suppression %) |

5 independent qualitative matches from 5 different groups is stronger evidence than 1 quantitative fit to Bozic.

## Consequences

### Positive
- 6 comparators (5 new + Bozic) from 5 institutions — not cherry-picked
- Tests different aspects of the model (resistance, persistence, kinetics, dosing, pharmacology)
- Any single failure is informative (identifies model boundary) not catastrophic
- Combined evidence upgrades ASME V&V 40 credibility assessment from "limited" to "moderate"

### Negative
- Each comparison is shallow (qualitative, not quantitative)
- 5 shallow validations may be less convincing to domain experts than 1 deep one
- Spreads implementation effort across 5 experiments instead of perfecting 1

### Risks
- If 3+ validations fail, the model's credibility is damaged more than if we hadn't tried

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Deepen Bozic only (quantitative fit with cell counts, weeks, mutation rates) | 1 deep fit can be gamed; doesn't prove generality |
| Validate against 10+ papers | Diminishing returns — 5 from different phenomena covers the important axes |
| Validate against our own experiments (internal consistency only) | Circular — doesn't test against external reality |

## References

- `docs/sprints/PAPER_VALIDATION/README.md` — 5 papers selected
- `docs/regulatory/04_validation/VALIDATION_REPORT.md` (RD-4.4) — existing Bozic validation
- ASME V&V 40:2018 §6 — "validation evidence should span the applicability domain"
