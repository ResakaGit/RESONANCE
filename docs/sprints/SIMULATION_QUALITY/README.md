# Track — Simulation Quality (SQ)

Master index: [`../README.md`](../README.md).

Refactor track for `src/simulation/` — code smells, anti-patterns, SRP violations.
Zero behavior change; pure quality improvement.

Extracted from STRUCTURE_MIGRATION after SM-1–SM-7 closure.

## Sprints

| Sprint | File | Scope | Status |
|--------|------|-------|--------|
| **SM-8** | [SPRINT_SM8_SIMULATION_CODE_QUALITY.md](SPRINT_SM8_SIMULATION_CODE_QUALITY.md) | `simulation/thermodynamic/`, `reactions.rs`, `input.rs` — god-systems, magic numbers, SRP violations | ⏳ |

## Context

SM-8 was authored during the STRUCTURE_MIGRATION track as a post-audit. The `simulation/metabolic/` subdirectory scores 9.7/10 and is considered reference-quality. Work targets the remaining 30% surface: `thermodynamic/` (2 god-systems), root `reactions.rs`, and `input.rs`.

## References

- `CLAUDE.md` — Coding rules (max 4 fields, one system one transformation, math in blueprint/equations)
- `docs/arquitectura/` — Module contracts
