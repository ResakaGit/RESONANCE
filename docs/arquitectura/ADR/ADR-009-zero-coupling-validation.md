# ADR-009: Zero-Coupling Paper Validation

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** Paper validation track (PV-1 through PV-5)

## Context

RESONANCE needs to validate against 5+ published papers. Each validation is an experiment with its own config, protocol, and success criteria. The question is whether to integrate these into existing experiment infrastructure or isolate them.

## Decision

Each paper validation is a single new file (`use_cases/experiments/paper_*.rs`) that imports existing `batch/` and `equations/` but creates zero dependencies in the other direction. No existing file is modified except 1 line per module in `mod.rs` and 1 `[[bin]]` entry in `Cargo.toml`.

Contract: `PaperXConfig → run_paper_x() → PaperXReport`. Pure function. No shared state. No new crates.

## Consequences

### Positive
- Deleting any paper validation removes zero functionality from the engine
- Each validation is independently compilable and testable
- No risk of regression in existing 3,113 tests
- New validations can be added by anyone without understanding the full codebase

### Negative
- Some code duplication across paper experiments (arena setup, population counting)
- Cannot share intermediate results between experiments without explicit wiring

### Risks
- If `batch/` API changes, all paper_*.rs files need updating (mitigated: batch API is stable, 156 tests)

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Extend `pathway_inhibitor_exp.rs` with paper configs | Couples unrelated papers to one module, bloats a 1,300-line file |
| Generic `PaperValidation<T>` trait | Premature abstraction — 5 experiments don't share enough structure |
| Separate crate (`resonance-validation`) | Overkill for 5 files, adds workspace complexity |

## References

- `docs/sprints/PAPER_VALIDATION/README.md` — track design
- `src/use_cases/experiments/pathway_inhibitor_exp.rs` — existing pattern (Config → Report)
- CLAUDE.md §Inference Protocol: "Don't add abstractions for one-time operations"
