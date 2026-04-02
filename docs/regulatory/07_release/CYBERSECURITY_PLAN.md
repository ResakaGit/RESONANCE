---
document_id: RD-7.4
title: Cybersecurity Plan
standard: FDA §524B FD&C, IMDRF Cybersecurity Guidance
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
---

# Cybersecurity Plan

## 1. Purpose

This document defines the cybersecurity plan for RESONANCE, addressing threats to software integrity, supply chain security, and data confidentiality. It satisfies the cybersecurity documentation expectations of FDA §524B of the Federal Food, Drug, and Cosmetic Act (as amended by the Consolidated Appropriations Act, 2023) and the IMDRF Principles and Practices for Medical Device Cybersecurity (N60, 2020).

RESONANCE is a research simulation tool classified as IEC 62304 Class A (RD-1.2) that runs entirely offline on a local workstation. It does not process patient data, does not communicate over a network during operation, and does not interface with medical devices or clinical systems. The cybersecurity risk profile is correspondingly low. This plan is maintained voluntarily as regulatory preparedness.

**Cross-references:**
- RD-1.1: Intended Use Statement (offline research tool, no patient data)
- RD-1.2: Software Safety Classification (Class A)
- RD-2.1: Risk Management Plan (H-05: SOUP vulnerability)
- RD-3.2: SOUP Analysis (dependency risk assessment)
- RD-3.3: SBOM (dependency inventory with checksums)
- RD-3.4: Configuration Management Plan (Cargo.lock pinning, Git integrity)
- RD-7.1: Part 11 Compliance Assessment (access controls, audit trail)
- RD-7.3: Audit Trail Procedure (change tracking, anomaly detection)

## 2. System Characterization

### 2.1 System Architecture (Cybersecurity Perspective)

```
┌─────────────────────────────────────────────────────────┐
│                   Researcher Workstation                 │
│                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────┐ │
│  │  RESONANCE   │    │   Cargo      │    │  Git      │ │
│  │  Binary      │    │   (build)    │    │  (VCS)    │ │
│  │              │    │              │    │           │ │
│  │  No network  │    │  crates.io   │←───│  GitHub   │ │
│  │  No PHI/PII  │    │  (fetch)     │    │  (remote) │ │
│  │  Local I/O   │    │              │    │           │ │
│  └──────────────┘    └──────────────┘    └───────────┘ │
│                                                         │
│  Attack surface: build-time only (crate fetch)          │
│  Runtime: zero network, zero external input             │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Connectivity Profile

| Interface | Status | Description |
|-----------|--------|-------------|
| Network (runtime) | **None** | RESONANCE binaries make zero network calls during execution. No HTTP, no WebSocket, no IPC to remote services. |
| Network (build-time) | **Outbound only** | `cargo build` fetches crates from `crates.io` via HTTPS. One-time per dependency version; cached locally thereafter. |
| Network (VCS) | **Bidirectional** | `git push`/`git pull` communicates with GitHub via SSH or HTTPS. Developer-initiated only. |
| USB/Peripheral | **None** | No device I/O beyond keyboard, mouse, and display (Bevy windowing). |
| File system (runtime) | **Local read/write** | Reads map configs (`assets/maps/*.ron`), writes simulation output (PPM, CSV, JSON) to local filesystem. |
| Cloud services | **None** | No cloud backend, no telemetry, no analytics, no license server. |

### 2.3 Data Classification

| Data Category | Present | Classification | Justification |
|---------------|---------|----------------|---------------|
| Protected Health Information (PHI) | **No** | N/A | RESONANCE does not accept, process, store, or transmit patient data |
| Personally Identifiable Information (PII) | **No** | N/A | No user accounts, no registration, no data collection |
| Intellectual property (source code) | **Yes** | Public (AGPL-3.0) | Source code is intentionally public; no trade secrets |
| Simulation output | **Yes** | Non-sensitive | Abstract energy units (qe); no patient-identifiable content |
| Configuration (Cargo.lock, maps) | **Yes** | Non-sensitive | Build configuration; no credentials or secrets |

---

## 3. Threat Model

### 3.1 Threat Identification

Threats are assessed using the STRIDE framework (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege) adapted for an offline research tool.

| Threat ID | STRIDE Category | Threat Description | Likelihood | Impact | Risk Level |
|-----------|-----------------|-------------------|------------|--------|------------|
| T-01 | Tampering | **Simulation tampering:** Malicious modification of simulation code to produce biased results (e.g., to falsely validate a drug combination strategy) | Low | Medium | Low |
| T-02 | Tampering | **Supply chain compromise:** A direct or transitive dependency (crate) is compromised to inject malicious code, affecting simulation integrity or host system | Low | High | Medium |
| T-03 | Information Disclosure | **Data exfiltration:** Extraction of sensitive data from the RESONANCE system | Very Low | Very Low | Negligible |
| T-04 | Denial of Service | **Resource exhaustion:** RESONANCE consumes excessive CPU/memory, rendering the workstation unusable | Very Low | Low | Negligible |
| T-05 | Tampering | **Git history manipulation:** Unauthorized force-push or rebase that rewrites commit history to obscure changes | Low | Medium | Low |
| T-06 | Spoofing | **Commit author spoofing:** Attacker commits code under a false identity to inject malicious changes without attribution | Very Low | Medium | Low |
| T-07 | Tampering | **Build reproducibility attack:** Modification of Cargo.lock or build environment to produce a different binary from the same source | Very Low | Medium | Low |
| T-08 | Elevation of Privilege | **Local privilege escalation via RESONANCE** | Very Low | Low | Negligible |

### 3.2 Threat Detail Sheets

#### T-01: Simulation Tampering

**Attack vector:** A developer (or compromised developer account) modifies simulation equations or constants to produce biased output that favors or disfavors a particular drug strategy.

**Existing controls:**
- All code changes recorded in Git audit trail (RD-7.3)
- 3,113 automated tests detect equation changes that alter expected output
- Pure math in `src/blueprint/equations/` is isolated from side-effect-producing systems
- Property-based tests fuzz conservation invariants
- Code review (Verificador role) checks math correctness

**Residual risk:** Low. Deliberate tampering would require modifying tests to match the tampered equations, which would be visible in the Git diff. The public repository makes such changes visible to any reviewer.

#### T-02: Supply Chain Compromise

**Attack vector:** A crate published to crates.io is compromised (account takeover, typosquatting, dependency confusion), and the compromised version is pulled into RESONANCE's dependency tree.

**Existing controls:**
- `Cargo.lock` pins exact versions with SHA-256 checksums — upgrades require explicit developer action
- Hard Block HB-2 (CLAUDE.md): "NO external crates without approval — only what's in Cargo.toml"
- SOUP Analysis (RD-3.2) assesses risk for each direct dependency
- `cargo audit` checks against RUSTSEC advisory database
- 14 direct runtime dependencies (minimized surface)
- crates.io integrity: checksums verified by cargo on download

**Residual risk:** Medium. The ~424 transitive dependencies (primarily from Bevy's rendering subsystem) represent a large attack surface that RESONANCE cannot directly audit. Mitigation relies on the Rust ecosystem's security practices (crates.io, RUSTSEC advisories, cargo audit).

#### T-03: Data Exfiltration

**Attack vector:** Extraction of sensitive data from the RESONANCE system.

**Assessment:** Negligible. RESONANCE contains no PHI, no PII, no credentials, and no trade secrets. The entire source code is public (AGPL-3.0). Simulation output consists of abstract energy values with no patient-identifiable content. There is nothing of confidential value to exfiltrate.

#### T-04: Denial of Service

**Attack vector:** RESONANCE consumes excessive computational resources, rendering the workstation unusable.

**Assessment:** Negligible. RESONANCE runs locally on the operator's workstation. The operator controls execution (start/stop). Batch simulations (`src/batch/`) use rayon for parallelism but are bounded by the number of worlds and ticks specified at invocation. There is no remote-triggerable execution path.

#### T-05: Git History Manipulation

**Attack vector:** Unauthorized force-push to `main` that rewrites commit history, obscuring evidence of code changes.

**Existing controls:**
- Development practice prohibits force-push to `main` (CLAUDE.md, RD-3.4 §3.1)
- Every developer clone contains the full pre-manipulation history
- Quarterly audit trail review (RD-7.3 §5) checks commit count monotonicity
- GitHub event logs record push events independently of Git history

**Residual risk:** Low. Force-push is detectable by comparing any pre-manipulation clone with the repository.

#### T-06: Commit Author Spoofing

**Attack vector:** An attacker sets `git config user.name` and `git config user.email` to impersonate a legitimate developer.

**Existing controls:**
- GitHub account binding (commits are associated with GitHub accounts via email matching)
- Planned: GPG-signed commits (RD-7.1 Gap G-04)

**Residual risk:** Low. Spoofing requires write access to the repository (controlled by GitHub permissions). The attack is further mitigated by the public repository's transparency.

---

## 4. Mitigations

### 4.1 Git SHA Integrity

| Control | Implementation |
|---------|----------------|
| Commit integrity | SHA-1 hash chain: every commit cryptographically references its parent and content |
| Tamper detection | `git fsck --full` verifies object database integrity |
| Distributed verification | Multiple clones provide independent integrity verification |
| Future hardening | Git SHA-256 transition (when stable) will strengthen hash integrity |

### 4.2 Cargo Audit

| Control | Implementation |
|---------|----------------|
| Tool | `cargo audit` (RustSec Advisory Database) |
| Scope | All direct and transitive dependencies |
| Frequency | Before each release; triggered by RUSTSEC advisory notification |
| Action on finding | Assess severity; patch within 30 days for critical/high; 90 days for medium/low (see §5) |
| Command | `cargo audit` |

### 4.3 Cargo.lock Pinning

| Control | Implementation |
|---------|----------------|
| Mechanism | `Cargo.lock` committed to repository; pins exact versions with SHA-256 checksums |
| Policy | `Cargo.lock` changes require explicit commit with justification in commit message |
| Verification | `cargo verify-project` confirms lock file consistency |
| Audit trail | All `Cargo.lock` changes visible via `git log -- Cargo.lock` |

### 4.4 Branch Protection (Planned)

| Control | Implementation | Status |
|---------|----------------|--------|
| Required reviewers | GitHub branch protection: minimum 1 reviewer for PRs to `main` | Planned (RD-7.1 Gap G-02) |
| Required status checks | `cargo test` and `cargo check` must pass before merge | Planned |
| Signed commits | GPG-signed commits required | Planned (RD-7.1 Gap G-04) |
| No force-push | Enforce at GitHub level (currently enforced by convention only) | Planned |

### 4.5 Minimal Attack Surface

| Control | Implementation |
|---------|----------------|
| No network at runtime | RESONANCE binaries make zero network calls; no listening ports, no HTTP clients, no WebSocket |
| No unsafe code | Hard Block HB-1 (CLAUDE.md): "NO `unsafe` — zero tolerance. No exceptions." |
| No dynamic loading | No `dlopen`, no plugin system, no WASM execution |
| No user input parsing (network) | All input is local files (RON configs, CLI args); no network-received input |
| Minimal binary count | 25 binary targets, all compiled from the same source; no separate service components |

### 4.6 Dependency Governance

| Control | Implementation |
|---------|----------------|
| Approval required | Hard Block HB-2: new dependencies require explicit approval (documented in Cargo.toml) |
| SOUP analysis | RD-3.2 assesses risk for each of the 14 direct runtime dependencies |
| License audit | All dependencies are MIT, Apache-2.0, or Zlib licensed (compatible with AGPL-3.0) |
| Transitive awareness | `cargo tree` enumerates full dependency graph (~424 unique packages) |
| Dev-dependency isolation | 5 dev-only dependencies (criterion, naga, pollster, proptest, wgpu) are not present in release binaries |

---

## 5. SBOM Maintenance

### 5.1 SBOM Cross-Reference

The Software Bill of Materials is maintained in RD-3.3 (`docs/regulatory/03_traceability/SBOM.md`). It lists all 14 direct runtime dependencies and 5 dev-only dependencies with:
- Component name and version
- License
- Publisher
- SHA-256 checksum from `Cargo.lock`
- Relationship (direct/dev)

### 5.2 SBOM Update Triggers

| Trigger | Action |
|---------|--------|
| Any change to `Cargo.toml` | Update RD-3.3 SBOM; update RD-3.2 SOUP Analysis if new dependency |
| Any change to `Cargo.lock` | Verify SBOM accuracy; update checksums if dependency versions changed |
| RUSTSEC advisory published | Run `cargo audit`; assess impact; update SOUP Analysis risk assessment |
| Quarterly audit trail review | Verify SBOM matches current `Cargo.lock` |
| Major release | Full SBOM regeneration and verification |

### 5.3 SBOM Generation

```bash
# Verify current dependency tree
cargo tree --depth 1

# Check for known vulnerabilities
cargo audit

# Verify Cargo.lock integrity
cargo verify-project

# Count transitive dependencies
cargo tree --prefix none --no-dedupe | sort -u | wc -l
```

---

## 6. Patching Cadence

### 6.1 Vulnerability Response Timeline

| RUSTSEC Severity | Response Time | Action |
|------------------|---------------|--------|
| Critical (CVSS 9.0+) | 7 days | Assess impact; patch or mitigate; update SBOM; notify stakeholders |
| High (CVSS 7.0-8.9) | 30 days | Assess impact; schedule patch; update SBOM |
| Medium (CVSS 4.0-6.9) | 90 days | Assess impact; schedule patch in next sprint |
| Low (CVSS 0.1-3.9) | Next release | Document in SOUP Analysis; patch opportunistically |
| Informational | No action required | Document for awareness |

### 6.2 Patching Procedure

1. **Detection:** `cargo audit` identifies advisory
2. **Assessment:** Evaluate whether the vulnerable code path is reachable in RESONANCE's usage
3. **Decision:** Patch (update dependency), mitigate (workaround), or accept (if not reachable)
4. **Implementation:** Update `Cargo.toml` and `Cargo.lock`; run full test suite (`cargo test`)
5. **Verification:** Confirm vulnerability resolved (`cargo audit` clean)
6. **Documentation:** Update RD-3.2 SOUP Analysis; update RD-3.3 SBOM; commit with justification
7. **Review:** Verificador confirms patch correctness and test results

### 6.3 Current Vulnerability Status

As of commit `971c7acb99decde45bf28860e6e10372718c51e2`:

| Check | Command | Status |
|-------|---------|--------|
| Known vulnerabilities | `cargo audit` | To be verified at release |
| Outdated dependencies | `cargo outdated` | Informational only; not all updates are required |
| Dependency tree size | `cargo tree --prefix none --no-dedupe \| sort -u \| wc -l` | ~424 unique packages |

---

## 7. No PHI/PII Declaration

### 7.1 Formal Declaration

RESONANCE does not process, store, transmit, or have access to Protected Health Information (PHI) as defined by HIPAA (45 CFR §160.103) or Personally Identifiable Information (PII) as defined by NIST SP 800-122.

### 7.2 Evidence

| Claim | Evidence |
|-------|----------|
| No patient data input | No file parsers for DICOM, HL7, FHIR, or any clinical data format in `src/` |
| No patient data output | All output is simulation data in abstract energy units (qe) or calibrated units with explicit disclaimers |
| No user accounts | No authentication system, no user database, no login mechanism |
| No telemetry | No analytics, no crash reporting, no usage tracking |
| No network communication at runtime | No HTTP client, no WebSocket, no IPC; verified by absence of network crates in runtime dependencies |
| No credentials stored | No API keys, no tokens, no passwords in source code or configuration |
| No cloud integration | No cloud backend, no SaaS dependency, no external service calls |

### 7.3 Implications

Because RESONANCE processes no PHI/PII:
- HIPAA Security Rule (45 CFR §164.312) does not apply
- GDPR data protection requirements (EU 2016/679) do not apply
- No Data Protection Impact Assessment (DPIA) is required
- No breach notification obligations exist
- No data encryption at rest or in transit is required for regulatory compliance (though Git uses HTTPS/SSH for transport)

---

## 8. Incident Response

### 8.1 Cybersecurity Incident Types

| Incident Type | Example | Response |
|---------------|---------|----------|
| Dependency compromise | RUSTSEC advisory for a direct dependency | Patching cadence (§6); CAPA if shipped (RD-5.7) |
| Repository compromise | Unauthorized commit or force-push | Immediate investigation; compare with backup clones; revoke compromised credentials; restore history |
| Build environment compromise | Developer workstation compromised | Revoke repository access; audit recent commits from compromised machine; rebuild from clean environment |
| Simulation integrity incident | Results cannot be reproduced from declared seed + commit | Determinism investigation; check for platform-dependent behavior; file CAPA (RD-5.7) |

### 8.2 Incident Response Process

1. **Detection:** Automated (`cargo audit`, `git fsck`, test failure) or manual (reported by user or reviewer)
2. **Containment:** Halt releases; isolate affected component
3. **Assessment:** Determine scope, impact, and root cause
4. **Remediation:** Patch, revert, or mitigate
5. **Recovery:** Verify fix; resume normal operations
6. **Documentation:** File CAPA (RD-5.7); update relevant regulatory documents
7. **Lessons learned:** Update threat model if new threat category identified

---

## 9. Conclusion

RESONANCE's cybersecurity risk profile is inherently low due to its offline operation, absence of patient data, and public source code. The primary cybersecurity concern is supply chain integrity (T-02), which is mitigated by `Cargo.lock` pinning, `cargo audit`, SOUP analysis, and the Hard Block governance model for dependency approval.

| Threat | Risk Level | Mitigation Status |
|--------|------------|-------------------|
| T-01: Simulation tampering | Low | ✅ Fully mitigated (Git audit trail, 3,113 tests, code review) |
| T-02: Supply chain compromise | Medium | ⚠️ Partially mitigated (Cargo.lock + cargo audit; ~424 transitive deps not individually audited) |
| T-03: Data exfiltration | Negligible | ✅ No sensitive data exists |
| T-04: Denial of service | Negligible | ✅ Local execution, operator-controlled |
| T-05: Git history manipulation | Low | ✅ Convention + distributed verification; planned: branch protection |
| T-06: Commit author spoofing | Low | ⚠️ Partially mitigated; planned: GPG-signed commits |
| T-07: Build reproducibility attack | Low | ✅ Cargo.lock pinning + SHA-256 checksums |
| T-08: Local privilege escalation | Negligible | ✅ No unsafe code, no elevated permissions required |

---

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial plan |
