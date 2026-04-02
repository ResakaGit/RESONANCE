# ADR-008: ALCOA+ Compliance via Git (No Electronic Signatures)

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** 21 CFR Part 11, EU Annex 11, Data Integrity

## Context

21 CFR Part 11 (Electronic Records; Electronic Signatures) and EU Annex 11 (Computerised Systems) require controls for data integrity when electronic records replace paper records. The harmonized framework for data integrity is ALCOA+, which requires that records be: **A**ttributable, **L**egible, **C**ontemporaneous, **O**riginal, **A**ccurate, **C**omplete, **C**onsistent, **E**nduring, and **A**vailable.

RESONANCE's regulatory documentation (43 documents), source code (113K LOC), and test results (3,113 tests) are all electronic records stored in Git. The question is whether Git infrastructure satisfies ALCOA+ requirements or whether a dedicated electronic signature and record management system is needed.

## Decision

Implement **ALCOA+ data integrity principles through Git infrastructure** rather than deploying a dedicated electronic signature system. Accept known gaps in access control and electronic signatures, with planned mitigations in RI-2.

ALCOA+ mapping to Git:

| ALCOA+ Principle | Git Implementation | Status |
|-----------------|-------------------|--------|
| **Attributable** | `git log --format="%an <%ae>"` -- every commit has author name and email | Implemented (self-asserted identity) |
| **Legible** | Markdown format for all documentation, source code in Rust with `///` doc comments | Implemented |
| **Contemporaneous** | `git log --format="%ai"` -- commit timestamp recorded at time of creation | Implemented |
| **Original** | Git blob storage with SHA content addressing -- content is immutable once committed | Implemented |
| **Accurate** | 3,113 tests verify correctness; `cargo clippy` enforces code quality; sprint DOD verification | Implemented |
| **Complete** | Full Git history from project inception; no selective deletion possible without SHA chain breakage | Implemented |
| **Consistent** | `Cargo.lock` pins all dependency versions; deterministic hash-based RNG (no external randomness) | Implemented |
| **Enduring** | Git repository with GitHub remote; content-addressed storage survives format changes | Implemented |
| **Available** | Public GitHub repository (AGPL-3.0); `git clone` provides complete copy | Implemented |

## Consequences

### Positive
- All 9 ALCOA+ principles satisfied without additional tooling or cost.
- Evidence is inherently tamper-evident -- modifying any historical record breaks the SHA chain.
- Single source of truth -- code, documentation, and test results share one audit trail.
- Any auditor can independently verify the complete history with standard Git tools.

### Negative
- Author identity is self-asserted via `git config`. No cryptographic proof of identity without GPG signatures (planned in RI-2).
- No electronic signatures meeting 21 CFR Part 11 Section 11.50 requirements (signature linked to record, includes printed name, date/time, and meaning).
- Access control (Section 11.10(d)) is limited to GitHub repository permissions -- no application-level role-based access control distinguishing "author," "reviewer," and "approver" roles.
- Audit trail export requires Git expertise. No standardized report format for regulatory submission.

### Risks
- If a regulatory body requires formal Part 11 compliance (not voluntary), the lack of electronic signatures would be a finding. Mitigation: GPG signing planned in RI-2 sprint provides cryptographic attribution; full Part 11 electronic signature system would require dedicated tooling.
- Git history rewriting (`--force` push, `rebase`) could compromise the "Original" and "Complete" principles. Mitigation: branch protection planned in RI-1; current single-developer context and published GitHub remote provide practical safeguard.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| DocuSign / Adobe Sign | Addresses only the signature gap, not the full ALCOA+ framework. Adds per-document cost. Does not integrate with code versioning. |
| Blockchain timestamping (e.g., OpenTimestamps) | Provides cryptographic timestamp proof but adds complexity without proportionate benefit. Git SHA chain already provides tamper evidence. |
| Dedicated Part 11 platform (Montrium, Veeva Vault) | Enterprise-grade cost ($50K-200K/year). Designed for pharmaceutical companies with hundreds of users. Vastly disproportionate for a single-developer research project. |
| GPG-signed commits only | Addresses Attributable (cryptographic identity) but does not address the full ALCOA+ framework. Planned as incremental improvement in RI-2, not as a complete solution. |

## References

- `docs/regulatory/01_foundation/RD-7.1` -- 21 CFR Part 11 Compliance Assessment
- `docs/regulatory/01_foundation/RD-7.2` -- Data Integrity Policy (ALCOA+ Framework)
- `docs/regulatory/01_foundation/RD-7.3` -- Audit Trail Specification
- 21 CFR Part 11 -- Electronic Records; Electronic Signatures
- EU Annex 11 -- Computerised Systems (EudraLex Volume 4)
- WHO Guidance on Good Data and Record Management Practices (2016)
- RI-2 sprint plan -- GPG signing + enhanced access controls
