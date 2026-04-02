---
document_id: RD-7.5
title: Release Package Definition
standard: IEC 62304:2006+Amd1:2015 §5.8
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
---

# Release Package Definition

## 1. Purpose

This document defines the release package for RESONANCE, specifying the criteria, artifacts, procedures, and post-release activities required for each software release. It satisfies IEC 62304:2006+Amd1:2015 §5.8 (Software release) and §6.3.2 (Release verification).

For RESONANCE (IEC 62304 Class A), §5.8 requires:
- Definition of what constitutes a release
- Verification that release criteria are met
- Documentation of the release package contents
- Procedures for making the release available

**Cross-references:**
- RD-1.2: Software Safety Classification (Class A)
- RD-1.4: Software Development Plan (lifecycle model, sprint process)
- RD-2.6: Risk Management Report (overall risk acceptability)
- RD-3.3: SBOM (dependency inventory)
- RD-3.4: Configuration Management Plan (versioning, Cargo.lock pinning)
- RD-4.3: Verification Report (test results)
- RD-4.4: Validation Report (experiment validation)
- RD-5.3: Document Control Procedure (document status tracking)
- RD-7.1: Part 11 Compliance Assessment (electronic records controls)
- RD-7.3: Audit Trail Procedure (change tracking)
- RD-7.4: Cybersecurity Plan (vulnerability status)

## 2. Release Types

### 2.1 Release Classification

| Release Type | Version Bump | Scope | Examples |
|--------------|-------------|-------|---------|
| **Patch** | 0.1.x | Bug fixes, documentation updates, test additions. No API or behavior changes. | Fix floating-point edge case in `derived_thresholds.rs`; add missing test; correct documentation error |
| **Minor** | 0.x.0 | New features, new drug models, new calibration profiles, new emergence systems. Backward-compatible. | Add new pathway inhibitor mode; add clinical calibration profile; register new emergence system |
| **Major** | x.0.0 | Breaking changes to axioms, fundamental constants, layer definitions, or batch simulator API. | Modify fundamental constant value; add L14+ layer; change batch simulator API |

**Current version:** 0.1.0 (pre-1.0). Per semver convention, any minor version bump may include breaking changes during pre-1.0 development (RD-3.4 §4.1).

### 2.2 Release Identification

Each release is identified by:

| Identifier | Format | Example |
|------------|--------|---------|
| Semantic version | `MAJOR.MINOR.PATCH` | `0.2.0` |
| Git tag | `v{MAJOR}.{MINOR}.{PATCH}` | `v0.2.0` |
| Git commit hash | 40-character SHA-1 | `971c7acb99decde45bf28860e6e10372718c51e2` |
| `Cargo.toml` version | `version = "{MAJOR}.{MINOR}.{PATCH}"` | `version = "0.2.0"` |

**Note:** As of commit `971c7ac`, Git tags are not yet used for releases (RD-3.4 §3.3 identifies this as a gap). This release package definition formalizes the tagging process.

---

## 3. Release Criteria

All criteria must be satisfied before a release is authorized. The release is blocked if any criterion fails.

### 3.1 Mandatory Release Criteria

| ID | Criterion | Verification Method | Acceptance Threshold |
|----|-----------|--------------------|--------------------|
| RC-01 | All tests pass | `cargo test` | 0 failures, 0 errors |
| RC-02 | No compiler warnings | `cargo check 2>&1` | 0 warnings |
| RC-03 | No known vulnerabilities | `cargo audit` | 0 advisories with severity >= Medium that affect runtime dependencies |
| RC-04 | All active sprint closures green | Sprint README closure criteria verified | All checkboxes checked |
| RC-05 | Regulatory documentation complete | All RD-1 through RD-7 documents present and current | Status = DRAFT or APPROVED for all documents |
| RC-06 | Risk management signed off | RD-2.6 Risk Management Report conclusion: overall risk acceptable | "Acceptable" or "ALARP with justification" |
| RC-07 | Clinical evaluation current | RD-6.1/RD-6.2 Clinical Evaluation reflects current capabilities | Clinical evaluation date within 6 months of release |
| RC-08 | Version number updated | `Cargo.toml` `version` field reflects the release version | Version matches intended release |
| RC-09 | SBOM current | RD-3.3 SBOM matches current `Cargo.lock` | All dependency versions and checksums match |
| RC-10 | Determinism verified | Determinism test suite passes | `cargo test --lib determinism` — 0 failures |
| RC-11 | Bozic validation confirmed | 10-seed validation produces expected result | `cargo run --release --bin bozic_validation` — combo > mono 10/10 seeds |
| RC-12 | Changelog complete | Release notes document all changes since previous release | See §7 (release notes template) |

### 3.2 Conditional Release Criteria

These criteria apply only to specific release types.

| ID | Criterion | Applies To | Verification Method |
|----|-----------|-----------|--------------------|
| RC-13 | Backward compatibility verified | Minor releases | No existing test changes required (new tests added only) |
| RC-14 | Migration guide provided | Major releases | Release notes §Migration section documents all breaking changes |
| RC-15 | Benchmark regression check | Performance-sensitive releases | `cargo bench --bench batch_benchmark` — no >10% regression |
| RC-16 | New SOUP assessed | Releases adding dependencies | RD-3.2 SOUP Analysis updated for new dependency |

---

## 4. Release Procedure

### 4.1 Pre-Release Checklist

The following steps are executed sequentially by the release owner (Verificador role).

```
Step 1: Verify all sprint closures are green
        └── Check docs/sprints/README.md — all active sprints closed or deferred

Step 2: Run full test suite
        └── cargo test
        └── Verify: 0 failures, all tests accounted for

Step 3: Run compiler check
        └── cargo check 2>&1 | grep -c "warning"
        └── Verify: 0 warnings

Step 4: Run vulnerability audit
        └── cargo audit
        └── Verify: 0 medium/high/critical advisories on runtime deps

Step 5: Run Bozic validation
        └── cargo run --release --bin bozic_validation
        └── Verify: combo > mono confirmed 10/10 seeds

Step 6: Run determinism verification
        └── cargo test --lib determinism
        └── Verify: 0 failures

Step 7: Verify SBOM accuracy
        └── Compare Cargo.lock checksums with RD-3.3 entries
        └── Verify: all match

Step 8: Verify regulatory documentation
        └── Confirm all RD-1 through RD-7 documents exist and are current

Step 9: Update version number
        └── Edit Cargo.toml: version = "X.Y.Z"
        └── Commit: "chore: bump version to X.Y.Z"

Step 10: Write release notes
         └── Per template in §7

Step 11: Create Git tag
         └── git tag -a vX.Y.Z -m "Release X.Y.Z: <summary>"

Step 12: Push tag and commit
         └── git push origin main --tags
```

### 4.2 Release Authorization

| Step | Actor | Action |
|------|-------|--------|
| 1 | Verificador | Completes pre-release checklist (§4.1) |
| 2 | Verificador | Documents checklist results in release notes |
| 3 | Verificador | Creates Git tag (serves as release authorization) |
| 4 | Development team | Pushes tag to GitHub |
| 5 | Development team | Creates GitHub Release with release notes |

---

## 5. Release Artifacts

### 5.1 Artifact Inventory

| Artifact | Format | Location | Mandatory |
|----------|--------|----------|-----------|
| Source code snapshot | Git tag + SHA | `git tag vX.Y.Z` → GitHub Releases | Yes |
| Source archive | `.tar.gz` / `.zip` | GitHub Releases (auto-generated from tag) | Yes |
| Release notes | Markdown | GitHub Release body | Yes |
| Test results | Text (CLI output) | Included in release notes or attached | Yes |
| SBOM | Markdown (RD-3.3) | `docs/regulatory/03_traceability/SBOM.md` at tagged commit | Yes |
| Regulatory documentation snapshot | Markdown (all RD-*) | `docs/regulatory/` at tagged commit | Yes |
| Cargo.lock | TOML | Repository root at tagged commit | Yes |
| Release binaries | Platform-specific executables | GitHub Releases (attached) | Optional |
| Benchmark results | HTML (Criterion) | Attached to release if performance-relevant | Optional |
| Bozic validation output | Text (CLI output) | Included in release notes or attached | Yes (for minor/major) |

### 5.2 Binary Build Procedure

Release binaries are optional artifacts. When provided, they are built as follows:

```bash
# Build release binaries
cargo build --release

# Key binaries (from Cargo.toml [[bin]] sections)
# Located in target/release/ after build:
#   resonance              (main simulation with rendering)
#   headless_sim           (headless simulation → PPM image)
#   bozic_validation       (10-seed Bozic validation)
#   cancer_therapy         (cytotoxic drug experiment)
#   pathway_inhibitor      (pathway inhibitor experiment)
#   adaptive_therapy       (adaptive therapy experiment)
#   lab                    (interactive simulation lab)
#   survival               (survival analysis)
#   + additional experiment binaries
```

**Platform matrix:**

| Platform | Architecture | Build Environment | Status |
|----------|-------------|-------------------|--------|
| macOS | aarch64 (Apple Silicon) | Primary development platform | Verified |
| macOS | x86_64 | Cross-compilation or CI | Planned |
| Linux | x86_64 | CI or dedicated build | Planned |
| Windows | x86_64 | CI or dedicated build | Planned |

**Note:** Cross-platform binary distribution is not yet established. Current releases are source-only. Users build from source using `cargo build --release`.

### 5.3 Reproducibility Guarantee

Any release can be reproduced from its Git tag:

```bash
# Clone at exact release point
git clone --branch vX.Y.Z https://github.com/ResakaGit/RESONANCE.git
cd RESONANCE

# Build (Cargo.lock ensures identical dependency resolution)
cargo build --release

# Run tests (should produce identical results)
cargo test

# Reproduce simulation output (deterministic)
cargo run --release --bin headless_sim -- --ticks 10000 --scale 8 --out output.ppm
```

The combination of Git tag (exact source), `Cargo.lock` (exact dependencies), and deterministic RNG (`src/blueprint/equations/determinism.rs`) ensures that any release can be reproduced on any compatible platform.

---

## 6. Version Numbering

### 6.1 Semantic Versioning Policy

RESONANCE follows Semantic Versioning 2.0.0 (semver.org):

```
MAJOR.MINOR.PATCH

MAJOR: Breaking changes to axioms, constants, layers, or public API
MINOR: New features, backward-compatible
PATCH: Bug fixes, documentation, tests
```

### 6.2 Version Lifecycle

```
0.1.0 (current)
  │
  ├── 0.1.1 (patch: bug fix)
  ├── 0.1.2 (patch: documentation)
  │
  ├── 0.2.0 (minor: new emergence system registered)
  │    ├── 0.2.1 (patch)
  │
  ├── 0.3.0 (minor: new drug model)
  │
  └── 1.0.0 (major: stable API, axioms frozen, public commitment)
       ├── 1.0.1 (patch)
       ├── 1.1.0 (minor: new calibration profile)
       └── 2.0.0 (major: L14+ layer, axiom revision)
```

### 6.3 Pre-1.0 Convention

During pre-1.0 development (current state), minor versions may include breaking changes per semver convention. The project will declare 1.0.0 when:
- All 8 axioms are fully implemented and tested
- All 14 layers are stable (no planned structural changes)
- Batch simulator API is stable
- At least 3 drug experiments validated against published data
- Regulatory documentation track (RD-1 through RD-7) is complete

### 6.4 Version Source of Truth

The authoritative version is the `version` field in `Cargo.toml` line 3. All other references (documentation, SBOM, release notes) must match this value.

---

## 7. Release Notes Template

```markdown
# RESONANCE vX.Y.Z — Release Notes

**Date:** YYYY-MM-DD
**Commit:** <full SHA>
**Previous version:** vA.B.C

## Summary

<1-3 sentence summary of the release purpose>

## Changes

### New Features
- <feature description> (`src/path/to/file.rs`)

### Bug Fixes
- <fix description> (`src/path/to/file.rs`)

### Documentation
- <doc change description>

### Dependencies
- <dependency change: added/upgraded/removed, with version numbers>

## Release Criteria Verification

| Criterion | Result |
|-----------|--------|
| cargo test | X tests, 0 failures, Xs elapsed |
| cargo check | 0 warnings |
| cargo audit | 0 advisories |
| Sprint closures | All green |
| Regulatory docs | Complete (RD-1 through RD-7) |
| Risk management | Acceptable (RD-2.6) |
| Clinical evaluation | Current (RD-6.2) |
| SBOM | Matches Cargo.lock |
| Determinism | Verified (23 tests pass) |
| Bozic validation | 10/10 seeds confirm combo > mono |

## Known Limitations

<List any known limitations specific to this release — cross-ref RD-6.3>

## Migration Guide (major releases only)

<Breaking changes and migration instructions>

## Checksums

| Artifact | SHA-256 |
|----------|---------|
| Source archive (.tar.gz) | <hash> |
| resonance (macOS aarch64) | <hash> |
| headless_sim (macOS aarch64) | <hash> |
```

---

## 8. Post-Release Monitoring

### 8.1 Monitoring Channels

| Channel | Purpose | Frequency |
|---------|---------|-----------|
| GitHub Issues | Bug reports, feature requests, usage questions from the community | Continuous (monitored by development team) |
| RUSTSEC Advisories | Dependency vulnerability notifications | Continuous (automated via `cargo audit`) |
| Zenodo record | Citation tracking, download metrics | Quarterly review |
| Cargo test (regression) | Verify release integrity on new platforms or Rust versions | Per Rust toolchain update |

### 8.2 Post-Release Defect Handling

| Severity | Definition | Response Time | Action |
|----------|-----------|---------------|--------|
| Critical | Axiom violation, conservation bug, determinism broken | 48 hours | Immediate patch release; file CAPA (RD-5.7) |
| High | Incorrect simulation output for documented experiment | 7 days | Patch release; update affected validation results |
| Medium | Non-critical bug (UI, performance, documentation error) | 30 days | Schedule fix in next minor release |
| Low | Cosmetic, enhancement request | Next minor release | Triage and prioritize |

### 8.3 Post-Release Review

Within 30 days of each minor or major release:

| Activity | Owner | Output |
|----------|-------|--------|
| Review GitHub Issues filed since release | Development team | Issue triage and priority assignment |
| Run `cargo audit` against current RUSTSEC database | Development team | Vulnerability status report |
| Verify Bozic validation still passes on current Rust stable | Development team | Regression check |
| Update SBOM if dependencies have published new advisories | Development team | Updated RD-3.3 |
| Assess whether clinical evaluation needs updating | Development team | Decision: update RD-6.2 or defer |

---

## 9. Rollback Procedure

### 9.1 When to Rollback

A rollback is warranted when:
- A released version contains a critical defect (axiom violation, conservation bug, determinism failure)
- A dependency vulnerability is discovered that cannot be patched forward quickly
- A release was made prematurely (release criteria not actually met)

### 9.2 Rollback Steps

```
Step 1: Identify the last known-good release
        └── git tag -l   (list all tags)
        └── Identify vA.B.C (the previous release)

Step 2: Communicate the rollback
        └── GitHub Issue documenting the defect and rollback decision

Step 3: Create a revert release
        └── git checkout vA.B.C
        └── Verify: cargo test (0 failures)
        └── Verify: cargo run --release --bin bozic_validation (10/10)
        └── Tag: git tag -a vX.Y.Z+1 -m "Rollback to vA.B.C due to <defect>"
        └── Push: git push origin main --tags

Step 4: Create GitHub Release for the revert
        └── Document the defect and rollback in release notes

Step 5: File CAPA (RD-5.7)
        └── Root cause analysis of why the defective release passed criteria
        └── Corrective action to prevent recurrence

Step 6: Forward fix
        └── Fix the defect on main
        └── Create new release (vX.Y.Z+2) with fix
        └── Verify all release criteria (§3)
```

### 9.3 Rollback Limitations

- Git tags are immutable once pushed; a rolled-back release's tag remains in history (with a superseding tag documented in release notes)
- Users who have already cloned or built the defective release must be notified via GitHub Issue
- Binary artifacts (if distributed) cannot be recalled from users who have already downloaded them; the GitHub Release is marked as pre-release or deleted, with a notification pointing to the replacement

---

## 10. Release History

| Version | Date | Commit | Type | Summary |
|---------|------|--------|------|---------|
| 0.1.0 | 2025-03-25 | `42834c2` (initial) through `971c7ac` (current) | Initial development | Pre-release development phase; no formal tagged release yet |

**Note:** As of this document's creation, RESONANCE has not yet made a formal tagged release. The first formal release will follow this procedure and produce the first entry in the release history table.

---

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial release package definition |
