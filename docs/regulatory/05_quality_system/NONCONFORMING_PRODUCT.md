---
document_id: RD-5.6
title: Nonconforming Product Procedure
standard: ISO 13485:2016 §8.3
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Nonconforming Product Procedure

## 1. Purpose

This procedure defines the process for identifying, documenting, evaluating, segregating, and disposing of nonconforming product within the RESONANCE Quality Management System, in accordance with ISO 13485:2016 §8.3.

In the context of RESONANCE (a software product), "nonconforming product" means source code, documentation, or configuration that fails to meet specified requirements --- including test failures, axiom violations, hard block breaches, coding rule violations, and safety rule breaches.

## 2. Scope

This procedure applies to all outputs of the development process:
- Source code in `src/` (113K LOC)
- Configuration files (`Cargo.toml`, `Cargo.lock`, `assets/maps/*.ron`)
- Sprint documentation (`docs/sprints/`)
- Regulatory documentation (`docs/regulatory/`)
- Test infrastructure
- Binary outputs (simulation results, evolved genomes)

## 3. Definition of Nonconformance

### 3.1 Nonconformance Categories

| Category | Severity | Definition | Detection Method | Examples |
|----------|----------|------------|------------------|----------|
| **NC-CRIT: Axiom Violation** | Critical | Code behavior contradicts any of the 8 foundational axioms or 4 fundamental constants | Verificador review (verdict: BLOCK); test failure in `derived_thresholds.rs`; conservation property test failure | Energy created from nothing (Axiom 5); hardcoded behavior bypassing emergence (Axiom 6); non-decreasing interaction over distance (Axiom 7) |
| **NC-HIGH: Hard Block Breach** | High | Violation of absolute hard blocks (HB-1 through HB-5) | grep audit (§4.3 of RD-5.5); `cargo check` errors | `unsafe` code (HB-1); unapproved crate added (HB-2); `async fn` introduced (HB-3); `Arc<Mutex>` used (HB-4); `static mut` declared (HB-5) |
| **NC-HIGH: Test Failure** | High | Any test in the 3,113-test suite fails | `cargo test` execution | Unit test failure in `blueprint/equations/`; integration test failure; property test (proptest) failure; batch test failure |
| **NC-MED: Safety Rule Breach** | Medium | Violation of strong default hard blocks (SD-1 through SD-12) without inline `// DEBT:` justification | Code review (Observador/Verificador); grep audit | `unwrap()` in system without DEBT comment (SD-6); inline formula in system (SD-7); `HashMap` in hot path without benchmark justification (SD-1) |
| **NC-MED: Coding Rule Violation** | Medium | Violation of the 14 coding rules defined in `CLAUDE.md` §Coding Rules | Code review | Component with >4 fields (Rule 2); system accessing >5 component types (Rule 3); gameplay system in `Update` schedule (Rule 7) |
| **NC-LOW: Documentation Gap** | Low | Controlled document missing, outdated, or inconsistent with code | Document review; annual audit (RD-5.5 §5.2) | Sprint doc missing closure criteria; ARCHITECTURE.md not reflecting current module structure; regulatory doc cross-reference broken |
| **NC-LOW: Warning** | Low | Compiler or clippy warning | `cargo check`; `cargo clippy` | Unused import; dead code; unnecessary clone |

### 3.2 Nonconformance vs. Improvement Opportunity

Not every deviation is a nonconformance. The distinction:

| Characteristic | Nonconformance | Improvement Opportunity |
|----------------|---------------|------------------------|
| Requirement violated | Yes (axiom, hard block, test, coding rule) | No (current code is conforming) |
| Action required | Yes (must fix before merge) | Optional (may improve) |
| Record required | Yes (sprint doc or CAPA) | Optional (OBSERVATION in audit) |
| Example | Test failure, `unsafe` code | Better variable naming, refactoring for clarity |

## 4. Detection Methods

### 4.1 Automated Detection

| Method | Tool | What It Detects | Frequency |
|--------|------|-----------------|-----------|
| **Test execution** | `cargo test` | Test failures (NC-HIGH) | Every sprint closure; recommended before every commit |
| **Compilation check** | `cargo check` | Compilation errors and warnings (NC-LOW) | Every sprint closure |
| **Lint analysis** | `cargo clippy` | Lint warnings, potential bugs (NC-LOW to NC-MED) | Every sprint closure |
| **Grep audit** | `grep -r` (per RD-5.5 §4.3) | Hard block violations (NC-HIGH), coding rule violations (NC-MED) | Every sprint track closure |
| **Property testing** | `cargo test` (proptest) | Conservation violations, mathematical property failures (NC-CRIT) | Every sprint closure (proptest runs as part of `cargo test`) |

### 4.2 Manual Detection

| Method | Performed By | What It Detects | Frequency |
|--------|-------------|-----------------|-----------|
| **Verificador review** | Verificador role | Axiom violations (NC-CRIT), math errors, DOD violations, determinism issues | Every sprint review phase |
| **Observador review** | Observador role | Coding rule violations (NC-MED), component bloat, system scope creep | During code development |
| **Planificador review** | Planificador role | Design-level nonconformances (wrong layer decomposition, missing orthogonality) | Sprint scoping phase |
| **Inference Protocol** | Any contributor | Premature abstraction, scope creep, orphan components (Red Lines in CLAUDE.md §Inference Protocol) | Continuous |

### 4.3 External Detection

| Method | Source | What It Detects |
|--------|--------|-----------------|
| **GitHub issue** | External user or contributor | Bug report, unexpected behavior |
| **Journal review** | Peer reviewer | Scientific error, reproducibility failure |
| **Pharma partner feedback** | Evaluation partner | Validation gap, documentation inadequacy |

## 5. Disposition

### 5.1 Disposition Decision

The trunk-based development model on `main` determines the disposition workflow: **nonconforming code must be fixed before it can be merged to `main`**. There is no separate "quarantine" or "concession" pathway for software.

| Disposition | When Used | Action |
|-------------|-----------|--------|
| **Fix** | All NC-CRIT, NC-HIGH, and most NC-MED | Correct the code, add regression test, verify fix |
| **Justify** | NC-MED (strong default violations only) | Add `// DEBT: <reason>` inline justification; violation remains but is documented |
| **Accept** | NC-LOW (documentation gaps, warnings being addressed) | Document the gap; schedule correction in next sprint |
| **Revert** | NC-CRIT (axiom violation) if fix is complex | Revert the offending change entirely; re-approach in a new sprint |

### 5.2 Disposition by Severity

| Severity | Disposition | Timeline | Authority |
|----------|-------------|----------|-----------|
| NC-CRIT (axiom violation) | Fix or revert immediately; no merge permitted | Before next commit to `main` | Verificador (BLOCK verdict) |
| NC-HIGH (hard block / test failure) | Fix before sprint closure | Within current sprint | Verificador |
| NC-MED (safety rule / coding rule) | Fix or justify before sprint closure | Within current sprint | Observador or Verificador |
| NC-LOW (doc gap / warning) | Accept and schedule fix | Within next sprint or next annual audit | Planificador |

### 5.3 Segregation

In software, "segregation" means preventing nonconforming code from reaching the `main` branch:

| Mechanism | How It Prevents Nonconforming Product |
|-----------|---------------------------------------|
| **Trunk-based development** | All changes go to `main` only after review. No unreviewed code in production. |
| **Sprint closure gate** | `cargo test` must pass with 0 failures before sprint closure. Failing sprint = no archive. |
| **Verificador BLOCK** | BLOCK verdict halts the sprint. No override without written justification. |
| **Grep audit** | Hard block grep commands (RD-5.5 §4.3) detect violations before they are committed. |

**Known gap:** There is no branch protection rule enforced by GitHub (no required PR reviews, no required CI checks). Protection is procedural (Verificador review) rather than automated. This is acceptable for the current project structure but should be automated if the contributor count increases.

## 6. Nonconformance Records

### 6.1 Record Content

Each nonconformance record must contain:

| Field | Content |
|-------|---------|
| **NC ID** | Unique identifier (sprint ID + sequence, e.g., "DC-4/NC-1") |
| **Date detected** | ISO 8601 date |
| **Severity** | NC-CRIT / NC-HIGH / NC-MED / NC-LOW |
| **Description** | What was found, where, and how it violates requirements |
| **Detection method** | How it was found (cargo test, grep, review, external report) |
| **Root cause** | Why the nonconformance occurred (documented in sprint "Problema" section) |
| **Disposition** | Fix / Justify / Accept / Revert |
| **Corrective action** | What was done to fix the specific instance |
| **Preventive action** | What was done to prevent recurrence (new test, new grep criterion, new closure criterion) |
| **Verification** | Evidence that the fix works (test pass, grep result) |
| **Status** | OPEN / CLOSED |

### 6.2 Record Location

Nonconformance records are stored within the sprint documentation that discovered or addressed them:

| Context | Record Location |
|---------|-----------------|
| Found during sprint implementation | Sprint README.md "Problema" section or inline in sprint doc |
| Found during sprint closure audit | Sprint README.md closure criteria (failed item) |
| Found during annual audit | Dedicated audit record in `docs/regulatory/05_quality_system/` |
| Found externally | New sprint created to address the finding |

### 6.3 Record Retention

Per RD-5.4 (Record Control), nonconformance records are retained indefinitely in Git history.

## 7. Real Examples

### 7.1 Example 1: DC-4 Hardcoded dt (NC-MED, Fixed)

| Field | Detail |
|-------|--------|
| **NC ID** | DC-AUDIT/NC-1 |
| **Date** | 2026-04-01 |
| **Severity** | NC-MED (coding rule violation: inline magic number) |
| **Description** | `epigenetic_adaptation.rs` contained hardcoded `0.016` as the timestep delta. This violated Coding Rule 10 (constants in constants modules) and created a coupling to a specific timestep that would silently break if `Time<Fixed>` configuration changed. |
| **Detection** | Discovered during DECOUPLING_AUDIT codebase review |
| **Root cause** | Original implementation used a fixed value during prototyping; not updated when `Time<Fixed>` was introduced |
| **Disposition** | Fix |
| **Corrective action** | Replaced `0.016` with `time.delta_secs()` in `epigenetic_adaptation.rs` |
| **Preventive action** | Added grep criterion to DECOUPLING_AUDIT closure: verify no hardcoded dt values remain. Sprint closure checklist now includes magic number scan. |
| **Verification** | `cargo test` pass; grep for `0.016` in `src/simulation/` returns 0 matches |
| **Status** | CLOSED |
| **Record location** | `docs/sprints/archive/DECOUPLING_AUDIT/README.md` line 23 |

### 7.2 Example 2: COMMENSALISM_INTAKE Hardcoded (NC-MED, Fixed)

| Field | Detail |
|-------|--------|
| **NC ID** | DC-AUDIT/NC-2 |
| **Date** | 2026-04-01 |
| **Severity** | NC-MED (axiom-adjacent: constant not derived from fundamentals) |
| **Description** | `COMMENSALISM_INTAKE` in `symbiosis_effect.rs` was a hardcoded constant not derived from any of the 4 fundamental constants. The AXIOMATIC_INFERENCE sprint established that all lifecycle constants must derive from fundamentals. |
| **Detection** | Discovered during DECOUPLING_AUDIT constants audit |
| **Root cause** | Constant predated the AXIOMATIC_INFERENCE sprint; was not migrated when the derivation chain was established |
| **Disposition** | Fix |
| **Corrective action** | Derived `COMMENSALISM_INTAKE` from `DISSIPATION_SOLID`; derived `MUTUALISM` from `2 * DISSIPATION_SOLID` |
| **Preventive action** | All new constants must reference a derivation chain back to one of the 4 fundamentals, documented inline |
| **Verification** | 17 tests in `derived_thresholds.rs` pass; grep confirms no standalone numeric constants in symbiosis module |
| **Status** | CLOSED |
| **Record location** | `docs/sprints/archive/DECOUPLING_AUDIT/README.md` line 25 |

### 7.3 Example 3: Dead Code (NC-LOW, Fixed)

| Field | Detail |
|-------|--------|
| **NC ID** | DC-AUDIT/NC-3 |
| **Date** | 2026-04-01 |
| **Severity** | NC-LOW (dead code --- no functional impact but maintenance burden) |
| **Description** | 7 dead items found: function `dimension_base_frequency` in `pathway_inhibitor_exp.rs`, 4 orphaned spawn functions in `heroes.rs` and `world_entities.rs`, dead constants `EMERGENT_INITIAL_RADIUS` etc. in `abiogenesis/constants.rs` |
| **Detection** | Codebase audit + compiler warnings |
| **Root cause** | Code from earlier sprints that was superseded but not cleaned up |
| **Disposition** | Fix (remove dead code) |
| **Corrective action** | Removed all 7 dead items |
| **Preventive action** | Zero-warning closure criterion (QO-5) catches dead code via `cargo clippy` warnings |
| **Verification** | `cargo check` = 0 warnings; `cargo test` = 3,113 passed |
| **Status** | CLOSED |
| **Record location** | `docs/sprints/archive/DECOUPLING_AUDIT/README.md` lines 24, 30--33 |

## 8. Nonconformance Trending

### 8.1 Current Metrics

| Period | NC-CRIT | NC-HIGH | NC-MED | NC-LOW | Total |
|--------|---------|---------|--------|--------|-------|
| DECOUPLING_AUDIT (2026-04-01) | 0 | 0 | 2 | 1 (7 items) | 3 |

### 8.2 Trending Analysis

The DECOUPLING_AUDIT represents the only formal nonconformance audit to date. Findings were:
- **NC-CRIT:** 0 --- no axiom violations detected
- **NC-HIGH:** 0 --- no hard block breaches (confirmed by grep)
- **NC-MED:** 2 (hardcoded dt, underived constant) --- both fixed
- **NC-LOW:** 1 instance with 7 dead items --- all removed

The NC-MED findings share a common root cause: code from early development that was not retroactively updated when later sprints (AXIOMATIC_INFERENCE) established stricter derivation requirements. This is a one-time legacy cleanup pattern, not a systemic process failure.

### 8.3 Known Limitation

Nonconformance trending is currently manual. There is no automated NC tracking system. For the current project scale (single contributor, research tool), manual tracking in sprint docs is sufficient. If the project scales, a formal NC log (e.g., a dedicated `docs/regulatory/05_quality_system/NC_LOG.md`) should be established.

## 9. Cross-References

| Document | Path | Relationship |
|----------|------|-------------|
| RD-5.5 Internal Audit | `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` | Audits detect nonconformances |
| RD-5.7 CAPA | `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` | Systemic nonconformances trigger CAPA |
| RD-5.2 Quality Policy | `docs/regulatory/05_quality_system/QUALITY_POLICY.md` | Objectives define conformance targets |
| CLAUDE.md | Repository root | Defines axioms, hard blocks, coding rules (conformance requirements) |
| DECOUPLING_AUDIT | `docs/sprints/archive/DECOUPLING_AUDIT/README.md` | Real nonconformance records |

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. Nonconformance categories (7 levels), detection methods (automated + manual), disposition (fix/justify/accept/revert), 3 real examples from DECOUPLING_AUDIT, trending baseline. |
