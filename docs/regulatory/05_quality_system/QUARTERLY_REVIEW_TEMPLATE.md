---
document_id: RD-5.11
title: Quarterly Management Review Template
standard: ISO 13485:2016 §5.6, ISO 14971:2019 §9
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Quarterly Management Review Template

## 1. Purpose

This document provides the template for quarterly management reviews of the RESONANCE Quality Management System. Quarterly reviews satisfy ISO 13485:2016 §5.6 (Management review) and ISO 14971:2019 §9 (Post-production monitoring) by ensuring periodic evaluation of the risk file, SOUP dependencies, SBOM currency, post-production feedback, and test suite integrity.

**First scheduled review:** Q3 2026 (July 2026).

**Cross-references:**
- RD-2.1 through RD-2.7 --- Risk management file
- RD-3.2 `docs/regulatory/03_traceability/SOUP_ANALYSIS.md` --- SOUP risk assessment
- RD-3.3 `docs/regulatory/03_traceability/SBOM.md` --- Software Bill of Materials
- RD-2.7 `docs/regulatory/02_risk_management/POST_PRODUCTION_MONITORING.md` --- Monitoring channels
- RD-5.5 `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` --- Internal audit procedure
- RD-5.9 `docs/regulatory/05_quality_system/CCB_CHARTER.md` --- Change Control Board

## 2. Review Period

| Field | Value |
|-------|-------|
| **Review date** | _YYYY-MM-DD_ |
| **Review period** | _From YYYY-MM-DD to YYYY-MM-DD_ |
| **Reviewer(s)** | _Name(s) and role(s)_ |
| **Review type** | Quarterly / Annual / Ad hoc |

## 3. Scope

This quarterly review covers:

| Document/System | RD Reference | What to Check |
|----------------|-------------|---------------|
| Risk analysis | RD-2.2 | New hazards identified? Existing hazard severity/probability changed? |
| Risk controls | RD-2.4 | Controls still effective? New controls needed? |
| Risk evaluation | RD-2.3 | Acceptability criteria still appropriate? |
| Residual risk | RD-2.5, RD-2.6 | Overall risk still acceptable? |
| SOUP analysis | RD-3.2 | Versions match Cargo.lock? New CVEs reported? |
| SBOM | RD-3.3 | Regenerated if dependencies changed? Accurate? |
| Post-production monitoring | RD-2.7 | Issues received? Advisories issued? Actions taken? |
| Test suite | `cargo test` | Still passing? Count regression? |
| CCB dispositions | RD-5.9 | Findings from CCB reviews requiring follow-up? |

## 4. Review Checklist

### 4.1 Risk Analysis

- [ ] Reviewed RD-2.2 (Risk Analysis) for completeness against current codebase
- [ ] No new hazards identified since last review (or new hazards documented)
- [ ] Existing risk controls (RD-2.4) verified as still effective
- [ ] No axiom violations detected in the review period
- [ ] Residual risk levels (RD-2.5) unchanged (or changes documented)

**Findings:** _Document any new hazards, control failures, or risk level changes here._

### 4.2 SOUP Dependencies

- [ ] Ran `cargo audit` --- output reviewed for CVEs
- [ ] SOUP versions in RD-3.2 match current `Cargo.lock`
- [ ] No new CVEs with CVSS >= 7.0 for runtime dependencies
- [ ] No new CVEs with CVSS >= 9.0 (critical) for any dependency
- [ ] Any CVEs found have been assessed for exploitability in RESONANCE context

**`cargo audit` output summary:** _Paste summary or "0 vulnerabilities found"._

**Findings:** _Document any CVEs, version mismatches, or SOUP concerns here._

### 4.3 SBOM

- [ ] SBOM (RD-3.3) regenerated if dependencies changed since last review
- [ ] SBOM accurately reflects current `Cargo.lock` contents
- [ ] No new dependencies added without CCB review (RD-5.9)

**Findings:** _Document any SBOM discrepancies here._

### 4.4 Post-Production Monitoring

- [ ] Reviewed GitHub Issues for new reports (count: ___)
- [ ] Reviewed GitHub Discussions for new threads (count: ___)
- [ ] Checked email for direct feedback (count: ___)
- [ ] Checked Google Scholar for DOI citations (count: ___)
- [ ] No clinical use reports detected
- [ ] Advisories issued in review period: ___ (list or "None")
- [ ] Actions taken per RD-2.7 §5: ___ (list or "None")

**Findings:** _Document any feedback, clinical use concerns, or actions here._

### 4.5 Test Suite

- [ ] `cargo test` passes with 0 failures
- [ ] Test count: ___ (previous: ___, delta: ___)
- [ ] No test count regression (count >= previous review)
- [ ] `cargo check` produces 0 warnings
- [ ] `cargo clippy` produces 0 warnings

**Findings:** _Document any test failures, regressions, or warning increases here._

### 4.6 CCB Dispositions

- [ ] All CCB dispositions from the review period documented in REVIEW_LOG.md
- [ ] No DEFERRED items past their resolution deadline
- [ ] No REJECTED changes resubmitted without addressing rationale

**Findings:** _Document any open CCB items requiring follow-up here._

## 5. Output

### 5.1 Review Summary

| Item | Status | Notes |
|------|--------|-------|
| Risk file | _Current / Needs update_ | |
| SOUP | _Current / CVEs found_ | |
| SBOM | _Current / Needs regeneration_ | |
| Post-production | _No issues / Issues found_ | |
| Test suite | _Passing / Failures found_ | |
| CCB | _No open items / Open items_ | |

### 5.2 Corrective Actions

| # | Finding | Action Required | Owner | Deadline | Status |
|---|---------|----------------|-------|----------|--------|
| 1 | _Description_ | _Action_ | _Role_ | _YYYY-MM-DD_ | _Open / Closed_ |

_If no corrective actions needed, state: "No corrective actions identified."_

### 5.3 Preventive Actions

| # | Observation | Preventive Action | Owner | Deadline | Status |
|---|------------|-------------------|-------|----------|--------|
| 1 | _Description_ | _Action_ | _Role_ | _YYYY-MM-DD_ | _Open / Closed_ |

_If no preventive actions needed, state: "No preventive actions identified."_

## 6. Sign-Off

| Role | Name | Date | Disposition |
|------|------|------|------------|
| Reviewer (Verificador) | | _YYYY-MM-DD_ | _Accepted / Accepted with actions / Not accepted_ |
| Reviewer (Observador) | | _YYYY-MM-DD_ | _Accepted / Accepted with actions / Not accepted_ |

**Next scheduled review:** _YYYY-MM-DD_ (per §5 cadence: quarterly)

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial quarterly review template. Checklist covers risk file, SOUP, SBOM, post-production monitoring, test suite, and CCB dispositions. First scheduled review: Q3 2026. |
