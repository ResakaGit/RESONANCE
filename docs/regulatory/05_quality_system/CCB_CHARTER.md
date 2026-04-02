---
document_id: RD-5.9
title: Change Control Board Charter
standard: ISO 13485:2016 §5.6, IEC 62304 §8
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Change Control Board Charter

## 1. Purpose

This document establishes the Change Control Board (CCB) for the RESONANCE project. The CCB is responsible for reviewing and approving changes to regulatory documentation, safety-critical code changes, and any modifications that affect the project's compliance posture under ISO 13485:2016 and IEC 62304.

The CCB ensures that changes to controlled documents and safety-relevant systems are evaluated for impact before implementation, preventing uncontrolled drift in regulatory documentation and maintaining traceability between design decisions and their justifications.

**Cross-references:**
- RD-5.1 `docs/regulatory/05_quality_system/QUALITY_MANUAL.md` --- QMS structure and management responsibility
- RD-5.3 `docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md` --- Document control procedures
- RD-5.5 `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` --- Audit procedures
- RD-5.8 `docs/regulatory/05_quality_system/COMPETENCE_RECORDS.md` --- Role competence requirements
- RD-5.10 `docs/regulatory/05_quality_system/TRAINING_MATRIX.md` --- Training and competence matrix
- `CLAUDE.md` §Roles --- Alquimista, Observador, Planificador, Verificador

## 2. Membership

### 2.1 Permanent Members

| Role | CCB Function | Justification |
|------|-------------|---------------|
| **Verificador** | Chair | Full DOD enforcement capability; PASS/WARN/BLOCK verdict authority; regulatory awareness (RD-5.8 §3.4) |
| **Observador** | Permanent member | Code review expertise; axiom compliance verification; math correctness auditing (RD-5.8 §3.2) |

### 2.2 Advisory Members (As Needed)

| Role | CCB Function | When Involved |
|------|-------------|---------------|
| **Alquimista** | Technical advisor | Changes affecting simulation code, equations, or constants |
| **Planificador** | Architecture advisor | Changes affecting layer design, sprint scope, or system decomposition |

### 2.3 Current State

**Gap acknowledged:** RESONANCE is a single-developer project. All 4 roles are performed by the same individual (RD-5.8 §2). The CCB structure is defined for organizational readiness and process integrity, not to create fictional separation of duties. When additional contributors join, CCB membership will map to distinct individuals.

## 3. Scope of Authority

The CCB has review and disposition authority over the following change categories:

### 3.1 Mandatory CCB Review

| Change Category | Examples | Rationale |
|----------------|----------|-----------|
| **Regulatory document changes** | Any modification to `docs/regulatory/**/*.md` (RD-1.* through RD-7.*) | Regulatory documents are controlled; changes must be evaluated for compliance impact |
| **Safety class reclassification** | Moving from IEC 62304 Class A to Class B or higher | Reclassification cascades through the entire regulatory file; requires full impact assessment |
| **DOD exceptions** | Any new `// DEBT:` annotation in simulation code | Strong defaults (CLAUDE.md §Hard Blocks) require documented justification |
| **SOUP/SBOM updates** | Adding, removing, or upgrading dependencies in `Cargo.toml` | Dependency changes affect SOUP analysis (RD-3.2) and SBOM (RD-3.3) |
| **Axiom-adjacent changes** | Modifications to `blueprint/equations/derived_thresholds.rs` or fundamental constants | 4 constants and 8 axioms are constitutional; any change requires maximum scrutiny |

### 3.2 Routine Changes (No CCB Review Required)

| Change Category | Examples | Rationale |
|----------------|----------|-----------|
| Bug fixes within existing safety class | Correcting a calculation error in equations | Does not change compliance posture |
| New tests | Adding test coverage for existing features | Improves quality without risk |
| Sprint documentation | New sprint READMEs, closure criteria | Process artifacts, not controlled documents |
| Cosmetic/rendering changes | Visual tuning, color adjustments | Not safety-relevant for Class A research tool |

## 4. Process

### 4.1 Change Proposal

All changes requiring CCB review are submitted via GitHub Pull Request with the `ccb-review` label.

```
Change Proposal Flow:
  1. Author creates PR with changes
  2. Author adds `ccb-review` label
  3. PR description includes:
     - Change category (§3.1)
     - Documents affected (RD-X.Y identifiers)
     - Impact assessment (what changes, what doesn't)
     - Axiom compliance statement (confirm no axiom violated)
  4. CCB review proceeds per §4.2
```

### 4.2 Review

| Step | Action | Responsible |
|------|--------|-------------|
| 1 | Verify completeness of change proposal (all required fields present) | Verificador (Chair) |
| 2 | Assess regulatory impact: does the change affect safety classification, risk controls, or validation claims? | Observador |
| 3 | Assess technical correctness: are cross-references updated, constants consistent, tests passing? | Verificador |
| 4 | For axiom-adjacent changes: verify no axiom weakened, bypassed, or contradicted | Observador |
| 5 | Record disposition (§4.3) | Verificador (Chair) |

### 4.3 Disposition

| Disposition | Meaning | Action |
|-------------|---------|--------|
| **APPROVE** | Change is acceptable; no compliance impact or impact is positive | Merge PR; update REVIEW_LOG.md |
| **REJECT** | Change introduces compliance risk, axiom violation, or unjustified regression | Close PR with documented rationale; author may revise and resubmit |
| **DEFER** | Change requires additional information, analysis, or external input | PR remains open; action items documented; resubmit when resolved |

### 4.4 Records

All CCB dispositions are recorded in:
1. **GitHub PR comments** --- the disposition (APPROVE/REJECT/DEFER) with rationale, posted as a review comment on the PR
2. **REVIEW_LOG.md** --- a cumulative log of all CCB decisions, maintained in `docs/regulatory/05_quality_system/REVIEW_LOG.md`

## 5. Cadence

| Activity | Frequency | Scope |
|----------|-----------|-------|
| **Per-sprint CCB review** | At the end of each active sprint track | All PRs with `ccb-review` label from that sprint |
| **Quarterly risk/SOUP review** | Every 3 months (aligned with QUARTERLY_REVIEW_TEMPLATE.md, RD-5.11) | Risk file currency (RD-2.*), SOUP versions (RD-3.2), SBOM (RD-3.3) |
| **Ad hoc review** | As needed | Triggered by safety class reclassification proposals, critical SOUP CVEs, or axiom challenges |

## 6. Escalation

### 6.1 Standard Escalation

| Situation | Resolution |
|-----------|------------|
| Verificador and Observador agree | Disposition stands |
| Verificador and Observador disagree on a routine change | Planificador reviews and decides |
| Verificador and Observador disagree on a regulatory change | Full discussion; Planificador mediates; if no consensus, change is DEFERRED |

### 6.2 Safety Class Changes

Safety class reclassification (e.g., Class A to Class B) requires **unanimity** among all CCB members (permanent + advisory). No single role can unilaterally reclassify.

Rationale: Reclassification triggers cascade effects across the entire regulatory file (new documentation requirements, formal verification, independent testing). The cost of incorrect reclassification is high in both directions: premature reclassification wastes effort; missed reclassification creates compliance gaps.

### 6.3 Current State

**Gap acknowledged:** With a single developer, escalation paths are aspirational. The CCB process is maintained for organizational readiness. Escalation becomes meaningful when the team grows to 2+ contributors.

## 7. Records

### 7.1 Record Types

| Record | Location | Content |
|--------|----------|---------|
| CCB disposition | GitHub PR review comments | APPROVE/REJECT/DEFER + rationale |
| Cumulative decision log | `docs/regulatory/05_quality_system/REVIEW_LOG.md` | All CCB decisions with PR reference, date, disposition, and rationale |
| Quarterly review minutes | Per QUARTERLY_REVIEW_TEMPLATE.md (RD-5.11) | Findings, actions, sign-off |

### 7.2 Record Retention

All CCB records are retained indefinitely in Git history per RD-5.4 (Record Control) §7.

## 8. Quorum

| Change Type | Required Quorum | Rationale |
|-------------|----------------|-----------|
| **Regulatory document changes** | 2 of 2 permanent members (Verificador + Observador) | Regulatory changes affect compliance posture; both perspectives required |
| **Safety class reclassification** | Unanimous (all participating members) | Maximum scrutiny for highest-impact changes |
| **Routine changes with `ccb-review` label** | 1 of 2 permanent members | Lower risk; single reviewer sufficient for DOD exceptions, minor SOUP updates |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial charter. Membership, scope, process, cadence, escalation, quorum defined. Single-developer gap acknowledged honestly. |
