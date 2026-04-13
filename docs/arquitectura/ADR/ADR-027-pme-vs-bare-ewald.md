# ADR-027: PME vs Bare Ewald

**Status:** Proposed | **Sprint:** MD_REFACTOR R2 | **Date:** 2026-04-13

## Context

Current Ewald implementation is "bare" — O(N * k_max³). For N=1000 with k_max=5,
this is ~1.3M operations per step. GROMACS switched to PME in 2001 because bare
Ewald doesn't scale past N=5000.

PME (Particle Mesh Ewald) uses a 3D FFT grid to compute reciprocal-space
contributions in O(N log N). This is 100x faster for N=10K.

## Options

### Option A: Implement PME with `rustfft` crate
- **Pro:** Standard algorithm, well-understood, O(N log N)
- **Con:** Requires `rustfft` crate approval (Hard Block #2)
- **Effort:** 2 weeks
- **Reference:** Darden et al., JCP 1993

### Option B: Implement bare FFT (no crate)
- **Pro:** No external dependency
- **Con:** FFT from scratch is error-prone; Cooley-Tukey for 3D is ~500 lines
- **Effort:** 3 weeks
- **Reference:** Numerical Recipes Ch. 12

### Option C: Smooth Particle Mesh Ewald (SPME)
- **Pro:** Better accuracy than PME for same grid size
- **Con:** More complex (B-spline interpolation), still needs FFT
- **Effort:** 3 weeks
- **Reference:** Essmann et al., JCP 1995

### Option D: Gaussian Split Ewald (Desmond approach)
- **Pro:** More cache-friendly than PME, better for modern CPUs
- **Con:** Less well-known, fewer reference implementations
- **Effort:** 3 weeks
- **Reference:** Shan et al., JCP 2005

## Recommendation

**Option A** (PME with `rustfft`). Least effort, most validated, standard
in all production MD codes. The `rustfft` crate is pure Rust, no unsafe,
well-maintained (>10M downloads). Request crate approval.

If crate not approved: **Option B** (bare FFT). Radix-2 Cooley-Tukey is
well-understood enough to implement safely.

## Axiom compliance

Axiom 7 (Distance Attenuation): PME computes exact long-range Coulomb in
periodic systems. More accurate than bare Ewald with truncated k-vectors.
