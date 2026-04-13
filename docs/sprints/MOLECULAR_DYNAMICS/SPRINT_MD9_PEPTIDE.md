# MD-9: Peptide in Vacuum

**Effort:** 1 week | **Blocked by:** MD-5, MD-6, MD-7, MD-8 | **Blocks:** MD-15

**ADR:** [ADR-025 Ewald vs Cutoff Decision](../../arquitectura/ADR/ADR-025-ewald-vs-cutoff-decision.md) (decision gate post-MD-9)

## Problem

Before folding proteins, we need to verify that a small peptide (alanine
dipeptide) samples the correct conformational space. The Ramachandran plot
(phi/psi dihedral angles) is the standard validation.

## Purpose

This is the **decision gate** for the shortcut path:
- If peptide in vacuum samples correct phi/psi → Go model viable without solvent
- Skip Phase 2 (water, Ewald) → save 8 weeks

## Implementation

### Binary: `src/bin/peptide_vacuum.rs`

- Load hardcoded alanine dipeptide geometry (22 atoms, 3D)
- Topology: bonds, angles, dihedrals from AMBER force field (hardcoded params)
- Non-bonded: LJ with cutoff (no Coulomb — vacuum, no long-range needed)
- Thermostat: Langevin at T=300K (reduced units)
- Measure phi/psi every 100 steps
- Run 100K steps, output Ramachandran distribution

### Alanine dipeptide

```
CH3-CO-NH-CH(CH3)-CO-NH-CH3
     phi        psi
```

22 atoms. 2 backbone dihedrals: phi (C-N-CA-C) and psi (N-CA-C-N).
Expected: alpha-helix basin (~-60, -45) and beta-sheet basin (~-135, 135).

### Pure math: `blueprint/equations/md_observables.rs` (extend)

```rust
pub fn ramachandran_bin(phi: f32, psi: f32, n_bins: usize) -> (usize, usize)
```

## Validation

| Observable | Expected | Tolerance |
|-----------|----------|-----------|
| phi/psi sampling | Two basins visible | Qualitative |
| Energy conservation (NVE) | < 1e-3 drift / 10K steps | Strict |
| Bond lengths stable | Within 5% of r0 | Strict |

## Acceptance Criteria

- [x] `src/bin/peptide_vacuum.rs` runs alanine dipeptide
- [x] Ramachandran plot shows two basins (alpha + beta)
- [x] NVE energy conservation passes
- [x] Bond lengths stay near equilibrium
- [x] Decision gate: proceed to Phase 3 shortcut or Phase 2
