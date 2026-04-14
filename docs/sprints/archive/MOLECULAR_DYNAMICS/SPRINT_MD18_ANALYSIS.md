# MD-18: Analysis Suite

**Effort:** 1 week | **Blocked by:** MD-15 | **Blocks:** —

## Problem

MD trajectories produce millions of frames. Need standard analysis tools:
RMSD, radius of gyration, contact maps, potential of mean force (PMF).

## Implementation

### `blueprint/equations/md_analysis.rs`

```rust
/// RMSD between two structures (after optimal alignment via Kabsch).
pub fn rmsd(coords_a: &[[f64; 3]], coords_b: &[[f64; 3]]) -> f64

/// Radius of gyration: Rg = sqrt(sum(m_i * |r_i - r_com|^2) / sum(m_i))
pub fn radius_of_gyration(coords: &[[f64; 3]], masses: &[f64]) -> f64

/// Contact map: NxN matrix of pairwise distances.
pub fn contact_map(coords: &[[f64; 3]], n: usize) -> Vec<f64>

/// Native contact fraction Q: fraction of native contacts present.
pub fn native_fraction(
    coords: &[[f64; 3]],
    native_contacts: &[(u16, u16, f64)],
    tolerance: f64,
) -> f64

/// PMF from histogram: F(x) = -kT * ln(P(x))
pub fn pmf_from_histogram(bins: &[u64], k_b_t: f64) -> Vec<f64>
```

### Kabsch alignment (for RMSD)

Optimal rotation matrix that minimizes RMSD between two coordinate sets.
SVD-based (3x3 matrix — can implement without external crate).

## Tests

| Test | Criterion |
|------|-----------|
| `rmsd_identical_is_zero` | RMSD(A, A) = 0 |
| `rmsd_translated_is_zero` | RMSD invariant under translation |
| `rg_single_atom_is_zero` | Rg = 0 for single particle |
| `rg_sphere` | Rg = sqrt(3/5)*R for uniform sphere |
| `native_fraction_at_native` | Q = 1.0 for native structure |
| `pmf_uniform_is_flat` | F(x) = const for uniform distribution |

## Acceptance Criteria

- [x] RMSD with Kabsch alignment
- [x] Rg, contact map, native Q, PMF
- [x] >= 6 tests
- [x] All functions pure, stateless
