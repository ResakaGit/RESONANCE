# MD-8: Cutoff + Shifted LJ

**Effort:** 3 days | **Blocked by:** MD-3, MD-7 | **Blocks:** MD-9

## Problem

LJ potential has a discontinuity at r_cut: V(r_cut-) != 0 but V(r_cut+) = 0.
This creates an energy jump when particles cross the cutoff boundary, breaking
energy conservation and introducing artifacts in pressure measurements.

## Solution

Shifted LJ potential: `V_shifted(r) = V(r) - V(r_cut)` for r < r_cut, 0 otherwise.
Plus analytical tail corrections for thermodynamic properties.

Already partially implemented in `md_observables::lj_potential_reduced` (MD-4).
This sprint formalizes it and adds force-shifted variant + tail corrections.

## Implementation

### `blueprint/equations/md_observables.rs` (extend)

```rust
/// Force-shifted LJ: both potential AND force are continuous at r_cut.
/// V_fs(r) = V(r) - V(r_cut) - (r - r_cut) * dV/dr(r_cut)
pub fn lj_potential_force_shifted(r: f64, r_cut: f64) -> f64

/// LJ tail correction for energy: U_tail = (8/3) * pi * rho * N * epsilon * sigma^3 * [...]
pub fn lj_tail_correction_energy(n: usize, density: f64, r_cut: f64) -> f64

/// LJ tail correction for pressure: P_tail = (16/3) * pi * rho^2 * epsilon * sigma^3 * [...]
pub fn lj_tail_correction_pressure(density: f64, r_cut: f64) -> f64
```

## Tests

| Test | Criterion |
|------|-----------|
| `force_shifted_continuous_at_cutoff` | V(r_cut) = 0 AND dV/dr(r_cut) = 0 |
| `tail_correction_sign` | U_tail < 0, P_tail < 0 (attractive contribution) |
| `tail_correction_scales_with_density` | P_tail(2*rho) / P_tail(rho) = 4 |

## Acceptance Criteria

- [x] Force-shifted potential + tail corrections in `md_observables.rs`
- [x] Energy conservation improved vs simple cutoff
- [x] >= 3 tests
