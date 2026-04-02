# Contributing to Resonance

Thank you for your interest in RESONANCE. This document explains how to contribute effectively.

## Before You Start

RESONANCE has **8 inviolable axioms** and **4 fundamental constants** that cannot be changed, weakened, or bypassed by any contribution. If your change conflicts with an axiom, the change is wrong — not the axiom. Read `CLAUDE.md` for the full list.

## Development Workflow

1. **Fork** the repository and clone your fork
2. **Create a branch** from `main` (e.g., `feat/my-feature` or `fix/my-bugfix`)
3. **Make changes** following the coding rules below
4. **Run tests** locally: `cargo test --workspace` (all 3,113+ tests must pass)
5. **Run checks**: `cargo clippy --workspace -- -D warnings` (zero warnings)
6. **Format**: `cargo fmt`
7. **Push** your branch and open a **Pull Request** against `main`
8. CI runs 5 automated checks — all must pass
9. A reviewer (Verificador role) reviews using the PASS/WARN/BLOCK protocol

## Coding Rules

These are the hard constraints. See `CLAUDE.md` for the complete list.

### Absolute (never violate)

- **NO `unsafe`** — zero tolerance, no exceptions
- **NO external crates** without prior approval (only what's in `Cargo.toml`)
- **NO `async`/`await`** — Bevy schedule only
- **NO `Arc<Mutex<T>>`** — use `Resource` or `Local`
- **NO shared mutable state** outside Bevy Resources

### Architecture

- **Math in `blueprint/equations/`** — pure functions, no ECS, no side effects
- **Constants in `constants/`** — tuning values centralized per module
- **One system, one transformation** — no god-systems (>5 component types)
- **Max 4 fields per component** — split into layers if more
- **Phase assignment required** — every gameplay system in `FixedUpdate` + `Phase::X`
- **`#[derive(Component, Reflect, Debug, Clone)]`** on every component

### Style

- English identifiers only
- `///` doc comments: bilingual (Spanish first, English below), one line each
- Dense but readable — vertical alignment, early return, functional over imperative
- Test naming: `<function>_<condition>_<expected>`

## What to Contribute

### Good first contributions

- Tests for untested pure functions in `blueprint/equations/`
- Documentation improvements in `docs/`
- Bug fixes with a failing test that proves the bug

### Needs discussion first

- New ECS layers (must pass the 5-question orthogonality test in `CLAUDE.md`)
- New dependencies (must justify against the no-external-crates rule)
- Changes to emergence systems (may affect simulation behavior)

### Do not submit

- Changes to the 8 axioms or 4 fundamental constants
- `unsafe` code
- Code that bypasses the `Phase` scheduling system
- Inline formulas in systems (must be in `blueprint/equations/`)

## Testing

```bash
cargo test --workspace          # All tests (must pass, ~36 sec)
cargo clippy -- -D warnings     # Zero warnings
cargo fmt --check               # Formatting
cargo audit                     # No known vulnerabilities
```

Tests are organized in 4 layers:
- **Unit** — pure math in `blueprint/equations/` (boundary inputs, invariants)
- **Integration** — `MinimalPlugins` + spawn + update + assert
- **Property** — `proptest` fuzz in `tests/property_conservation.rs`
- **Batch** — headless simulator in `src/batch/` (156 tests, no Bevy)

## Review Process

PRs are reviewed by the **Verificador** role using this checklist:

1. **Contract** — does the change respect interfaces and invariants?
2. **Math** — are equations correct? Are they in `blueprint/equations/`?
3. **DOD (Definition of Done)** — coding rules followed?
4. **Determinism** — does the change preserve bit-exact reproducibility?
5. **Performance** — no regressions in hot paths?
6. **Tests** — new behavior has tests? Existing tests still pass?

Verdict: **PASS** (merge), **WARN** (merge with noted concern), **BLOCK** (must fix before merge). Math or determinism doubt = automatic BLOCK.

## Regulatory Note

RESONANCE maintains voluntary compliance documentation per IEC 62304, ISO 14971, and ISO 13485. Changes to `docs/regulatory/` require CCB (Change Control Board) review via the `ccb-review` label. See [docs/regulatory/05_quality_system/CCB_CHARTER.md](./docs/regulatory/05_quality_system/CCB_CHARTER.md).

## Communication

- **GitHub Issues** — bugs, features, regulatory feedback (templates provided)
- **Pull Requests** — all code changes
- **Tone:** peer-to-peer, direct, professional
- **Language:** Spanish default in comments, English for identifiers and docs

## License

By contributing, you agree that your contributions will be licensed under AGPL-3.0.
