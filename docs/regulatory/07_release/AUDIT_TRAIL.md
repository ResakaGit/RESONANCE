---
document_id: RD-7.3
title: Audit Trail Procedure
standard: 21 CFR Part 11 §11.10(e)
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Audit Trail Procedure

## 1. Purpose

This document defines the audit trail procedures for RESONANCE, satisfying 21 CFR Part 11 §11.10(e) requirements for secure, computer-generated, time-stamped audit trails that independently record the date and time of operator entries and actions that create, modify, or delete electronic records.

RESONANCE uses Git as its primary audit trail mechanism. Git provides cryptographically-linked, timestamped, attributed, append-only records of all changes to source code, documentation, and configuration. This procedure defines three audit trail types, prescribes extraction commands, establishes a review schedule, and documents known limitations.

**Cross-references:**
- RD-3.4: Configuration Management Plan (Git conventions, branch strategy)
- RD-5.3: Document Control Procedure (controlled document lifecycle)
- RD-5.4: Record Control Procedure (record types and retention)
- RD-7.1: Part 11 Compliance Assessment §3.5 (§11.10(e) compliance status)
- RD-7.2: Data Integrity Policy (ALCOA+ mapping)

## 2. Audit Trail Types

RESONANCE maintains three categories of audit trails, each serving a distinct regulatory purpose.

### 2.1 Type 1: Code Change Audit Trail

| Attribute | Value |
|-----------|-------|
| Scope | All files under Git version control (`src/`, `docs/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `CLAUDE.md`) |
| Mechanism | Git commit history |
| Granularity | Per-commit (one logical change per commit) |
| Attribution | Author name + email per commit |
| Timestamp | Author date (ISO 8601, timezone-aware) |
| Content | Full diff of changes (additions, deletions, modifications) |
| Integrity | SHA-1 hash chain — each commit references parent hash; any modification breaks the chain |
| Retention | Indefinite (Git history is permanent and append-only) |

**What is recorded:**
- Every line of code added, modified, or deleted
- Every document created, updated, or reorganized
- Every dependency change (via `Cargo.toml` and `Cargo.lock`)
- Every configuration change (map files, feature flags)
- The identity and timestamp of the person who made each change
- A human-readable description of why the change was made (commit message)

### 2.2 Type 2: Simulation Record Audit Trail

| Attribute | Value |
|-----------|-------|
| Scope | All simulation executions (interactive, headless, batch, experiment binaries) |
| Mechanism | Deterministic reproducibility: seed + commit + parameters = exact output |
| Granularity | Per-simulation-run |
| Attribution | Commit hash identifies the code version; operator identity is external (not recorded by the binary) |
| Timestamp | Simulation start time (system clock, not embedded in deterministic output) |
| Content | Input parameters (seed, tick count, map, archetype) + output (PPM, CSV, JSON, CLI text) |
| Integrity | Determinism: replaying with identical inputs on the same commit produces bit-identical output |
| Retention | Output files: operator's responsibility. Reproducibility: guaranteed as long as the commit exists in Git |

**What is recorded:**
- Input parameters are either CLI arguments or hardcoded in experiment source (`src/use_cases/experiments/`, `src/bin/`)
- Output is deterministic: the same seed, commit, and parameters always produce the same result
- The audit trail for simulation records is the ability to reproduce, not the preservation of every output file

**Key files:**
- `src/blueprint/equations/determinism.rs` — hash-based RNG (functions: `hash_f32_slice`, `next_u64`, `unit_f32`, `range_f32`, `gaussian_f32`; 23 tests for bit-exactness)
- `src/bin/headless_sim.rs` — headless simulator: `--ticks N --scale S --out file.ppm`
- `src/bin/bozic_validation.rs` — 10-seed Bozic validation
- `src/bin/pathway_inhibitor.rs` — pathway inhibitor experiment

### 2.3 Type 3: Configuration Audit Trail

| Attribute | Value |
|-----------|-------|
| Scope | Build configuration (`Cargo.toml`, `Cargo.lock`), map configurations (`assets/maps/*.ron`), project constitution (`CLAUDE.md`) |
| Mechanism | Git commit history (same as Type 1, but isolated to configuration items) |
| Granularity | Per-commit |
| Attribution | Author name + email per commit |
| Timestamp | Author date (ISO 8601) |
| Content | Exact diff showing what configuration changed and why |
| Integrity | SHA-1 hash chain + `Cargo.lock` SHA-256 checksums for each dependency |
| Retention | Indefinite |

**What is recorded:**
- Dependency additions, upgrades, and removals (visible in `Cargo.toml` diff + `Cargo.lock` diff)
- Map configuration changes (new maps, parameter modifications)
- Coding standard changes (modifications to `CLAUDE.md`)
- Feature flag changes

---

## 3. Audit Trail Properties

### 3.1 Security

| Property | Implementation |
|----------|----------------|
| Tamper evidence | SHA-1 hash chain: modifying any commit changes its hash and all descendant hashes; detectable by `git fsck` or by comparing with any other clone |
| Append-only | New commits are added; existing commits are never modified (force-push to `main` prohibited by development practice) |
| Non-repudiation | Commit author is bound to the commit content via the hash chain (content + metadata = hash input) |
| Distributed verification | Every clone contains the full audit trail; multiple independent copies can be compared to detect tampering |

### 3.2 Completeness

| Property | Implementation |
|----------|----------------|
| No gaps | Git records every committed change; there is no mechanism to skip the audit trail for a committed change |
| Deleted files tracked | `git log --diff-filter=D` shows all file deletions with timestamp, author, and reason |
| Renamed files tracked | `git log --follow <file>` tracks history across renames |
| Uncommitted changes | Not tracked by the audit trail (this is by design: only committed changes are permanent records) |

### 3.3 Independence

| Property | Implementation |
|----------|----------------|
| Computer-generated | Git timestamps are generated by the system, not manually entered |
| Separate from content | Audit trail metadata (author, date, message) is distinct from file content; both are stored in the commit object |
| Cannot be disabled | There is no way to commit to Git without generating an audit trail entry |

---

## 4. Extraction Commands

The following commands extract audit trail data for inspection, review, or regulatory submission. All commands are run from the repository root.

### 4.1 Complete Audit Trail

```bash
# Full commit history with hash, author, date, and message
git log --format="%H | %ai | %an <%ae> | %s"
```

**Example output:**
```
971c7acb99decde45bf28860e6e10372718c51e2 | 2026-04-02 ... | Dev Name <dev@example.com> | docs: honesty pass — 5 known limitations added to Rosie case (README + paper)
d9b60fa... | 2026-04-01 ... | Dev Name <dev@example.com> | paper: add Experiment 7 (Rosie canine mast cell) + London references
```

### 4.2 Audit Trail for a Specific File

```bash
# Complete history of a single file
git log --follow --format="%H | %ai | %an | %s" -- src/blueprint/equations/derived_thresholds.rs
```

### 4.3 Per-Line Attribution

```bash
# Who last modified each line of a file
git blame src/blueprint/equations/derived_thresholds.rs
```

### 4.4 Changes Between Two Points in Time

```bash
# All changes between two commits
git log --format="%H | %ai | %an | %s" <commit1>..<commit2>

# Detailed diff between two commits
git diff <commit1>..<commit2>

# Summary statistics
git diff --stat <commit1>..<commit2>
```

### 4.5 Changes by Date Range

```bash
# All commits in a date range
git log --format="%H | %ai | %an | %s" --after="2026-01-01" --before="2026-04-02"
```

### 4.6 Changes by Author

```bash
# All commits by a specific author
git log --format="%H | %ai | %an | %s" --author="Author Name"
```

### 4.7 Changes to Dependencies

```bash
# All commits that modified Cargo.lock (dependency changes)
git log --format="%H | %ai | %an | %s" -- Cargo.lock

# Diff of dependency changes between two commits
git diff <commit1>..<commit2> -- Cargo.lock
```

### 4.8 Deleted Files

```bash
# All file deletions with timestamp and reason
git log --diff-filter=D --summary --format="%H | %ai | %an | %s"
```

### 4.9 Search for Specific Code Changes

```bash
# Find when a specific string was added or removed (pickaxe search)
git log -S "DISSIPATION_SOLID" --format="%H | %ai | %an | %s"

# Find commits whose messages match a pattern
git log --grep="axiom" --format="%H | %ai | %an | %s"
```

### 4.10 Repository Integrity Verification

```bash
# Verify object database integrity
git fsck --full

# Compare local and remote refs
git fetch origin && git log origin/main..main --format="%H %s"
git log main..origin/main --format="%H %s"
```

### 4.11 Configuration Snapshot at a Specific Commit

```bash
# View Cargo.toml at a specific historical commit
git show <commit>:Cargo.toml

# View Cargo.lock at a specific historical commit
git show <commit>:Cargo.lock

# View full file tree at a specific commit
git ls-tree -r --name-only <commit>
```

### 4.12 Simulation Reproducibility Verification

```bash
# Reproduce a simulation run from a specific commit
git checkout <commit>
cargo run --release --bin headless_sim -- --ticks 10000 --scale 8 --out reproduced.ppm

# Reproduce Bozic validation
git checkout <commit>
cargo run --release --bin bozic_validation

# Return to current HEAD
git checkout main
```

---

## 5. Review Procedure

### 5.1 Quarterly Audit Trail Review

| Attribute | Value |
|-----------|-------|
| Frequency | Quarterly (Q1: January, Q2: April, Q3: July, Q4: October) |
| Owner | Verificador role |
| Scope | All commits in the preceding quarter |
| Duration | 1-2 hours (estimated for current repository size) |

**Review checklist:**

| # | Check | Command | Expected Result |
|---|-------|---------|-----------------|
| 1 | Repository integrity | `git fsck --full` | No errors, no dangling objects |
| 2 | No force-pushes detected | Compare commit count: `git rev-list --count main` vs. previous quarter's count | Count is monotonically increasing |
| 3 | All commits attributed | `git log --format="%an" \| sort -u` | All authors are known team members |
| 4 | Commit messages are meaningful | `git log --format="%s" --after="<quarter_start>"` | All messages have type prefix; no empty messages |
| 5 | Test suite passes | `cargo test` | 0 failures |
| 6 | No unauthorized dependency changes | `git log --format="%H %s" -- Cargo.lock --after="<quarter_start>"` | All dependency changes have corresponding commit messages explaining the change |
| 7 | Configuration items current | Verify `Cargo.toml` version matches documented version | Version in `Cargo.toml` matches RD-3.3 SBOM and RD-3.4 CM Plan |
| 8 | Determinism verified | Run determinism tests: `cargo test --lib determinism` | All determinism tests pass |

### 5.2 Review Output

Each quarterly review produces a record consisting of:
1. Date of review
2. Reviewer identity
3. Quarter covered
4. Checklist results (pass/fail for each item)
5. Anomalies detected (if any)
6. Corrective actions taken (if any)

This record is committed to the repository as evidence of the review. Location: sprint documentation or a dedicated audit log (appended to this document or committed as a separate record).

### 5.3 Event-Triggered Reviews

In addition to quarterly reviews, an audit trail review is triggered by:

| Trigger | Scope | Owner |
|---------|-------|-------|
| Major version release (x.0.0) | Full audit trail since last major release | Verificador |
| Security advisory (RUSTSEC) | Dependency audit trail + `cargo audit` | Development team |
| Reported anomaly (test failure, nondeterminism) | Targeted investigation of affected commits | Development team |
| External audit request | Full audit trail extraction per §4 | Verificador |

---

## 6. Anomaly Detection

### 6.1 Anomaly Types

| Anomaly | Detection Method | Severity | Response |
|---------|-----------------|----------|----------|
| Force-push detected (history rewrite) | Commit count decreased; `git reflog` shows reset | Critical | Investigate immediately; restore from backup clone; file CAPA (RD-5.7) |
| Unknown author in commit log | `git log --format="%an" \| sort -u` shows unrecognized name | High | Verify identity; if unauthorized, investigate and revoke access |
| Commit without test execution | Sprint closure criteria not met; test count decreased | Medium | Rerun tests; file nonconformance (RD-5.6) if tests fail |
| Dependency change without justification | `Cargo.lock` modified but commit message does not explain dependency change | Low | Document justification retroactively; update commit message conventions |
| `git fsck` reports errors | `git fsck --full` outputs warnings or errors | High | Investigate corruption source; restore from verified backup clone |
| Simulation nondeterminism | Same seed + commit produces different output | Critical | Investigate determinism.rs; check for platform-dependent floating point behavior; file CAPA |

### 6.2 Escalation Path

```
Anomaly detected
    │
    ▼
Low severity ──── Document in quarterly review record
    │
Medium severity ── File nonconformance per RD-5.6
    │
High severity ──── File CAPA per RD-5.7; notify development team lead
    │
Critical severity ─ Immediate investigation; halt releases until resolved; file CAPA
```

---

## 7. Known Limitations

### 7.1 Git Author Identity

Git commit author identity is self-asserted via `git config user.name` and `git config user.email`. An operator could configure a false identity. Mitigation: GitHub account binding provides a second layer of identity; GPG-signed commits (planned, see RD-7.1 Gap G-04) would provide cryptographic identity verification.

### 7.2 Author Date Override

Git allows the `--date` flag to override the author date on a commit. An operator could backdate or forward-date a commit. Mitigation: GitHub records a separate server-side timestamp (the push timestamp in the GitHub API) that cannot be forged by the committer. For forensic purposes, the GitHub push event timestamp takes precedence.

### 7.3 Local Uncommitted Changes

The audit trail only records committed changes. Work-in-progress changes in the working tree or staging area are not part of the audit trail until committed. This is by design — only finalized changes are permanent records.

### 7.4 Simulation Output Retention

The audit trail does not automatically preserve simulation output files (PPM, CSV, JSON). Simulation reproducibility is guaranteed by determinism (seed + commit = exact output), but the operator must explicitly preserve output files if needed. Output files generated during experiments or validation runs should be committed or archived separately if they constitute evidence.

### 7.5 Granularity

Git operates at file-commit granularity, not at real-time keystroke granularity. Changes between commits are not recorded. A developer could make multiple logical changes in a single commit, reducing audit trail granularity. Mitigation: development practice mandates "one logical change per commit" (RD-3.4 §3.3).

### 7.6 SHA-1 Collision Risk

Git currently uses SHA-1 for object hashing. Theoretical SHA-1 collision attacks exist (SHAttered, 2017). Practical risk for RESONANCE is negligible: (1) collisions require deliberate effort with ~2^63 operations, (2) Git has hardened SHA-1 detection (SHA-1DC), (3) migration to SHA-256 is underway in Git. No action required at this time.

---

## 8. Audit Trail Retention

| Record Type | Retention Period | Mechanism | Disposal |
|-------------|-----------------|-----------|----------|
| Git commit history | Indefinite | Git object database (append-only) | Never — history is permanent |
| GitHub hosting | Duration of GitHub account + 12 months after account closure (GitHub ToS) | GitHub infrastructure | Mitigated by local clones |
| Local clones | Developer discretion | Local filesystem | Developers should retain at least one clone indefinitely |
| Zenodo deposit | Indefinite (CERN-hosted; institutional commitment to long-term preservation) | Zenodo infrastructure | Not applicable |

---

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial procedure |
