---
document_id: RD-5.3
title: Document Control Procedure
standard: ISO 13485:2016 §4.2.4
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Document Control Procedure

## 1. Purpose

This procedure defines the controls for creating, reviewing, approving, distributing, and maintaining documents within the RESONANCE Quality Management System, in accordance with ISO 13485:2016 §4.2.4. It ensures that current, approved versions of all quality-relevant documents are available at points of use, and that obsolete documents are identifiable.

## 2. Scope

This procedure applies to all documents that govern or evidence the design, development, verification, and maintenance of RESONANCE software. Documents are categorized as either **controlled** (require formal change control) or **reference** (informational, no formal approval required).

## 3. Controlled Documents

### 3.1 Document Registry

| Category | Document | Path | Authority Level |
|----------|----------|------|-----------------|
| **Constitution** | `CLAUDE.md` | Repository root | Highest --- governs all development. Changes require explicit justification and version control. |
| **Architecture** | `docs/ARCHITECTURE.md` | `docs/` | Canonical architecture reference. Must reflect current codebase state. |
| **Design Specs** | `docs/design/*.md` (6 files) | `docs/design/` | Code-referenced design specifications (TOPOLOGY, ECO_BOUNDARIES, BRIDGE_OPTIMIZER, AXIOMATIC_CLOSURE, SIMULATION_CORE_DECOUPLING, TERRAIN_MESHER) |
| **Module Contracts** | `docs/arquitectura/*.md` (4 files) | `docs/arquitectura/` | Runtime behavior contracts per module |
| **Regulatory Docs** | `docs/regulatory/**/*.md` (RD-1 through RD-7) | `docs/regulatory/` | QMS, risk, traceability, validation, clinical, release documents |
| **Sprint Docs** | `docs/sprints/{TRACK}/SPRINT_*.md` | `docs/sprints/` | Active sprint scope, design, closure criteria |
| **Sprint Archives** | `docs/sprints/archive/{TRACK}/*.md` | `docs/sprints/archive/` | Completed sprint documentation (immutable post-archive) |
| **Package Config** | `Cargo.toml` | Repository root | Dependencies, features, build configuration |
| **Dependency Lock** | `Cargo.lock` | Repository root | Exact pinned dependency versions |
| **Map Configs** | `assets/maps/*.ron` | `assets/maps/` | World map definitions (RON format) |
| **Sprint Backlog** | `docs/sprints/README.md` | `docs/sprints/` | Active/archived sprint tracking, wave status |
| **Archive Index** | `docs/sprints/archive/README.md` | `docs/sprints/archive/` | Completed sprint index with dates and test counts |

### 3.2 Document Identification

Every controlled document is identified by:

| Element | Format | Example |
|---------|--------|---------|
| **Document ID** | `RD-{sprint}.{sequence}` for regulatory; track-based for sprints | `RD-5.3`, `DC-4` |
| **Title** | Descriptive title in YAML frontmatter or Markdown H1 | "Document Control Procedure" |
| **Version** | `{major}.{minor}` in YAML frontmatter | `1.0` |
| **Date** | ISO 8601 in YAML frontmatter | `2026-04-02` |
| **Status** | `DRAFT` / `APPROVED` / `OBSOLETE` in YAML frontmatter | `DRAFT` |
| **File path** | Relative to repository root | `docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md` |
| **Git SHA** | Commit hash at time of approval | `971c7acb99decde45bf28860e6e10372718c51e2` |

Non-regulatory documents (`CLAUDE.md`, `Cargo.toml`, sprint docs) use Git commit history as their version record rather than YAML frontmatter.

## 4. Document Lifecycle

### 4.1 Creation

| Step | Action | Responsibility |
|------|--------|----------------|
| 1 | Identify need (new feature, regulatory requirement, audit finding) | Planificador |
| 2 | Draft document following applicable template | Alquimista (code docs), Planificador (design/regulatory docs) |
| 3 | Assign document ID, version `1.0`, status `DRAFT` | Author |
| 4 | Place in correct directory per §3.1 registry | Author |
| 5 | Commit to `main` branch with descriptive commit message | Author |

### 4.2 Review and Approval

| Step | Action | Responsibility |
|------|--------|----------------|
| 1 | Review document for accuracy, completeness, and conformance to standards | Observador (technical docs), Verificador (regulatory docs) |
| 2 | Issue verdict: PASS / WARN / BLOCK | Reviewer |
| 3 | If BLOCK: return to author for rework | Reviewer |
| 4 | If PASS: change status from `DRAFT` to `APPROVED` in frontmatter | Reviewer |
| 5 | Merge to `main` | Reviewer or Author |

**Approval mechanism:** Merge to the `main` branch constitutes approval. Git records the author, timestamp, and commit message. For trunk-based development, the act of committing to `main` after review is the approval action.

**Known gap:** There is no formal digital signature on approvals. Git commit authorship provides attribution but not electronic signature per 21 CFR Part 11 standards. This gap is acceptable for current research-tool status and is noted in RD-1.5 §5.6 (Part 11 gap analysis).

### 4.3 Distribution

| Channel | Mechanism | Audience |
|---------|-----------|----------|
| **Primary** | GitHub public repository (`https://github.com/ResakaGit/RESONANCE`) | All users (public, open source) |
| **Local** | `git clone` or `git pull` to developer workstation | Contributors |
| **Offline** | Git repository can be fully cloned for offline use | Air-gapped environments |

All controlled documents are distributed via the same Git repository as the source code. There is no separate document management system. This ensures that documents and code are always in sync --- any `git checkout` at a given commit retrieves the exact documents that correspond to that code version.

### 4.4 Change Control

| Step | Action | Record |
|------|--------|--------|
| 1 | Identify need for change (defect, new requirement, audit finding) | Sprint backlog entry or CAPA record |
| 2 | Make change in document | `git diff` shows exact delta |
| 3 | Update version number in YAML frontmatter (minor: non-breaking; major: structural) | Frontmatter `version` field |
| 4 | Update revision history table at end of document | Revision History section |
| 5 | Review change (same process as §4.2) | Reviewer |
| 6 | Commit to `main` with descriptive message referencing reason for change | Commit message |

**Change evidence:** Every document change is recorded in Git history. `git log --follow -- <path>` provides the complete change history for any document. `git diff <old-commit>..<new-commit> -- <path>` shows the exact text delta.

### 4.5 Obsolescence

Documents are made obsolete by:
1. Changing the `status` field in YAML frontmatter to `OBSOLETE`
2. Adding a note at the top of the document pointing to the replacement document
3. Committing the change to `main`

Obsolete documents are **not deleted**. They remain in the Git repository for historical traceability. Git history provides permanent access to any previous version.

For sprint documents, archival (move to `docs/sprints/archive/`) is the equivalent of "obsolescence" --- the sprint is completed and its document becomes a historical record.

## 5. External Documents

### 5.1 Referenced Standards

The following external documents are referenced by the QMS but are not controlled within the repository. They are maintained by their respective standards bodies.

| Standard | Version | Source |
|----------|---------|--------|
| ISO 13485 | 2016 | ISO |
| IEC 62304 | 2006+Amd1:2015 | IEC |
| ISO 14971 | 2019 | ISO |
| ASME V&V 40 | 2018 | ASME |
| 21 CFR Part 11 | Current | FDA |
| IMDRF SaMD N10 | R4:2013 | IMDRF |

### 5.2 External References in Code

Published references cited in the codebase (Bozic 2013, Gatenby 2009, London 2009) are identified by author, year, and journal in code comments and the paper (`docs/paper/resonance_arxiv.tex`). These are reference documents, not controlled documents.

## 6. Document Storage and Backup

### 6.1 Primary Storage

All documents are stored in the Git repository at `https://github.com/ResakaGit/RESONANCE`. Git provides:

| Feature | Mechanism |
|---------|-----------|
| **Integrity** | SHA-256 content-addressed storage. Any bit-flip is detectable. |
| **Immutability** | Commit history is append-only. Previous versions are permanently accessible via commit SHA. |
| **Attribution** | Every commit records author, email, and timestamp. |
| **Branching** | Trunk-based development on `main`. No long-lived feature branches. |

### 6.2 Backup

| Backup | Mechanism | Frequency |
|--------|-----------|-----------|
| GitHub cloud | GitHub repository hosting (primary remote) | Continuous (on push) |
| Local clones | Developer workstations | On pull |
| Release tags | Git tags marking specific versions | Per release |

### 6.3 Retention

All documents are retained indefinitely in Git history. There is no document destruction policy. Even deleted files remain accessible via `git show <commit>:<path>`.

**Retention justification:** Digital storage cost is negligible. Regulatory best practice favors permanent retention. Git's append-only model makes deletion of history a deliberate and detectable action.

## 7. Periodic Review

### 7.1 Review Cycle

| Document Category | Review Trigger | Reviewer |
|-------------------|----------------|----------|
| `CLAUDE.md` | Any modification to axioms, constants, coding rules, or hard blocks | Planificador |
| Architecture docs | Sprint that modifies module boundaries or pipeline | Observador |
| Regulatory docs | Annual review; any change to intended use or regulatory status | Planificador |
| Sprint docs (active) | Each sprint phase (scope, design, implement, test, review) | Role performing that phase |
| Sprint docs (archived) | No review required (immutable historical records) | N/A |
| `Cargo.toml` / `Cargo.lock` | Any dependency addition, removal, or version change | Verificador (crate approval per HB-2) |

### 7.2 Review Records

Review completion is evidenced by:
- Git commit merging the reviewed change to `main`
- Sprint README.md closure criteria checkboxes (for sprint-related reviews)
- Revision history table update in the reviewed document (for regulatory docs)

## 8. Templates

### 8.1 Regulatory Document Template

All regulatory documents (RD-1 through RD-7) follow this YAML frontmatter template:

```yaml
---
document_id: RD-X.Y
title: Document Title
standard: ISO/IEC reference §clause
version: 1.0
date: YYYY-MM-DD
status: DRAFT
author: Resonance Development Team
---
```

### 8.2 Sprint Document Template

Sprint documents follow the template defined in RD-1.4 §10.3:

```
# {SPRINT_ID}: {Title}
Objetivo: ...
Estado: PENDIENTE | EN PROGRESO | COMPLETADO
Esfuerzo: Bajo | Medio | Alto
Bloqueado por: ...
Desbloquea: ...

## Entregables
## Scope definido
## Criterios de cierre
```

## 9. Codebase References

| Reference | Path | Relevance |
|-----------|------|-----------|
| Project constitution | `CLAUDE.md` | Highest-authority controlled document |
| Architecture | `docs/ARCHITECTURE.md` | Canonical architecture reference |
| Design specs | `docs/design/*.md` | 6 design specifications |
| Module contracts | `docs/arquitectura/*.md` | 4 runtime contracts |
| Sprint backlog | `docs/sprints/README.md` | Active sprint tracking |
| Sprint archive | `docs/sprints/archive/README.md` | 78 archived sprint index |
| Regulatory docs | `docs/regulatory/` | 7 subdirectories (RD-1 through RD-7) |
| Package config | `Cargo.toml` | Dependency declarations |
| Dependency lock | `Cargo.lock` | Exact pinned versions |
| Git history | `.git/` | Complete immutable audit trail |

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. Document registry, lifecycle (creation through obsolescence), change control via Git, storage and retention, periodic review triggers, templates. |
