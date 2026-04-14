# MD-14: Solvated Peptide

**Effort:** 1 week | **Blocked by:** MD-10, MD-11, MD-12, MD-13 | **Blocks:** —

## Problem

MD-9 (peptide in vacuum) lacks the solvation environment that drives real
folding. This sprint combines peptide + explicit water + electrostatics.

## Implementation

### Binary: `src/bin/peptide_solvated.rs`

1. Load alanine dipeptide (hardcoded or from FF loader)
2. Place in center of TIP3P water box (MD-10)
3. Apply SHAKE constraints on water (MD-11)
4. Coulomb via Ewald or reaction field (MD-12)
5. Thermostat at 300K
6. Measure: hydration shell RDF, phi/psi angles

### Setup

- Peptide: 22 atoms (alanine dipeptide)
- Water: ~500-1000 molecules (1500-3000 atoms)
- Box: ~30 A per side
- dt: 2 fs (SHAKE allows larger step)
- Equilibration: 50 ps, Production: 100 ps

## Validation

| Observable | Expected | Tolerance |
|-----------|----------|-----------|
| Water density in bulk | 0.997 g/cm^3 | +/- 0.01 |
| Hydration shell peak | ~3.0 A from peptide O/N | +/- 0.3 |
| Peptide stable | No atom > 10 A from initial | — |
| phi/psi more constrained | Narrower basins than vacuum | Qualitative |

## Acceptance Criteria

- [x] Solvated peptide binary runs stably for 100 ps
- [x] Water density correct
- [x] Hydration shell visible in RDF
- [x] phi/psi basins narrower than vacuum (solvent effect)
