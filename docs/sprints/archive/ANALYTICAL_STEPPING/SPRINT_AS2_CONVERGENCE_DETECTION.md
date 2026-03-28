# Sprint AS-2 — Convergence Detection: Skip Stabilized Diffusion

**Modulo:** `src/blueprint/equations/radial_field.rs`, `src/batch/systems/internal_field.rs`
**Tipo:** Ecuación pura + system optimization.
**Onda:** AS-1 → AS-2.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post AS-1)

- `radial_diffuse()` runs every tick on 128 nodes (16×8).
- Field converges exponentially — after ~50-100 ticks, changes are < 0.1%/tick.
- No mechanism to detect convergence and stop diffusing.

---

## Objetivo

Detect when the radial energy field has stabilized and skip remaining diffusion ticks.
Saves ~90% of `internal_diffusion` cost (the most expensive system with 128 nodes).

**Contract:** If `max(|field_t - field_{t-1}|) < CONVERGENCE_EPSILON` for all 128 nodes,
the field is stable. Skip further diffusion. Resume only if external event changes qe.

---

## Responsabilidades

### AS-2A: Convergence metric (pure equation)

```rust
// src/blueprint/equations/radial_field.rs — ampliar

/// Maximum absolute change across all nodes between two field states.
/// O(AXIAL × RADIAL). Returns 0.0 if identical.
pub fn radial_max_delta(a: &RadialField, b: &RadialField) -> f32;

/// Check if field has converged: max delta < epsilon.
pub fn radial_converged(before: &RadialField, after: &RadialField, epsilon: f32) -> bool {
    radial_max_delta(before, after) < epsilon
}
```

### AS-2B: Convergence flag in EntitySlot

```rust
// src/batch/arena.rs — add flag to EntitySlot

/// True if qe_field has converged and diffusion can be skipped.
/// Reset to false when qe changes externally (photosynthesis, predation, etc.).
pub field_converged: bool,
```

### AS-2C: System update

```rust
// src/batch/systems/internal_field.rs — modify internal_diffusion

pub fn internal_diffusion(world: &mut SimWorldFlat) {
    // ...
    // Skip diffusion if converged and qe hasn't changed
    if e.field_converged {
        // Just sync qe from field (in case other systems changed qe)
        let field_sum = radial_field::radial_total(&e.qe_field);
        if (field_sum - e.qe).abs() > GUARD_EPSILON {
            // qe changed externally → reconverge
            radial_field::radial_rescale(&mut e.qe_field, e.qe);
            e.field_converged = false;
        }
        // Otherwise skip — field is stable
        continue;
    }

    let before = e.qe_field;
    e.qe_field = radial_field::radial_diffuse(&e.qe_field, conductivity, dt);

    if radial_field::radial_converged(&before, &e.qe_field, CONVERGENCE_EPSILON) {
        e.field_converged = true;
    }
    // ...
}
```

### AS-2D: Reset convergence on external qe change

Systems that modify `entity.qe` must reset `field_converged = false`:
- `dissipation` — drains qe
- `photosynthesis` — adds qe
- `trophic_predation` — transfers qe
- `collision` — exchanges qe

One-liner per system: `e.field_converged = false;` after modifying qe.

---

## Constantes

```rust
/// Convergence epsilon for radial field diffusion.
/// Below this max-delta, field is considered stable.
pub const CONVERGENCE_EPSILON: f32 = 0.001;
```

---

## NO hace

- No modifica diffusion math — same equations, just skips when stable.
- No implements event-driven pipeline — eso es AS-3.
- No changes analytical stepping — eso es AS-1.

---

## Criterios de aceptacion

### Convergence detection
- Uniform field → converged after 0 ticks (already stable).
- Spike field → converges after ~50-100 ticks (with conductivity=0.05).
- After convergence, `internal_diffusion` is O(1) per entity (just check flag).

### Reset on external change
- Photosynthesis adds qe → `field_converged` reset → re-diffuses next tick.
- After re-convergence, skips again.

### Conservation
- `radial_total(field) == entity.qe` invariant preserved.
- Skip diffusion doesn't change field values.

### Performance
- 200 worlds × 3000 ticks: measurable speedup on `internal_diffusion`.
- Benchmark: convergence detection overhead < 5% of diffusion cost.

---

## Referencias

- `src/blueprint/equations/radial_field.rs` — `radial_diffuse`
- `src/batch/systems/internal_field.rs` — current system
