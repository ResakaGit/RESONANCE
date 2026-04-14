# MD-6: Molecular Topology

**Effort:** 1 week | **Blocked by:** MD-5 | **Blocks:** MD-13

**ADR:** [ADR-021 Bonded Force Architecture](../../arquitectura/ADR/ADR-021-bonded-force-architecture.md)

## Problem

MD-5 adds bonded force math, but bond/angle/dihedral lists are inline in
experiments. A proper molecular topology data structure is needed to:
- Define connectivity graphs (which atoms are bonded)
- Group atoms into residues (for Go model, analysis)
- Map atom types to force field parameters (for MD-13)

## Solution

`batch/topology.rs` — a standalone data structure describing molecular
connectivity. Immutable during force computation (bond breaking is a separate
phase). No heap allocation in force loops (pre-built lists).

## Implementation

### `batch/topology.rs`

```rust
#[derive(Clone, Debug)]
pub struct BondParams { pub r0: f32, pub k: f32 }

#[derive(Clone, Debug)]
pub struct AngleParams { pub theta0: f32, pub k: f32 }

#[derive(Clone, Debug)]
pub struct DihedralParams { pub k: f32, pub n: u8, pub delta: f32 }

#[derive(Clone, Debug)]
pub struct Topology {
    pub bonds: Vec<(u16, u16, BondParams)>,
    pub angles: Vec<(u16, u16, u16, AngleParams)>,
    pub dihedrals: Vec<(u16, u16, u16, u16, DihedralParams)>,
    pub residues: Vec<ResidueInfo>,
    pub atom_types: Vec<u8>,
    pub n_atoms: usize,
}

#[derive(Clone, Debug)]
pub struct ResidueInfo {
    pub name: [u8; 4],        // e.g. b"ALA\0"
    pub first_atom: u16,
    pub atom_count: u16,
}
```

### Builders

```rust
impl Topology {
    pub fn new(n_atoms: usize) -> Self
    pub fn add_bond(&mut self, i: u16, j: u16, params: BondParams)
    pub fn add_angle(&mut self, i: u16, j: u16, k: u16, params: AngleParams)
    pub fn add_dihedral(&mut self, i: u16, j: u16, k: u16, l: u16, params: DihedralParams)
    pub fn add_residue(&mut self, info: ResidueInfo)
    pub fn infer_angles_from_bonds(&mut self)   // auto-detect i-j-k from bond graph
    pub fn infer_dihedrals_from_bonds(&mut self) // auto-detect i-j-k-l
}
```

### Bonded force system: `batch/systems/bonded_forces.rs`

```rust
pub fn compute_bonded_forces(
    positions: &[[f32; 2]],
    topology: &Topology,
    forces: &mut [[f64; 2]],
)
```

Iterates topology lists, calls `equations::bonded::*`, accumulates into force array.

## Tests

| Test | Criterion |
|------|-----------|
| `topology_add_bond` | Bond stored and retrievable |
| `topology_infer_angles` | i-j, j-k bonds → angle i-j-k detected |
| `topology_infer_dihedrals` | i-j, j-k, k-l bonds → dihedral detected |
| `topology_residue_boundaries` | Atoms correctly assigned to residues |
| `bonded_forces_linear_chain` | N-particle chain has N-1 bonds, N-2 angles |

## Acceptance Criteria

- [x] `batch/topology.rs` with Topology struct + builders
- [x] `batch/systems/bonded_forces.rs` with force accumulation
- [x] Angle/dihedral inference from bond graph
- [x] >= 5 tests
- [x] All existing tests pass
