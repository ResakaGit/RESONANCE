# MD-0: Velocity Verlet Integrator

**Effort:** 3 days | **Blocked by:** nothing | **Blocks:** MD-1, MD-2

## Problem

`movement_integrate` in `batch/systems/atomic.rs:39` uses Euler integration:
```
v += a * dt
x += v * dt
```

Euler has O(dt) energy error per step. Over thousands of steps, total energy drifts
monotonically — the system heats up or cools down without physical cause. This makes
any thermodynamic measurement meaningless.

## Solution

Replace with Velocity Verlet (symplectic integrator, O(dt^2) per step, bounded drift):

```
x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt^2    // position half-step
a(t+dt) = F(x(t+dt)) / m                      // recompute forces at new position
v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt      // velocity full step
```

Symplectic = conserves phase space volume = energy oscillates around true value
instead of drifting. This is the standard integrator for molecular dynamics.

## Implementation

### 1. Pure math: `blueprint/equations/verlet.rs`

```rust
/// Velocity Verlet position half-step: x += v*dt + 0.5*a*dt^2.
pub fn verlet_position_step(x: f64, v: f64, a: f64, dt: f64) -> f64 {
    x + v * dt + 0.5 * a * dt * dt
}

/// Velocity Verlet velocity full step: v += 0.5*(a_old + a_new)*dt.
pub fn verlet_velocity_step(v: f64, a_old: f64, a_new: f64, dt: f64) -> f64 {
    v + 0.5 * (a_old + a_new) * dt
}
```

Tests: reversibility (step forward + backward = original), zero-force (uniform motion),
constant-force (exact parabola), energy conservation for harmonic oscillator.

### 2. EntitySlot extension: `batch/arena.rs`

Add per-entity force storage (needed for "old acceleration" in Verlet):

```rust
// In EntitySlot:
pub force: [f32; 2],       // accumulated force from previous tick
```

Force is zeroed at start of tick, accumulated by `particle_forces` and any other
force system, then used by Verlet for the velocity step.

### 3. System change: `batch/systems/atomic.rs`

Replace `movement_integrate` with `verlet_integrate`:

```rust
pub fn verlet_integrate(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let mass = e.particle_mass.max(0.01);
        let ax = e.force[0] / mass;
        let ay = e.force[1] / mass - GRAVITY_ACCELERATION;

        // Position half-step (using old force)
        e.position[0] += e.velocity[0] * dt + 0.5 * ax * dt * dt;
        e.position[1] += e.velocity[1] * dt + 0.5 * ay * dt * dt;

        // Store old acceleration for velocity step
        e.force[0] = ax;  // temporarily store a_old
        e.force[1] = ay;

        // Ground collision
        if e.position[1] < 0.0 {
            e.position[1] = 0.0;
            e.velocity[1] = 0.0;
        }
    }
    // NOTE: forces are recomputed by particle_forces (next in pipeline)
    // Then verlet_velocity_finish completes the velocity step
}

pub fn verlet_velocity_finish(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let mass = e.particle_mass.max(0.01);
        let ax_new = e.force[0] / mass;  // new force from this tick
        let ay_new = e.force[1] / mass - GRAVITY_ACCELERATION;

        // Retrieve a_old (stored in force fields by verlet_integrate)
        // PROBLEM: force was overwritten by particle_forces
        // SOLUTION: need separate old_force storage
    }
}
```

**Key insight:** Velocity Verlet requires TWO force evaluations per step (or storing
old forces). In the pipeline, this means:

```
1. verlet_position_step    (uses old forces to advance positions)
2. zero forces
3. compute forces           (at new positions)
4. verlet_velocity_step     (uses old + new forces to advance velocities)
```

This changes the pipeline order. Currently: forces -> integrate.
New: position_step -> forces -> velocity_step.

### 4. Pipeline change: `batch/pipeline.rs`

```
// Before (Euler):
particle_forces → movement_integrate

// After (Verlet):
verlet_position_step → zero_forces → particle_forces → verlet_velocity_step
```

### 5. EntitySlot field addition

```rust
pub old_acceleration: [f32; 2],  // a(t) stored for Verlet velocity step
```

This adds 8 bytes to EntitySlot. At 128 entities = 1KB. Negligible.

## Tests

| Test | What it validates |
|------|-------------------|
| `verlet_position_exact_for_constant_force` | x = x0 + v0*t + 0.5*a*t^2 (parabolic) |
| `verlet_velocity_exact_for_constant_force` | v = v0 + a*t |
| `verlet_energy_drift_harmonic_1k_steps` | E oscillates, drift < 1e-4 relative |
| `verlet_energy_drift_harmonic_10k_steps` | E oscillates, drift < 1e-3 relative |
| `verlet_reversibility` | step(+dt) then step(-dt) recovers initial state within epsilon |
| `verlet_vs_euler_drift_comparison` | Verlet drift << Euler drift at same dt |
| `verlet_conserves_momentum_two_body` | p_total constant for isolated pair |

## Acceptance Criteria

- [x] `movement_integrate` replaced by `verlet_position_step` + `verlet_velocity_finish`
- [x] Pipeline reordered: position -> forces -> velocity
- [x] Energy drift < 1e-4 over 10K steps (harmonic oscillator test)
- [x] All existing batch tests pass (regression)
- [x] `old_acceleration` field added to EntitySlot
- [x] Pure math in `blueprint/equations/verlet.rs` with >= 7 tests
