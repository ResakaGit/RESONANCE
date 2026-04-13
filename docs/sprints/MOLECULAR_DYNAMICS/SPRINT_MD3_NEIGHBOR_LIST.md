# MD-3: Neighbor Lists (Cell List)

**Effort:** 1 week | **Blocked by:** MD-2 | **Blocks:** MD-4

## Problem

`accumulate_forces` in `coulomb.rs` is O(N^2): every particle checks every other.
At N=128 this is 8K pairs — fine. At N=1000 it's 500K pairs. At N=4096 (target for
MD fluids) it's 8.4M pairs per tick — too slow for 60 Hz.

But LJ and Coulomb (with cutoff) decay to negligible values beyond r_cut. Most of
those 8.4M pair computations return ~zero force. We only need pairs within r_cut.

## Solution: Cell Lists

Divide the simulation box into cells of side >= r_cut. For each cell, only check
neighboring cells (9 cells in 2D, 27 in 3D). This gives O(N) force computation
for uniform density.

```
+---+---+---+---+
| . | . | x | . |   Particle in cell (2,1) only checks
+---+---+---+---+   cells (1,0)..(3,2) — 9 cells total.
| . | * | x | . |   All other cells are beyond r_cut.
+---+---+---+---+
| . | x | x | . |
+---+---+---+---+
```

### Why Cell List (not Verlet List)

| | Cell List | Verlet List |
|---|---|---|
| Rebuild | Every step (cheap: O(N)) | Every ~10-20 steps |
| Memory | O(N) + cell array | O(N * avg_neighbors) |
| Complexity | Simple | Needs skin parameter + rebuild trigger |
| Correctness risk | None (rebuilt fresh) | Missed rebuilds → wrong forces |

Cell list is simpler and sufficient for N < 10K. Verlet list is an optimization
for larger N (add later if needed).

## Implementation

### 1. Data structure: `batch/neighbor_list.rs`

```rust
/// Cell list for spatial acceleration of pairwise force computation.
pub struct CellList {
    /// Cell dimensions [nx, ny]. Each cell side >= r_cut.
    pub grid_dims: [u32; 2],
    /// Cell side length.
    pub cell_size: f64,
    /// Head-of-chain per cell. head[cell_idx] = first particle index, or u16::MAX.
    pub head: Vec<u16>,
    /// Next-in-chain per particle. next[particle_idx] = next particle in same cell.
    pub next: Vec<u16>,
    /// Cutoff radius squared.
    pub r_cut_sq: f64,
}

impl CellList {
    /// Build cell list from positions. O(N).
    pub fn build(
        positions: &[[f32; 2]],
        alive_mask: u64,
        sim_box: &SimBox,
        r_cut: f64,
    ) -> Self { ... }

    /// Iterate all pairs within r_cut. Calls `f(i, j, dx, dy, r_sq)` for each.
    /// Uses minimum image convention internally.
    pub fn for_each_pair(
        &self,
        positions: &[[f32; 2]],
        sim_box: &SimBox,
        mut f: impl FnMut(usize, usize, f64, f64, f64),
    ) { ... }
}
```

### 2. Cell index computation

```rust
fn cell_index(pos: &[f32; 2], cell_size: f64, grid_dims: &[u32; 2]) -> u32 {
    let cx = ((pos[0] as f64) / cell_size).floor() as u32;
    let cy = ((pos[1] as f64) / cell_size).floor() as u32;
    let cx = cx.min(grid_dims[0] - 1);
    let cy = cy.min(grid_dims[1] - 1);
    cy * grid_dims[0] + cx
}
```

### 3. Neighbor cell iteration (2D)

For cell (cx, cy), check 9 cells: (cx-1..cx+1, cy-1..cy+1).
With PBC: wrap indices modulo grid_dims.

```rust
fn neighbor_cells_2d(cx: u32, cy: u32, dims: &[u32; 2]) -> impl Iterator<Item = u32> {
    (-1i32..=1).flat_map(move |dy| {
        (-1i32..=1).map(move |dx| {
            let nx = (cx as i32 + dx).rem_euclid(dims[0] as i32) as u32;
            let ny = (cy as i32 + dy).rem_euclid(dims[1] as i32) as u32;
            ny * dims[0] + nx
        })
    })
}
```

### 4. Integration with particle_forces

```rust
pub fn particle_forces_with_cells(world: &mut SimWorldFlat, strategy: ForceStrategy, dt: f32) {
    if strategy == ForceStrategy::Disabled { return; }

    let Some(sim_box) = &world.sim_box else {
        // Fallback to O(N^2) for non-PBC worlds
        particle_forces(world, strategy, dt);
        return;
    };

    let r_cut = LJ_CUTOFF_RATIO * LENNARD_JONES_SIGMA as f64;
    let cells = CellList::build(&positions, world.alive_mask, sim_box, r_cut);

    // Zero forces
    // ...

    // Accumulate via cell list
    cells.for_each_pair(&positions, sim_box, |i, j, dx, dy, r_sq| {
        let force = coulomb::net_force_from_rsq(charge_i, charge_j, r_sq, dx, dy);
        forces[i] += force;
        forces[j] -= force;  // Newton 3
    });

    // Apply forces to velocities
    // ...
}
```

### 5. r_cut selection

LJ standard: `r_cut = 2.5 * sigma`. Beyond this, LJ < 1% of well depth.

```rust
// In constants/molecular_dynamics.rs:
pub const LJ_CUTOFF_RATIO: f64 = 2.5;
```

With `sigma = 1/DENSITY_SCALE = 0.05`: `r_cut = 0.125`. This may be too small
for the current coordinate system. Need to validate during MD-4.

**Alternative for current coordinates:** Use `r_cut` as a parameter in CellList,
not hardcoded. Let the simulation binary set it based on the system being modeled.

## Risks and Mitigations

### Cell list overhead for small N

**Problem:** At N < 100, O(N^2) is faster than cell list (no allocation, no indirection).

**Mitigation:** Threshold: if N < 128, use O(N^2) directly. Cell list only for N >= 128.

### Cell size vs. box size mismatch

**Problem:** If box_length < 3 * cell_size, there are fewer than 3 cells per dimension.
Neighbor iteration wraps around and may double-count pairs.

**Mitigation:** Assert `grid_dims[d] >= 3` for all dimensions. If box is too small,
fall back to O(N^2).

### Non-uniform density

**Problem:** If all particles cluster in one cell, cell list degrades to O(N^2) for
that cell.

**Mitigation:** This is inherent to cell lists. For the expected use case (fluid
simulations with roughly uniform density), this is not a problem. For clustered
systems, a tree-based approach (Barnes-Hut, already in PC-1 design) would be needed.

### Memory allocation per tick

**Problem:** Rebuilding cell list every tick allocates head/next arrays.

**Mitigation:** Pre-allocate in CellList and reuse across ticks. `build()` clears
and refills without reallocation if capacity is sufficient.

## Tests

| Test | Pass criterion |
|------|---------------|
| `cell_index_corner_cases` | (0,0) → 0, (nx-1, ny-1) → last |
| `build_single_particle` | One particle in correct cell |
| `build_two_particles_same_cell` | Both in same chain |
| `for_each_pair_finds_all_neighbors` | Same pairs as brute force for r < r_cut |
| `for_each_pair_skips_distant` | No pairs with r > r_cut |
| `for_each_pair_pbc_wrap` | Particles at box edges detected as neighbors |
| `cell_list_vs_brute_force_forces` | Force vectors identical within epsilon |
| `cell_list_newton3` | Sum of all forces = 0 |
| `cell_list_empty_world` | No crash, no pairs |
| `performance_cell_vs_brute_n1000` | Cell list >= 5x faster at N=1000 |

## Acceptance Criteria

- [x] `batch/neighbor_list.rs` with CellList struct
- [x] O(N) build, O(N * avg_neighbors) iteration
- [x] PBC minimum image used in pair iteration
- [x] Falls back to O(N^2) for N < 128 or no PBC
- [x] Pre-allocated, reusable across ticks
- [x] Force results identical to brute force within epsilon
- [x] >= 10 tests
- [x] N=1000 benchmark shows >= 5x speedup over O(N^2)
