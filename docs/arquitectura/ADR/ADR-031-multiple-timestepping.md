# ADR-031: Multiple Timestepping (r-RESPA)

**Status:** Proposed | **Sprint:** MD_REFACTOR R5 | **Date:** 2026-04-13

## Context

Different force components vary at different rates:
- Bonded (bonds, angles): change fast → need small dt
- Non-bonded short-range (LJ): change moderately → medium dt
- Non-bonded long-range (Coulomb/Ewald reciprocal): change slowly → large dt

Currently all forces use the same dt. This wastes computation on slowly-varying
forces that don't need frequent updates.

r-RESPA (reversible Reference System Propagator Algorithm) evaluates different
force groups at different timesteps while maintaining time-reversibility.

## Scheme

```
Inner loop (dt_inner = 1 fs):     bonded forces
Middle loop (dt_mid = 2 fs):      short-range non-bonded (LJ + real Ewald)
Outer loop (dt_outer = 4 fs):     long-range (reciprocal Ewald)
```

Effective speedup: ~3x for solvated systems (long-range is 60% of cost).

## Options

### Option A: 2-level r-RESPA (bonded/non-bonded split)
- **Pro:** Simple, 2x speedup, well-tested
- **Con:** Doesn't separate short/long-range
- **Effort:** 3 days
- **Reference:** Tuckerman et al., JCP 1992

### Option B: 3-level r-RESPA (bonded/short/long split)
- **Pro:** 3-4x speedup, used by NAMD
- **Con:** More complex pipeline, resonance artifacts at certain dt ratios
- **Effort:** 1 week
- **Reference:** Grubmüller et al., Mol. Simul. 1991

### Option C: Langevin middle integrator
- **Pro:** Better thermostat stability than velocity Verlet + Langevin
- **Con:** Not directly a timestepping improvement, but enables larger dt
- **Effort:** 2 days
- **Reference:** Zhang et al., JCP 2019

## Recommendation

**Option A** first (simple 2-level), then **B** if profiling shows long-range
is the bottleneck (likely after PME implementation).

**Option C** is complementary — can be combined with either A or B.

## Axiom compliance

Axiom 4 (Dissipation): each timescale has its own dissipation rate.
r-RESPA respects this naturally — fast modes dissipate at inner dt,
slow modes at outer dt.

Axiom 7 (Distance Attenuation): the force split is exactly along
distance ranges — short-range forces update fast, long-range slow.
