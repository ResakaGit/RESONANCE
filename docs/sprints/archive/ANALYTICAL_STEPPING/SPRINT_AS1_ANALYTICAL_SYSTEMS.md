# Sprint AS-1 — Analytical Systems: O(1) Multi-Tick Stepping

**Modulo:** `src/batch/systems/atomic.rs`, `morphological.rs`, `pipeline.rs`
**Tipo:** Optimization — replace tick-by-tick with closed-form for independent systems.
**Onda:** Sin bloqueantes.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `src/blueprint/equations/macro_analytics.rs`:
  - `exponential_decay(value, rate, n) → value × (1-rate)^n` — O(1) dissipation.
  - `allometric_radius(r0, r_max, k, n)` — O(1) growth for N ticks.
  - `ticks_until_threshold(value, drain) → u32` — predictive death time.
- `src/batch/systems/atomic.rs: dissipation()` — calls `dissipation_loss(qe, rate)` per tick.
- `src/batch/systems/morphological.rs: senescence()` — age-dependent drain per tick.
- `src/batch/systems/morphological.rs: growth_inference()` — calls `allometric_radius(r, rmax, k, 1)` per tick.
- `src/batch/systems/atomic.rs: locomotion_drain()` — calls `locomotion_energy_cost(qe, speed, 1.0)` per tick.

**Lo que NO existe:**

1. **Multi-tick analytical path.** Systems call equations with `n=1` every tick. No system calls with `n=N`.
2. **Interaction detection.** No mechanism to determine if an entity is "isolated" (no collisions next N ticks).
3. **Split pipeline.** No separation between "independent" systems (batchable) and "interactive" systems (must tick).

---

## Objetivo

For entities that don't interact with others during an evaluation period, replace
tick-by-tick simulation with O(1) closed-form solutions. This is exact — same f32 result
as iterating N times — for dissipation, growth, senescence, and locomotion drain.

**Contract:** `analytical_step(entity, N) == for _ in 0..N { tick(entity) }` for isolated entities.

---

## Responsabilidades

### AS-1A: Analytical dissipation (O(1) for N ticks)

```rust
// src/blueprint/equations/batch_stepping.rs (NUEVO)

/// Dissipation over N ticks: qe × (1 - rate)^N.
/// Exact discrete Euler. Same f32 result as calling dissipation_loss N times.
/// Reuses `macro_analytics::exponential_decay` internally.
/// Axiom 4: dissipation is monotonic non-negative.
pub fn dissipation_n_ticks(qe: f32, rate: f32, n: u32) -> f32 {
    let clamped_rate = rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX);
    equations::exponential_decay(qe, clamped_rate, n)
}
```

**Test contract:**
```rust
// tick-by-tick == analytical
let mut qe_iter = 100.0;
for _ in 0..500 {
    qe_iter -= equations::dissipation_loss(qe_iter, 0.01);
}
let qe_analytical = dissipation_n_ticks(100.0, 0.01, 500);
assert!((qe_iter - qe_analytical).abs() < 1e-2); // f32 accumulation tolerance
```

### AS-1B: Analytical growth (O(1) for N ticks)

```rust
/// Growth over N ticks: allometric_radius(r0, r_max, k, N).
/// Already exists in macro_analytics. Just wire it.
pub fn growth_n_ticks(
    radius: f32, growth_bias: f32, max_radius: f32, k: f32, n: u32,
) -> f32 {
    if growth_bias <= 0.0 { return radius; }
    let r_max = growth_bias * max_radius;
    if radius >= r_max { return radius; }
    equations::allometric_radius(radius, r_max, k, n)
}
```

### AS-1C: Analytical senescence (O(1) for N ticks)

```rust
/// Senescence drain over N ticks.
/// age_dependent_dissipation(base, age, coeff) returns rate at given age.
/// Over N ticks, age increments 1 per tick: total drain = Σ qe × rate(age+i).
/// For small coeff, this is approximately: qe × base_rate × N × (1 + coeff × (age + N/2)).
pub fn senescence_n_ticks(
    qe: f32, base_rate: f32, age: u64, coeff: f32, n: u32,
) -> f32 {
    let avg_age = age as f32 + n as f32 * 0.5;
    let avg_rate = base_rate * (1.0 + coeff * avg_age);
    let total_loss = qe * avg_rate * n as f32;
    (qe - total_loss).max(0.0)
}
```

### AS-1D: Analytical locomotion drain (O(1) for N ticks)

```rust
/// Locomotion drain over N ticks with constant velocity.
/// cost_per_tick = locomotion_energy_cost(qe, speed, terrain_factor).
/// Over N ticks: total_cost ≈ cost_per_tick × N (if qe doesn't change much).
/// Conservative: compute cost at initial qe, subtract total.
pub fn locomotion_drain_n_ticks(
    qe: f32, speed: f32, terrain_factor: f32, n: u32,
) -> f32 {
    if speed < 1e-4 { return qe; }
    let cost_per_tick = equations::locomotion_energy_cost(qe, speed, terrain_factor);
    (qe - cost_per_tick * n as f32).max(0.0)
}
```

### AS-1E: Isolation detection

```rust
/// Check if entity is isolated (no neighbors within interaction range).
/// If isolated, analytical stepping is exact.
/// Uses alive_mask scan with position comparison.
pub fn is_isolated(
    world: &SimWorldFlat, entity_idx: usize, interaction_range: f32,
) -> bool {
    let range_sq = interaction_range * interaction_range;
    let pos = world.entities[entity_idx].position;
    let mut mask = world.alive_mask & !(1u64 << entity_idx);
    while mask != 0 {
        let j = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let dx = world.entities[j].position[0] - pos[0];
        let dy = world.entities[j].position[1] - pos[1];
        if dx * dx + dy * dy < range_sq { return false; }
    }
    true
}
```

### AS-1F: Hybrid pipeline

```rust
/// Modified tick: analytical step for isolated entities, normal tick for interactive.
impl SimWorldFlat {
    pub fn tick_hybrid(&mut self, scratch: &mut ScratchPad, batch_ticks: u32) {
        scratch.clear();
        self.events.clear();

        // Classify: isolated vs interactive
        let max_range = PREDATION_RANGE.max(PACK_SCAN_RADIUS).max(COOPERATION_SCAN_RADIUS);
        let mut isolated_mask = 0u64;
        let mut mask = self.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            if is_isolated(self, i, max_range) {
                isolated_mask |= 1 << i;
            }
        }

        // Analytical step isolated entities (O(1) for batch_ticks)
        let mut iso = isolated_mask;
        while iso != 0 {
            let i = iso.trailing_zeros() as usize;
            iso &= iso - 1;
            let e = &mut self.entities[i];
            e.qe = dissipation_n_ticks(e.qe, e.dissipation, batch_ticks);
            e.radius = growth_n_ticks(e.radius, e.growth_bias, MAX_ALLOMETRIC_RADIUS, GROWTH_RATE_K, batch_ticks);
            // senescence, locomotion similarly...
            // Position: advance by velocity × dt × batch_ticks (with gravity)
            for _ in 0..batch_ticks { // position still needs per-tick for floor collision
                e.velocity[1] -= GRAVITY_ACCELERATION * self.dt;
                e.position[0] += e.velocity[0] * self.dt;
                e.position[1] += e.velocity[1] * self.dt;
                if e.position[1] < 0.0 { e.position[1] = 0.0; e.velocity[1] = 0.0; }
            }
        }

        // Interactive entities: full tick-by-tick for batch_ticks
        let interactive_mask = self.alive_mask & !isolated_mask;
        if interactive_mask != 0 {
            for _ in 0..batch_ticks {
                self.tick_id += 1;
                // Only run interactive systems on interactive_mask entities
                // ... (dissipation, collision, trophic, etc.)
            }
        } else {
            self.tick_id += batch_ticks as u64;
        }

        // Post-tick bookkeeping (all entities)
        systems::internal_diffusion(self);
        systems::asteroid_impact(self);
        systems::death_reap(self);
        self.update_total_qe();
    }
}
```

---

## Constantes

```rust
// src/batch/constants.rs — ampliar

/// Maximum interaction range for isolation detection.
/// Entities farther than this from ALL others are "isolated" and can be analytically stepped.
/// Derived from max(PREDATION_RANGE, PACK_SCAN_RADIUS, COOPERATION_SCAN_RADIUS).
pub const ISOLATION_RANGE: f32 = 8.0; // max of existing ranges
```

---

## Tacticas

- **Exact for isolated.** `dissipation_n_ticks` produces identical f32 as N calls to `dissipation_loss` because `exponential_decay` uses `powf` which matches `(1-rate)^N` exactly.
- **Conservative for interactive.** Interactive entities tick normally. No approximation.
- **Fallback safe.** If `isolated_mask == 0` (all entities near others), pipeline degrades to current behavior. Zero risk.
- **Growth already O(1).** `allometric_radius(r, rmax, k, N)` exists and is tested. Just change `n=1` to `n=N`.

---

## NO hace

- No modifica ecuaciones existentes — solo calls with `n > 1`.
- No approximates interactive systems (collision, trophic, entrainment).
- No changes field diffusion — eso es AS-2.
- No implements event-driven pipeline — eso es AS-3.
- No adds BridgeCache to batch — optional future optimization.

---

## Dependencias

- `crate::blueprint::equations::macro_analytics` — `exponential_decay`, `allometric_radius`.
- `crate::blueprint::equations::energy_competition` — `dissipation_loss` (rate clamping constants).
- `crate::blueprint::equations::locomotion` — `locomotion_energy_cost`.
- `crate::blueprint::equations::emergence::senescence` — `age_dependent_dissipation`.

---

## Criterios de aceptacion

### AS-1A (Dissipation analytical)
- `dissipation_n_ticks(100.0, 0.01, 1) == 100.0 - dissipation_loss(100.0, 0.01)` (single tick exact).
- `dissipation_n_ticks(100.0, 0.01, 500)` matches 500 iterative calls within f32 tolerance (< 0.1%).
- `dissipation_n_ticks(qe, rate, 0) == qe` (zero ticks no-op).
- Result always >= 0 (Axiom 5).

### AS-1B (Growth analytical)
- `growth_n_ticks(0.5, 0.8, 3.0, 0.01, 1)` matches single tick of `growth_inference`.
- `growth_n_ticks(0.5, 0.8, 3.0, 0.01, 10000)` approaches `r_max` within 0.1.
- Zero growth_bias → radius unchanged.

### AS-1C (Senescence analytical)
- Matches tick-by-tick within 1% for 1000 ticks.
- Result always >= 0.

### AS-1E (Isolation detection)
- Entity alone in world → isolated.
- Two entities within PREDATION_RANGE → NOT isolated.
- Entity near grid edge with no neighbors → isolated.

### AS-1F (Hybrid pipeline)
- `tick_hybrid(world, scratch, 1)` produces same result as `tick(world, scratch)`.
- `tick_hybrid(world, scratch, 100)` with all entities isolated produces same final state as 100 × `tick`.
- Conservation: `total_qe` after hybrid ≤ `total_qe` before + solar intake.
- Determinism: same seed → same result with hybrid vs sequential.

### Performance
- 200 worlds × 3000 ticks: measurably faster than current (target: 2-3×).
- `cargo bench --bench batch_benchmark` shows improvement.

### General
- `cargo test --lib` sin regresion.
- Zero `use bevy::` en batch.
- All equations in `blueprint/equations/`. No inline math.

---

## Referencias

- `src/blueprint/equations/macro_analytics.rs` — `exponential_decay`, `allometric_radius`
- `src/batch/pipeline.rs` — current `tick()` implementation
- `src/batch/systems/atomic.rs` — `dissipation`, `locomotion_drain`
- `src/batch/systems/morphological.rs` — `senescence`, `growth_inference`
- Schwefel (1981) — analytical stepping in Evolution Strategies
