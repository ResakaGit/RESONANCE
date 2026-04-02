# ADR-005: Competence-Through-Delivery Model (No Formal Training Certificates)

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** ISO 13485 Competence and Training Requirements

## Context

ISO 13485 Section 6.2 requires that personnel performing work affecting product quality be competent on the basis of appropriate education, training, skills, and experience. The organization must determine the necessary competence, provide training or take other actions, evaluate the effectiveness of those actions, and maintain records.

Traditional compliance approaches rely on formal training certificates: course completions, certification exams (e.g., Rust certification, Bevy training), regulatory training (GMP, GxP), and documented sign-offs by a quality manager.

RESONANCE is a single-developer project. There is no quality manager to sign off on training records, no HR department to track certifications, and no team to cross-train. The developer's competence is demonstrated entirely through the work product itself.

## Decision

Demonstrate developer competence through **sprint closures, test results, and code quality metrics** rather than formal training certificates.

The competence evidence chain:

| Competence Area | Evidence |
|----------------|---------|
| Rust language proficiency | 113K LOC compiling on stable 2024 edition, zero `unsafe`, zero `clippy` warnings |
| Bevy 0.15 ECS architecture | 14-layer system with correct scheduling, phase ordering, change detection |
| Simulation mathematics | 45+ pure math modules in `blueprint/equations/`, 3,113 tests passing |
| Regulatory knowledge | 43 regulatory documents authored, mapping to 7 international standards |
| Software lifecycle management | 78 archived sprint closures, each with DOD criteria met |
| Testing methodology | Unit, integration, property-based, batch, and headless simulation tests |

Each sprint closure (78 archived in `docs/sprints/archive/`) requires:
1. All tests pass (`cargo test` -- 3,113 tests)
2. Zero compiler warnings
3. Definition of Done criteria met
4. Grep-verified acceptance criteria green

## Consequences

### Positive
- Competence evidence is objective, verifiable, and tied to actual project work -- not to abstract course completion.
- The evidence is immutable (Git history) and auditable (sprint archives, test results).
- No cost for external training programs or certification exams.
- Competence record grows organically with the project -- every sprint closure adds evidence.

### Negative
- No third-party validation of competence. Self-assessed competence may not satisfy all auditors.
- Difficult to compare against industry benchmarks (e.g., "equivalent to 5 years Rust experience").
- If a second developer joins, the competence model needs formalization (onboarding checklist, supervised work period).

### Risks
- If RESONANCE reclassifies to Class B/C, formal training records may be required by the notified body. Mitigation: the delivery-based evidence can be supplemented with targeted training courses at that point, and the existing record provides a strong baseline.
- An auditor unfamiliar with software development may not recognize sprint closures as competence evidence. Mitigation: RD-5.8 includes a mapping table from ISO 13485 Section 6.2 requirements to specific RESONANCE evidence artifacts.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Formal Rust/Bevy training courses | Available but does not prove project-specific competence. A certificate says "completed course," not "built a 113K LOC simulation engine with 3,113 tests." |
| External code audit / peer review | Cost prohibitive for a single-developer research project. Would provide point-in-time validation, not ongoing competence record. |
| Pair programming logs | No second developer. Would require hiring or recruiting a volunteer, introducing coordination overhead without proportionate benefit. |
| Self-signed training certificates | Form without substance. Would satisfy the letter of ISO 13485 Section 6.2 but not the spirit. Dishonest. |

## References

- `docs/regulatory/01_foundation/RD-5.8` -- Competence and Training Records
- `docs/regulatory/01_foundation/RD-5.2` -- Quality Objectives (QO-1 through QO-6)
- `docs/sprints/archive/` -- 78 sprint closure records
- ISO 13485:2016, Section 6.2
- `CLAUDE.md` -- Testing section (3,113 tests, property tests, batch tests)
