---
document_id: RD-5.5
title: Internal Audit Procedure
standard: ISO 13485:2016 §8.2.4
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Internal Audit Procedure

## 1. Purpose

This procedure defines the process for conducting internal audits of the RESONANCE Quality Management System, in accordance with ISO 13485:2016 §8.2.4. Internal audits verify that the QMS conforms to planned arrangements (ISO 13485, IEC 62304 Class A) and to the project's own requirements (8 axioms, coding rules, hard blocks, sprint closure criteria).

## 2. Scope

Internal audits cover all QMS processes and their outputs:
- Sprint lifecycle compliance (scope, design, implementation, testing, review, closure)
- Coding standards conformance (`CLAUDE.md` rules and hard blocks)
- Axiom compliance (8 axioms inviolable)
- Test suite integrity (pass count, zero failures, zero warnings)
- Document control (controlled documents current and accessible)
- CAPA effectiveness (corrective actions verified by regression tests)
- Record completeness (sprint docs, closure criteria, grep evidence)

## 3. Audit Frequency and Triggers

### 3.1 Scheduled Audits

| Trigger | Frequency | Scope |
|---------|-----------|-------|
| **Sprint track closure** | At the completion of each sprint track (group of related sprints) | Full checklist (§5) against the track's deliverables |
| **Regulatory sprint closure** | At the completion of each RD-sprint (RD-1 through RD-7) | Regulatory document completeness and cross-reference integrity |
| **Annual review** | Every 12 months from initial QMS establishment | Full QMS scope per §2 |

### 3.2 Unscheduled Audits

Unscheduled audits are triggered by:
- Detection of a nonconforming product (RD-5.6) that suggests a systemic process failure
- CAPA investigation (RD-5.7) that identifies a recurring defect pattern
- Change to intended use (RD-1.1) or safety classification (RD-1.2)
- External inquiry from a regulatory body, journal reviewer, or pharma partner

## 4. Audit Method: Grep-Based Verification

### 4.1 Methodology

RESONANCE internal audits use **grep-based verification** as the primary audit method. This is a deterministic, reproducible, and automated approach where specific code patterns are searched across the codebase to verify conformance. Each closure criterion maps to one or more grep commands whose output constitutes the audit evidence.

This method was established during the DECOUPLING_AUDIT sprint track and has been validated across 3 completed sprints (DC-1, DC-3, DC-4).

### 4.2 Grep Verification Protocol

For each audit criterion:

1. **Define the grep command** that would detect a violation. Example: `grep -r "use crate::layers::" src/blueprint/equations/` --- should return 0 results if equations have no layer imports.
2. **Execute the command** against the current codebase state.
3. **Record the result** (match count) in the sprint README.md or audit record.
4. **Determine pass/fail**: 0 matches = PASS (no violations found); >0 matches = FAIL (violations detected).

### 4.3 Automated Verification Commands

The following grep commands constitute the standing audit checklist. They can be executed as a batch to verify overall QMS conformance.

| Criterion | Command | Expected Result |
|-----------|---------|-----------------|
| No `unsafe` code (HB-1) | `grep -r "unsafe " src/ --include="*.rs"` | 0 matches |
| No `async`/`await` (HB-3) | `grep -r "async fn\|\.await" src/ --include="*.rs"` | 0 matches |
| No `Arc<Mutex` (HB-4) | `grep -r "Arc<Mutex" src/ --include="*.rs"` | 0 matches |
| No `static mut` (HB-5) | `grep -r "static mut" src/ --include="*.rs"` | 0 matches |
| No `lazy_static.*Mutex` (HB-5) | `grep -r "lazy_static.*Mutex" src/ --include="*.rs"` | 0 matches |
| Equations purity: no layer imports | `grep -r "use crate::layers::" src/blueprint/equations/ --include="*.rs"` | 0 matches |
| Rendering decoupled from simulation | `grep -r "use crate::simulation" src/rendering/ --include="*.rs"` | 0 matches |
| All tests pass | `cargo test 2>&1 \| tail -1` | "test result: ok" |
| Zero compiler warnings | `cargo check 2>&1 \| grep -c "warning"` | 0 |
| Zero clippy warnings | `cargo clippy 2>&1 \| grep -c "warning"` | 0 |

## 5. Audit Checklist

### 5.1 Sprint Track Closure Audit Checklist

This checklist is executed at every sprint track closure. Each item must be verified before the track is archived.

| # | Category | Criterion | Verification Method | Pass Criteria |
|---|----------|-----------|--------------------|----|
| A1 | **Axiom Compliance** | No axiom violation detected in sprint scope | Verificador review verdict (no BLOCK for axiom conflict) | 0 axiom-related BLOCKs |
| A2 | **Axiom Compliance** | Derived constants unchanged unless justified | `git diff HEAD~N -- src/blueprint/equations/derived_thresholds.rs` | No unexplained changes |
| T1 | **Test Integrity** | `cargo test` passes | Execute `cargo test` | 0 failures |
| T2 | **Test Integrity** | Test count non-regressing | Compare test count to previous sprint closure | count >= previous |
| T3 | **Test Integrity** | New features have co-located tests | `grep -c "#\[test\]" <new-files>` | >0 for each new module |
| W1 | **Warnings** | Zero compiler warnings | `cargo check` | 0 warnings |
| W2 | **Warnings** | Zero clippy warnings | `cargo clippy` | 0 warnings |
| H1 | **Hard Blocks** | No `unsafe` | grep (§4.3) | 0 matches |
| H2 | **Hard Blocks** | No unapproved crates | `diff <(git show HEAD:Cargo.toml) <(git show HEAD~N:Cargo.toml)` | No new `[dependencies]` without justification |
| H3 | **Hard Blocks** | No async/await | grep (§4.3) | 0 matches |
| H4 | **Hard Blocks** | No shared mutable state | grep (§4.3) | 0 matches for Arc<Mutex>, static mut, lazy_static.*Mutex |
| D1 | **Documentation** | Sprint README.md exists with scope and criteria | File existence check | File present |
| D2 | **Documentation** | All closure criteria checkboxes checked | Read sprint README.md | All `[x]` |
| D3 | **Documentation** | Sprint-specific grep criteria pass | Execute each grep in closure criteria section | All criteria met |
| D4 | **DOD** | No `// DEBT:` introduced without justification | `grep -r "// DEBT:" <changed-files>` | 0 new unjustified DEBT |

### 5.2 Annual QMS Audit Checklist

This extended checklist is executed annually in addition to the sprint closure checklist.

| # | Category | Criterion | Verification Method |
|---|----------|-----------|---------------------|
| Q1 | **QMS Documents** | Quality Manual (RD-5.1) current and accessible | Read file, verify status != OBSOLETE |
| Q2 | **QMS Documents** | Quality Policy (RD-5.2) reviewed within 12 months | Check revision history date |
| Q3 | **QMS Documents** | All 6 QMS procedures exist | List `docs/regulatory/05_quality_system/` |
| Q4 | **Regulatory** | Intended Use (RD-1.1) reflects current product scope | Compare RD-1.1 to README.md and CLAUDE.md |
| Q5 | **Regulatory** | No SaMD reclassification trigger hit | Review against RD-1.5 §7.2 triggers |
| Q6 | **Records** | Sprint archive index (`docs/sprints/archive/README.md`) current | Compare to `ls docs/sprints/archive/` |
| Q7 | **CAPA** | All open CAPAs have documented status | Review active sprint docs for unresolved "Problema" sections |
| Q8 | **Configuration** | `Cargo.lock` committed and current | `git status Cargo.lock` |
| Q9 | **Objectives** | Quality objectives (QO-1 through QO-6) met | Review per RD-5.2 §4 |

## 6. Audit Execution

### 6.1 Audit Roles

| Role | Responsibility |
|------|----------------|
| **Auditor** | Executes the checklist, records findings. Must not audit their own work. |
| **Auditee** | Provides access to records and answers questions. Typically the Alquimista who performed the sprint work. |

In the current project structure, the Verificador role serves as the internal auditor for sprint closures. The Planificador role serves as auditor for annual QMS reviews.

**Independence requirement:** The auditor must not have been the primary author of the work being audited. For a single-contributor project, the sprint closure grep-based verification provides objective evidence independent of judgment. When the project has multiple contributors, formal auditor independence must be maintained.

### 6.2 Audit Procedure

| Step | Action | Record |
|------|--------|--------|
| 1 | Identify audit scope (sprint track or annual) | Audit scope noted in sprint README or separate audit record |
| 2 | Select applicable checklist (§5.1 or §5.2) | Checklist items documented |
| 3 | Execute each checklist item | Command output or review result recorded |
| 4 | Document findings: PASS / FAIL / OBSERVATION per item | Sprint README checkboxes or audit record |
| 5 | For FAIL items: create nonconformance per RD-5.6 | Reference to NC record |
| 6 | For OBSERVATION items: note as improvement opportunity | No NC required; may trigger preventive CAPA |
| 7 | Summarize audit result: PASS (all items pass) or FAIL (any item fails) | Summary in sprint README or audit record |
| 8 | Archive audit evidence (commit to Git) | Git commit |

### 6.3 Audit Findings Classification

| Classification | Definition | Required Action |
|----------------|------------|-----------------|
| **PASS** | Criterion met; evidence documented | None |
| **FAIL** | Criterion not met; violation detected | Nonconformance per RD-5.6; CAPA per RD-5.7 if systemic |
| **OBSERVATION** | No violation, but improvement opportunity identified | Optional preventive action; document for future consideration |

## 7. Audit Records

### 7.1 Record Content

Each audit record must contain:
- Audit date
- Audit scope (sprint track ID or "annual QMS")
- Auditor identity
- Checklist used (§5.1 or §5.2)
- Result per item (PASS / FAIL / OBSERVATION)
- Evidence per item (grep output, test count, file reference)
- Summary verdict
- References to any nonconformances or CAPAs opened

### 7.2 Record Storage

Audit records are stored per RD-5.4 (Record Control):
- **Sprint track closure audits:** Recorded as closure criteria checkboxes in the sprint track README.md, then archived to `docs/sprints/archive/{TRACK}/README.md`.
- **Annual QMS audits:** Recorded in a dedicated audit record document in `docs/regulatory/05_quality_system/` or appended to this procedure's revision history.

### 7.3 Record Retention

Audit records are retained indefinitely in Git history per RD-5.4 §7.

## 8. Real Example: DECOUPLING_AUDIT Closure

The DECOUPLING_AUDIT sprint track provides a concrete example of this internal audit procedure in practice.

### 8.1 Context

The DECOUPLING_AUDIT track comprised 5 sprints (DC-1 through DC-5) addressing architectural coupling violations. 3 sprints (DC-1, DC-3, DC-4) were completed; 2 (DC-2, DC-5) remain pending.

### 8.2 Audit Evidence (from `docs/sprints/archive/DECOUPLING_AUDIT/README.md`)

| Criterion | Grep Command | Result | Status |
|-----------|-------------|--------|--------|
| Equations purity | `grep -r "use crate::layers::" src/blueprint/equations/` | 0 results (was 41) | PASS |
| Inline math eliminated | Inline math count in systems | 0 (was 8) | PASS |
| Worldgen state decoupled | Worldgen controlling sim state | 0 transitions (was 2) | PASS |
| Rendering decoupled | `grep "use crate::simulation" src/rendering/quantized_color/systems.rs` | 0 results (was 1) | PASS |
| terrain_blocks_vision moved | `grep "terrain_blocks_vision" src/simulation/thermodynamic/physics.rs` | 0 results | PASS |
| Hardcoded dt fixed | Hardcoded 0.016 dt occurrences | 0 (was 1) | PASS |
| Dead code removed | Orphaned spawns, unused functions | 0 items (was 7) | PASS |
| Test suite | `cargo test` | 3,113 passed, 0 failed | PASS |
| Warnings | `cargo check` | 0 warnings (was 4) | PASS |

### 8.3 Post-Audit Corrective Actions

During the DECOUPLING_AUDIT, several issues were discovered and fixed as inline corrections:

| Finding | Corrective Action | Preventive Action |
|---------|-------------------|-------------------|
| Hardcoded `0.016` dt in `epigenetic_adaptation.rs` | Changed to `time.delta_secs()` | Added grep criterion to prevent recurrence |
| `COMMENSALISM_INTAKE` hardcoded | Derived from `DISSIPATION_SOLID` | Centralized in axiom-derived constants |
| Dead function `dimension_base_frequency` | Removed | Sprint closure now checks for unused functions |
| 4 cargo warnings | Fixed (unused imports, dead code) | Zero-warning closure criterion mandatory |

This demonstrates the audit-to-CAPA loop: audit finding triggers corrective action, which is verified by grep, and a preventive measure (new closure criterion) prevents recurrence.

## 9. Audit Independence

### 9.1 Current State

In the current project structure (primary single-contributor with AI-assisted development), full auditor independence is achieved through:
- **Deterministic verification:** Grep commands produce objective results independent of auditor judgment.
- **Automated testing:** `cargo test` results are machine-verified, not subjective.
- **Documented criteria:** Closure criteria are defined before implementation begins (in the sprint README), preventing post-hoc rationalization.

### 9.2 Known Limitation

For subjective audit criteria (e.g., "design is orthogonal to existing layers"), single-contributor projects cannot achieve true independence. This is acceptable for Class A software (IEC 62304) and research-tool classification but would need to be addressed for SaMD pursuit through:
- External code review (open-source community)
- Contracted regulatory audit
- Additional contributor with formal auditor role

## 10. Codebase References

| Reference | Path | Relevance |
|-----------|------|-----------|
| DECOUPLING_AUDIT closure | `docs/sprints/archive/DECOUPLING_AUDIT/README.md` | Real audit example with 7 grep-verified criteria |
| DC-4 sprint doc | `docs/sprints/archive/DECOUPLING_AUDIT/SPRINT_DC4_PURE_MATH_BOUNDARY.md` | 9 closure criteria, all grep-verifiable |
| Sprint archive index | `docs/sprints/archive/README.md` | 78 sprints with closure evidence |
| Hard blocks | `CLAUDE.md` §Hard Blocks | Source of audit criteria H1-H4 |
| Axioms | `CLAUDE.md` §The 8 Foundational Axioms | Source of audit criterion A1 |
| Derived thresholds | `src/blueprint/equations/derived_thresholds.rs` | Source of audit criterion A2 (17 tests) |
| Determinism module | `src/blueprint/equations/determinism.rs` | Source of determinism verification (23 tests) |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. Grep-based verification methodology, sprint closure checklist (16 items), annual checklist (9 items), real DECOUPLING_AUDIT example, audit independence assessment. |
