---
document_id: RD-1.8
title: Software Problem Resolution Process
standard: IEC 62304:2006+Amd1:2015 §9
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Software Problem Resolution Process

## 1. Purpose

This document defines the process for resolving software problems in RESONANCE, satisfying IEC 62304:2006+Amd1:2015 §9 (Software problem resolution process). The standard requires a process to analyze, investigate, and resolve problems discovered in released software, including trending of problem reports and assessment of advisory need.

IEC 62304 §9 requires:
- §9.1: Prepare problem reports
- §9.2: Investigate the problem
- §9.3: Advise relevant parties of the results of the investigation
- §9.4: Use change control to approve changes
- §9.5: Maintain records of problem resolution
- §9.6: Analyze problem trends
- §9.7: Verify problem resolution
- §9.8: Test the software product

**Cross-references:**
- RD-1.7 `docs/regulatory/01_foundation/SOFTWARE_MAINTENANCE_PLAN.md` --- Maintenance strategy and change evaluation
- RD-5.6 `docs/regulatory/05_quality_system/NONCONFORMING_PRODUCT.md` --- NC detection and disposition
- RD-5.7 `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` --- Corrective/preventive actions
- RD-2.4 `docs/regulatory/02_risk_management/RISK_CONTROLS.md` --- Risk controls that may be affected by problems
- RD-7.5 `docs/regulatory/07_release/RELEASE_PACKAGE.md` --- Release criteria including problem resolution

## 2. Scope

This process applies to all software problems in RESONANCE, including:

- Defects in simulation logic (incorrect energy calculations, conservation violations, non-deterministic output)
- Defects in drug models (incorrect Hill response, wrong inhibition mode behavior, Bozic validation failure)
- Defects in batch simulator (incorrect system behavior, genome corruption, bridge round-trip failure)
- Test failures (unit, integration, property, or experiment)
- SOUP vulnerabilities (security advisories for dependencies)
- Documentation errors (regulatory docs, README, CLAUDE.md)
- Usability issues (confusing CLI output, incorrect map behavior)

## 3. Problem Reporting

### 3.1 Detection Sources

| Source | Detection Method | Typical Severity |
|--------|-----------------|------------------|
| `cargo test` failure | Automated --- test suite execution before merge | High to Critical (test caught it) |
| `cargo audit` alert | Automated --- SOUP vulnerability scan | Variable (depends on CVSS) |
| Sprint review | Manual --- grep-based closure criteria verification | Medium (missed during implementation) |
| Property test (`proptest`) | Automated --- fuzz testing with arbitrary inputs | High (conservation or invariant violation) |
| GitHub Issue | External --- user report | Variable |
| Publication review | External --- journal/conference feedback | Variable (scientific validity) |
| Self-discovery | Internal --- developer finds issue during unrelated work | Variable |

### 3.2 Problem Report Contents

Each problem is recorded with the following information:

| Field | Description | Required |
|-------|-------------|----------|
| Problem ID | `PROB-{YYYY}-{SEQ}` (e.g., `PROB-2026-001`) | Yes |
| Date reported | Date of initial report | Yes |
| Source | Detection source per §3.1 | Yes |
| Reporter | Person or system that detected the problem | Yes |
| Description | What is wrong, what was expected, what happened | Yes |
| Reproduction steps | Commands, configuration, seed to reproduce | Yes (if reproducible) |
| RESONANCE version | Git commit SHA | Yes |
| Severity | Per §3.3 classification | Yes |
| Affected requirements | RF-xx, RS-xx, RP-xx, RI-xx IDs if identifiable | If known |
| Affected axioms | Axiom numbers if the problem involves axiom compliance | If applicable |

### 3.3 Severity Classification

| Severity | Definition | Response Time | Example |
|----------|------------|---------------|---------|
| **Critical (NC-CRIT)** | Axiom violation, energy conservation breach, or safety requirement failure. Simulation produces fundamentally wrong results. | Immediate --- stop all other work | Energy created from nothing (violates Axiom 5). `unsafe` block found in simulation code (violates RS-01). |
| **High (NC-HIGH)** | Hard block breach, test failure in `blueprint/equations/`, determinism broken, drug model produces incorrect qualitative predictions. | Next sprint | Unit test in `pathway_inhibitor.rs` fails. Bozic validation produces incorrect ranking. Non-deterministic output across runs. |
| **Medium (NC-MED)** | Coding rule violation, missing test coverage, documentation inconsistency, non-critical SOUP vulnerability (CVSS 4.0--6.9). | Planned sprint | `HashMap` used in hot path without benchmark. Missing boundary test for edge case. Regulatory document references wrong section. |
| **Low (NC-LOW)** | Cosmetic issue, comment typo, non-functional code style deviation, SOUP vulnerability CVSS < 4.0. | Backlog | Misaligned comment. Unused import. Non-functional naming inconsistency. |

### 3.4 Axiom-Specific Severity Escalation

Any problem that involves a potential axiom violation is automatically escalated to **Critical (NC-CRIT)**, regardless of its apparent impact. The 8 axioms are the constitution of the project (`CLAUDE.md` §The 8 Foundational Axioms):

> No change, feature, refactor, or optimization may contradict, bypass, or weaken ANY of the 8 axioms or 4 fundamental constants. If a proposed change conflicts with an axiom, the change is WRONG --- not the axiom.

A problem that reveals an existing axiom violation means the codebase has a constitutional defect that must be resolved before any other work.

## 4. Problem Investigation

### 4.1 Investigation Process

```
Problem Report --> Reproduce --> Root Cause Analysis --> Impact Assessment --> Resolution Plan
```

#### Step 1: Reproduce

| Action | Method |
|--------|--------|
| Identify minimal reproduction | Isolate the smallest test case or configuration that triggers the problem |
| Verify with `cargo test` | Run full suite to check if problem is caught by existing tests |
| Check determinism | Run twice with same seed/config to confirm reproducibility |
| Identify affected commit | Use `git bisect` if problem is a regression (determine when it was introduced) |

#### Step 2: Root Cause Analysis

Root cause analysis follows the sprint "Problema" methodology --- identify what went wrong and why it was not caught earlier.

| Analysis Method | When Used |
|----------------|-----------|
| Code inspection | Default for all problems --- read the code, trace the logic |
| Test gap analysis | Problem should have been caught by a test that does not exist |
| Axiom trace | Problem involves simulation logic --- trace back to which axiom is affected |
| SOUP analysis | Problem may originate in a dependency --- check upstream issue trackers |

#### Step 3: Impact Assessment

| Question | Method |
|----------|--------|
| Does this affect safety classification? | Review RD-1.2 criteria; if problem creates a patient harm pathway, reclassification may be needed |
| Does this affect risk controls? | Review RD-2.4; if a risk control is compromised, residual risk must be re-evaluated |
| Does this affect validation results? | Review RD-4.4; if Bozic validation or calibration profiles are affected, validation must be re-run |
| Does this affect other requirements? | Review RD-3.1 traceability matrix for downstream impacts |
| Does this affect published claims? | Review README, paper, and Zenodo record; if published claims are invalidated, advisories needed |

### 4.2 Investigation Time Limits

| Severity | Max Investigation Time | Escalation If Exceeded |
|----------|----------------------|------------------------|
| Critical | 24 hours | Halt all development until root cause identified |
| High | 1 sprint | Escalate to Planificador for scope expansion |
| Medium | 2 sprints | Re-prioritize in backlog if no progress |
| Low | No limit | Remains in backlog until addressed |

## 5. Advisories

### 5.1 When an Advisory Is Required

An advisory is required when a problem:

1. **Invalidates a published claim** --- e.g., Bozic validation no longer holds, conservation is violated, determinism is broken
2. **Affects safety classification** --- e.g., a previously unknown patient harm pathway is identified
3. **Involves a SOUP vulnerability with CVSS >= 7.0** that could affect simulation correctness
4. **Was reported by an external user** --- the reporter must be notified of the resolution

### 5.2 Advisory Mechanisms

| Audience | Mechanism |
|----------|-----------|
| GitHub users | GitHub Issue comment with resolution details; GitHub Release notes if fix is in a tagged release |
| Paper readers | If the problem invalidates a paper claim: Zenodo version update with errata; contact Zenodo for DOI versioning |
| README readers | Update `README.md` disclaimers or validation table (line 141) if validation status changes |
| CLAUDE.md users | Update `CLAUDE.md` if coding rules, axioms, or architecture sections are affected |

### 5.3 Current Advisory Infrastructure

**Gap acknowledged:** No formal advisory distribution list or notification mechanism exists. Advisories would be communicated through GitHub Issues/Releases and README updates. For a research tool with a small user base, this is adequate. If RESONANCE gains a larger user base or is reclassified as SaMD, a formal advisory process (e.g., FSCA --- Field Safety Corrective Action) would be required.

## 6. Resolution

### 6.1 Resolution Process

Problem resolution follows the CAPA procedure (RD-5.7):

1. **Detection:** Problem report filed per §3
2. **Investigation:** Root cause identified per §4
3. **Action plan:** Corrective action defined in sprint scope
4. **Implementation:** Fix implemented following standard sprint cycle (RD-1.4 §2.1)
5. **Verification:** Fix verified per §7
6. **Closure:** Problem report closed with resolution record

### 6.2 Resolution Options

| Option | When Appropriate | Example |
|--------|-----------------|---------|
| Code fix | Root cause is a software defect | Fix incorrect formula in `blueprint/equations/` |
| Test addition | Problem was not caught because test coverage was insufficient | Add boundary test for discovered edge case |
| Documentation update | Problem is a documentation error or the code is correct but documentation is misleading | Update regulatory document with correct cross-reference |
| SOUP update | Problem originates in a dependency | Update `Cargo.toml` version constraint, run regression |
| Design change | Root cause is a design flaw, not a coding error | Redesign system interaction to eliminate root cause |
| Accept (with justification) | Problem is cosmetic or the cure is worse than the disease | Low-severity style issue that would require invasive refactoring; document with `// DEBT:` |

### 6.3 Resolution Constraints

- Resolution must not violate any axiom (§3.4 applies to fixes as well as to the original code)
- Resolution must not introduce new `unsafe` blocks (Hard Block HB-1)
- Resolution must not add unapproved external crates (Hard Block HB-2)
- Resolution must maintain bit-exact determinism (RS-03)
- Resolution must pass the full regression test suite (§7)

## 7. Verification of Resolution

### 7.1 Verification Methods

| Verification | Method | Required For |
|--------------|--------|--------------|
| Regression test | `cargo test` --- full suite, 3,113 tests, 0 failures | All resolutions |
| Specific test | New test written to reproduce the original problem and confirm the fix prevents recurrence | All code fixes |
| Grep verification | `grep -rn "{pattern}" src/ --include="*.rs"` to confirm absence of problematic pattern | Hard block violations, axiom violations |
| Property test | `cargo test --test property_conservation` for conservation-related fixes | Conservation, pool invariant fixes |
| Bozic validation | `cargo run --release --bin bozic_validation` for drug model fixes | Drug model fixes |
| Determinism check | Two runs with identical config produce identical output | Any fix touching simulation logic |
| Document review | Affected regulatory documents reviewed for consistency | Documentation fixes, design changes |

### 7.2 Verification Record

Each resolved problem must have a verification record containing:

| Field | Content |
|-------|---------|
| Problem ID | `PROB-{YYYY}-{SEQ}` |
| Fix commit | Git commit SHA of the fix |
| Tests added | List of new tests that prevent recurrence |
| `cargo test` result | Pass/fail count, execution time |
| Regression confirmed | Yes/No --- did any existing test break? |
| Resolution verified by | Role: Verificador |
| Date verified | Date of verification |

## 8. Problem Trend Analysis

### 8.1 Trend Categories

Problems are categorized for trend analysis:

| Category | Description | Indicator |
|----------|-------------|-----------|
| Conservation | Energy creation/destruction violations | Axiom 2, 4, 5 |
| Determinism | Non-reproducible output | RS-03 |
| Math correctness | Incorrect formula in `blueprint/equations/` | RF-01 through RF-17 |
| SOUP | Dependency vulnerabilities or incompatibilities | RD-3.2 |
| Documentation | Inconsistencies, missing references, stale content | All RD-xx |
| Process | Sprint closure criteria not met, reviews missed | RD-1.4 |

### 8.2 Trend Review

Problem trends are reviewed:
- At each sprint closure (part of retrospective)
- Quarterly (per RD-2.7 post-production monitoring schedule)
- At each release (per RD-7.5 release criteria)

If a trend is detected (e.g., recurring problems in the same module or category), a **preventive action** is initiated per RD-5.7 §3.2.

### 8.3 Current Trend Data

**Gap acknowledged:** As of commit `971c7ac`, 0 formal problem reports have been filed. All defects to date have been caught and resolved within sprint cycles (before release), documented in sprint archive docs rather than formal problem reports. The trend analysis process exists but has no data to analyze.

This is consistent with a pre-1.0 research tool where the developer and the user are the same person. The gap would become significant if:
- External users begin filing issues
- RESONANCE is reclassified above Class A
- A formal release cycle (tagged versions) is established

## 9. Records

### 9.1 Record Types

| Record | Storage | Retention |
|--------|---------|-----------|
| Problem reports | GitHub Issues | Indefinite (GitHub retention) |
| Investigation notes | Sprint docs in `docs/sprints/` | Indefinite (Git history) |
| Fix commits | Git log | Indefinite (Git immutable history) |
| Test results | `cargo test` output | Not formally archived (gap --- see §9.2) |
| Verification records | Sprint closure docs | Indefinite (Git history) |
| Trend analysis | Quarterly review notes (planned) | To be defined |

### 9.2 Record Gaps

| Gap | Severity | Mitigation |
|-----|----------|------------|
| `cargo test` output not formally archived per run | Medium | Git commit history implicitly records that tests passed. Formal CI/CD with archived logs would close this gap. |
| No structured problem report database | Low | GitHub Issues provides structured issue tracking. Adequate for current scale. |
| No formal quarterly trend review records | Low | Process defined but not yet executed (0 problems to date). First review to be conducted Q3 2026 or upon first formal release, whichever comes first. |

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial problem resolution process. Severity classification, investigation, advisory, resolution, verification, and trend analysis procedures defined. 0 formal problem reports to date --- process exists but untested at scale. |
