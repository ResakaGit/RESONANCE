# MD-13: Force Field Loader

**Effort:** 2 weeks | **Blocked by:** MD-6 | **Blocks:** MD-14

## Problem

Bond/angle/dihedral/LJ/charge parameters are currently hardcoded per
experiment. A real peptide simulation needs hundreds of parameters from
standard force fields (AMBER, OPLS).

## Solution

Minimal parser for AMBER-format parameter files. Not a full AMBER suite —
only extract essentials: atom types, bond params, angle params, dihedral
params, LJ params, charges.

## Implementation

### `batch/ff/mod.rs`

```rust
pub struct ForceFieldParams {
    pub atom_types: Vec<AtomTypeInfo>,
    pub bond_params: Vec<(String, String, BondParams)>,
    pub angle_params: Vec<(String, String, String, AngleParams)>,
    pub dihedral_params: Vec<(String, String, String, String, DihedralParams)>,
    pub lj_params: Vec<(String, f64, f64)>,  // type, sigma, epsilon
}

pub fn load_amber_params(data: &str) -> Result<ForceFieldParams, String>
```

### `batch/ff/amber.rs`

Parse AMBER `.dat` format:
- MASS section: atom types + masses
- BOND section: k_bond, r_eq
- ANGLE section: k_angle, theta_eq
- DIHE section: k_phi, n, delta
- NONBON section: sigma, epsilon

### Parameter assignment

Given a Topology (MD-6) and ForceFieldParams:

```rust
pub fn assign_parameters(topology: &mut Topology, ff: &ForceFieldParams)
```

Maps atom types in topology to force field entries, fills BondParams etc.

## Tests

| Test | Criterion |
|------|-----------|
| `parse_amber_bond_section` | Correct k and r_eq extracted |
| `parse_amber_lj_section` | Correct sigma and epsilon |
| `assign_params_alanine` | All bonds/angles get parameters |
| `unknown_atom_type_error` | Clear error for missing params |

## Acceptance Criteria

- [x] AMBER .dat parser in `batch/ff/amber.rs`
- [x] Parameter assignment to Topology
- [x] Alanine dipeptide params loadable
- [x] >= 4 tests
