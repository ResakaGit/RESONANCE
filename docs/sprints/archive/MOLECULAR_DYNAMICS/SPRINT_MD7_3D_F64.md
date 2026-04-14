# MD-7: 3D + f64 Upgrade

**Effort:** 2 weeks | **Blocked by:** MD-5 | **Blocks:** MD-8, MD-9, MD-10, MD-12

**ADR:** [ADR-022 3D/f64 Migration Strategy](../../arquitectura/ADR/ADR-022-3d-f64-migration.md)

## Problem

EntitySlot uses `[f32; 2]` for position/velocity. Molecular dynamics requires
3D (proteins fold in 3D) and f64 precision (accumulated error over millions
of steps in f32 causes drift).

## Solution

Phased migration with backward compatibility:
1. Add `position_3d: [f64; 3]` and `velocity_3d: [f64; 3]` alongside 2D fields
2. Bridge function: `pos_2d(slot) -> [f32; 2]` for legacy systems
3. Feature gate: `#[cfg(feature = "md_3d")]` — default off for batch runs
4. MD systems use 3D; legacy systems use 2D unchanged

## Implementation

### EntitySlot extension

```rust
#[cfg(feature = "md_3d")]
pub position_3d: [f64; 3],
#[cfg(feature = "md_3d")]
pub velocity_3d: [f64; 3],
#[cfg(feature = "md_3d")]
pub old_acceleration_3d: [f64; 3],
```

+48 bytes per entity. At N=4096: 192KB (fits in L2 cache).

### LjWorld migration

The standalone `LjWorld` in `lj_fluid.rs` migrates to `[f64; 3]` positions
and velocities. This doesn't affect EntitySlot (separate data structure).

### PBC 3D

`pbc.rs` gains `minimum_image_3d` and `wrap_3d`:

```rust
pub fn minimum_image_3d(pos_a: [f64; 3], pos_b: [f64; 3], box_lengths: [f64; 3]) -> [f64; 3]
pub fn wrap_3d(pos: [f64; 3], box_lengths: [f64; 3]) -> [f64; 3]
```

### Cell list 3D

`neighbor_list.rs` gains a 3D variant with 27 neighbor cells (3^3).

### Force computation 3D

`lj_force_reduced` gains a 3D variant. `force_from_displacement` gains 3D.

## Risk: R8 — 2D→3D Migration Breaks Tests

All 33+ existing batch systems assume 2D. The feature gate ensures they compile
unchanged. The 3D path is opt-in via `--features md_3d` or used directly by
the standalone LjWorld.

## Tests

| Test | Criterion |
|------|-----------|
| `position_3d_wrap` | Wraps correctly in all 3 dimensions |
| `minimum_image_3d_symmetric` | d(i,j) = d(j,i) in 3D |
| `lj_force_3d_matches_2d_in_plane` | z=0 for all → same as 2D |
| `cell_list_3d_finds_all_pairs` | Matches brute force in 3D |
| `verlet_3d_energy_conservation` | Drift < 1e-4 in 3D NVE |
| `legacy_2d_tests_unchanged` | All existing tests pass without feature |

## Acceptance Criteria

- [x] 3D positions/velocities in LjWorld (f64)
- [x] PBC wrap + minimum image in 3D
- [x] Cell list 3D (27 neighbors)
- [x] LJ force in 3D
- [x] Verlet integration in 3D
- [x] Feature gate: existing 2D tests unaffected
- [x] >= 6 tests
