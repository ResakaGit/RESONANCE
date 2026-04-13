# ADR-026: Shared Math Utilities

**Status:** Proposed | **Sprint:** MD_REFACTOR R1 | **Date:** 2026-04-13

## Context

`erfc_approx()` is duplicated in `thermostat.rs` and `ewald.rs` with identical
magic numbers (Abramowitz & Stegun 7.1.26 coefficients). Degree-to-radian
conversion is computed inline in 6+ locations. TIP3P geometry constants are
hardcoded in `constraints.rs` instead of imported from `water.rs`.

## Decision

Create two shared modules:

### `blueprint/equations/special_functions.rs`
- `erfc_approx(x: f64) -> f64` — single implementation, documented reference
- `erfc_approx_f32(x: f32) -> f32` — f32 variant if needed
- Named constants for all polynomial coefficients

### `blueprint/equations/unit_conversion.rs`
- `DEG_TO_RAD: f64 = PI / 180.0`
- `RAD_TO_DEG: f64 = 180.0 / PI`
- `KCAL_TO_KJ: f64 = 4.184`
- `ANGSTROM_TO_NM: f64 = 0.1`
- Unit-aware conversion functions

### Constant consolidation
- All MD-specific named constants in `batch/constants.rs` or per-module
- `NUMERICAL_GRAD_STEP: f32 = 1e-4` (used in bonded.rs 3 places)
- `BARNES_HUT_THETA: f64 = 0.5`
- `BRUTE_FORCE_THRESHOLD: usize = 64`

## Consequences

- Single source of truth for math approximations
- Bugs fixed in one place propagate everywhere
- Named constants are searchable and auditable
