# ADR-030: SETTLE vs SHAKE for Water Constraints

**Status:** Proposed | **Sprint:** MD_REFACTOR R5 | **Date:** 2026-04-13

## Context

SHAKE is iterative (~5 iterations for water). For a system with 1000 water
molecules (3000 atoms, 2000 O-H constraints), SHAKE dominates constraint time.

SETTLE is an algebraic (non-iterative) constraint solver specifically for
3-site water models. It solves the exact analytic equations for 3 atoms in
one pass. Used by GROMACS since 1992.

## Options

### Option A: SETTLE for water, keep SHAKE for general constraints
- **Pro:** 3-5x speedup for water constraints. Exact (no convergence issues).
- **Con:** Only works for 3-site models (TIP3P, SPC). More code.
- **Effort:** 3 days
- **Reference:** Miyamoto & Kollman, J. Comput. Chem. 1992

### Option B: LINCS for all constraints
- **Pro:** Replaces both SHAKE and SETTLE. Parallelizable. O(1) per constraint.
- **Con:** Less accurate than SHAKE for coupled constraints (water OK).
- **Effort:** 1 week
- **Reference:** Hess et al., J. Comput. Chem. 1997

### Option C: Keep SHAKE, optimize iteration
- **Pro:** Already working. Minimal risk.
- **Con:** Still iterative. Can't parallelize easily.
- **Effort:** 1 day

## Recommendation

**Option A.** SETTLE for water (the bottleneck), SHAKE for everything else.
This is exactly what GROMACS does and it's proven optimal for mixed systems.

LINCS (Option B) is better long-term but higher implementation risk.

## Axiom compliance

Axiom 2 (Pool Invariant): SETTLE is exact — constraint forces do exactly
zero work. No iterative convergence error accumulating over millions of steps.
