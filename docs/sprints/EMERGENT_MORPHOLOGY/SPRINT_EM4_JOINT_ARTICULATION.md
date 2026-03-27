# Sprint EM-4 — Joint Articulation: Valleys → Segmented Appendages

**Modulo:** `src/blueprint/equations/radial_field.rs`, `src/geometry_flow/creature_builder.rs`
**Tipo:** Equations + geometry refinement.
**Onda:** EM-3 → EM-4.
**Estado:** ⏳ Pendiente

---

## Objetivo

Energy valleys along appendages become **joints** — points of reduced rigidity.
Appendage meshes split into **segments** separated by thin joints.
Joint flexibility is proportional to `1 / bond_energy` at the valley.

Zero hardcoded `MAX_BONES`. Zero `Joint` components. Joints are valleys in the field.

---

## Responsabilidades

### EM-4A: Appendage sub-field extraction

```rust
/// Extract 1D energy profile along an appendage from its peak outward.
/// Returns energy values from peak center toward body surface.
/// Used to find internal valleys (joints) within an appendage.
pub fn extract_appendage_profile(
    field: &RadialField, peak_ax: u8, peak_rad: u8, direction: (i8, i8),
) -> [f32; AXIAL_NODES];
```

### EM-4B: Joint detection within appendage

```rust
/// Find valleys (local minima) in 1D appendage profile.
/// Each valley with qe < JOINT_FLEX_THRESHOLD × mean → joint point.
/// Returns (position_along_appendage, flexibility) pairs.
/// Axiom 6: joints are energy valleys, never hardcoded positions.
pub fn detect_appendage_joints(
    profile: &[f32], bond_energy: f32,
) -> Vec<(f32, f32)>;  // (t ∈ [0,1] along appendage, flexibility ∈ [0,1])
```

### EM-4C: Segmented appendage mesh

```rust
/// Build appendage mesh with joints: thin at valleys, thick at peaks.
/// Each segment between joints gets its own GF1 sub-spine.
/// Joint radius = base_radius × (valley_qe / peak_qe).sqrt()
/// Axiom 1: geometry from energy, not templates.
pub fn build_segmented_appendage(
    peak_qe: f32,
    base_radius: f32,
    length: f32,
    joints: &[(f32, f32)],  // (position, flexibility)
    direction: Vec3,
    start: Vec3,
    tint: [f32; 3],
) -> Mesh;
```

### EM-4D: Integration in creature_builder

Modify `build_creature_mesh_radial` (EM-3):
- For each detected peak, extract appendage profile
- Detect joints within profile
- If joints found: call `build_segmented_appendage`
- If no joints: call existing GF1 single tube (current behavior)

---

## Emergence validation

- Appendage with uniform energy → no joints (single tube).
- Appendage with 1 valley → 2 segments (upper + lower leg).
- Appendage with 2 valleys → 3 segments (shoulder + arm + hand).
- High bond_energy → rigid (no visible joints). Low → visible constriction.
- Different appendages on same entity can have different joint counts.

---

## NO hace

- No implementa forward/inverse kinematics — futuro.
- No anima joints — las poses son estáticas (deformación futura).
- No implementa mesh volumétrico (marching cubes) — futuro nivel 7.

---

## Criterios de aceptacion

- Appendage with artificial valley → visually narrower at valley point.
- Joint count varies between appendages on same entity.
- `detect_appendage_joints(uniform_profile)` returns empty.
- `detect_appendage_joints([10, 1, 10])` returns 1 joint at middle.
- Conservation: segmented mesh has same total volume as non-segmented.
- `evolve_and_view` shows segmented bilateral creatures.
- All batch tests pass.

---

## Referencias

- `docs/design/EMERGENT_MORPHOLOGY.md`
- `src/blueprint/equations/radial_field.rs` — EM-1 equations
- `src/geometry_flow/creature_builder.rs` — EM-3 radial builder
