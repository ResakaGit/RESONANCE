# Sprint EM-3 — Appendage Inference: Peaks → Sub-Meshes

**Modulo:** `src/geometry_flow/creature_builder.rs`, `src/blueprint/equations/radial_field.rs`
**Tipo:** Geometry generation from 2D field peaks.
**Onda:** EM-2 → EM-3.
**Estado:** ⏳ Pendiente

---

## Objetivo

Lateral peaks in the radial field generate sub-meshes (appendages).
Each peak's **aspect ratio** determines shape (bulb vs tube vs taper).
Bilateral symmetry emerges from isotropic initialization — not mirroring.
Zero labels (`Head`, `Leg`). Only physics.

---

## Responsabilidades

### EM-3A: Peak → 3D position mapping

```rust
/// Map a 2D field peak (axial_idx, radial_sector) to a 3D offset from body center.
/// Axiom 7: position proportional to field gradient direction.
pub fn peak_to_3d_offset(
    ax: u8, rad: u8, body_length: f32, body_radius: f32,
) -> Vec3;
```

Sector 0 = dorsal (+Y), sector 1 = right (+X), sector 2 = ventral (-Y), sector 3 = left (-X).
Axial station → Z position along body axis.

### EM-3B: Peak → geometry type (from physics, not labels)

```rust
/// Determine GF1 spine params from peak properties.
/// aspect_ratio high → long thin tube (limb-like).
/// aspect_ratio low → short wide bulb (head-like).
/// aspect_ratio mid → tapered cone (tail-like).
/// Axiom 6: shape from physics, not from OrganRole enum.
pub fn peak_to_spine_params(
    peak_qe: f32, aspect_ratio: f32, base_radius: f32,
) -> (f32, f32, f32);  // (length_budget, radius_base, detail)
```

### EM-3C: creature_builder_radial

```rust
/// Build creature mesh from 2D radial field.
/// 1. Trunk: axial radii from radial_to_axial_radii()
/// 2. Detect peaks in radial field
/// 3. For each peak above APPENDAGE_QE_MIN:
///    - 3D offset from peak_to_3d_offset()
///    - Shape from peak_to_spine_params(peak_qe, aspect_ratio)
///    - Direction from gradient_at()
///    - Build GF1 sub-mesh
/// 4. merge_meshes([trunk, ...appendages])
pub fn build_creature_mesh_radial(
    growth_bias: f32,
    mobility_bias: f32,
    branching_bias: f32,
    resilience: f32,
    frequency_hz: f32,
    qe_field: &[[f32; 4]; 8],
    freq_field: &[[f32; 4]; 8],
) -> Mesh;
```

### EM-3D: evolve_and_view update

Update `src/bin/evolve_and_view.rs` to call `build_creature_mesh_radial` instead of `build_creature_mesh_with_field`.

---

## Emergence validation

- Isotropic init + 200 diffusion steps → peaks at sectors 1 and 3 (bilateral pair).
- High branching_bias → more lateral peaks → more appendages.
- High growth_bias → axial tip peaks → elongated "head" and "tail".
- High resilience → center peak → thick "torso", thin extremes.
- Asymmetric mutation → broken symmetry → one side stronger than other.

---

## NO hace

- No detecta joints ni articula — eso es EM-4.
- No programa `Head`/`Leg` labels.
- No fuerza bilateral symmetry — emerge de isotropic init.

---

## Criterios de aceptacion

- Uniform field → trunk only (no appendages).
- Field with 2 bilateral peaks → 2 symmetric sub-meshes.
- Field with 1 asymmetric peak → 1 sub-mesh (asymmetry preserved).
- High aspect_ratio peak → long tube. Low aspect_ratio → bulb.
- `evolve_and_view` shows bilateral organisms, not tubes.
- All batch tests pass.

---

## Referencias

- `docs/design/EMERGENT_MORPHOLOGY.md`
- `src/geometry_flow/creature_builder.rs` — current 1D builder
- `src/blueprint/equations/entity_shape.rs` — `lateral_offset()` reference
