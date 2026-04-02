---
document_id: RD-5.7
title: CAPA Procedure
standard: ISO 13485:2016 §8.5.2-8.5.3
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# CAPA Procedure

## 1. Purpose

This procedure defines the process for implementing Corrective Actions and Preventive Actions (CAPA) within the RESONANCE Quality Management System, in accordance with ISO 13485:2016 §8.5.2 (Corrective Action) and §8.5.3 (Preventive Action).

- **Corrective Action:** Action to eliminate the cause of a detected nonconformance or other undesirable situation, to prevent recurrence.
- **Preventive Action:** Action to eliminate the cause of a potential nonconformance or other undesirable potential situation, to prevent occurrence.

## 2. Scope

This procedure applies to all nonconformances detected in the RESONANCE development process (per RD-5.6) and to potential nonconformances identified through:
- Internal audits (RD-5.5)
- Test results analysis
- Trend analysis of defect patterns
- Sprint retrospectives
- External feedback (journal reviews, partner evaluations)
- Inference Protocol red lines (premature abstraction, scope creep, orphan components)

## 3. CAPA Triggers

### 3.1 Corrective Action Triggers

A corrective action is required when:

| Trigger | Severity | Example |
|---------|----------|---------|
| Axiom violation detected | NC-CRIT | Conservation test fails --- energy being created |
| Hard block breach | NC-HIGH | `unsafe` code found in `src/` |
| Test failure | NC-HIGH | Unit test in `blueprint/equations/` fails after code change |
| Recurring defect pattern | NC-MED (repeated) | Same type of coding rule violation found in multiple sprints |
| External defect report | Variable | User reports simulation produces non-deterministic output |
| Audit finding (FAIL) | Per audit | Sprint closure grep reveals hardcoded constant |

### 3.2 Preventive Action Triggers

A preventive action is initiated when:

| Trigger | Source | Example |
|---------|--------|---------|
| Near-miss detected | Code review | Math function passes tests but lacks boundary case coverage --- could fail with extreme inputs |
| Trend analysis | Sprint archive review | Multiple sprints introduce dead code, suggesting cleanup is not part of standard practice |
| New risk identified | Planificador analysis | New drug model could be misinterpreted as clinical if disclaimers are insufficient |
| Standards change | External | New IMDRF guidance changes classification criteria |
| Architectural evolution | Sprint design | Adding a 15th layer could violate orthogonality if not carefully designed |
| Property test gap | Test coverage analysis | New equation lacks proptest coverage for conservation invariant |

## 4. CAPA Process

### 4.1 Process Flow

```
Detection ──> Investigation ──> Root Cause ──> Action Plan ──> Implementation ──> Verification ──> Closure
    │              │                │               │                │                │              │
    │              │                │               │                │                │              │
    v              v                v               v                v                v              v
  NC Record    Sprint doc       "Problema"     Sprint scope      Code + test      cargo test    Sprint
  (RD-5.6)    analysis         section         + criteria        + grep           + grep        archive
```

### 4.2 Step 1: Detection and Recording

| Action | Responsibility | Record |
|--------|---------------|--------|
| Detect nonconformance (or potential NC) | Any role; automated tools | NC record per RD-5.6 |
| Assign CAPA ID | Planificador | `CAPA-{YYYY}-{SEQ}` format |
| Classify: Corrective or Preventive | Planificador | CAPA record |
| Assess severity and urgency | Planificador | NC severity per RD-5.6 §3.1 |

**CAPA ID format:** `CAPA-2026-001`, `CAPA-2026-002`, etc. Sequential within calendar year.

### 4.3 Step 2: Investigation

| Action | Responsibility | Record |
|--------|---------------|--------|
| Gather evidence: which files, which tests, which sprint | Alquimista or Verificador | Sprint doc "Problema" section |
| Determine scope of impact: what else might be affected | Alquimista | Sprint doc analysis |
| Identify related nonconformances (same root cause?) | Planificador | Cross-reference to NC records |

### 4.4 Step 3: Root Cause Analysis

Root cause analysis is documented in the sprint document's "Problema" section. The standard RESONANCE root cause format:

```
## Problema

{Description of what was found}

{Why it happened --- trace back to the process failure, not just the code error}

{Why existing controls (tests, grep, review) did not catch it}
```

**Root cause categories:**

| Category | Description | Example |
|----------|-------------|---------|
| **Legacy code** | Code predates a rule or practice that was established later | DC-AUDIT/NC-2: `COMMENSALISM_INTAKE` predated AXIOMATIC_INFERENCE sprint |
| **Incomplete migration** | Sprint established a new pattern but did not migrate all existing code | DC-AUDIT/NC-1: hardcoded `0.016` dt not updated when `Time<Fixed>` was introduced |
| **Missing test coverage** | Edge case or invariant not tested | Potential: equation works for positive inputs but not zero |
| **Unclear requirement** | Axiom or coding rule ambiguous in a specific context | Rule 8 (math in equations/) unclear about whether guard logic counts as "math" |
| **Human error** | Mistake during implementation despite clear requirements | Developer used `unwrap()` out of habit despite SD-6 |
| **Tool limitation** | Development tool (compiler, clippy) does not detect the violation | Axiom violations are not mechanically detectable by the Rust compiler |

### 4.5 Step 4: Action Plan

The action plan is expressed as a sprint (for corrective actions) or a sprint closure criterion (for preventive actions).

**Corrective Action plan:**

| Element | Content |
|---------|---------|
| **Sprint or sub-task** | Sprint ID that will implement the fix |
| **Scope** | What files will be changed |
| **Expected deliverables** | Code changes, new tests, documentation updates |
| **Closure criteria** | Verifiable criteria (grep commands, test assertions) that prove the fix works |
| **Timeline** | Within current sprint (NC-CRIT/NC-HIGH) or next sprint (NC-MED/NC-LOW) |

**Preventive Action plan:**

| Element | Content |
|---------|---------|
| **New test** | Regression test that would have caught the issue |
| **New grep criterion** | Audit criterion added to sprint closure checklist (RD-5.5 §5.1) |
| **New coding rule or hard block** | If the root cause reveals a gap in `CLAUDE.md` rules |
| **Documentation update** | If the root cause is unclear requirements |

### 4.6 Step 5: Implementation

Corrective and preventive actions are implemented through the standard sprint lifecycle:

1. **Code fix** (corrective): Change source code to eliminate the nonconformance
2. **Regression test** (corrective + preventive): Add a test that would fail if the nonconformance recurred
3. **Grep criterion** (preventive): Add a grep command to the sprint closure checklist
4. **Documentation update** (if needed): Update `CLAUDE.md`, architecture docs, or sprint templates

All implementation follows the coding rules and hard blocks defined in `CLAUDE.md`.

### 4.7 Step 6: Verification of Effectiveness

Effectiveness verification uses the same mechanisms as sprint closure:

| Verification Method | What It Proves | Record |
|--------------------|----|--------|
| **`cargo test` passes** | Regression test catches the defect; fix does not break anything else | Test count in sprint closure |
| **Grep verification passes** | New grep criterion detects the violation pattern; current code is clean | Grep output in sprint closure criteria |
| **Verificador PASS** | Human review confirms the fix is correct and the preventive action is adequate | Verificador verdict in sprint doc |
| **Subsequent sprint closure** | Preventive action (new closure criterion) is applied in the next sprint and does not find new violations | Next sprint's closure criteria |

**Effectiveness assessment timing:**
- **Immediate:** Test pass + grep verification at CAPA sprint closure
- **Sustained:** New grep criterion is verified at every subsequent sprint closure. If the same type of nonconformance recurs despite the preventive action, the CAPA is reopened.

### 4.8 Step 7: Closure

A CAPA is closed when:

1. Corrective action is implemented and verified (test passes, grep clean)
2. Preventive action is in place (new test, new closure criterion, or rule update)
3. Effectiveness is verified at least once (current sprint closure)
4. CAPA record is complete (all fields in §5 filled)
5. Sprint containing the CAPA is archived

Closure is recorded by:
- Checking the CAPA closure criterion in the sprint README.md
- Updating CAPA status to CLOSED
- Archiving the sprint to `docs/sprints/archive/`

## 5. CAPA Record Template

```
## CAPA-{YYYY}-{SEQ}: {Title}

**Type:** Corrective / Preventive
**Severity:** NC-CRIT / NC-HIGH / NC-MED / NC-LOW
**Date opened:** YYYY-MM-DD
**Date closed:** YYYY-MM-DD
**Sprint:** {Sprint ID where CAPA is implemented}

### Detection
{How and where the nonconformance (or potential NC) was found}

### Investigation
{Evidence gathered, files examined, impact assessment}

### Root Cause
{Why it happened, per §4.4 categories}

### Corrective Action
{Code changes, file paths, specific fixes}

### Preventive Action
{New tests, grep criteria, rule updates, documentation changes}

### Verification of Effectiveness
{Test results, grep output, Verificador verdict}

### Status: OPEN / CLOSED
```

## 6. Real Examples

### 6.1 Example 1: Hardcoded dt --- Corrective + Preventive

```
## CAPA-2026-001: Hardcoded timestep in epigenetic_adaptation

**Type:** Corrective + Preventive
**Severity:** NC-MED
**Date opened:** 2026-04-01
**Date closed:** 2026-04-01
**Sprint:** DC-AUDIT (DECOUPLING_AUDIT track)

### Detection
During DECOUPLING_AUDIT codebase review, `epigenetic_adaptation.rs` was found to
contain hardcoded `0.016` as the timestep delta instead of `time.delta_secs()`.

### Investigation
- File: `src/simulation/emergence/epigenetic_adaptation.rs`
- The value `0.016` corresponds to 1/60 (approximate 60 FPS fixed timestep)
- If `Time<Fixed>` configuration changed, epigenetic adaptation rate would
  silently diverge from intended behavior
- No test specifically validated that the system used the configurable timestep

### Root Cause
Legacy code: the hardcoded value was introduced during initial prototyping before
`Time<Fixed>` was standardized across all systems. The system was not updated when
the timestep configuration was centralized.

### Corrective Action
Replaced `0.016` with `time.delta_secs()` in `epigenetic_adaptation.rs`.
Commit evidence in DECOUPLING_AUDIT sprint archive.

### Preventive Action
1. Added grep criterion to sprint closure: verify no hardcoded dt values remain
   in simulation systems.
   Command: `grep -r "0\.016\|0\.0167" src/simulation/ --include="*.rs"`
   Expected: 0 matches.
2. Coding Rule 10 (constants in constants modules) re-emphasized in sprint primer.
3. Quality objective QO-5 (zero warnings) catches some instances via clippy's
   `approx_constant` lint.

### Verification of Effectiveness
- `cargo test`: 3,113 passed, 0 failures
- Grep for hardcoded dt: 0 matches in `src/simulation/`
- Grep criterion added to DECOUPLING_AUDIT closure checklist (verified)

### Status: CLOSED
```

**Record location:** `docs/sprints/archive/DECOUPLING_AUDIT/README.md` line 23.

### 6.2 Example 2: Conservation Violation Risk --- Preventive

```
## CAPA-2026-002: Conservation property fuzzing (preventive)

**Type:** Preventive
**Severity:** N/A (preventive --- no NC detected)
**Date opened:** 2026-03-25
**Date closed:** 2026-03-25
**Sprint:** SIMULATION_RELIABILITY (R1: units + conservation)

### Detection
During SIMULATION_RELIABILITY sprint planning, the Planificador identified that
conservation invariants (Axiom 2: Pool Invariant, Axiom 5: Conservation) were
tested only with specific hand-chosen inputs. A sophisticated violation (e.g.,
conservation holding for normal inputs but failing at boundary values) could
escape detection.

### Investigation
- Conservation equations in `blueprint/equations/` had unit tests with specific
  inputs but no fuzzing
- A conservation violation would be NC-CRIT (Axiom violation)
- The risk of undetected violation was proportional to input space coverage

### Root Cause
Missing test coverage category: property-based testing was not part of the
original testing strategy. Only deterministic inputs were used.

### Preventive Action
1. Added `proptest` as dev-dependency in `Cargo.toml`
2. Created `tests/property_conservation.rs` with strategies generating arbitrary
   valid inputs
3. Tests verify: energy conservation (sum never increases), pool invariant
   (children never exceed parent), dissipation positivity (loss always >= 0)
4. Property tests run as part of `cargo test` --- no separate invocation needed

### Verification of Effectiveness
- `cargo test`: all proptest cases pass (50+ fuzzing runs per property)
- No conservation violations found under arbitrary inputs
- Proptest is now a standing part of the test suite (runs at every sprint closure)

### Status: CLOSED
```

**Record location:** `docs/sprints/archive/SIMULATION_RELIABILITY/` and `tests/property_conservation.rs`.

### 6.3 Example 3: Underived Constant --- Corrective + Preventive

```
## CAPA-2026-003: COMMENSALISM_INTAKE not derived from fundamentals

**Type:** Corrective + Preventive
**Severity:** NC-MED
**Date opened:** 2026-04-01
**Date closed:** 2026-04-01
**Sprint:** DC-AUDIT (DECOUPLING_AUDIT track)

### Detection
During DECOUPLING_AUDIT constants audit, `COMMENSALISM_INTAKE` in
`symbiosis_effect.rs` was identified as a hardcoded constant not derived from
any of the 4 fundamental constants.

### Investigation
- The AXIOMATIC_INFERENCE sprint (7/7 archived) established that ALL lifecycle
  constants must derive from the 4 fundamentals
- `COMMENSALISM_INTAKE` predated AXIOMATIC_INFERENCE and was never migrated
- Similarly, `MUTUALISM` constant was hardcoded

### Root Cause
Incomplete migration: AXIOMATIC_INFERENCE sprint focused on core lifecycle
constants (thresholds, senescence, pressure) but did not audit emergence/
symbiosis module constants.

### Corrective Action
- Derived `COMMENSALISM_INTAKE` from `DISSIPATION_SOLID` (the fundamental
  constant governing solid-state energy loss)
- Derived `MUTUALISM` from `2 * DISSIPATION_SOLID`
- Documented derivation inline in `symbiosis_effect.rs`

### Preventive Action
1. All new constants must include inline comment documenting their derivation
   chain back to one of the 4 fundamentals
2. Any constant without a derivation comment is flagged during Verificador review
3. The `derived_thresholds.rs` module (17 tests) serves as the reference
   implementation for the derivation pattern

### Verification of Effectiveness
- `cargo test`: 3,113 passed (including 17 derived_thresholds tests)
- No standalone numeric constants remain in symbiosis module
- Derivation chain documented inline

### Status: CLOSED
```

**Record location:** `docs/sprints/archive/DECOUPLING_AUDIT/README.md` lines 25-26.

## 7. CAPA Log

### 7.1 Current CAPA Summary

| CAPA ID | Title | Type | Severity | Date Opened | Date Closed | Status |
|---------|-------|------|----------|-------------|-------------|--------|
| CAPA-2026-001 | Hardcoded dt in epigenetic_adaptation | Corrective + Preventive | NC-MED | 2026-04-01 | 2026-04-01 | CLOSED |
| CAPA-2026-002 | Conservation property fuzzing | Preventive | N/A | 2026-03-25 | 2026-03-25 | CLOSED |
| CAPA-2026-003 | COMMENSALISM_INTAKE underived | Corrective + Preventive | NC-MED | 2026-04-01 | 2026-04-01 | CLOSED |

### 7.2 CAPA Metrics

| Metric | Value |
|--------|-------|
| Total CAPAs opened | 3 |
| Total CAPAs closed | 3 |
| Open CAPAs | 0 |
| Average time to close | <1 day |
| CAPAs with preventive component | 3/3 (100%) |
| CAPAs that required CAPA reopening | 0 |

### 7.3 Known Limitation

The CAPA log above reflects only formally documented CAPAs from sprint archives. Earlier sprints (pre-DECOUPLING_AUDIT) may have included corrective actions that were not formally tracked as CAPAs. The CAPA process is retroactively formalized starting from this document. All future nonconformances requiring root cause analysis will be tracked using the CAPA record template in §5.

## 8. CAPA Effectiveness Review

### 8.1 Short-Term Effectiveness

Short-term effectiveness is verified at CAPA closure (§4.7): test passes, grep clean, Verificador PASS.

### 8.2 Long-Term Effectiveness

Long-term effectiveness is verified by monitoring for recurrence:

| Mechanism | What It Monitors | Frequency |
|-----------|-----------------|-----------|
| Sprint closure grep criteria | Same violation pattern detected in new code | Every sprint closure |
| `cargo test` regression tests | Same defect recurring in modified code | Every sprint closure (every `cargo test` run) |
| Annual audit (RD-5.5 §5.2) | Systemic recurrence patterns across sprints | Annual |

If a violation recurs despite a preventive action, the original CAPA is reopened and the preventive action is strengthened (e.g., adding a compile-time check, tightening the grep pattern, adding a clippy lint).

### 8.3 Escalation

If a CAPA is reopened more than once for the same root cause:
1. The issue is escalated to Planificador for systemic analysis
2. A structural solution is required (not just another grep criterion)
3. Examples of structural solutions: adding a compile-time constraint (type system enforcement), modifying `CLAUDE.md` to add a new hard block, or creating an automated CI gate

## 9. Cross-References

| Document | Path | Relationship |
|----------|------|-------------|
| RD-5.6 Nonconforming Product | `docs/regulatory/05_quality_system/NONCONFORMING_PRODUCT.md` | Nonconformances trigger corrective CAPAs |
| RD-5.5 Internal Audit | `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` | Audit findings may trigger CAPAs; audit verifies CAPA effectiveness |
| RD-5.2 Quality Policy | `docs/regulatory/05_quality_system/QUALITY_POLICY.md` | Quality objectives (QO-1 through QO-6) define conformance targets |
| RD-1.4 SDP | `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` | Sprint lifecycle governs CAPA implementation |
| CLAUDE.md | Repository root | Source of truth for axioms, hard blocks, coding rules |
| DECOUPLING_AUDIT | `docs/sprints/archive/DECOUPLING_AUDIT/README.md` | Real CAPA examples (3 CAPAs) |
| AXIOMATIC_INFERENCE | `docs/sprints/archive/AXIOMATIC_INFERENCE/` | Establishes derivation chain (context for CAPA-2026-003) |
| Property conservation tests | `tests/property_conservation.rs` | Preventive action from CAPA-2026-002 |

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. CAPA process (7 steps), triggers (corrective + preventive), root cause categories (6 types), record template, 3 real examples, CAPA log (3 entries, all closed), effectiveness review mechanism. |
