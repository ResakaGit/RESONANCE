# MD-2: Periodic Boundary Conditions

**Effort:** 1 week | **Blocked by:** MD-0 | **Blocks:** MD-3, MD-8, MD-12

## Problem

Current simulation has a floor at y=0 and no walls. Particles can fly off to infinity.
This models a droplet in vacuum — not bulk matter.

Molecular dynamics simulates bulk by repeating a small box infinitely in all directions.
A particle leaving the right edge reappears on the left. Forces use the **minimum image
convention**: each particle interacts with the closest copy of every other particle.

## Why PBC Matters

Without PBC:
- Surface effects dominate (all particles are "on the surface")
- No well-defined pressure or density
- Long-range correlations can't develop
- Temperature gradients from boundaries

With PBC:
- Bulk properties emerge (pressure, RDF, diffusion)
- No surface artifacts
- Axiom 7 (distance attenuation) applies on a torus — well-defined

## Implementation

### 1. Pure math: `batch/pbc.rs`

```rust
/// Simulation box dimensions.
pub struct SimBox {
    pub length: [f64; 2],  // 2D initially, extend to 3D in MD-7
}

/// Wrap coordinate into [0, L) range.
pub fn wrap(x: f64, length: f64) -> f64 {
    x - length * (x / length).floor()
}

/// Minimum image displacement: shortest vector between two points on a torus.
/// dr = x_j - x_i, then adjusted to [-L/2, L/2).
pub fn minimum_image(dr: f64, length: f64) -> f64 {
    dr - length * (dr / length + 0.5).floor()
}

/// Minimum image distance squared between two positions.
pub fn minimum_image_distance_sq(
    pos_i: &[f64; 2],
    pos_j: &[f64; 2],
    box_length: &[f64; 2],
) -> f64 {
    let dx = minimum_image(pos_j[0] - pos_i[0], box_length[0]);
    let dy = minimum_image(pos_j[1] - pos_i[1], box_length[1]);
    dx * dx + dy * dy
}
```

### 2. SimWorldFlat extension

```rust
// In SimWorldFlat:
pub sim_box: Option<SimBox>,  // None = no PBC (backward compatible)
```

When `sim_box` is Some, all force computations use minimum image distances
and position wrapping happens after integration.

### 3. Force computation change

`particle_forces` currently computes distance directly:
```rust
let dx = pos_j[0] - pos_i[0];
let dy = pos_j[1] - pos_i[1];
```

With PBC:
```rust
let dx = if let Some(box_) = &sim_box {
    minimum_image((pos_j[0] - pos_i[0]) as f64, box_.length[0]) as f32
} else {
    pos_j[0] - pos_i[0]
};
```

This change is in `particle_forces.rs` `extract_particles` or in
`coulomb::accumulate_forces`. Prefer modifying the extraction step so
`accumulate_forces` stays pure (doesn't know about PBC).

### 4. Position wrapping system

New system after `verlet_velocity_finish`:

```rust
pub fn wrap_positions(world: &mut SimWorldFlat) {
    let Some(box_) = &world.sim_box else { return; };
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        e.position[0] = wrap(e.position[0] as f64, box_.length[0]) as f32;
        e.position[1] = wrap(e.position[1] as f64, box_.length[1]) as f32;
    }
}
```

### 5. Ground collision removal (conditional)

With PBC, there is no floor. The ground collision in `verlet_integrate`:
```rust
if e.position[1] < 0.0 { e.position[1] = 0.0; e.velocity[1] = 0.0; }
```
Must be conditional:
```rust
if world.sim_box.is_none() && e.position[1] < 0.0 { ... }
```

### 6. Pipeline placement

```
verlet_position_step → wrap_positions → zero_forces → particle_forces(with min image) → verlet_velocity_step → langevin
```

## Risks and Mitigations

### Minimum image vs. Axiom 7

**Concern:** Does wrapping break distance attenuation?

**No.** Axiom 7 says interaction decays with physical distance. On a torus, the
physical distance IS the minimum image distance. The axiom holds on the torus
manifold. Document this in the code as a comment.

### Box size too small

**Problem:** If box_length < 2 * r_cut, a particle interacts with its own image.
This is unphysical.

**Mitigation:** Assert `box_length[d] >= 2.0 * r_cut` at configuration time.
For LJ with r_cut = 2.5 * sigma: box must be >= 5.0 * sigma per dimension.

### Backward compatibility

**Strategy:** `sim_box: Option<SimBox>`. None = no PBC = current behavior.
All existing tests pass without modification. New MD tests set sim_box.

### Energy discontinuity at box boundary

**Problem:** When a particle crosses the box edge, its position jumps from L to 0.
If any system depends on absolute position (not displacement), this causes a spike.

**Mitigation:** Audit all batch systems for absolute position dependence:
- `containment_check` → uses absolute position → skip if PBC active
- `collision` → uses pairwise distance → use minimum image
- `behavior_assess` → uses pairwise distance → use minimum image
- Gravity toward y=0 → disable if PBC active

## Tests

| Test | Pass criterion |
|------|---------------|
| `wrap_identity_inside_box` | x in [0, L) → wrap(x) = x |
| `wrap_positive_overflow` | wrap(L + 0.1, L) = 0.1 |
| `wrap_negative_underflow` | wrap(-0.1, L) = L - 0.1 |
| `minimum_image_small_displacement` | dr < L/2 → unchanged |
| `minimum_image_large_displacement` | dr > L/2 → wrapped to [-L/2, L/2) |
| `minimum_image_distance_symmetric` | d(i,j) = d(j,i) |
| `pbc_particle_reappears_on_other_side` | pos > L → pos wraps to [0, L) |
| `pbc_force_uses_nearest_image` | Two atoms at (0.1, 0) and (L-0.1, 0) attract (not repel) |
| `pbc_no_self_interaction` | Single particle in box has zero force |
| `pbc_disabled_backward_compatible` | sim_box=None → identical to current behavior |
| `pbc_box_too_small_panics` | box_length < 2*r_cut → config error |

## Acceptance Criteria

- [x] `batch/pbc.rs` with SimBox, wrap, minimum_image, >= 6 pure functions
- [x] `SimWorldFlat.sim_box: Option<SimBox>` added
- [x] Force computation uses minimum image when PBC active
- [x] Position wrapping system in pipeline
- [x] Ground collision conditional on PBC
- [x] All existing tests pass (backward compatible via Option)
- [x] >= 11 tests for PBC correctness
