# MD-11: SHAKE Constraint Solver

**Effort:** 1 week | **Blocked by:** MD-10 | **Blocks:** MD-14

## Problem

TIP3P water has rigid bond lengths. Harmonic bonds with high k require tiny
timesteps. SHAKE enforces exact bond lengths as holonomic constraints,
allowing larger dt while maintaining geometry.

## Theory

After Verlet position update, positions violate constraints. SHAKE iteratively
corrects positions to satisfy `|r_i - r_j| = d_ij` for each constraint.
RATTLE does the same for velocities.

## Implementation

### `blueprint/equations/constraints.rs`

```rust
/// One SHAKE iteration for a single distance constraint.
/// Returns position corrections (delta_i, delta_j).
pub fn shake_pair(
    r_i: [f64; 3], r_j: [f64; 3],
    r_i_old: [f64; 3], r_j_old: [f64; 3],
    d_target: f64, m_i: f64, m_j: f64,
) -> ([f64; 3], [f64; 3])

/// RATTLE velocity correction for a constrained pair.
pub fn rattle_pair(
    r_i: [f64; 3], r_j: [f64; 3],
    v_i: [f64; 3], v_j: [f64; 3],
    d_target: f64, m_i: f64, m_j: f64,
) -> ([f64; 3], [f64; 3])

/// Iterate SHAKE until convergence (max_iter, tolerance).
pub fn shake_solve(
    positions: &mut [[f64; 3]],
    old_positions: &[[f64; 3]],
    constraints: &[(u16, u16, f64)],
    masses: &[f64],
    tolerance: f64,
    max_iter: u32,
) -> u32  // iterations used
```

### Pipeline

```
Verlet position step → SHAKE position correction → forces → Verlet velocity → RATTLE velocity
```

### Axiom mapping

Axiom 2 (Pool Invariant): constraint forces do no work (perpendicular to
constraint surface), so no energy is created or destroyed.

## Tests

| Test | Criterion |
|------|-----------|
| `shake_maintains_bond_length` | |r_i - r_j| = d +/- tolerance after correction |
| `shake_converges` | < 10 iterations for water |
| `shake_preserves_energy` | KE + PE unchanged by constraint |
| `rattle_zero_constraint_velocity` | v_ij . r_ij = 0 after correction |
| `water_geometry_rigid` | O-H and H-H distances stable over 10K steps |

## Acceptance Criteria

- [x] SHAKE + RATTLE in `equations/constraints.rs`
- [x] Iterative solver with convergence check
- [x] Water geometry maintained to tolerance 1e-6
- [x] Energy conservation not degraded
- [x] >= 5 tests
