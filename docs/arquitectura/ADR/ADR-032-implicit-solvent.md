# ADR-032: Implicit Solvent Option (GB/SA)

**Status:** Proposed | **Sprint:** MD_REFACTOR R5 | **Date:** 2026-04-13

## Context

Explicit water (TIP3P, MD-10) is accurate but expensive:
- 80% of atoms are water
- 80% of computation is water-water interactions
- Water equilibration takes 50% of simulation time

Implicit solvent replaces water with a continuum dielectric model.
The protein "feels" the solvent effect without simulating individual
water molecules.

## Options

### Option A: Generalized Born / Surface Area (GB/SA)
- **Pro:** 10-100x faster than explicit water. No PBC needed.
- **Con:** Misses specific water-protein interactions (bridges, channels)
- **Effort:** 2 weeks
- **Reference:** Still et al., JACS 1990; Onufriev et al., Proteins 2004

### Option B: Distance-Dependent Dielectric (simple)
- **Pro:** Trivial to implement: ε(r) = 4r instead of ε = 1
- **Con:** Very crude. Misses solvation free energy entirely.
- **Effort:** 1 day

### Option C: Reference Interaction Site Model (RISM)
- **Pro:** More accurate than GB/SA, includes solvent structure
- **Con:** Complex integral equation solver, FFT required
- **Effort:** 4 weeks

## Recommendation

**Option A** (GB/SA). It's the standard in pharma (used by Schrödinger's Prime,
AMBER's igb=8). For Go model folding, GB/SA is often better than explicit water
because it removes the noise of water molecule collisions.

**Option B** as interim solution — one line of code, immediate benefit for
quick folding scans.

## Connection to Axiom 8

Implicit solvent with Axiom 8: the solvent dielectric modulates frequency
coherence. Water as a medium has its own characteristic frequency. The
GB/SA model can incorporate a frequency-dependent screening:

```
ε_eff(r, f_i, f_j) = ε_bulk * (1 - alignment(f_water, f_avg) * exp(-r/λ_D))
```

This makes implicit solvent frequency-aware — another original contribution.

## Axiom compliance

Axiom 1: Solvent energy captured as continuum contribution to qe.
Axiom 4: Solvent dissipation modeled via friction coefficient (Langevin γ).
Axiom 7: GB screening decays with distance (Born radii).
Axiom 8: Optional frequency-dependent dielectric — novel extension.
