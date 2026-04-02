---
document_id: RD-5.10
title: Training and Competence Matrix
standard: ISO 13485:2016 §6.2, IEC 62304 §5.1
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Training and Competence Matrix

## 1. Purpose

This document provides the formal training and competence matrix for all roles involved in RESONANCE development, review, and maintenance. It satisfies ISO 13485:2016 §6.2 (Human Resources) by mapping each role to its required competencies, the evidence type that demonstrates competence, and the source of that evidence.

This matrix complements RD-5.8 (Competence Records) by providing a concise, auditable summary of competence status per role.

**Cross-references:**
- RD-5.8 `docs/regulatory/05_quality_system/COMPETENCE_RECORDS.md` --- Detailed competence requirements, training sources, gaps
- RD-1.4 `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` --- Roles and responsibilities (§3)
- RD-5.1 `docs/regulatory/05_quality_system/QUALITY_MANUAL.md` --- QMS organizational structure
- `CLAUDE.md` §Roles --- Alquimista, Observador, Planificador, Verificador

## 2. Competence Matrix

### 2.1 Alquimista (Developer)

| Required Competence | Evidence Type | Evidence Source | Status |
|-------------------|---------------|-----------------|--------|
| Rust 2024 edition (ownership, traits, generics, iterators, `f32` precision) | Sprint closures (code merged + tests green) | `docs/sprints/archive/` (78 closures) | Active |
| Bevy 0.15 ECS (Query, Res/ResMut, Commands, SystemSet, FixedUpdate, observers) | Sprint closures (code merged + tests green) | `docs/sprints/archive/` (78 closures) | Active |
| ECS design patterns (SparseSet, change detection, query width, event chaining) | Code review compliance with CLAUDE.md §Coding Rules | Git commit history + sprint reviews | Active |
| 8 foundational axioms (energy, conservation, dissipation, oscillation) | Zero axiom violations in 113K LOC | Codebase audit via grep (RD-5.5 §4.3) | Active |
| 14 orthogonal ECS layers (L0--L13, composition model) | Layer implementations authored and tested | `src/layers/` (24+ files) | Active |
| DOD compliance (hard blocks, coding rules, design templates) | Zero unjustified `// DEBT:` annotations | Sprint closure grep verification | Active |

### 2.2 Observador (Code Reviewer)

| Required Competence | Evidence Type | Evidence Source | Status |
|-------------------|---------------|-----------------|--------|
| Code review methodology (DOD violation detection, axiom compliance) | PR review history (>=10 reviews) | GitHub PR review comments | Active |
| Axiom compliance verification (detect energy creation, hardcoded trophic classes) | Sprint closure reviews (0 axiom BLOCKs) | `docs/sprints/archive/` closure criteria | Active |
| Math correctness auditing (boundary conditions, conservation, monotonicity) | Equation review evidence in sprint closures | Sprint review notes + test verification | Active |
| Bevy 0.15 pattern compliance (tuple spawn, no Bundle derive, StateScoped) | Code review against CLAUDE.md §Bevy 0.15 Patterns | GitHub PR review comments | Active |

### 2.3 Planificador (Architect/Planner)

| Required Competence | Evidence Type | Evidence Source | Status |
|-------------------|---------------|-----------------|--------|
| ECS architecture (14-layer orthogonality, interaction matrix, 5-test validation) | Sprint track designs (>=3 tracks) | `docs/sprints/` track READMEs | Active |
| Layer orthogonality assessment (new layer 5-test, vertical slice decomposition) | Architecture documentation authored | `docs/ARCHITECTURE.md`, `CLAUDE.md` §New Layer | Active |
| Sprint design (scope, entregables, closure criteria, dependency ordering) | 112 sprints planned; 78 archived with verified closure | `docs/sprints/archive/` index | Active |
| Risk identification (axiom destabilization, conservation invariant breaks) | 12 hazards identified with codebase evidence | RD-2.2 Risk Analysis | Active |

### 2.4 Verificador (PR Reviewer)

| Required Competence | Evidence Type | Evidence Source | Status |
|-------------------|---------------|-----------------|--------|
| Full DOD enforcement (6-point checklist: contract, math, DOD, determinism, perf, tests) | PASS/WARN/BLOCK verdicts (>=5 PRs) | GitHub PR review verdicts | Active |
| Determinism verification (hash-based RNG, f32 reproducibility, canonical ordering) | Determinism test suite maintenance (23 tests) | `src/blueprint/equations/determinism.rs` | Active |
| Performance awareness (query width, system scope, SparseSet, hot path analysis) | Performance-related review comments | GitHub PR review comments | Active |
| Bevy 0.15 compliance (API patterns, migration compliance) | Code review against CLAUDE.md §Bevy 0.15 Patterns | GitHub PR review comments | Active |
| Regulatory awareness (ISO 13485, IEC 62304, ISO 14971 process knowledge) | Regulatory document authorship and review | `docs/regulatory/` (37+ documents) | Active |

## 3. Competence-Through-Delivery Model

RESONANCE uses a **competence-through-delivery** model as defined in ADR-005 (architectural decision record) and detailed in RD-5.8 §5.1. In this model, successful completion of sprint deliverables demonstrates competence in the required skills.

### 3.1 Rationale

For a research project with a single principal developer, formal training certificates (ISO Lead Implementer, Bevy certification) are either unavailable (Bevy) or disproportionate to the project's risk class (Class A). Competence is instead demonstrated by:

1. **Working software:** 113K LOC, 3,113 passing tests, zero `unsafe`, zero axiom violations
2. **Documented process:** 78 archived sprints with grep-verified closure criteria
3. **Published work:** Zenodo paper (DOI: 10.5281/zenodo.19342036) with limitations honestly stated
4. **Regulatory documentation:** 37+ regulatory documents authored, internally consistent, cross-referenced

### 3.2 Limitations

The competence-through-delivery model has known limitations:

| Limitation | Impact | Mitigation |
|-----------|--------|------------|
| No formal certificates | Cannot demonstrate formal training to an external auditor | Deliverable evidence (§3.1) substitutes; acceptable for Class A |
| No independent assessor | Same person performs and assesses work | Grep-based verification provides objective evidence (RD-5.5 §4) |
| No peer benchmarking | Cannot compare competence to industry norms | Publication on Zenodo invites external scrutiny |

## 4. Upgrade Path

### 4.1 Reclassification Trigger

If RESONANCE is reclassified to **Class B or higher** (IEC 62304), the following formal training would be required:

| Training | Standard | Provider Examples | Timeline |
|----------|----------|-------------------|----------|
| IEC 62304 Software Lifecycle | IEC 62304:2006+Amd1:2015 | BSI, TUV, Johner Institut | Within 6 months of reclassification |
| ISO 14971 Risk Management | ISO 14971:2019 | BSI, TUV, ISPE | Within 6 months of reclassification |
| ISO 13485 Lead Implementer | ISO 13485:2016 | BSI, SGS, TUV | Within 12 months of reclassification |
| GAMP 5 (if pharma partnership) | ISPE GAMP 5 | ISPE | Within 12 months of partnership |

### 4.2 Current Classification

RESONANCE is classified as IEC 62304 Class A and IMDRF SaMD Category I (RD-1.2). The competence-through-delivery model is appropriate for this classification. Formal training is not required but would be obtained if reclassification occurs.

## 5. Refresh Cycle

| Activity | Frequency | Trigger |
|----------|-----------|---------|
| Matrix review | Annually | Calendar (first review: Q1 2027) |
| Competence re-verification | Per sprint | Sprint closure criteria verification |
| Matrix update | When new standard added to scope | Addition of new regulatory standard reference |
| Role competence re-assessment | When team membership changes | New contributor onboarding |

## 6. Gaps

### 6.1 Acknowledged Gaps

| Gap | Severity | Acceptable for Class A? | Resolution Path |
|-----|----------|------------------------|-----------------|
| No formal ISO 13485 certificate | Medium | Yes --- competence demonstrated through QMS documentation quality | Obtain if reclassified to Class B+ |
| No formal ISO 14971 certificate | Medium | Yes --- risk file demonstrates practical competence | Obtain if reclassified to Class B+ |
| No formal IEC 62304 certificate | Medium | Yes --- software lifecycle practices documented in 78 sprints | Obtain if reclassified to Class B+ |
| No Bevy certification (none exists) | N/A | Yes --- no industry certification available | Competence demonstrated by 113K LOC working codebase |
| No independent competence assessor | Low | Yes --- grep-based verification provides objectivity | Add independent reviewer if team grows |

### 6.2 Honest Assessment

This matrix documents **actual competence as evidenced by deliverables**, not aspirational qualifications. No training certificates are claimed that do not exist. No organizational structures are fabricated. The gaps in §6.1 are real and would need to be closed for regulatory submission or reclassification above Class A.

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial training matrix. 4 roles mapped to competencies with evidence sources. Competence-through-delivery model documented. Upgrade path for Class B+ defined. Gaps acknowledged honestly. |
