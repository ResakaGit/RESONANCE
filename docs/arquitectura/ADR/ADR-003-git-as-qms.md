# ADR-003: Git as QMS Infrastructure

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** ISO 13485 Quality Management System Infrastructure

## Context

ISO 13485 requires document control (Section 4.2.4), record control (Section 4.2.5), complete audit trails, and traceability from requirements through implementation to verification. Traditionally, these requirements are met by dedicated electronic Quality Management System (eQMS) tools such as MasterControl, Greenlight Guru, or Qualio, which provide role-based access control, digital signatures, workflow automation, and regulatory-ready audit exports.

RESONANCE is a single-developer open-source research project (AGPL-3.0). The codebase, all 43 regulatory documents, sprint archives (78 closures), and test infrastructure are already versioned in Git. The question is whether Git infrastructure can satisfy QMS requirements without a dedicated eQMS tool.

## Decision

Use **Git (commits, history, branches, tags)** as the primary quality management system infrastructure. All code and all documentation are versioned together in a single repository, providing atomic change tracking across the entire quality system.

The mapping to ISO 13485 requirements:

| ISO 13485 Requirement | Git Implementation |
|----------------------|-------------------|
| Document control (4.2.4) | Git version history, commit messages as change descriptions |
| Record control (4.2.5) | Immutable Git blobs, SHA-256 content addressing |
| Approval and review | Commit messages document rationale; sprint closures document review |
| Change history | `git log`, `git blame`, `git diff` -- complete, immutable, attributable |
| Audit trail | Full commit history with author, timestamp, content hash |
| Traceability | File cross-references, sprint-to-code mapping via commit messages |
| Controlled copies | `git clone` produces exact, verified copy; `git status` detects drift |

## Consequences

### Positive
- Zero additional tooling cost -- Git is already the development infrastructure.
- Code and documentation changes are atomic -- a regulatory doc update and its corresponding code change exist in the same commit.
- Complete, immutable history from project inception. No retroactive fabrication possible (SHA integrity).
- 78 archived sprint closures provide structured change control records beyond individual commits.
- Open-source transparency -- any auditor can verify the complete history.

### Negative
- No built-in digital signatures. Author identity is self-asserted via Git config (mitigated: GPG signing planned in RI-2).
- No role-based access control beyond GitHub repository permissions. A dedicated eQMS provides granular read/write/approve/release permissions per document type.
- No automated workflow (e.g., "document requires approval before release"). Process discipline is manual.
- Audit export requires Git expertise -- no one-click "generate audit report" feature.

### Risks
- If RESONANCE reclassifies to Class B/C or pursues formal SaMD status, a dedicated eQMS may become necessary for notified body acceptance. Mitigation: all Git-based records are exportable and would seed an eQMS migration.
- Git history can be rewritten with `--force` push. Mitigation: branch protection planned in RI-1; current single-developer context makes this a low-probability risk.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| MasterControl / Greenlight Guru / Qualio | Cost prohibitive ($20K-100K/year) for a single-developer research project. Capabilities exceed current needs. |
| SharePoint / Google Drive | No immutable history, no atomic commits, no content-addressed integrity. Version control is manual and error-prone. |
| No QMS infrastructure | Non-compliant with ISO 13485. Would undermine the voluntary compliance strategy (ADR-002). |
| Hybrid (Git for code, eQMS for docs) | Splits the audit trail across two systems. Loses atomic code+doc change tracking. Adds synchronization burden. |

## References

- `docs/regulatory/01_foundation/RD-5.3` -- Document Control Procedure
- `docs/regulatory/01_foundation/RD-5.4` -- Record Control Procedure
- `docs/regulatory/01_foundation/RD-7.1` -- 21 CFR Part 11 Compliance
- `docs/regulatory/01_foundation/RD-7.2` -- Data Integrity Policy
- `docs/regulatory/01_foundation/RD-7.3` -- Audit Trail Specification
- ISO 13485:2016, Sections 4.2.4, 4.2.5
- `docs/sprints/archive/` -- 78 sprint closure records
