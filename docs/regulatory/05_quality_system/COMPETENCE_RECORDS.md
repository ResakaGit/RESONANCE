---
document_id: RD-5.8
title: Personnel Competence and Training Records
standard: ISO 13485:2016 §6.2, ISO 14971:2019 §4.3
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Personnel Competence and Training Records

## 1. Purpose

This document defines the competence requirements, training sources, and verification methods for personnel involved in the development, review, and maintenance of RESONANCE. It satisfies ISO 13485:2016 §6.2 (Human Resources --- Competence, training, and awareness) and ISO 14971:2019 §4.3 (Qualification of personnel performing risk management activities).

ISO 13485:2016 §6.2 requires that:
> Personnel performing work affecting product quality shall be competent on the basis of appropriate education, training, skills, and experience. The organization shall determine the necessary competence, provide training or take other actions to achieve the necessary competence, evaluate the effectiveness of the actions taken, and ensure that its personnel are aware of the relevance and importance of their activities.

**Cross-references:**
- RD-1.4 `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` --- Roles and responsibilities (§3)
- RD-2.1 `docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md` --- Risk management responsibilities
- RD-5.1 `docs/regulatory/05_quality_system/QUALITY_MANUAL.md` --- QMS organizational structure
- RD-5.5 `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` --- Audit competence requirements
- `CLAUDE.md` §Roles --- Alquimista, Observador, Planificador, Verificador

## 2. Organizational Context

RESONANCE is developed by a small team operating as a research project, not a commercial organization. The development team consists of a single principal developer (author/architect) supported by AI-assisted development tooling (Claude Code). This context is critical for interpreting this document:

- **Team size:** 1 principal developer + AI tooling
- **Organizational structure:** Flat --- all 4 roles (§3) are performed by the same individual
- **Employment relationship:** Independent researcher, not employed by a medical device manufacturer
- **Regulatory status:** Research tool, not regulated as a medical device (RD-1.1, RD-1.5)

**Gap acknowledged:** In a regulated medical device organization, ISO 13485 §6.2 would require formal training records, competence assessments signed by management, and periodic re-evaluation. RESONANCE's single-developer model means the "organization" and the "personnel" are the same person. This document records the competence profile honestly rather than creating fictional organizational structures.

## 3. Competence Requirements by Role

### 3.1 Alquimista (Developer)

| Competence Area | Required Knowledge | Evidence of Competence |
|-----------------|-------------------|----------------------|
| Rust programming | Rust 2024 edition idioms, ownership model, trait system, generics, iterators, `#[derive]` macros, `f32` precision constraints | 113K LOC authored in Rust; zero `unsafe` in simulation code (RS-01); 3,113 compiling tests |
| Bevy 0.15 ECS | Entity-Component-System architecture, `Query`, `Res`/`ResMut`, `Commands`, `SystemSet`, `FixedUpdate`, component storage modes, observer patterns | 14 ECS layers implemented (`src/layers/`), 6-phase pipeline (`src/simulation/pipeline.rs`), plugin registration |
| ECS design patterns | Sparse set vs table storage, change detection, query width minimization, event chaining, phase ordering | `CLAUDE.md` §Coding Rules (17 rules enforced by code review) |
| Simulation physics | Energy conservation, Kleiber scaling, dissipation dynamics, Coulomb/LJ potentials, Hill pharmacokinetics, Kuramoto oscillators | `src/blueprint/equations/` --- 45+ domain files, all pure functions with unit tests |
| Mathematical modeling | Sigmoid functions, Gaussian alignment, gradient descent optimization, DAG flow networks, Union-Find, HP lattice folding | Equations implemented with boundary tests, edge case coverage, and property-based fuzzing |
| Deterministic computation | Hash-based RNG, floating-point reproducibility (`f32::to_bits`), canonical ordering, avoiding non-deterministic iteration | `src/blueprint/equations/determinism.rs` --- 23 tests; bit-exact reproducibility demonstrated |

### 3.2 Observador (Code Reviewer)

| Competence Area | Required Knowledge | Evidence of Competence |
|-----------------|-------------------|----------------------|
| Code review methodology | DOD (Definition of Done) violation detection, axiom compliance verification, math correctness auditing | Sprint closure reviews documented in `docs/sprints/archive/` (78 archived sprints) |
| Axiom compliance | Understanding of 8 axioms + 4 constants, ability to detect violations (energy creation, hardcoded trophic classes, non-emergent behavior) | All sprint reviews include axiom compliance check; zero axiom violations in codebase |
| Bevy 0.15 compliance | Bevy 0.15 API patterns (tuple spawn, no `#[derive(Bundle)]`, `StateScoped`, `#[require]`) | Code follows Bevy 0.15 patterns per `CLAUDE.md` §Bevy 0.15 Patterns |
| Performance awareness | Query width, system scope (max 5 component types), `SparseSet` for transients, sorted `Vec` over `HashMap` in hot paths | Performance rules documented and enforced in `CLAUDE.md` §Coding Rules |

### 3.3 Planificador (Architect/Planner)

| Competence Area | Required Knowledge | Evidence of Competence |
|-----------------|-------------------|----------------------|
| ECS architecture | 14-layer orthogonality, layer interaction matrix, 5-test new-layer validation | `docs/ARCHITECTURE.md` canonical architecture; `CLAUDE.md` §New Layer checklist |
| Decomposition methodology | Data-first decomposition (component -> system -> event -> equation -> archetype), vertical slice pattern | Sprint scope definitions in `docs/sprints/` track the decomposition per feature |
| Risk identification | Ability to identify where changes destabilize axioms, conservation invariants, or determinism | Risk analysis (RD-2.2) identifies 12 hazards with codebase-specific evidence |
| Sprint planning | Scope definition, entregables, closure criteria (grep-verifiable), dependency ordering (waves) | 112 total sprints planned; 78 archived with verified closure criteria |

### 3.4 Verificador (PR Reviewer / Verification)

| Competence Area | Required Knowledge | Evidence of Competence |
|-----------------|-------------------|----------------------|
| Testing methodology | Unit testing (pure math), integration testing (MinimalPlugins), property testing (proptest), batch testing | 3,113 tests across 4 testing layers; `tests/property_conservation.rs` (19 proptest fuzz tests) |
| Mathematical verification | Ability to verify equation correctness: boundary conditions, conservation properties, monotonicity, convergence | All `blueprint/equations/` modules have dedicated test suites with edge cases |
| Determinism verification | Ability to verify bit-exact reproducibility across runs | Determinism test suite: 23 tests + cross-experiment determinism checks |
| DOD enforcement | 6-point checklist: (1) contract (2) math (3) DOD (4) determinism (5) perf (6) tests; PASS/WARN/BLOCK verdict | `CLAUDE.md` §Roles --- Verificador checklist documented |

## 4. Training Sources

### 4.1 Primary Training Material

| Source | Content | Role Coverage |
|--------|---------|---------------|
| `CLAUDE.md` | Project constitution: axioms, constants, architecture, coding rules, hard blocks, design templates, checklists, inference protocol | All 4 roles |
| `docs/ARCHITECTURE.md` | Canonical architecture documentation: layers, pipeline, drug models, energy cycle, evolution | Planificador, Observador |
| `docs/design/*.md` | Design specifications: topology, eco boundaries, bridge optimizer, axiomatic closure, simulation decoupling, terrain mesher, quantized color, folder structure | Planificador, Alquimista |
| `docs/arquitectura/*.md` | Module contracts: batch simulator, bridge optimizer, axiomatic closure, blueprint math | Planificador, Verificador |

### 4.2 External Training References

| Source | Content | Role Coverage |
|--------|---------|---------------|
| The Rust Programming Language (rustbook) | Rust language fundamentals, ownership, lifetimes, generics | Alquimista |
| Bevy 0.15 documentation (bevyengine.org) | ECS API, system scheduling, component storage, plugin architecture | Alquimista, Observador |
| Bevy 0.15 migration guide | Breaking changes from 0.14, new patterns (`#[require]`, `StateScoped`) | Alquimista |
| ISO 14971:2019 | Risk management for medical devices (referenced, not formally trained) | Planificador (risk analysis) |
| IEC 62304:2006+Amd1:2015 | Medical device software lifecycle (referenced, not formally trained) | All roles (process awareness) |
| Bozic et al. 2013 (eLife) | Evolutionary dynamics of cancer resistance --- validation reference | Verificador |
| Gatenby et al. 2009 (Cancer Research) | Adaptive therapy --- calibration reference | Verificador |

### 4.3 Training Gaps

| Gap | Impact | Mitigation |
|-----|--------|------------|
| No formal ISO 13485 training certificate | Cannot demonstrate formal QMS training to an auditor | Competence demonstrated through QMS documentation quality (37+ regulatory documents authored). If pharma partnership or regulatory submission pursued, formal ISO 13485 Lead Implementer training would be obtained. |
| No formal ISO 14971 training certificate | Cannot demonstrate formal risk management training | Risk management file (RD-2.1 through RD-2.6) demonstrates practical competence. Formal training available from BSI, TUV, or equivalent if required. |
| No formal IEC 62304 training certificate | Cannot demonstrate formal software lifecycle training | Software lifecycle practices documented in RD-1.4 and demonstrated by 78 archived sprints. |
| No GxP / GAMP 5 training certificate | Cannot demonstrate pharmaceutical validation competence | GAMP 5 principles applied in validation documentation (RD-4.1 through RD-4.5). Formal training available from ISPE if required. |
| No Bevy certification (none exists) | No industry certification available for Bevy | Competence demonstrated by working codebase: 113K LOC, 14 layers, 6-phase pipeline, 3,113 passing tests. |

## 5. Training Verification Method

### 5.1 Competence-Through-Delivery Model

In the absence of formal training certificates, RESONANCE uses a **competence-through-delivery** model: successful completion of sprint deliverables demonstrates competence in the required skills.

| Verification Method | Evidence | Applies To |
|---------------------|----------|------------|
| Sprint closure | Sprint README closure criteria verified (grep-based) | All roles |
| Test passage | `cargo test` --- 3,113 tests, 0 failures | Alquimista, Verificador |
| Axiom compliance | No axiom violations detected in codebase | Observador |
| Math correctness | All `blueprint/equations/` modules have boundary tests | Alquimista, Verificador |
| Documentation quality | 37+ regulatory documents authored, internally consistent, cross-referenced | All roles |
| Property test coverage | `tests/property_conservation.rs` --- 19 proptest fuzz tests pass | Verificador |
| Determinism verification | Bit-exact reproducibility demonstrated across experiments | Alquimista, Verificador |
| Publication | Paper published on Zenodo (DOI: 10.5281/zenodo.19342036) with peer-accessible methodology | All roles |

### 5.2 Ongoing Competence Maintenance

| Activity | Frequency | Record |
|----------|-----------|--------|
| Sprint delivery | Per sprint (irregular cadence) | Sprint archive in `docs/sprints/archive/` |
| Full test suite execution | Before every merge | `cargo test` output (not formally archived --- gap) |
| Code review against CLAUDE.md | Every code change | Git commit history + sprint review notes |
| Regulatory document review | When new documents are authored | Document version history (§Revision History in each document) |
| SOUP vulnerability check | Per release or when `cargo audit` reports findings | `cargo audit` output (RD-3.2 SOUP Analysis) |

### 5.3 Verification Gaps

| Gap | Severity | Mitigation |
|-----|----------|------------|
| No formal competence assessment signed by management | Medium | Single-developer project --- no management layer to sign. Competence evidenced by deliverables. |
| No periodic re-evaluation schedule | Low | Competence implicitly re-verified every sprint. Formal schedule would add bureaucratic overhead without safety benefit for Class A tool. |
| `cargo test` results not formally archived per run | Low | Git commit history records that tests passed before merge (implicit). Formal CI/CD with archived test logs would close this gap. |
| No training plan for onboarding new contributors | Low | Currently single-developer. `CLAUDE.md` serves as the onboarding document. Formal onboarding plan would be created if team expands. |

## 6. Risk Management Personnel Qualification (ISO 14971 §4.3)

ISO 14971:2019 §4.3 requires that personnel performing risk management activities have "knowledge and experience appropriate to the tasks assigned to them."

### 6.1 Risk Management Roles

| Risk Management Activity | RESONANCE Role | Qualification Evidence |
|--------------------------|----------------|----------------------|
| Hazard identification | Planificador | 12 hazards identified in RD-2.2 with codebase-specific evidence |
| Risk estimation | Planificador | Severity and probability ratings with justification per hazard |
| Risk evaluation | Planificador + Observador | Acceptability determination using 5x5 P-S matrix (RD-2.3) |
| Risk control implementation | Alquimista | 52 controls implemented with verification evidence (RD-2.4) |
| Risk control verification | Verificador | Each control verified via test, grep, or document review (RD-2.4) |
| Residual risk assessment | Planificador | Post-control risk levels assessed (RD-2.5), overall risk acceptable (RD-2.6) |

### 6.2 Qualification Basis

The risk management personnel qualification is based on:

1. **Domain knowledge:** Developer has authored the entire simulation engine (113K LOC) and understands all failure modes, abstractions, and limitations.
2. **Standards awareness:** ISO 14971, IEC 62304, and IMDRF SaMD frameworks referenced and applied (though without formal certification --- gap documented in §4.3).
3. **Scientific literacy:** Published paper with limitations section (Zenodo DOI: 10.5281/zenodo.19342036) demonstrates ability to identify and communicate scope boundaries.
4. **Honest scope acknowledgment:** README, CLAUDE.md, and in-code disclaimers consistently acknowledge what RESONANCE is NOT --- demonstrating risk awareness.

### 6.3 Limitation

A single individual performing all risk management roles creates a concentration risk: there is no independent reviewer to challenge assumptions or identify blind spots. This is mitigated by:

- The Inference Protocol (`CLAUDE.md` §Inference Protocol) which mandates self-critique, alternatives, and red-line challenges.
- AI-assisted development tooling providing a second perspective on code and documentation.
- Publication on Zenodo, inviting external scrutiny.

This limitation would need to be resolved (by adding an independent reviewer) if RESONANCE were reclassified above Class A.

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. Competence requirements per role, training sources, verification-through-delivery model, gaps documented honestly. |
