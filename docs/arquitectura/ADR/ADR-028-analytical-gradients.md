# ADR-028: Analytical vs Numerical Gradients for Bonded Forces

**Status:** Proposed | **Sprint:** MD_REFACTOR R2 | **Date:** 2026-04-13

## Context

Current bonded forces (angles, dihedrals) use numerical central differences
with step `h = 1e-4`. This requires 6 energy evaluations per force component
(2 per dimension × 3 dimensions = 6 per atom in a 3-body angle).

For a system with 100 angles, this is 600 energy evaluations per step just
for angles. Analytical gradients would require 0 extra evaluations.

## Analysis

### Harmonic angle: analytical is straightforward
```
V = 0.5 * k * (θ - θ₀)²
∂V/∂r_i = k * (θ - θ₀) * ∂θ/∂r_i
```
where `∂θ/∂r_i` has a known closed-form expression (see Allen & Tildesley, App. C).

### Proper dihedral: analytical is messy but doable
```
V = k * (1 + cos(nφ - δ))
∂V/∂r_i = -k * n * sin(nφ - δ) * ∂φ/∂r_i
```
The dihedral angle derivative involves cross products and is error-prone but
well-documented (Bekker et al., GROMACS manual, Ch. 4.9).

## Options

### Option A: Full analytical gradients
- **Pro:** 6x speedup for angles, 8x for dihedrals. Exact (no truncation error).
- **Con:** Complex derivation for dihedrals. Risk of implementation bugs.
- **Effort:** 1 week
- **Reference:** GROMACS manual Ch. 4.2 (angles), 4.9 (dihedrals)

### Option B: Keep numerical, optimize step size
- **Pro:** Zero risk of analytical bugs. Already working.
- **Con:** Still 6-8x slower. Truncation error at `h = 1e-4`.
- **Effort:** 1 day

### Option C: Autodiff via dual numbers
- **Pro:** Exact derivatives, no manual derivation needed.
- **Con:** Requires dual number types, ~2x overhead vs analytical.
- **Effort:** 1 week for dual number infrastructure

## Recommendation

**Option A** for angles (straightforward). **Option A** for dihedrals with
numerical gradient as validation fallback in tests.

Keep current numerical implementation as `_numerical` suffix functions for
cross-validation in `#[cfg(test)]`.

## Axiom compliance

Axiom 1 (Everything is Energy): analytical gradients compute exact ∂E/∂r,
ensuring energy conservation is not degraded by numerical truncation.
