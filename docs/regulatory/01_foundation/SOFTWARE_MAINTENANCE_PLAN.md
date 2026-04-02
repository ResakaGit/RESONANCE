---
document_id: RD-1.7
title: Software Maintenance Plan
standard: IEC 62304:2006+Amd1:2015 §6
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Software Maintenance Plan

## 1. Purpose

This document defines the software maintenance plan for RESONANCE, satisfying IEC 62304:2006+Amd1:2015 §6 (Software Maintenance Process). It describes the strategy, processes, and responsibilities for maintaining the software after initial release, including problem reporting, change evaluation, modification implementation, regression testing, and release management.

IEC 62304 §6 requires:
- §6.1: Establish a software maintenance plan
- §6.2: Monitor feedback from users and external sources
- §6.3: Evaluate and classify change requests and problem reports
- §6.3.2: Verify that modifications do not introduce new hazards

**Cross-references:**
- RD-1.4 `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` --- Lifecycle model, sprint methodology
- RD-1.8 `docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md` --- Problem resolution process
- RD-2.7 `docs/regulatory/02_risk_management/POST_PRODUCTION_MONITORING.md` --- Post-production feedback channels
- RD-3.4 `docs/regulatory/03_traceability/CONFIGURATION_MANAGEMENT.md` --- Version control, Cargo.lock pinning
- RD-5.7 `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` --- Corrective/preventive actions
- RD-7.5 `docs/regulatory/07_release/RELEASE_PACKAGE.md` --- Release criteria and procedures

## 2. Scope

This maintenance plan applies to:

- All source code under `src/` (~113K LOC, Rust 2024 edition)
- All pure math in `src/blueprint/equations/` (45+ domain files)
- All batch simulation code in `src/batch/` (33 systems)
- All test infrastructure (3,113 tests)
- Configuration files (`Cargo.toml`, `Cargo.lock`, `assets/maps/*.ron`)
- Regulatory documentation (`docs/regulatory/`)
- Project constitution (`CLAUDE.md`)

**Out of scope:** Third-party crate internals (SOUP) --- maintained by upstream projects. SOUP monitoring is handled by RD-2.7 and the `cargo audit` process defined in RD-3.2.

## 3. Maintenance Strategy

### 3.1 Iterative Sprint-Based Maintenance

RESONANCE uses the same sprint-based development model for maintenance as for new development (RD-1.4 §2.1). There is no distinction between "development" and "maintenance" phases --- all changes follow the same lifecycle:

```
Problem/Request --> Evaluation --> Sprint Scope --> Design --> Implement --> Test --> Review --> Archive
```

This is consistent with IEC 62304 §6.2.2, which permits using the same development processes for maintenance activities.

### 3.2 Maintenance Categories

| Category | Description | Process | Example |
|----------|-------------|---------|---------|
| Corrective | Fix defects that cause incorrect simulation output, axiom violations, or test failures | Problem Resolution (RD-1.8) + CAPA (RD-5.7) | Energy conservation violation detected by proptest |
| Adaptive | Modify software for new SOUP versions, Rust edition updates, or Bevy version upgrades | Standard sprint cycle with regression testing | Bevy 0.15 -> 0.16 migration |
| Perfective | Improve performance, add features, or enhance usability without changing existing behavior | Standard sprint cycle | New emergence system registration, new calibration profile |
| Preventive | Proactively address potential problems before they manifest | Preventive action per RD-5.7 | Increase test coverage for uncovered edge case |

### 3.3 Maintenance Triggers

| Trigger | Source | Response |
|---------|--------|----------|
| Test failure | `cargo test` | Immediate corrective action (axiom violation = NC-CRIT per RD-5.6) |
| SOUP vulnerability | `cargo audit` or RustSec advisory | Evaluate CVSS; update if CVSS >= 7.0 (RD-2.7) |
| User-reported problem | GitHub Issues | Triage per Problem Resolution (RD-1.8) |
| Feature request | GitHub Issues or Discussions | Evaluate fit with intended use; scope into sprint if accepted |
| Rust/Bevy version update | Upstream release | Evaluate compatibility; plan migration sprint if breaking changes |
| Regulatory feedback | External audit, partner evaluation | Evaluate impact on classification; update documentation |
| Publication feedback | Journal review, conference discussion | Evaluate scientific validity challenge; update limitations if needed |

## 4. Problem Reporting

### 4.1 Reporting Channels

| Channel | Medium | Target Audience |
|---------|--------|-----------------|
| GitHub Issues | `https://github.com/ResakaGit/RESONANCE/issues` | External users, contributors |
| Test suite | `cargo test` (automated, pre-merge) | Developer |
| Sprint review | Sprint closure grep verification | Observador, Verificador |
| `cargo audit` | Automated SOUP vulnerability scan | Developer |

### 4.2 Current State

**Gap acknowledged:** As of commit `971c7ac`, 0 GitHub Issues have been filed. The problem reporting process exists (GitHub Issues is enabled) but has not been exercised at scale. The primary problem detection mechanism is the automated test suite (3,113 tests) and sprint review process.

### 4.3 Problem Report Format

When a problem is reported via GitHub Issues, the following information should be captured:

| Field | Description |
|-------|-------------|
| Title | Brief description of the problem |
| Steps to reproduce | Commands, configuration, map preset, or seed that triggers the issue |
| Expected behavior | What should happen according to axioms/requirements |
| Actual behavior | What happens instead |
| RESONANCE version | Git commit SHA or `Cargo.toml` version |
| Platform | OS, Rust version, hardware |
| Severity estimate | Critical / High / Medium / Low (reporter's assessment; triaged by team) |

## 5. Change Evaluation

### 5.1 Evaluation Process

All proposed changes --- whether corrective, adaptive, perfective, or preventive --- are evaluated before implementation using the following criteria:

| Criterion | Evaluator | Method |
|-----------|-----------|--------|
| Axiom compliance | Observador | Review proposed change against 8 axioms; any violation = BLOCK |
| Safety impact | Planificador | Assess whether change affects safety classification (RD-1.2) |
| Risk impact | Planificador | Determine if change introduces new hazards or modifies existing risk controls (RD-2.4) |
| Conservation invariant | Verificador | Determine if change could violate energy conservation (Axiom 5) |
| Determinism impact | Verificador | Assess whether change could break bit-exact reproducibility (RS-03) |
| Regression scope | Verificador | Identify tests that must pass to confirm no regression |
| SOUP impact | Alquimista | Determine if change requires SOUP version update (RD-3.2) |

### 5.2 Change Classification

| Class | Description | Required Activities |
|-------|-------------|---------------------|
| Trivial | Documentation fix, comment correction, test addition with no code change | Review + merge. No sprint required. |
| Minor | Bug fix, constant adjustment, new test, performance optimization | Sprint scope + implement + `cargo test` pass |
| Significant | New feature, new equation, new layer, SOUP update | Full sprint cycle: scope + design + implement + test + review + archive |
| Critical | Axiom modification, fundamental constant change, safety classification change | Full sprint + risk re-evaluation (RD-2.1) + regulatory documentation update |

### 5.3 Impact on Regulatory Documentation

Any change classified as Significant or Critical must be evaluated for impact on the regulatory documentation set:

| Change Type | Potentially Affected Documents |
|-------------|-------------------------------|
| New requirement | RD-1.3 (SRS), RD-3.1 (Traceability), RD-4.6 (URS) |
| New risk or hazard | RD-2.2 through RD-2.6 (Risk file) |
| SOUP version change | RD-3.2 (SOUP Analysis), RD-3.3 (SBOM) |
| Safety classification change | RD-1.2 (Safety Class), RD-1.6 (MDF), all downstream |
| New calibration profile | RD-6.2 (Clinical Evaluation), RD-6.5 (Reference Data) |
| Intended use change | RD-1.1 (Intended Use) --- triggers full re-evaluation |

## 6. Modification Implementation

### 6.1 Standard Sprint Cycle

All modifications beyond Trivial follow the standard sprint cycle defined in RD-1.4 §2.1:

1. **Scope:** Define objectives, deliverables, closure criteria in sprint README
2. **Design:** Decompose into layers, validate orthogonality, identify equations
3. **Implement:** Write code respecting `CLAUDE.md` rules, 14 layers, Phase assignment
4. **Test:** Unit tests (pure math), integration tests (MinimalPlugins), property tests as needed
5. **Review:** Verificador checklist: contract, math, DOD, determinism, perf, tests
6. **Archive:** Verify closure criteria (grep-based), move to `docs/sprints/archive/`

### 6.2 Hard Blocks During Modification

All `CLAUDE.md` Hard Blocks apply during maintenance with the same force as during initial development:

- **Absolute:** NO `unsafe`, NO unapproved external crates, NO `async`, NO `Arc<Mutex>`, NO shared mutable state
- **Strong defaults:** NO `HashMap` in hot paths, NO `String` in components, NO `Box<dyn Trait>`, NO inline formulas, etc. (violable with `// DEBT:` justification)

### 6.3 Axiom Inviolability During Modification

Per `CLAUDE.md` §The 8 Foundational Axioms:

> No change, feature, refactor, or optimization may contradict, bypass, or weaken ANY of the 8 axioms or 4 fundamental constants. If a proposed change conflicts with an axiom, the change is WRONG --- not the axiom.

This applies with equal force during maintenance. Any maintenance change that would violate an axiom must be rejected, regardless of the urgency of the problem being addressed. The correct response is to find an axiom-compliant solution.

## 7. Regression Testing

### 7.1 Regression Test Strategy

Before any change is merged, the full test suite must pass:

```bash
cargo test
```

This executes 3,113 tests covering:

| Test Layer | Count (approx.) | Coverage |
|------------|-----------------|----------|
| Unit tests (blueprint/equations/) | ~1,800 | Pure math: boundary cases, conservation, convergence |
| Integration tests (simulation/) | ~400 | System behavior: minimal ECS app, single update, assert delta |
| Batch tests (batch/) | ~200 | Headless simulator: 33 systems, arena, genome, harness, bridge |
| Property tests (proptest) | ~19 | Fuzz: conservation invariants with arbitrary inputs |
| Experiment tests (use_cases/) | ~150 | End-to-end: cancer therapy, pathway inhibitor, Bozic validation |
| Other (layers, entities, plugins) | ~500 | Component construction, archetype spawning, plugin registration |

### 7.2 Targeted Regression for Specific Changes

| Change Domain | Additional Regression |
|---------------|----------------------|
| `blueprint/equations/` | Run property tests: `cargo test --test property_conservation` |
| `blueprint/constants/` | Verify derived thresholds: `cargo test derived_thresholds` |
| `batch/` | Run batch benchmark: `cargo bench --bench batch_benchmark` |
| Drug models | Run Bozic validation: `cargo run --release --bin bozic_validation` |
| Worldgen / abiogenesis | Run map validation: `RESONANCE_MAP=genesis_validation cargo run` |
| Determinism | Run determinism tests: `cargo test determinism` + experiment determinism checks |

### 7.3 Regression Test Gaps

| Gap | Severity | Mitigation |
|-----|----------|------------|
| No automated CI/CD pipeline | Medium | Tests run locally before merge. GitHub Actions planned but not implemented. |
| No benchmark regression tracking | Low | Criterion benchmarks available (`cargo bench --bench batch_benchmark`) but no historical tracking. |
| No visual regression testing | Low | Visual output is non-critical (research tool). Headless PPM output can be compared manually. |

## 8. Release Management

### 8.1 Release Process

Software releases follow the process defined in RD-7.5 (Release Package Definition):

1. All 12 release criteria must be met (RD-7.5 §3)
2. `cargo test` passes with 0 failures
3. `cargo audit` shows no unmitigated CVSS >= 7.0 vulnerabilities
4. All regulatory documentation affected by changes is updated
5. Git tag created: `v{MAJOR}.{MINOR}.{PATCH}`
6. `Cargo.toml` version updated to match tag

### 8.2 Release Types

| Type | Version Bump | When |
|------|-------------|------|
| Patch | 0.1.x | Bug fixes, doc updates, test additions |
| Minor | 0.x.0 | New features, new drug models, new calibration profiles |
| Major | x.0.0 | Breaking changes to axioms, constants, layers, or APIs |

### 8.3 Release Gaps

| Gap | Severity | Mitigation |
|-----|----------|------------|
| No Git tags used for releases yet | Medium | RD-7.5 formalizes tagging. First tagged release will establish the practice. |
| No formal release notes template | Low | Sprint archive docs serve as release notes. Formal template planned for first tagged release. |

## 9. Feedback Monitoring

Post-production feedback monitoring is defined in RD-2.7 (Post-Production Monitoring Plan). The maintenance plan relies on RD-2.7's monitoring channels to feed problems and change requests into the maintenance process.

**Current channels:**
- GitHub Issues (primary)
- GitHub Discussions (secondary)
- Paper citation tracking (scientific validity challenges)
- `cargo audit` / RustSec (SOUP vulnerabilities)

**Gap acknowledged:** No formal feedback mechanism exists beyond GitHub Issues. For a research tool with a small user base, this is adequate. If the user base grows significantly or RESONANCE is reclassified as SaMD, a more structured feedback collection process would be needed (e.g., structured intake form, SLA for response time, formal triage process).

## 10. Roles and Responsibilities

| Activity | Primary Role | Supporting Role |
|----------|-------------|-----------------|
| Problem triage and classification | Planificador | Observador |
| Change impact assessment | Planificador | Verificador |
| Modification implementation | Alquimista | --- |
| Regression testing | Verificador | Alquimista |
| Risk re-evaluation (if needed) | Planificador | Observador |
| Release decision | Planificador | Verificador |
| Documentation update | Alquimista | Observador |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial maintenance plan. Sprint-based maintenance strategy, change evaluation process, regression testing, release management. Gaps in CI/CD and formal feedback documented. |
