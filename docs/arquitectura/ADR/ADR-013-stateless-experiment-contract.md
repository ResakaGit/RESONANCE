# ADR-013: Stateless Experiment Contract (Config → Report)

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** All experiments (existing + paper validation track)

## Context

RESONANCE has 15+ experiment modules. Each needs a consistent interface for composition (orchestrators), testing (BDD), and binary execution (CLI). The question is what contract to enforce.

## Decision

Every experiment is a pure function: `Config → Report`. No side effects, no global state, no IO.

```rust
// Config: all parameters needed to reproduce the experiment
pub struct XConfig { /* fields with pub, Clone, Debug */ }
impl Default for XConfig { /* paper-derived defaults */ }

// Report: all results, no interpretation
pub struct XReport { /* fields with pub, Clone, Debug */ }

// Pure function: deterministic, no IO, no RNG crate
pub fn run_x(config: &XConfig) -> XReport { ... }
```

**Rules:**
1. `run_x` never prints, never reads files, never accesses network
2. All randomness from `determinism::next_u64` (hash-based, no external RNG)
3. `XConfig::default()` reproduces the paper's setup — the binary just calls `run_x(&XConfig::default())`
4. Binaries handle IO: parse CLI args, call `run_x`, format output
5. Tests call `run_x` directly — no binary execution, no subprocess

## Consequences

### Positive
- Every experiment is reproducible: same Config → same Report (bit-exact)
- Composable: `orchestrators.rs` can `ablate()`, `ensemble()`, `sweep()` over any experiment
- Testable: BDD assertions on Report fields, no mocking needed
- Parallelizable: no shared state → `rayon::par_iter` over configs trivially

### Negative
- Binary logic (CLI parsing, output formatting) is duplicated per binary
- Cannot incrementally checkpoint long experiments (stateless = no resume)

### Risks
- If an experiment needs statefulness (e.g., interactive feedback), this pattern doesn't fit. Mitigated: adaptive therapy feedback loop is implemented as a state machine inside `run_x`, not as external state.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Trait `Experiment { fn run(&self) -> Box<dyn Report> }` | Trait object adds heap allocation; experiments don't share enough interface to justify |
| Builder pattern (`Experiment::new().with_drug(...).run()`) | More API surface for the same result; Config struct is simpler |
| Side-effectful experiments (print as they run) | Breaks composition, breaks testing, breaks parallelism |

## References

- `src/use_cases/experiments/pathway_inhibitor_exp.rs` — canonical example (InhibitorConfig → InhibitorReport)
- `src/use_cases/experiments/cancer_therapy.rs` — TherapyConfig → TherapyReport
- `src/use_cases/orchestrators.rs` — HOF composition depends on this contract
- CLAUDE.md §Coding Rules #8: "Math in blueprint/equations/. Systems call pure fns."
