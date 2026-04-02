# ADR-006: Trunk-Based Development as Change Control

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** IEC 62304 Configuration Management and Change Control

## Context

IEC 62304 Section 8 requires configuration management including: identification of configuration items, change control processes, configuration status accounting, and configuration audits. The standard expects that changes to medical device software are controlled, reviewed, and traceable.

Traditional approaches in regulated software development use feature branches, merge requests with mandatory reviewer approval, and gated CI pipelines before any change reaches the main branch. These processes assume a multi-developer team where peer review serves as a quality gate.

RESONANCE is a single-developer project where the developer is also the reviewer. The question is which branching strategy provides the best balance of change control rigor and development velocity for this context.

## Decision

Use **trunk-based development** (single `main` branch, no long-lived feature branches) as the primary change control strategy.

Change control is provided at two levels:

1. **Commit level:** Each commit to `main` represents a discrete, tested change. The developer runs `cargo test` (3,113 tests) and `cargo clippy` (zero warnings) before pushing. Commit messages document the change rationale.

2. **Sprint level:** Changes are organized into sprints with explicit scope, acceptance criteria, and Definition of Done. Sprint closures (78 archived) provide structured change control at a higher granularity than individual commits. Each sprint closure verifies: all tests pass, zero warnings, DOD met, grep-verified acceptance criteria.

The combination provides: atomic change tracking (Git commits) + structured change packages (sprints) + regression verification (3,113 tests) + immutable history (Git SHA integrity).

## Consequences

### Positive
- Zero merge complexity -- no feature branches diverging, no merge conflicts, no stale branches.
- All changes are immediately visible in `main` -- no hidden work-in-progress in long-lived branches.
- Sprint-based change control provides review granularity that matches IEC 62304 expectations (planned changes, verified outcomes).
- Simple mental model: `main` is always the current state, `git log` is the complete change history.

### Negative
- No pre-merge review gate -- changes reach `main` without a second pair of eyes. In a multi-developer team, this would be unacceptable.
- A broken commit directly impacts `main`. No staging branch to catch regressions before they affect the primary branch.
- `git push --force` could destroy history. No technical safeguard currently in place (GitHub branch protection planned in RI-1).

### Risks
- If a second developer joins, trunk-based development without branch protection becomes risky. Mitigation: RI-1 sprint plans branch protection rules and mandatory CI checks before merge.
- An auditor may expect feature branch + PR workflow as evidence of change control. Mitigation: sprint closures provide equivalent (arguably stronger) evidence of controlled change packages with verification.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Gitflow (develop + feature + release + hotfix branches) | Overkill for a single developer. Creates merge debt, stale branches, and ceremony without a second reviewer to provide the intended quality gate. |
| Feature branches with pull requests | Adds process ceremony (create branch, open PR, self-review, merge) without adding quality -- a single developer self-reviewing a PR is theater, not review. |
| Release branches | Premature for a pre-1.0 project. No deployed production instances requiring hotfix isolation. |
| No version control discipline | Non-compliant with IEC 62304 Section 8. Would make change traceability impossible. |

## References

- `docs/regulatory/01_foundation/RD-3.4` -- Configuration Management Plan, Section 5
- `docs/regulatory/01_foundation/RD-1.4` -- Software Development Plan, Section 6
- IEC 62304:2006+A1:2015, Section 8 (Software Configuration Management)
- `docs/sprints/archive/` -- 78 sprint closure records
- RI-1 sprint plan -- GitHub Actions CI + branch protection
