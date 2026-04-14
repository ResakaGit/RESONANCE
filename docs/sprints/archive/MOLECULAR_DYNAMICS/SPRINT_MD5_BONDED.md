# MD-5: Bonded Potentials

**Effort:** 2 weeks | **Blocked by:** MD-4 | **Blocks:** MD-6, MD-7, MD-9

**ADR:** [ADR-021 Bonded Force Architecture](../../arquitectura/ADR/ADR-021-bonded-force-architecture.md)

## Problem

The MD engine only has non-bonded forces (Coulomb + LJ). Molecules require
bonded interactions: covalent bonds (harmonic), bond angles (harmonic), and
torsion angles (periodic dihedral). Without these, no peptide or protein
simulation is possible.

## Solution

Pure math functions for the three standard bonded potentials, plus a minimal
bond/angle/dihedral list structure for the LJ fluid world. Full topology
(connectivity graph, residues, atom types) is deferred to MD-6.

### Harmonic Bond

```
V(r) = 0.5 * k * (r - r0)^2
F_on_i = k * (r - r0) * (dx/r, dy/r)
```

Convention: positive force component along dx = toward j (attractive when stretched).

### Harmonic Angle

```
V(theta) = 0.5 * k * (theta - theta0)^2
```

Three-body: particles a-b-c, b is vertex. Forces on all three particles
derived from the gradient of V with respect to each position.

### Proper Dihedral (3D only)

```
V(phi) = k * (1 + cos(n*phi - delta))
```

Four-body: particles a-b-c-d. Angle between planes (a,b,c) and (b,c,d).
Implemented with `[f32; 3]` positions (ready for MD-7). Not applicable to
current 2D batch simulator.

## Implementation

### 1. Pure math: `blueprint/equations/bonded.rs`

```rust
// ── Bond ──
pub fn harmonic_bond_energy(r: f32, r0: f32, k: f32) -> f32
pub fn harmonic_bond_force(dx: f32, dy: f32, r0: f32, k: f32) -> [f32; 2]

// ── Angle ──
pub fn angle_from_vectors_2d(ba: [f32; 2], bc: [f32; 2]) -> f32
pub fn harmonic_angle_energy(theta: f32, theta0: f32, k: f32) -> f32
pub fn harmonic_angle_forces_2d(
    a: [f32; 2], b: [f32; 2], c: [f32; 2],
    theta0: f32, k: f32,
) -> [[f32; 2]; 3]

// ── Dihedral (3D) ──
pub fn dihedral_from_positions_3d(
    a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3],
) -> f32
pub fn dihedral_energy(phi: f32, k: f32, n: u8, delta: f32) -> f32
pub fn dihedral_forces_3d(
    a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3],
    k: f32, n: u8, delta: f32,
) -> [[f32; 3]; 4]
```

### 2. Bond list for LjWorld: `use_cases/experiments/lj_fluid.rs`

Minimal inline structure (not a full topology — that's MD-6):

```rust
struct BondDef { i: u16, j: u16, r0: f32, k: f32 }
struct AngleDef { i: u16, j: u16, k_idx: u16, theta0: f32, k: f32 }
```

Bonded forces accumulated alongside non-bonded in the same force array.
Newton 3 respected: f_on_i = -f_on_j for bonds.

### 3. Pipeline placement

Bonded forces computed AFTER non-bonded, added to the same force accumulator:

```
non-bonded forces (cell list / brute)
  +
bonded forces (bond list iteration)
  =
total force → Verlet velocity step
```

## Axiom Mapping

| Feature | Axiom | Derivation |
|---------|-------|-----------|
| Harmonic bond | 8 | Small-amplitude limit of oscillatory interaction |
| Harmonic angle | 8 | Angular oscillation at equilibrium geometry |
| Dihedral | 8 | Periodic oscillation around torsion axis |

## Tests

| Test | Criterion |
|------|-----------|
| `bond_energy_zero_at_equilibrium` | V(r0) = 0 |
| `bond_energy_symmetric` | V(r0+d) = V(r0-d) |
| `bond_force_restoring` | F points toward r0 when stretched/compressed |
| `bond_force_newton3` | f_on_i + f_on_j = 0 |
| `bond_oscillation_period` | Two bonded particles oscillate at sqrt(k/m) |
| `angle_energy_zero_at_equilibrium` | V(theta0) = 0 |
| `angle_force_sum_zero` | f_a + f_b + f_c = 0 (Newton 3) |
| `angle_restores_to_equilibrium` | Angle converges to theta0 under dynamics |
| `dihedral_periodic` | V(phi + 2pi/n) = V(phi) |
| `dihedral_forces_sum_zero` | f_a + f_b + f_c + f_d = 0 |

## Acceptance Criteria

- [x] `blueprint/equations/bonded.rs` with bond, angle, dihedral functions
- [x] Harmonic bond: energy + force (2D), >= 5 tests
- [x] Harmonic angle: energy + force (2D), >= 3 tests
- [x] Proper dihedral: energy + force (3D), >= 2 tests
- [x] Bond oscillation test passes (correct frequency)
- [x] All existing tests pass (regression)
