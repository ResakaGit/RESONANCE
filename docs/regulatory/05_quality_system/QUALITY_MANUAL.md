---
document_id: RD-5.1
title: Quality Manual
standard: ISO 13485:2016 §4.2.2
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Quality Manual

## 1. Purpose

This Quality Manual defines the Quality Management System (QMS) for the RESONANCE project in accordance with ISO 13485:2016 §4.2.2. It describes the scope, exclusions, organizational structure, process interactions, and references to the 6 supporting QMS procedures.

RESONANCE is an emergent life simulation engine (113K LOC, 3,113 tests, Rust 2024 / Bevy 0.15) developed under AGPL-3.0. It is classified as a **research tool** (not a medical device) per RD-1.1 Intended Use and RD-1.5 Regulatory Strategy. This QMS is maintained voluntarily as best practice for software quality, pharma partnership readiness, and future regulatory optionality.

**Cross-references:**
- RD-1.1 `docs/regulatory/01_foundation/INTENDED_USE.md` --- Intended use, IMDRF Category I
- RD-1.2 `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` --- IEC 62304 Class A
- RD-1.4 `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` --- Lifecycle model, roles, V&V
- RD-1.5 `docs/regulatory/01_foundation/REGULATORY_STRATEGY.md` --- Regulatory positioning, gap analysis

## 2. QMS Scope

### 2.1 Scope Statement

The QMS applies to the design, development, verification, validation, and maintenance of the RESONANCE biomedical simulation software, including:

- All source code under `src/` (113K LOC, Rust 2024 edition, MSRV 1.85)
- All pure mathematical functions in `src/blueprint/equations/` (50+ domain files)
- All batch simulation code in `src/batch/` (33 stateless systems, zero Bevy dependency)
- All test infrastructure (3,113 tests: unit, integration, property, batch)
- Configuration files (`Cargo.toml`, `Cargo.lock`, `assets/maps/*.ron`)
- Sprint documentation (`docs/sprints/`, `docs/sprints/archive/`)
- Design specifications (`docs/design/`, `docs/arquitectura/`)
- Regulatory documentation (`docs/regulatory/`)
- Coding standards and project constitution (`CLAUDE.md`)

### 2.2 Product Scope

RESONANCE simulates emergent life dynamics, therapeutic resistance evolution, and drug interaction strategies from first principles. It operates on abstract energy units (qe), not clinical data. All behavior emerges from 8 foundational axioms and 4 fundamental constants. It is not intended for clinical decision-making.

**Codebase references:**
- 8 axioms: `CLAUDE.md` §The 8 Foundational Axioms
- 4 constants: `CLAUDE.md` §The 4 Fundamental Constants, `src/blueprint/equations/derived_thresholds.rs` (17 tests)
- 14 ECS layers: `src/layers/mod.rs`

## 3. Exclusions

The following ISO 13485:2016 clauses are excluded from the scope of this QMS, with justification per §4.2.2.

### 3.1 §7.3 Design and Development --- Purchasing Controls (§7.4)

**Exclusion:** Purchasing controls for external suppliers of components, materials, or services.

**Justification:** RESONANCE is open-source software (AGPL-3.0) with no purchased components, contracted services, or supplier relationships. All dependencies are open-source Rust crates governed by Hard Block HB-2 ("NO external crates without approval --- only what's in `Cargo.toml`"). Dependency management is handled through `Cargo.toml` (semver constraints) and `Cargo.lock` (exact pinning). No supplier evaluation or incoming inspection applies.

**Mitigating control:** Crate approval process (HB-2) and SOUP analysis (planned in RD-3) serve as the equivalent of supplier qualification for open-source dependencies.

### 3.2 §7.5.1 Control of Production and Service Provision

**Exclusion:** Manufacturing process controls, production equipment validation, and process validation for production.

**Justification:** RESONANCE is software, not a manufactured product. There is no physical production, no assembly, no sterilization, and no manufacturing facility. "Production" in the software context is compilation, which is fully deterministic and reproducible via `Cargo.lock` pinning and the Rust stable toolchain.

**Mitigating control:** Build reproducibility is ensured by `Cargo.lock` (exact dependency versions), deterministic compilation (Rust stable 2024 edition), and headless simulation reproducibility (`src/blueprint/equations/determinism.rs` --- hash-based RNG, 23 tests).

### 3.3 §7.5.5 Preservation of Product

**Exclusion:** Physical preservation requirements (packaging, handling, storage conditions, shelf life).

**Justification:** RESONANCE is digital-only. Distribution is via Git repository (GitHub). There is no physical medium, no packaging, no shipping, and no storage conditions. Digital integrity is preserved by Git SHA-256 content-addressing.

**Mitigating control:** Git immutable history provides tamper-evident storage. Repository is publicly accessible at `https://github.com/ResakaGit/RESONANCE`. Specific versions are identified by commit SHA and `Cargo.toml` version field.

### 3.4 §7.5.3 Traceability (Implantable Devices)

**Exclusion:** Special traceability requirements for implantable medical devices.

**Justification:** RESONANCE is software with no physical device, no implantable component, and no UDI requirement under current classification (research tool, not a medical device).

### 3.5 §6.4 Work Environment (Contamination Control)

**Exclusion:** Contamination control, cleanroom requirements, and environmental monitoring.

**Justification:** Software development has no contamination vectors. Development occurs on standard computing equipment (macOS Darwin ARM64, Linux x86_64).

## 4. Process Interaction Diagram

### 4.1 Sprint Lifecycle (Primary Process)

All development, corrective, and preventive activities follow the same sprint-based process:

```
                    ┌────────────────────────────────────────────┐
                    │           SPRINT LIFECYCLE                 │
                    │                                            │
    ┌───────────┐   │  ┌──────┐  ┌────────┐  ┌──────────────┐   │   ┌─────────┐
    │  Backlog   │──>│  │Scope │─>│ Design │─>│ Implement    │   │──>│ Archive │
    │  (README)  │   │  │      │  │        │  │ + Test       │   │   │         │
    └───────────┘   │  └──────┘  └────────┘  └──────┬───────┘   │   └─────────┘
                    │                               │            │
                    │                          ┌────v─────┐      │
                    │                          │ Review    │      │
                    │                          │ (Verif.)  │      │
                    │                          └────┬─────┘      │
                    │                               │            │
                    │                    PASS ──────>│<── BLOCK   │
                    │                               │    (rework)│
                    │                          ┌────v─────┐      │
                    │                          │ Closure   │      │
                    │                          │ (grep +   │      │
                    │                          │  test)    │      │
                    │                          └──────────┘      │
                    └────────────────────────────────────────────┘
```

### 4.2 Process Interactions

```
┌──────────────────────────────┐     ┌──────────────────────────────┐
│  MANAGEMENT PROCESSES        │     │  MEASUREMENT / IMPROVEMENT   │
│                              │     │                              │
│  Quality Policy (RD-5.2)     │     │  Internal Audit (RD-5.5)     │
│  Management Review (sprint   │     │  Nonconforming Product       │
│    backlog review)           │     │    (RD-5.6)                  │
│  Resource Allocation         │     │  CAPA (RD-5.7)              │
│    (sprint planning)         │     │  Test Results (cargo test)   │
└──────────────┬───────────────┘     └──────────────┬───────────────┘
               │                                     │
               v                                     v
┌──────────────────────────────────────────────────────────────────┐
│                    CORE PROCESSES                                 │
│                                                                  │
│  Design Input ──> Design ──> Implementation ──> Verification     │
│  (sprint doc)    (eqs +     (src/ code)        (cargo test +     │
│                   consts)                        grep closure)    │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────┐     │
│  │ Blueprint   │  │ Simulation  │  │ Verification         │     │
│  │ (pure math) │  │ (ECS sys)   │  │ (3,113 tests +       │     │
│  │ equations/  │  │ simulation/ │  │  proptest + bench)    │     │
│  └─────────────┘  └─────────────┘  └──────────────────────┘     │
└──────────────────────────────────────────────────────────────────┘
               │                                     ^
               v                                     │
┌──────────────────────────────────────────────────────────────────┐
│                    SUPPORT PROCESSES                              │
│                                                                  │
│  Document Control (RD-5.3)    Record Control (RD-5.4)           │
│  Configuration Management     Infrastructure (Rust toolchain,   │
│    (Git + Cargo.lock)           Bevy 0.15, criterion)           │
│  Coding Standards (CLAUDE.md)                                    │
└──────────────────────────────────────────────────────────────────┘
```

### 4.3 Input/Output Matrix

| Process | Input | Output | Records |
|---------|-------|--------|---------|
| Sprint Scoping | Backlog, axiom requirements | Sprint README.md with scope, entregables, criteria | Sprint document |
| Design | Sprint scope, axioms, existing equations | New equations in `blueprint/equations/`, constants | Design notes in sprint doc |
| Implementation | Design, coding standards (CLAUDE.md) | Source code in `src/`, tests co-located | Commit history (Git) |
| Verification | Source code, closure criteria | Test results (cargo test), grep verification | Sprint README checkboxes, CI output |
| Review | Implementation + verification results | PASS / WARN / BLOCK verdict | Verificador review in sprint doc |
| Closure | All criteria met + PASS verdict | Sprint moved to `docs/sprints/archive/` | Archived sprint doc |
| CAPA | Nonconformance detection | Root cause analysis, corrective fix, regression test | Sprint doc "Problema" section, new test |
| Internal Audit | Sprint closure criteria | Grep-verified checklist | Sprint README checkmarks |

## 5. Organizational Structure

### 5.1 Roles and Responsibilities

RESONANCE defines 4 functional roles. Roles are functional, not organizational --- a single contributor may hold multiple roles across different sprints but must not self-verify.

| Role | Name | Responsibility | QMS Mapping |
|------|------|----------------|-------------|
| Developer | **Alquimista** | Write code respecting 14 layers, Phase assignment, equations in `blueprint/equations/`. Output: impact analysis, code, plugin registration. | ISO 13485 §6.2: Competence. Executes design and development activities. |
| Reviewer | **Observador** | Review for DOD violations, math correctness, pipeline ordering, performance, Bevy 0.15 compliance. | ISO 13485 §7.3.5: Design and development verification. |
| Planner | **Planificador** | Decompose features into layers, validate orthogonality, define interaction matrix. Output: data flow, systems, events, equations, archetypes, risks. | ISO 13485 §5.5.1: Responsibility and authority. Defines QMS-relevant activities. |
| Verifier | **Verificador** | PR-level verification: (1) contract, (2) math, (3) DOD, (4) determinism, (5) performance, (6) tests. Verdict: PASS / WARN / BLOCK. | ISO 13485 §7.3.6: Design and development validation. BLOCK = nonconforming output. |

**Escalation:** BLOCK verdicts halt the sprint. Resolution requires re-implementation and re-verification. No BLOCK may be overridden without written justification.

**Reference:** `CLAUDE.md` §Roles table.

### 5.2 Quality Responsibility

Quality responsibility is distributed, not centralized. Every role has quality obligations:

- **Alquimista:** Must follow coding rules, hard blocks, and axiom constraints. Must write co-located tests.
- **Observador:** Must flag DOD violations, math errors, and component bloat.
- **Planificador:** Must validate orthogonality and define closure criteria.
- **Verificador:** Must BLOCK on math doubt or determinism doubt. Must verify all closure criteria (grep + test).

### 5.3 Management Representative

In the current project structure, the Planificador role serves as the de facto management representative for QMS purposes. This role:
- Defines sprint scope and closure criteria (design control input)
- Reviews sprint backlog and prioritization (management review equivalent)
- Ensures regulatory documentation sprints (RD-1 through RD-7) are planned and executed

**Known gap:** There is no formal management review meeting. The sprint backlog review (`docs/sprints/README.md`) and sprint closure process serve as the functional equivalent. A formal management review process should be established if RESONANCE pursues SaMD classification.

## 6. QMS Procedures

This Quality Manual references the following 6 mandatory procedures required by ISO 13485:2016.

| Procedure | Document ID | ISO 13485 Clause | Path |
|-----------|-------------|-------------------|------|
| Quality Policy and Objectives | RD-5.2 | §5.3, §5.4.1 | `docs/regulatory/05_quality_system/QUALITY_POLICY.md` |
| Document Control | RD-5.3 | §4.2.4 | `docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md` |
| Record Control | RD-5.4 | §4.2.5 | `docs/regulatory/05_quality_system/RECORD_CONTROL.md` |
| Internal Audit | RD-5.5 | §8.2.4 | `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` |
| Nonconforming Product | RD-5.6 | §8.3 | `docs/regulatory/05_quality_system/NONCONFORMING_PRODUCT.md` |
| CAPA | RD-5.7 | §8.5.2, §8.5.3 | `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` |

## 7. Regulatory Documentation Cross-References

### 7.1 Foundation (RD-1)

| Document | Path | Relevance to QMS |
|----------|------|-------------------|
| RD-1.1 Intended Use | `docs/regulatory/01_foundation/INTENDED_USE.md` | Defines product scope and user profiles |
| RD-1.2 Safety Class | `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` | Determines lifecycle rigor (Class A) |
| RD-1.3 SRS | `docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md` | Requirements baseline for design control |
| RD-1.4 SDP | `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` | Lifecycle model, roles, V&V strategy |
| RD-1.5 Strategy | `docs/regulatory/01_foundation/REGULATORY_STRATEGY.md` | Regulatory positioning, gap analysis |

### 7.2 Risk Management (RD-2)

Planned. Will contain: Risk Management Plan, Hazard Analysis (FMEA), Risk Evaluation, Risk Controls, Residual Risk, Risk Management Report. Location: `docs/regulatory/02_risk_management/`.

### 7.3 Traceability (RD-3)

Planned. Will contain: Traceability Matrix, SOUP Analysis, SBOM, Configuration Management Plan. Location: `docs/regulatory/03_traceability/`.

### 7.4 Validation (RD-4)

Planned. Will contain: V&V Plan, Credibility Assessment (ASME V&V 40), Verification Report, Validation Report, Uncertainty Quantification. Location: `docs/regulatory/04_validation/`.

### 7.5 Clinical (RD-6)

Planned. Will contain: Clinical Evaluation Plan, Clinical Evaluation Report, Limitations, Reproducibility, Reference Data. Location: `docs/regulatory/06_clinical/`.

### 7.6 Release (RD-7)

Planned. Will contain: Part 11 Assessment, Data Integrity Policy, Audit Trail, Cybersecurity, Release Package. Location: `docs/regulatory/07_release/`.

## 8. Governing Documents

The QMS is built upon and references the following project-level governing documents:

| Document | Path | Authority |
|----------|------|-----------|
| `CLAUDE.md` | Repository root | Project constitution: 8 axioms, 4 constants, 14 coding rules, 17 hard blocks, 4 roles, inference protocol |
| `docs/ARCHITECTURE.md` | `docs/` | Canonical architecture: module map, pipeline, drug models, emergence status |
| `docs/sprints/README.md` | `docs/sprints/` | Sprint backlog: active/archived sprints, wave tracking, global metrics |
| `docs/sprints/archive/README.md` | `docs/sprints/archive/` | Archived sprint index: 49 tracks, 78 sprints completed |

## 9. Known Gaps

The following gaps between current practice and full ISO 13485:2016 compliance are documented for transparency.

| Gap | ISO 13485 Clause | Current State | Mitigation |
|-----|-------------------|---------------|------------|
| No formal management review meeting | §5.6 | Sprint backlog review serves as functional equivalent | Document formal review when pursuing SaMD |
| No training records | §6.2 | Roles defined in CLAUDE.md but no evidence of competence assessment | Not required for research tool; document if pursuing SaMD |
| No customer communication procedure | §7.2.3 | GitHub issues serve as ad hoc mechanism | Formalize if user base grows |
| No post-market surveillance | §8.2.1 | Not applicable (research tool, not marketed) | Required only if SaMD classification is pursued |
| No advisory notice procedure | §8.2.3 | Not applicable (no deployed medical device) | Required only if SaMD classification is pursued |
| Purchasing controls excluded | §7.4 | Open-source crate approval via HB-2 | SOUP analysis (RD-3) will formalize dependency assessment |
| No formal design review records | §7.3.4 | Sprint closure criteria + Verificador verdict serve as equivalent | Formalize review records if pursuing SaMD |

## 10. Codebase References

| Claim | Reference | Verification |
|-------|-----------|--------------|
| 113K LOC | `src/` directory | `find src -name '*.rs' \| xargs wc -l` |
| 3,113 tests | `cargo test` output | `cargo test` --- 3,113 passed, 0 failures |
| 49 archived tracks, 78 sprints | `docs/sprints/archive/README.md` | Count index entries |
| 4 roles | `CLAUDE.md` §Roles | Read document |
| 14 coding rules | `CLAUDE.md` §Coding Rules | Read document |
| 17 hard blocks (5 absolute + 12 strong defaults) | `CLAUDE.md` §Hard Blocks | Read document |
| 8 axioms, 4 constants | `CLAUDE.md` §Axioms, §Constants | Read document |
| AGPL-3.0 license | `LICENSE` file, `Cargo.toml` | Repository root |
| Git trunk-based on `main` | Git configuration | `git branch` |
| Current commit | `971c7acb99decde45bf28860e6e10372718c51e2` | `git log -1` |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. QMS scope, exclusions, process interactions, organizational structure, procedure references, known gaps. Retroactive formalization of existing practices per ISO 13485:2016 §4.2.2. |
