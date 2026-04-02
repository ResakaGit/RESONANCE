# ADR-007: GitHub Actions for CI/CD Pipeline

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** IEC 62304 Verification and Continuous Integration

## Context

The regulatory audit checklist (AUDIT_CHECKLIST Section 5.2) identifies "no CI/CD pipeline" as a Low-severity structural gap. Currently, all verification is performed locally by the developer before pushing to `main`: `cargo test` (3,113 tests), `cargo clippy` (zero warnings), and manual sprint DOD verification.

While local verification has been effective (78 sprint closures, zero known regression escapes), it relies entirely on developer discipline. There is no automated enforcement that prevents a broken commit from reaching `main`. IEC 62304 Section 5.5 (Software Integration and Integration Testing) and Section 5.7 (Software Verification) benefit from automated, reproducible verification that does not depend on a specific developer's local environment.

Sprint RI-1 targets closing this gap.

## Decision

Use **GitHub Actions** as the CI/CD platform for automated testing, security scanning, and release artifact generation.

Planned pipeline stages:

| Stage | Tool | Purpose |
|-------|------|---------|
| Build check | `cargo check` | Fast compilation verification |
| Lint | `cargo clippy -- -D warnings` | Zero-warning enforcement |
| Test | `cargo test` | 3,113 tests, all must pass |
| Security audit | `cargo audit` | Known vulnerability detection in dependencies |
| Artifact | `cargo build --release` | Release binary generation |

The pipeline will run on every push to `main` and on pull requests (if the branching model evolves per ADR-006). Results are visible in the GitHub repository, providing an independent, timestamped verification record.

## Consequences

### Positive
- Automated, environment-independent verification -- catches "works on my machine" failures.
- Timestamped CI run records provide auditable verification evidence beyond local test runs.
- `cargo audit` adds dependency vulnerability scanning that is not currently performed systematically.
- Free for public repositories -- no cost impact for the AGPL-3.0 project.
- YAML pipeline definition is versioned alongside code in the repository.

### Negative
- Adds ~5-10 minutes per push for full pipeline execution (Rust compilation is slow).
- GitHub Actions runner environment may differ from development environment (macOS local vs. Ubuntu runner) -- potential for environment-specific test failures.
- Dependency on GitHub infrastructure -- if GitHub Actions has an outage, CI is unavailable.

### Risks
- CI pipeline becomes a false sense of security if tests are incomplete or flaky. Mitigation: 3,113 existing tests with property-based fuzzing provide strong baseline; test coverage is tracked per sprint.
- Pipeline configuration (YAML) can become complex and hard to maintain. Mitigation: start with a minimal pipeline and expand incrementally.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Self-hosted CI (Jenkins, Drone) | Maintenance burden for infrastructure. No benefit over GitHub Actions for a public repository. |
| GitLab CI | Would require migrating the repository from GitHub to GitLab, or maintaining a mirror. Unnecessary complexity. |
| CircleCI / Travis CI | Additional external service with free-tier limitations. GitHub Actions is natively integrated and sufficient. |
| No CI (status quo) | Identified as a structural gap in the audit checklist. Local verification works but lacks independent, timestamped evidence and automated enforcement. |

## References

- `docs/regulatory/01_foundation/RD-7.5` -- Release Package Documentation
- `docs/regulatory/01_foundation/RD-3.4` -- Configuration Management Plan, Section 10
- RI-1 sprint documentation -- CI/CD pipeline implementation plan
- IEC 62304:2006+A1:2015, Section 5.5 (Integration Testing), Section 5.7 (Verification)
- GitHub Actions documentation: https://docs.github.com/en/actions
