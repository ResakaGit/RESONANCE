# Security Policy

## Scope

RESONANCE is a computational research tool. It does **not** process patient data (PHI/PII), does not connect to networks at runtime, and does not make clinical decisions. Its attack surface is limited to:

- **Supply chain** — compromised crate in the dependency tree
- **Simulation tampering** — malicious modification of equations producing wrong results
- **Build integrity** — modified binary distributed as authentic

## Supported Versions

| Version | Supported |
|---------|-----------|
| `main` (HEAD) | Yes |
| Tagged releases | Yes |
| Older commits | Best-effort |

## Reporting a Vulnerability

**Do NOT open a public GitHub Issue for security vulnerabilities.**

Instead, report via **GitHub Security Advisories**:

1. Go to https://github.com/ResakaGit/RESONANCE/security/advisories
2. Click "New draft security advisory"
3. Fill in the details (affected component, severity, reproduction steps)

Alternatively, email the maintainer directly (see GitHub profile).

### Response Timeline

| Step | Target |
|------|--------|
| Acknowledgment | 48 hours |
| Initial assessment | 7 days |
| Fix for Critical (CVSS ≥ 9.0) | 7 days |
| Fix for High (CVSS ≥ 7.0) | 30 days |
| Fix for Medium/Low | Next release |

## Security Measures in Place

| Measure | Implementation |
|---------|---------------|
| Dependency scanning | `cargo audit` in CI + Dependabot alerts |
| Dependency pinning | `Cargo.lock` committed, `--locked` builds |
| Zero `unsafe` policy | Hard block — zero tolerance, no exceptions |
| Determinism | Hash-based RNG (`blueprint/equations/determinism.rs`), bit-exact |
| No network | No networking crates in `Cargo.toml`, no runtime connections |
| No patient data | No DICOM, HL7, FHIR — only abstract energy units (qe) |
| Immutable audit trail | Git commit history (timestamped, attributed) |
| Signed commits | GPG/SSH signing (see `docs/PRE_DEPLOY_MANUAL_STEPS.md`) |

## Dependencies

RESONANCE has 14 direct runtime dependencies. All are from crates.io with permissive licenses (MIT/Apache-2.0). See:
- [docs/regulatory/03_traceability/SOUP_ANALYSIS.md](./docs/regulatory/03_traceability/SOUP_ANALYSIS.md) — risk assessment per crate
- [docs/regulatory/03_traceability/SBOM.md](./docs/regulatory/03_traceability/SBOM.md) — full bill of materials

## Cybersecurity Plan

For the complete threat model (STRIDE), mitigations, and patching cadence, see [docs/regulatory/07_release/CYBERSECURITY_PLAN.md](./docs/regulatory/07_release/CYBERSECURITY_PLAN.md).
