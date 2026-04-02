---
document_id: RD-2.7
title: Post-Production Monitoring Plan
standard: ISO 14971:2019 §9
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Post-Production Monitoring Plan

## 1. Purpose

This document defines the post-production monitoring plan for RESONANCE, satisfying ISO 14971:2019 §9 (Production and post-production activities). The standard requires that the manufacturer establish, document, and maintain a system to actively collect and review information about the medical device in the production and post-production phases.

ISO 14971:2019 §9 requires:
- §9.1: Establish a system to collect and review relevant post-production information
- §9.2: Determine the information to be collected
- §9.3: Evaluate collected information for relevance to safety
- §9.4: Take action when new hazards or increased risks are identified
- §9.5: Feed information back into the risk management process

**Important context:** RESONANCE is a research tool classified as IMDRF SaMD Category I and IEC 62304 Class A. It is not regulated as a medical device. Post-production monitoring is maintained voluntarily as best practice and for readiness in case of future reclassification.

**Cross-references:**
- RD-1.1 `docs/regulatory/01_foundation/INTENDED_USE.md` --- Intended use, excluded uses
- RD-1.2 `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` --- Safety classification (Class A)
- RD-1.7 `docs/regulatory/01_foundation/SOFTWARE_MAINTENANCE_PLAN.md` --- Maintenance strategy, change evaluation
- RD-1.8 `docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md` --- Problem resolution process
- RD-2.1 `docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md` --- Risk management scope and criteria
- RD-2.2 `docs/regulatory/02_risk_management/RISK_ANALYSIS.md` --- Identified hazards
- RD-2.4 `docs/regulatory/02_risk_management/RISK_CONTROLS.md` --- Risk control measures
- RD-2.6 `docs/regulatory/02_risk_management/RISK_MANAGEMENT_REPORT.md` --- Overall risk acceptability
- RD-3.2 `docs/regulatory/03_traceability/SOUP_ANALYSIS.md` --- SOUP risk assessment
- RD-5.7 `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` --- Corrective/preventive actions

## 2. Monitoring Channels

### 2.1 Channel Inventory

| Channel | Medium | Information Type | Monitoring Method | Current Status |
|---------|--------|-----------------|-------------------|----------------|
| GitHub Issues | `https://github.com/ResakaGit/RESONANCE/issues` | Bug reports, feature requests, usage questions | Manual review of new issues | **Active** --- enabled, 0 issues filed to date |
| GitHub Discussions | `https://github.com/ResakaGit/RESONANCE/discussions` | Community discussion, usage patterns, scientific questions | Manual review of new threads | **Configured** --- available but unused |
| Email | Author contact (via Zenodo record) | Direct feedback from researchers, collaboration inquiries | Manual review of incoming email | **Available** --- contact info in Zenodo metadata |
| Paper citation tracking | Google Scholar, Semantic Scholar | Citations that challenge model validity or report clinical use attempts | Periodic search for DOI citations | **Planned** --- DOI registered (10.5281/zenodo.19342036), not yet indexed |
| RustSec / `cargo audit` | `https://rustsec.org/` | SOUP vulnerabilities (CVE/RUSTSEC advisories) | `cargo audit` execution | **Active** --- can be run on demand |
| Crate release monitoring | `https://crates.io/` (Bevy, glam, etc.) | SOUP version updates, deprecations, breaking changes | Manual check of key dependencies | **Ad hoc** --- no automated monitoring |

### 2.2 Infrastructure Gaps

**Gap acknowledged:** The monitoring channels listed above are **defined but not actively managed**. Specifically:

| Gap | Description | Impact | Resolution Path |
|-----|-------------|--------|-----------------|
| No automated issue triage | GitHub Issues are reviewed manually by the sole developer | Issues could be missed during periods of low activity | Implement GitHub notification alerts or assign issue triage SLA |
| No automated citation monitoring | Paper citation tracking requires manual Google Scholar searches | Misuse of RESONANCE in clinical contexts could go undetected | Set up Google Scholar alerts for the Zenodo DOI |
| No automated `cargo audit` schedule | SOUP vulnerability scanning is manual | Vulnerabilities could accumulate undetected between releases | Implement GitHub Actions with `cargo audit` on schedule (weekly) |
| No structured feedback form | Users can only report via GitHub Issues (unstructured) or email | Important context may be omitted from reports | Low priority for current user base; revisit if user base grows |
| No formal monitoring log | No record of monitoring activities performed | Cannot demonstrate to an auditor that monitoring is occurring | Create quarterly monitoring log (see §5) |

## 3. Information Collection

### 3.1 Information Categories

| Category | What to Collect | Source | Relevance to Safety |
|----------|----------------|--------|---------------------|
| **User feedback** | Bug reports, unexpected behavior, usability issues, feature requests | GitHub Issues, Discussions, email | Direct --- may reveal defects affecting simulation correctness |
| **SOUP vulnerabilities** | CVE/RUSTSEC advisories for runtime dependencies | RustSec database, `cargo audit` | Indirect --- SOUP vulnerability could affect simulation integrity if exploited |
| **Scientific challenges** | Published criticism of model assumptions, axiom challenges, comparisons showing model failure | Paper citations, conference discussions, email | Indirect --- could invalidate validation claims (Bozic 2013, calibration profiles) |
| **Clinical use reports** | Any report of RESONANCE output being used in clinical decision-making | GitHub Issues, email, citations | Critical --- would require immediate reclassification assessment per RD-1.2 §5 |
| **Regulatory changes** | Changes to IMDRF SaMD framework, IEC 62304, ISO 14971, or FDA guidance | Regulatory body publications | Indirect --- could affect classification or documentation requirements |
| **Similar product incidents** | Safety incidents involving similar simulation tools (e.g., computational biology SaMD, oncology decision support) | FDA MAUDE, EU Vigilance, literature | Indirect --- could indicate systemic risks applicable to RESONANCE |

### 3.2 SOUP Monitoring Details

RESONANCE has 14 runtime dependencies (RD-3.3 SBOM). The following are monitored for security vulnerabilities:

| Dependency | Version (Cargo.lock) | Monitoring Source | Criticality |
|------------|---------------------|-------------------|-------------|
| bevy | 0.15.x | RustSec, GitHub Security Advisories | High --- core engine |
| glam | 0.29.x | RustSec | High --- math operations |
| rayon | (pinned) | RustSec | Medium --- batch parallelism |
| All others | (pinned via Cargo.lock) | RustSec via `cargo audit` | Per SOUP risk assessment (RD-3.2) |

**Monitoring command:**

```bash
cargo audit
```

This checks all dependencies in `Cargo.lock` against the RustSec Advisory Database.

## 4. Evaluation Criteria

### 4.1 When Collected Information Triggers Risk Re-Evaluation

Not all feedback requires risk re-evaluation. The following criteria determine when a piece of collected information triggers a review of the risk management file (RD-2.1 through RD-2.6):

| Trigger | Condition | Action |
|---------|-----------|--------|
| **New clinical use claim** | Any report that RESONANCE output was used to inform a clinical decision (treatment selection, dosing, diagnosis) | **Immediate:** Re-evaluate safety classification (RD-1.2). Assess whether intended use statement (RD-1.1) needs a stronger disclaimer. Consider issuing advisory (RD-1.8 §5). |
| **Axiom challenge** | Published paper or credible feedback that challenges the validity of one or more of the 8 axioms in the context where RESONANCE is applied | **High priority:** Review axiom justification. If challenge is valid, assess impact on all derived behavior and validation claims. |
| **Bozic validation failure** | Evidence that RESONANCE's qualitative Bozic 2013 prediction is incorrect or misleading in the context claimed | **High priority:** Re-run validation with updated parameters. Update RD-4.4 (Validation Report) and RD-6.3 (Limitations Report). |
| **SOUP CVE with CVSS >= 7.0** | A dependency has a vulnerability scored 7.0 or higher on the CVSS v3 scale | **Prompt:** Assess exploitability in RESONANCE context. If exploitable: update dependency, regression test, release patch. If not exploitable: document rationale, monitor for escalation. |
| **SOUP CVE with CVSS >= 9.0** | A dependency has a critical vulnerability | **Immediate:** Same as CVSS >= 7.0 but with expedited timeline. |
| **SOUP CVE with CVSS < 7.0** | A dependency has a low or medium vulnerability | **Routine:** Record in SOUP analysis (RD-3.2). Evaluate at next scheduled review. |
| **New IMDRF/IEC/ISO guidance** | Regulatory body publishes guidance that changes classification criteria or documentation requirements | **Routine:** Evaluate impact at next quarterly review. Update RD-1.5 (Regulatory Strategy) if needed. |
| **Similar product incident** | Another computational biology or oncology simulation tool involved in a safety incident | **Routine:** Evaluate whether the incident scenario applies to RESONANCE. Document assessment. |

### 4.2 Severity Classification for Feedback

| Severity | Definition | Response Timeline |
|----------|------------|-------------------|
| **Critical** | Clinical use detected, axiom violation discovered, or critical SOUP vulnerability | Within 72 hours |
| **High** | Validation challenge, SOUP CVSS >= 7.0, or reproducibility failure reported | Within 2 weeks |
| **Medium** | Scientific criticism, non-critical SOUP vulnerability, or regulatory change | Next quarterly review |
| **Low** | Feature request, usability feedback, cosmetic issue | Backlog |

## 5. Actions

### 5.1 Action Types

When evaluation criteria (§4) are met, the following actions may be taken:

| Action | When | Procedure |
|--------|------|-----------|
| **Update risk file** | New hazard identified or existing risk level changed | Re-analyze per RD-2.2, re-evaluate per RD-2.3, implement new controls per RD-2.4, update report per RD-2.6 |
| **Issue advisory** | Problem affects users who may have acted on RESONANCE output | Per RD-1.8 §5 (advisory mechanisms: GitHub Issue/Release, README update, Zenodo version) |
| **Release patch** | Defect or vulnerability requires code change | Per RD-7.5 (Release Package), RD-1.7 (Maintenance Plan) |
| **Update disclaimers** | New misuse scenario identified | Update README.md, CLAUDE.md, and/or in-code disclaimers per RD-1.1 §6 |
| **Reclassification assessment** | Evidence that intended use has shifted or that RESONANCE is being used clinically | Re-evaluate RD-1.2 (Safety Classification) per §5 conditional reclassification scenarios |
| **CAPA** | Recurring problem or systemic issue detected | Per RD-5.7 (CAPA Procedure) |
| **No action** | Information reviewed, no risk impact identified | Document "reviewed, no action" in monitoring log |

### 5.2 Risk Management Feedback Loop

Per ISO 14971:2019 §9.5, information collected through post-production monitoring must feed back into the risk management process:

```
Monitoring Channel --> Information Collection --> Evaluation (§4) --> Action (§5.1)
                                                                          |
                                                                          v
                                                              Risk Management File
                                                          (RD-2.1 through RD-2.6)
                                                                          |
                                                                          v
                                                              Updated Controls (RD-2.4)
                                                              Updated Residual Risk (RD-2.5)
                                                              Updated Report (RD-2.6)
```

## 6. Review Frequency

### 6.1 Scheduled Reviews

| Review Type | Frequency | Scope | Record |
|-------------|-----------|-------|--------|
| SOUP vulnerability check | Before each release + quarterly | `cargo audit` output, RustSec advisories | SOUP analysis update (RD-3.2) |
| GitHub Issues/Discussions review | Weekly (when active users exist) | New issues, discussion threads | Monitoring log entry |
| Citation monitoring | Quarterly | Google Scholar search for DOI | Monitoring log entry |
| Regulatory landscape scan | Quarterly | IMDRF, FDA, EU MDR updates relevant to simulation software | Monitoring log entry |
| Full monitoring review | Annually | All channels, all collected information, trend analysis | Annual monitoring report |

### 6.2 Frequency Adjustment Triggers

| Condition | New Frequency |
|-----------|---------------|
| Reclassified as SaMD (Category II+) | Monthly for all channels |
| Pharma partnership established | Monthly for SOUP + user feedback |
| >50 active GitHub users | Weekly issue triage with SLA |
| Clinical use detected | Continuous monitoring with dedicated owner |

### 6.3 Current Frequency

**Gap acknowledged:** The scheduled review frequencies above are targets, not current practice. As of commit `971c7ac`, no formal monitoring reviews have been conducted because:

1. The product is pre-1.0 with no tagged releases
2. The user base is effectively 1 (the developer)
3. 0 GitHub Issues have been filed
4. The Zenodo paper has not yet been indexed by citation databases

The first formal monitoring review will be conducted upon:
- First tagged release (v0.1.0 or equivalent), OR
- First external GitHub Issue filed, OR
- Q3 2026, whichever comes first

## 7. Monitoring Log Template

Each monitoring review should produce a log entry with the following structure:

| Field | Content |
|-------|---------|
| **Review date** | YYYY-MM-DD |
| **Review period** | From date to date |
| **Reviewer** | Name/role |
| **Channels checked** | List of channels reviewed |
| **Issues/feedback received** | Count and summary |
| **SOUP vulnerabilities found** | Count, CVE IDs, CVSS scores |
| **Citations found** | Count and relevance assessment |
| **Actions taken** | List of actions per §5.1, or "None" |
| **Risk file updated** | Yes/No. If yes, which documents. |
| **Next review date** | Scheduled per §6.1 |

### 7.1 Initial Log Entry

| Field | Content |
|-------|---------|
| **Review date** | 2026-04-02 |
| **Review period** | Initial --- no prior review |
| **Reviewer** | Resonance Development Team |
| **Channels checked** | GitHub Issues, GitHub Discussions, RustSec, Google Scholar |
| **Issues/feedback received** | 0 |
| **SOUP vulnerabilities found** | To be assessed (`cargo audit` pending formal execution) |
| **Citations found** | 0 (DOI not yet indexed) |
| **Actions taken** | None --- monitoring plan established (this document) |
| **Risk file updated** | No |
| **Next review date** | 2026-07-02 (Q3 2026) or first external issue, whichever comes first |

## 8. Specific Monitoring Scenarios

### 8.1 Scenario: Clinical Use Detected

If evidence emerges that RESONANCE output is being used to inform clinical decisions:

1. **Immediate:** Document the evidence (who, what, how, where)
2. **Within 72 hours:** Re-evaluate safety classification (RD-1.2 §5)
3. **If reclassification needed:** Halt distribution, update README with explicit prohibition, issue GitHub advisory
4. **If reclassification not needed:** Strengthen disclaimers, add specific warning addressing the detected use pattern
5. **Update risk file:** Add new hazard to RD-2.2, evaluate per RD-2.3, implement controls per RD-2.4

### 8.2 Scenario: SOUP Critical Vulnerability (CVSS >= 9.0)

1. **Immediate:** Assess whether the vulnerability is exploitable in RESONANCE context
2. **If exploitable:** Update dependency, run full regression (`cargo test`), release patch
3. **If not exploitable:** Document rationale in RD-3.2, add compensating statement
4. **Notify users:** If a tagged release includes the vulnerable dependency, issue GitHub Security Advisory

### 8.3 Scenario: Published Challenge to Bozic Validation

1. **Evaluate:** Read the challenging publication. Determine if the challenge applies to RESONANCE's qualitative claim.
2. **If valid:** Re-run Bozic validation with parameters addressing the challenge. Update RD-4.4 and RD-6.3.
3. **If invalid:** Document rebuttal with evidence. No action on risk file.
4. **If ambiguous:** Add to limitations (RD-6.3) with honest assessment of uncertainty.

### 8.4 Scenario: Axiom Validity Challenge

1. **Evaluate:** Assess whether the challenge applies within RESONANCE's intended use context (abstract energy simulation, not biophysical measurement)
2. **If valid in context:** This is a constitutional crisis. Halt all development. Reassess affected axiom with full rigor. If axiom must be modified, cascade impact through all derived behavior.
3. **If valid but out of context:** Document as a limitation in RD-6.3. Strengthen intended use boundaries in RD-1.1.
4. **If invalid:** Document rebuttal. No action.

## 9. Linked Artifacts

| Artifact | Reference | Description |
|----------|-----------|-------------|
| GitHub Issue Templates | `.github/ISSUE_TEMPLATE/bug_report.yml` | Structured bug reporting with severity classification (Critical/High/Medium/Low) |
| GitHub Issue Templates | `.github/ISSUE_TEMPLATE/feature_request.yml` | Feature requests with axiom impact assessment |
| GitHub Issue Templates | `.github/ISSUE_TEMPLATE/regulatory_feedback.yml` | Regulatory documentation gap reporting |
| Dependabot Configuration | `.github/dependabot.yml` | Automated SOUP dependency vulnerability monitoring |
| Quarterly Review Template | RD-5.11 `docs/regulatory/05_quality_system/QUARTERLY_REVIEW_TEMPLATE.md` | Periodic review of risk file, SOUP, SBOM, and post-production feedback |

**Note:** Monitoring channels activated in sprint RI-3. GitHub Issue templates provide structured intake for the monitoring channels defined in §2.1. Dependabot provides automated SOUP vulnerability scanning (§3.2). Quarterly reviews (RD-5.11) formalize the scheduled review cadence defined in §6.1.

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial post-production monitoring plan. 6 monitoring channels defined, evaluation criteria established, review frequency set. Infrastructure gaps documented honestly --- channels defined but not actively managed. |
