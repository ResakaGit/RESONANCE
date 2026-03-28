# Sprint AS-3 — Event-Driven Pipeline: Skip Empty Ticks

**Modulo:** `src/blueprint/equations/batch_stepping.rs` (nuevo), `src/batch/pipeline.rs`
**Tipo:** Ecuaciones puras + pipeline refactor.
**Onda:** AS-2 → AS-3.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post AS-2)

- AS-1: Isolated entities step analytically (O(1) for N ticks).
- AS-2: Converged fields skip diffusion.
- Pipeline still ticks uniformly: 3000 ticks even if nothing happens.

---

## Objetivo

Predict when the next "event" occurs (death, reproduction, collision, asteroid) and
skip directly to it. Between events, use analytical stepping for all entities.

**Contract:** `tick_event_driven(world, 3000)` produces identical final state as
`for _ in 0..3000 { tick(world) }`, but in fewer actual system calls.

---

## Responsabilidades

### AS-3A: Event prediction (pure equations)

```rust
// src/blueprint/equations/batch_stepping.rs (NUEVO)

/// Predict tick of next death for entity (qe reaches QE_MIN_EXISTENCE).
/// Uses ticks_until_threshold with dissipation + senescence combined drain.
pub fn predict_death_tick(qe: f32, dissipation_rate: f32, age: u64, senescence_coeff: f32) -> u64;

/// Predict tick of next reproduction (qe reaches REPRODUCTION_THRESHOLD).
/// Uses inverse of dissipation: qe_gain from photosynthesis vs drain.
pub fn predict_reproduction_tick(qe: f32, net_intake_per_tick: f32, threshold: f32) -> u64;

/// Predict next collision time between two entities.
/// Solves |pos_a(t) - pos_b(t)| = r_a + r_b for linear trajectories.
/// Returns None if no collision in N ticks.
pub fn predict_collision_tick(
    pos_a: [f32; 2], vel_a: [f32; 2], r_a: f32,
    pos_b: [f32; 2], vel_b: [f32; 2], r_b: f32,
    max_ticks: u32, dt: f32,
) -> Option<u32>;

/// Next event tick for the world: min of all predicted events.
pub fn next_event_tick(world: &SimWorldFlat, current_tick: u64, max_ticks: u32) -> u32;
```

### AS-3B: Event-driven pipeline

```rust
impl SimWorldFlat {
    /// Advance world by `total_ticks`, jumping between events.
    ///
    /// 1. Predict next event tick.
    /// 2. Analytically step all entities to event tick (AS-1).
    /// 3. Run full tick at event tick (collisions, reproduction, death).
    /// 4. Repeat until total_ticks consumed.
    pub fn tick_event_driven(&mut self, scratch: &mut ScratchPad, total_ticks: u32) {
        let mut remaining = total_ticks;
        while remaining > 0 {
            let jump = next_event_tick(self, self.tick_id, remaining).min(remaining);
            if jump > 1 {
                // Analytical step for jump-1 ticks (no events)
                self.analytical_step_all(jump - 1);
                remaining -= jump - 1;
            }
            // Full tick at event point
            self.tick(scratch);
            remaining -= 1;
        }
    }

    /// Analytically step all entities by N ticks (AS-1 functions).
    fn analytical_step_all(&mut self, n: u32) {
        // dissipation_n_ticks, growth_n_ticks, senescence_n_ticks, locomotion_n_ticks
        // for ALL entities (isolated or not — between events, no interactions by definition)
        // ...
        self.tick_id += n as u64;
    }
}
```

### AS-3C: Batch evaluation using event-driven

```rust
impl WorldBatch {
    /// Run evaluation using event-driven pipeline.
    pub fn run_evaluation_fast(&mut self, ticks: u32) {
        use rayon::prelude::*;
        self.worlds.par_iter_mut().for_each(|world| {
            THREAD_SCRATCH.with(|cell| {
                let mut scratch = cell.borrow_mut();
                world.tick_event_driven(&mut scratch, ticks);
            });
        });
    }
}
```

---

## Precision guarantees

| Aspect | Guarantee | How verified |
|--------|-----------|-------------|
| **Dissipation** | Exact f32 (exponential_decay) | Test: analytical == iterative for 3000 ticks |
| **Growth** | Exact f32 (allometric_radius) | Test: analytical == iterative for 3000 ticks |
| **Senescence** | <1% error (trapezoidal approx) | Test: compare at 1000 ticks |
| **Collision timing** | ±1 tick (linear prediction) | Test: collision detected within 1 tick of prediction |
| **Final state** | Identical to tick-by-tick | Integration test: same seed → same final qe, alive_mask |

---

## NO hace

- No modifica physics equations — same math, fewer calls.
- No implements BridgeCache integration — separate optimization.
- No handles non-linear velocity changes (drag) analytically — falls back to tick.

---

## Criterios de aceptacion

### Correctness
- `tick_event_driven(world, 3000)` == `for _ in 0..3000 { tick(world) }` for worlds without collisions.
- With collisions: final state matches within f32 tolerance.
- Determinism: same seed → bit-exact result.

### Performance
- 200 worlds × 3000 ticks: 5-10× faster than current.
- `cargo bench --bench batch_benchmark` shows improvement.
- Most ticks jumped (>80% of ticks skipped in typical worlds).

### Conservation
- `total_qe` invariant preserved across jumps.
- No entity gains energy from analytical stepping.

---

## Referencias

- `src/blueprint/equations/macro_analytics.rs` — `exponential_decay`, `ticks_until_threshold`
- `src/batch/pipeline.rs` — current `tick()`
- `src/batch/systems/` — all 34 systems
- AS-1 — analytical equations
- AS-2 — convergence detection
