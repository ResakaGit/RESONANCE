---
document_id: RD-7.1
title: 21 CFR Part 11 Compliance Assessment
standard: 21 CFR Part 11, EU Annex 11
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
---

# 21 CFR Part 11 Compliance Assessment

## 1. Purpose

This document assesses RESONANCE's compliance with 21 CFR Part 11 (Electronic Records; Electronic Signatures) and EU Annex 11 (Computerised Systems). It maps each regulatory requirement to the RESONANCE implementation, identifies gaps, and defines a remediation plan.

RESONANCE is a research simulation tool classified as IEC 62304 Class A (RD-1.2). It is not subject to mandatory 21 CFR Part 11 compliance because it does not generate electronic records for regulatory submissions and does not process patient data. This assessment is performed voluntarily as part of regulatory preparedness (RD-1.5 Regulatory Strategy §2.2).

**Cross-references:**
- RD-1.1: Intended Use Statement (research tool, no clinical data)
- RD-1.2: Software Safety Classification (Class A)
- RD-1.5: Regulatory Strategy (voluntary compliance rationale)
- RD-3.4: Configuration Management Plan (Git-based version control)
- RD-5.3: Document Control Procedure (document lifecycle)
- RD-5.4: Record Control Procedure (record retention and retrieval)
- RD-7.3: Audit Trail Procedure (audit trail implementation details)

## 2. System Description

| Attribute | Value |
|-----------|-------|
| System name | RESONANCE |
| Version | 0.1.0 |
| Classification | Closed system (21 CFR Part 11 §11.10) |
| Record types | Source code (`.rs`), configuration (`.toml`, `.ron`), documentation (`.md`), simulation output (`.ppm`, `.csv`, `.json`) |
| Record storage | Git repository (GitHub), local filesystem |
| Electronic signatures | Not used (no regulatory submissions) |
| Network connectivity | None required; offline tool |
| Patient data | None (no PHI/PII) |

### 2.1 Closed vs. Open System Determination

21 CFR Part 11 distinguishes between closed systems (§11.10) and open systems (§11.30). A closed system is one where "system access is controlled by persons who are responsible for the content of electronic records that are on the system."

**RESONANCE is a closed system because:**
- Repository access is controlled by the development team via GitHub permissions (SSH key + HTTPS authentication)
- The software runs locally on researcher workstations with no network communication
- No external parties have write access to the repository without explicit grant
- Simulation output files are stored locally on the operator's filesystem

**§11.30 (Open Systems) is excluded** because RESONANCE does not operate in an open system context. There are no web-accessible interfaces, no multi-tenant environments, and no external data exchange requiring encryption for regulatory purposes.

## 3. Subpart B: Electronic Records (§11.10)

### 3.1 §11.10(a) — Validation

> **Requirement:** Persons who use closed systems to create, modify, maintain, or transmit electronic records shall employ procedures and controls designed to ensure the authenticity, integrity, and, when appropriate, the confidentiality of electronic records, and to ensure that the signer cannot readily repudiate the signed record as not genuine. Such procedures and controls shall include the following: (a) Validation of systems to ensure accuracy, reliability, consistent intended performance, and the ability to discern invalid or altered records.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Accuracy | 3,113 automated tests verify mathematical correctness of all equations in `src/blueprint/equations/` (50+ domain files) | `cargo test` — 0 failures (RD-4.3 Verification Report) |
| Reliability | Deterministic RNG (`src/blueprint/equations/determinism.rs`, 23 tests) ensures bit-exact reproducibility | RD-4.4 Validation Report §3 (determinism validation) |
| Consistent intended performance | Property-based tests (`tests/property_conservation.rs`) fuzz conservation invariants across arbitrary inputs | RD-4.1 Validation Plan §2.1 |
| Discerning invalid/altered records | Git SHA-1 integrity — any modification to a committed record changes its hash, making alteration detectable | RD-3.4 Configuration Management Plan §3.1 |
| Validation plan | RD-4.1 Software Validation Plan defines validation strategy, acceptance criteria, and responsibilities | `docs/regulatory/04_validation/VALIDATION_PLAN.md` |

**Status:** ✅ Compliant

**Notes:** Validation is ongoing. Each sprint includes verification of closure criteria. The validation report (RD-4.4) documents experiment-level validation against published scientific data (Bozic et al. 2013).

---

### 3.2 §11.10(b) — Generating Accurate and Complete Copies

> **Requirement:** The ability to generate accurate and complete copies of records in both human readable and electronic form suitable for inspection, review, and copying by the agency.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Human-readable copies | All records are plain text: Rust source (`.rs`), Markdown (`.md`), TOML (`Cargo.toml`, `Cargo.lock`), RON (`assets/maps/*.ron`) | No binary formats for records (evolved genomes `.bin` are experimental artifacts, not regulatory records) |
| Electronic copies | `git archive` produces complete snapshot; `git clone` provides full history | RD-3.4 §3.1 |
| Export commands | `git log --format=...` extracts audit trail; `cargo test` produces machine-readable output | RD-7.3 Audit Trail Procedure §4 |
| Inspection-ready | GitHub web interface provides browsable view of all files at any commit | `https://github.com/ResakaGit/RESONANCE` |

**Status:** ✅ Compliant

---

### 3.3 §11.10(c) — Protection of Records

> **Requirement:** Protection of records to enable their accurate and ready retrieval throughout the records retention period.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Primary storage | GitHub-hosted Git repository (redundant datacenter storage) | `https://github.com/ResakaGit/RESONANCE` |
| Local copies | Every developer clone is a full backup of the entire repository history | Git distributed architecture |
| Retention period | Indefinite (Git history is immutable; GitHub repositories persist unless explicitly deleted) | RD-5.4 Record Control Procedure §5 |
| Retrieval | Any historical state recoverable via `git checkout <commit>` or `git show <commit>:<path>` | RD-3.4 §3.2 |

**Status:** ✅ Compliant

---

### 3.4 §11.10(d) — Limiting System Access

> **Requirement:** Limiting system access to authorized individuals.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Repository access | GitHub authentication (SSH key or HTTPS token) | GitHub settings |
| Write access | Restricted to development team members with explicit collaborator or organization membership | GitHub repository settings |
| Read access | Public repository (AGPL-3.0); read access does not compromise record integrity | License file; Git is append-only for authorized writers |
| Local execution | No multi-user access controls on local workstation (single-user research tool) | RD-1.1 §3 (use environment) |

**Status:** ⚠️ Partial

**Gap:** No role-based access control (RBAC) within the application itself. RESONANCE is a single-user CLI tool that relies on operating system and GitHub-level access controls rather than application-level user authentication. This is appropriate for a research tool with no patient data but would require enhancement for SaMD classification.

---

### 3.5 §11.10(e) — Audit Trails

> **Requirement:** Use of secure, computer-generated, time-stamped audit trails to independently record the date and time of operator entries and actions that create, modify, or delete electronic records. Record changes shall not obscure previously recorded information. Such audit trail documentation shall be retained for a period at least as long as that required for the subject electronic records and shall be available for agency review and copying.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Audit trail mechanism | Git commit history — every change is a timestamped, attributed, immutable entry | RD-7.3 Audit Trail Procedure |
| Time-stamped | Git commits include author date and commit date (UTC) | `git log --format="%H %ai %an %s"` |
| Attributed | Git commits identify the author (name + email) | `git log --format="%an <%ae>"` |
| Non-obscuring | Git history is append-only; previous versions are never overwritten (force-push to `main` prohibited by development practice) | RD-3.4 §3.3 ("Immutable" history), CLAUDE.md §Coding Rules |
| Retention | Git history retained indefinitely (same lifetime as repository) | GitHub storage policy |
| Agency access | Public repository; any auditor can clone and inspect | `https://github.com/ResakaGit/RESONANCE` |
| Simulation records | Deterministic seeds enable reproduction of any simulation run — seed + commit = exact output | `src/blueprint/equations/determinism.rs` |

**Status:** ✅ Compliant

**Notes:** Git provides a stronger audit trail than most bespoke audit trail systems because the cryptographic hash chain makes any retroactive modification of history computationally detectable. See RD-7.3 for detailed audit trail extraction procedures.

---

### 3.6 §11.10(f) — Operational System Checks

> **Requirement:** Use of operational system checks to enforce permitted sequencing of steps and events, as appropriate.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Build system | `cargo build` enforces compilation order; Rust type system prevents invalid state at compile time | Rust compiler |
| Pipeline sequencing | Bevy `SystemSet` (`Phase` enum) enforces execution order: `SimulationClockSet` → `Phase::Input` → `Phase::ThermodynamicLayer` → ... → `Phase::MorphologicalLayer` | `src/simulation/pipeline.rs`, `src/simulation/mod.rs` |
| Sprint sequencing | Sprint waves enforce dependency ordering (blocking dependencies resolved before downstream sprints) | RD-1.4 §2.2 |
| Test gating | All tests must pass (`cargo test` — 0 failures) before sprint closure | RD-1.4 §2.1 |

**Status:** ✅ Compliant

---

### 3.7 §11.10(g) — Authority Checks

> **Requirement:** Use of authority checks to ensure that only authorized individuals can use the system, electronically sign a record, access the operation or computer system input or output device, alter a record, or perform the operation at hand.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Repository write access | GitHub authentication + repository permissions | GitHub settings |
| Code review | Sprint closure includes review (Verificador role: contract, math, DOD, determinism, perf, tests) | RD-1.4 §2.1, CLAUDE.md §Roles |
| Branch protection | Trunk-based development on `main`; no force-push policy | RD-3.4 §5.1 |

**Status:** ⚠️ Partial

**Gap:** No formal branch protection rules enforced at the GitHub level (e.g., required reviewers, status checks). Current practice relies on development discipline rather than tooling enforcement. This is acceptable for a single-team research project but would need hardening for SaMD.

---

### 3.8 §11.10(h) — Device Checks

> **Requirement:** Use of device checks to determine, as appropriate, the validity of the source of data input or operational instruction.

**Implementation in RESONANCE:**

**Status:** N/A — Excluded

**Justification:** RESONANCE does not interface with physical devices (sensors, instruments, medical devices, or laboratory equipment). All input is either source code, configuration files (`.ron`, `.toml`), or CLI arguments. There is no device data acquisition pathway. This subsection does not apply.

---

### 3.9 §11.10(i) — Training

> **Requirement:** Determination that persons who develop, maintain, or use electronic record/electronic signature systems have the education, training, and experience to perform their assigned tasks.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Developer qualification | Development team has Rust, ECS, and simulation engineering expertise | Project constitution (`CLAUDE.md`) defines roles: Alquimista, Observador, Planificador, Verificador |
| Role definitions | Each role has defined focus and responsibilities | CLAUDE.md §Roles |
| Training records | Not formally maintained | Gap — see below |

**Status:** ⚠️ Partial

**Gap:** No formal training records or competency assessments are maintained. Developer expertise is evidenced by the codebase itself (113K LOC, 3,113 tests) but not documented in a training matrix. Remediation: create a training record log if SaMD classification is pursued.

---

### 3.10 §11.10(j) — Written Policies

> **Requirement:** The establishment of, and adherence to, written policies that hold individuals accountable and responsible for actions initiated under their electronic signatures, in order to deter record and signature falsification.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Accountability policy | Git commits attribute all changes to an identified author | `git log` author field |
| Coding standards | `CLAUDE.md` defines development rules, hard blocks, and inference protocol | Repository root |
| Quality policy | RD-5.2 Quality Policy defines quality objectives and management commitment | `docs/regulatory/05_quality_system/QUALITY_POLICY.md` |
| CAPA procedure | RD-5.7 defines corrective/preventive action for nonconformances | `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` |

**Status:** ✅ Compliant

**Notes:** Electronic signatures are not currently used (no regulatory submissions). Accountability is established through Git attribution. If electronic signatures become necessary, GPG-signed commits would be the implementation mechanism.

---

### 3.11 §11.10(k) — Controls for Systems Documentation

> **Requirement:** Use of appropriate controls over systems documentation including: (1) Adequate controls over the distribution of, access to, and use of documentation for system operation and maintenance. (2) Revision and change control procedures to maintain an audit trail that documents time-sequenced development and modification of systems documentation.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Document control | RD-5.3 Document Control Procedure governs creation, review, distribution, and maintenance | `docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md` |
| Version control | All documentation is in Git; every change is timestamped and attributed | RD-3.4 Configuration Management Plan |
| Change audit trail | `git log -- docs/` shows complete history of all documentation changes | See RD-7.3 |
| Distribution control | Public repository; controlled documents identified in RD-5.3 §3.1 | Document registry in RD-5.3 |

**Status:** ✅ Compliant

---

## 4. §11.50 — Signature Manifestations

> **Requirement:** (a) Signed electronic records shall contain information associated with the signing that clearly indicates all of the following: (1) The printed name of the signer; (2) The date and time when the signature was executed; and (3) The meaning (such as review, approval, responsibility, or authorship) associated with the signature.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Signer identification | Git commit author field: `name <email>` | `git log --format="%an <%ae>"` |
| Timestamp | Git commit date (ISO 8601, UTC-referenced) | `git log --format="%ai"` |
| Meaning | Commit message prefix conveys intent: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, `paper:` | RD-3.4 §3.3 |

**Status:** ⚠️ Partial

**Gap:** Git commit author identity is self-asserted (configurable via `git config`), not cryptographically verified. For full §11.50 compliance, GPG-signed commits with verified keys would be required. This is a known gap documented in RD-3.4 §3.3 ("Signed commits: Not currently required").

---

## 5. §11.70 — Signature/Record Linking

> **Requirement:** Electronic signatures and handwritten signatures executed to electronic records shall be linked to their respective electronic records to ensure that the signatures cannot be excised, copied, or otherwise transferred to falsify an electronic record by ordinary means.

**Implementation in RESONANCE:**

| Control | Implementation | Evidence |
|---------|----------------|----------|
| Signature-record binding | Git commits cryptographically bind author metadata to content via SHA-1 hash chain | Git internals: commit object = tree hash + parent hash + author + committer + message |
| Non-transferability | Changing any field (author, content, timestamp) changes the commit hash, breaking the chain | SHA-1 collision resistance |
| Tamper evidence | Any modification to history is detectable by comparing local and remote repository hashes | `git fsck`, `git log --verify-signatures` (when GPG is enabled) |

**Status:** ⚠️ Partial

**Gap:** Same as §11.50 — author identity is self-asserted without GPG verification. The cryptographic linking of content to metadata is strong (SHA-1 hash chain), but the identity assertion is weak without signed commits.

---

## 6. Subpart C: Electronic Signatures — Applicability

21 CFR Part 11 Subpart C (§11.100 through §11.300) governs electronic signatures intended as the legally binding equivalent of handwritten signatures.

**RESONANCE does not use electronic signatures in the regulatory sense.**

| §11 Subpart C Section | Applicability | Justification |
|------------------------|---------------|---------------|
| §11.100 — General requirements | N/A | No records are electronically signed for regulatory purposes |
| §11.200 — Electronic signature components and controls | N/A | No biometric or non-biometric e-signatures implemented |
| §11.300 — Controls for identification codes/passwords | N/A | No e-signature system exists; authentication is OS/GitHub-level |

**If electronic signatures become required** (e.g., SaMD classification pursued), the implementation path would be:
1. Enable GPG-signed Git commits for all developers
2. Establish a key management procedure (key generation, distribution, revocation)
3. Configure GitHub branch protection to require signed commits
4. Document the signature meaning conventions in a written policy

---

## 7. EU Annex 11 Cross-Reference

For international regulatory preparedness, this section maps key EU Annex 11 requirements to RESONANCE implementation.

| Annex 11 Clause | Requirement | RESONANCE Implementation | Status |
|------------------|-------------|--------------------------|--------|
| §1 Risk management | Risk management applied throughout lifecycle | RD-2.1 through RD-2.6 (ISO 14971 risk management) | ✅ |
| §3 Supplier and service providers | Agreements with third-party suppliers | No third-party suppliers; open-source dependencies governed by HB-2 (CLAUDE.md) + SOUP analysis (RD-3.2) | ✅ (adapted) |
| §4 Validation | Documented evidence of fitness for intended use | RD-4.1 through RD-4.5 (validation plan, credibility model, verification, validation, uncertainty) | ✅ |
| §5 Data | Checks for data integrity | 3,113 tests; property-based fuzzing; deterministic RNG; conservation invariants | ✅ |
| §6 Accuracy checks | Critical data entered manually verified | No manual data entry; all input is source code or configuration files under version control | N/A |
| §7.1 Data storage | Protection against damage and data loss | Git distributed architecture; GitHub redundancy; every clone is a full backup | ✅ |
| §7.2 Data storage | Backup and restore tested | Recoverable via `git clone` from any mirror; tested by routine development workflow | ✅ |
| §8 Printouts | Clear printouts of electronically stored data | All records are human-readable plain text (Markdown, Rust, TOML, RON) | ✅ |
| §9 Audit trails | Record changes with timestamps | Git commit history — see §11.10(e) above and RD-7.3 | ✅ |
| §10 Change and configuration management | Formal change control | RD-3.4 Configuration Management Plan; trunk-based development on `main` | ✅ |
| §11 Periodic evaluation | Regular review to confirm validated state | Quarterly audit trail review (RD-7.3 §5); internal audit (RD-5.5) | ⚠️ |
| §12 Security | Physical and logical access controls | GitHub authentication; local OS controls; no network exposure | ⚠️ |
| §13 Incident management | Reporting and assessment of incidents | CAPA procedure (RD-5.7); nonconforming product procedure (RD-5.6) | ✅ |
| §14 Electronic signature | Equivalent to handwritten signature | Not used — see §6 above | N/A |
| §16 Business continuity | Plans for system availability | Git distributed architecture; no single point of failure for source; GitHub uptime SLA | ✅ |
| §17 Archiving | Long-term readability and integrity | Git immutable history; plain-text formats; no proprietary file formats | ✅ |

---

## 8. Gap Summary and Remediation Plan

### 8.1 Gap Register

| Gap ID | Section | Gap Description | Severity | Current Mitigation |
|--------|---------|-----------------|----------|--------------------|
| G-01 | §11.10(d) | No application-level RBAC; reliance on OS and GitHub access controls | Low | RESONANCE is single-user CLI; GitHub permissions restrict write access |
| G-02 | §11.10(g) | No enforced branch protection rules at GitHub level | Low | Development discipline (no force-push policy, Verificador review role) |
| G-03 | §11.10(i) | No formal training records or competency matrix | Low | Developer expertise evidenced by codebase quality (113K LOC, 3,113 tests) |
| G-04 | §11.50, §11.70 | Git commit author identity is self-asserted, not GPG-verified | Medium | SHA-1 hash chain provides content-metadata binding; identity is bound to GitHub account |
| G-05 | Annex 11 §11 | Periodic evaluation process not yet executed (first cycle pending) | Low | Procedure defined in RD-7.3 and RD-5.5; first execution scheduled |

### 8.2 Remediation Plan

| Gap ID | Remediation Action | Priority | Trigger |
|--------|-------------------|----------|---------|
| G-01 | Implement application-level access controls if RESONANCE transitions to multi-user or SaMD | Deferred | SaMD classification decision |
| G-02 | Enable GitHub branch protection rules: required reviewers (1+), required status checks (`cargo test`, `cargo check`) | Medium | Next infrastructure sprint |
| G-03 | Create training record template; document developer qualifications | Low | SaMD classification decision |
| G-04 | Enable GPG-signed commits; establish key management procedure; configure GitHub signed-commit requirement | Medium | Next infrastructure sprint or SaMD classification decision |
| G-05 | Execute first quarterly audit trail review per RD-7.3 §5 | High | Q2 2026 (scheduled) |

### 8.3 Residual Risk Assessment

All identified gaps are rated **Low** or **Medium** severity in the context of RESONANCE's current classification as a Class A research tool with no patient data. None of the gaps expose patient safety risks. The gaps are relevant only for:
- Future SaMD classification (G-01, G-03)
- Strengthened regulatory posture for pharma partnerships (G-02, G-04)
- Procedural maturity (G-05)

---

## 9. Conclusion

RESONANCE achieves substantial compliance with 21 CFR Part 11 §11.10 through its Git-based development infrastructure, which provides timestamped, attributed, immutable audit trails without requiring a bespoke electronic records management system. Key strengths:

- **8 of 11 §11.10 subsections:** Fully compliant (a, b, c, e, f, j, k) or not applicable (h)
- **3 subsections:** Partially compliant (d, g, i) with low-severity gaps appropriate for Class A
- **§11.50 and §11.70:** Partially compliant; GPG-signed commits would close the gap
- **EU Annex 11:** 13 of 17 clauses fully compliant or not applicable; 2 partially compliant; 2 not applicable

The identified gaps (G-01 through G-05) are documented with remediation plans and severity assessments. None represent safety risks given RESONANCE's classification as a research tool with no patient data processing.

---

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial assessment |
