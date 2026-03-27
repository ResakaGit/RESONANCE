# Sprint EM-2 — Arena Integration: EntitySlot 2D + Pipeline Wiring

**Modulo:** `src/batch/arena.rs`, `src/batch/systems/internal_field.rs`, `src/batch/constants.rs`
**Tipo:** Data structure migration + system update.
**Onda:** EM-1 → EM-2.
**Estado:** ⏳ Pendiente

---

## Objetivo

Migrar EntitySlot de campo 1D (`[f32; 8]`) a 2D radial (`[[f32; 4]; 8]`).
Wire `internal_diffusion` system para llamar `radial_diffuse` en vez de `field_diffuse`.
Mantener backward compatibility: `qe` sigue siendo la suma total.

---

## Responsabilidades

### EM-2A: EntitySlot migration

```rust
// BEFORE:
pub qe_field:   [f32; 8],
pub freq_field:  [f32; 8],

// AFTER:
pub qe_field:   [[f32; 4]; 8],   // 8 axial × 4 radial
pub freq_field:  [[f32; 4]; 8],
```

Update `Default`, size assertion, all tests that reference `qe_field`.

### EM-2B: internal_diffusion system update

Replace calls to `internal_field::field_diffuse` with `radial_field::radial_diffuse`.
Replace `internal_field::distribute_to_field` with `radial_field::distribute_to_radial`.
Replace `internal_field::rescale_field` with 2D rescale variant.
Replace `internal_field::field_total` with `radial_field::radial_total`.

### EM-2C: Constants

Add to `batch/constants.rs`:
```rust
pub const PEAK_THRESHOLD_FACTOR: f32 = 1.8;
pub const APPENDAGE_QE_MIN: f32 = 0.5;
pub const JOINT_FLEX_THRESHOLD: f32 = 5.0;
```

---

## NO hace

- No modifica creature_builder — eso es EM-3.
- No detecta peaks ni joints — eso es EM-3/EM-4.

---

## Criterios de aceptacion

- EntitySlot compila con 2D fields. `Copy` + `repr(C)` preserved.
- `internal_diffusion` system uses 2D radial math.
- Conservation: `radial_total(entity.qe_field) == entity.qe` after every tick.
- All existing batch tests pass (adapted to 2D fields).
- Determinism test still bit-exact.

---

## Referencias

- `docs/design/EMERGENT_MORPHOLOGY.md`
- `src/batch/arena.rs` — EntitySlot struct
- `src/batch/systems/internal_field.rs` — current 1D system
