# MD-10: Water Model (TIP3P)

**Effort:** 1 week | **Blocked by:** MD-7 | **Blocks:** MD-11, MD-14

## Problem

Biological molecules exist in water. Solvation effects (hydrophobic effect,
hydrogen bonding) are critical for protein folding. Without explicit water,
the simulation lacks the medium that drives folding.

## Solution

TIP3P (Transferable Intermolecular Potential, 3-Point): simplest rigid water
model. 3 sites (O + 2H), fixed geometry, partial charges.

## TIP3P Parameters

| Parameter | Value |
|-----------|-------|
| O charge | -0.834 e |
| H charge | +0.417 e |
| O-H distance | 0.9572 A |
| H-O-H angle | 104.52 deg |
| O sigma (LJ) | 3.1507 A |
| O epsilon (LJ) | 0.1521 kcal/mol |
| H LJ | none (zero) |

## Implementation

### `batch/ff/water.rs`

```rust
pub const TIP3P_CHARGE_O: f64 = -0.834;
pub const TIP3P_CHARGE_H: f64 = 0.417;
pub const TIP3P_R_OH: f64 = 0.9572;       // Angstrom
pub const TIP3P_ANGLE_HOH: f64 = 104.52;  // degrees
pub const TIP3P_SIGMA_O: f64 = 3.1507;
pub const TIP3P_EPSILON_O: f64 = 0.1521;

pub fn create_water_topology(n_waters: usize) -> Topology
pub fn place_water_box(n_waters: usize, box_length: f64) -> Vec<[f64; 3]>
```

### Rigid geometry

TIP3P water is rigid (fixed bond lengths + angle). Requires SHAKE constraints
(MD-11). For this sprint, use harmonic bonds with very high k as approximation.
True SHAKE in MD-11.

## Validation

| Observable | Expected | Tolerance |
|-----------|----------|-----------|
| Density at 300K, 1 atm | 0.997 g/cm^3 | +/- 0.01 |
| O-O RDF first peak | ~2.76 A | +/- 0.1 |
| O-O RDF coordination number | ~4.5 | +/- 0.5 |

## Acceptance Criteria

- [x] TIP3P constants and topology builder
- [x] Water box initialization (cubic lattice)
- [x] Density within 1% of experimental at 300K
- [x] O-O RDF qualitatively correct
- [x] >= 4 tests
