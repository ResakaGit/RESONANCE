# Emergent Morphology — From Tubes to Vertebrates

## Problem

Current morphology produces **1D tubes with branches** (GF1 spine + 8-node axial field).
This limits organisms to level 3.5: proto-organs as bulges along a single axis.
Bilateral symmetry, extremities, joints, and body segmentation cannot emerge from a 1D field.

## Constraint: 100% Emergent

Every morphological feature must emerge from energy physics. The 8 axioms and 4 fundamental
constants are the only inputs. Zero labels (`Head`, `Leg`, `Arm`). Zero templates
(`Quadruped`, `Biped`). Zero hardcoded positions.

What we call "head" is a **large qe peak at the front tip**.
What we call "legs" are **bilateral lateral peaks**.
What we call "joints" are **qe valleys between peaks** (low bond energy = flexible).
What we call "bilateral symmetry" is **isotropic diffusion from a center point**.

## Solution: 2D Radial Energy Field

Replace the 1D axial field `[f32; 8]` with a **2D radial field** `[[f32; RADIAL]; AXIAL]`.
8 axial nodes × 4 radial sectors = 32 nodes per entity.

```
        Sector 0 (dorsal)
           ╱ ╲
    Sec 3 │   │ Sec 1 (left/right)
           ╲ ╱
        Sector 2 (ventral)

    × 8 axial stations along body axis
```

The radial dimension enables:
- **Bilateral peaks**: energy accumulates at sectors 1 and 3 → lateral protrusions
- **Dorsal/ventral differentiation**: sector 0 vs 2 can diverge
- **Rotational asymmetry**: not all sectors equal → non-radial body plans emerge

## Architecture

```
blueprint/equations/radial_field.rs     ← NEW: pure math (diffusion, peaks, gradients)
batch/arena.rs                          ← MODIFY: qe_field [f32;8] → [[f32;4];8]
batch/systems/internal_field.rs         ← MODIFY: call radial diffusion
batch/constants.rs                      ← MODIFY: add radial constants
geometry_flow/creature_builder.rs       ← MODIFY: radial field → appendage meshes
blueprint/equations/internal_field.rs   ← EXTEND: 2D variants of existing 1D functions

REUSE (no changes):
  blueprint/equations/entity_shape.rs   ← lateral_offset(), infer_symmetry_mode()
  blueprint/equations/inferred_world_geometry/body_plan.rs ← SymmetryMode enum
  geometry_flow/mod.rs                  ← build_flow_mesh_variable_radius()
  geometry_flow/mod.rs                  ← merge_meshes()
  topology/generators/slope.rs          ← gradient stencil patterns (reference)
  blueprint/equations/radiation_pressure.rs ← frequency alignment (reference)
```

## Equations (pure, stateless, in `blueprint/equations/radial_field.rs`)

### Constants (derivable from 4 fundamentals)

```rust
/// Radial sector count. 4 = minimal bilateral (dorsal/left/ventral/right).
pub const RADIAL_SECTORS: usize = 4;

/// Axial node count (same as existing).
pub const AXIAL_NODES: usize = 8;

/// Total node count.
pub const TOTAL_NODES: usize = AXIAL_NODES * RADIAL_SECTORS; // 32

/// Peak detection threshold. A node is a peak if qe > mean × this.
/// Derivable: DENSITY_SCALE × DISSIPATION_SOLID / DISSIPATION_LIQUID.
pub const PEAK_THRESHOLD_FACTOR: f32 = 1.8;

/// Minimum peak qe to generate an appendage mesh.
/// Derivable: QE_MIN_EXISTENCE × DENSITY_SCALE.
pub const APPENDAGE_QE_MIN: f32 = 0.5;

/// Joint flexibility threshold: bond_energy < this → flexible.
/// Derivable: DISSIPATION_LIQUID × DENSITY_SCALE.
pub const JOINT_FLEX_THRESHOLD: f32 = 5.0;
```

### Core functions

```rust
/// Type alias for the 2D radial field.
pub type RadialField = [[f32; RADIAL_SECTORS]; AXIAL_NODES];

/// Sum all nodes.
pub fn radial_total(field: &RadialField) -> f32;

/// 2D diffusion: axial + radial neighbors. Conservation guaranteed.
/// Axiom 7: only adjacent nodes exchange. Axiom 4: dissipation per transfer.
pub fn radial_diffuse(field: &RadialField, conductivity: f32, dt: f32) -> RadialField;

/// Detect peaks: nodes where qe > local_mean × PEAK_THRESHOLD_FACTOR.
/// Returns array of (axial_idx, radial_idx, qe) for up to MAX_PEAKS peaks.
/// Axiom 6: peaks are not programmed — they emerge from diffusion dynamics.
pub fn detect_peaks(field: &RadialField, threshold_factor: f32) -> Vec<(u8, u8, f32)>;

/// Gradient direction at a node: vector pointing from low to high qe.
/// Axiom 7: computed from adjacent neighbors only.
pub fn gradient_at(field: &RadialField, ax: usize, rad: usize) -> (f32, f32);

/// Aspect ratio of a peak: ratio of axial extent to radial extent.
/// High → elongated (tube/limb). Low → compact (bulb/head).
/// Axiom 6: shape classification from physics, not labels.
pub fn peak_aspect_ratio(field: &RadialField, ax: u8, rad: u8) -> f32;

/// Per-node radii for variable-thickness mesh. 2D version of field_to_radii.
/// Each axial station gets radius = base × f(radial_profile).
/// Axiom 1: radius ∝ sqrt(local_qe / mean_qe).
pub fn radial_to_axial_radii(field: &RadialField, base_radius: f32) -> [f32; AXIAL_NODES];

/// Distribute scalar qe into 2D field using genome biases.
/// growth → tips, resilience → center, branching → lateral peaks.
/// Initialized from center (axial midpoint, all radials equal) → isotropy.
/// Axiom 6: bilateral symmetry emerges from isotropic initialization.
pub fn distribute_to_radial(total_qe: f32, growth: f32, resilience: f32, branching: f32) -> RadialField;

/// Detect joints: axial stations where ALL radial sectors have qe below threshold.
/// A valley across the full cross-section = a natural segmentation point.
/// Axiom 6: joints are energy valleys, not programmed attachment points.
pub fn detect_joints(field: &RadialField, bond_profile: &[f32; AXIAL_NODES]) -> Vec<(u8, f32)>;

/// Frequency field for 2D radial (per-node tint variation).
pub fn radial_freq_entrain(freq: &RadialField, coupling: f32, dt: f32) -> RadialField;
```

## Data Structure Changes

### `EntitySlot` (batch/arena.rs)

```rust
// BEFORE:
pub qe_field:   [f32; 8],
pub freq_field:  [f32; 8],

// AFTER:
pub qe_field:   [[f32; 4]; 8],   // 8 axial × 4 radial = 32 nodes (128 bytes)
pub freq_field:  [[f32; 4]; 8],   // matching radial frequency field (128 bytes)
```

Impact: EntitySlot grows by +192 bytes (from 32 × 4 = 128 extra per field × 2 fields - 64 existing).
Total EntitySlot: ~400 bytes. 64 entities × 400 = 25.6 KB per world. 1M worlds = 25.6 GB.

### `creature_builder.rs` (geometry_flow/)

```rust
// BEFORE: build_creature_mesh_with_field(biases, qe_field: &[f32;8], ...)
// AFTER:  build_creature_mesh_radial(biases, qe_field: &[[f32;4];8], ...)

// New logic:
// 1. Trunk: axial radii from radial_to_axial_radii() (same as before, averaged across sectors)
// 2. Detect peaks in radial field
// 3. For each lateral peak: spawn sub-mesh at peak position
//    - Position: axial station × radial sector → 3D offset from spine
//    - Length: peak_aspect_ratio() → fineness → GF1 spine length
//    - Radius: sqrt(peak_qe / mean_qe) × base
//    - Direction: gradient_at() → outward from body axis
// 4. merge_meshes([trunk, ...appendages])
```

## Emergence Levels

### Level 4 — Bilateral (this design doc)

Isotropic initialization from center + 2D diffusion → symmetric field.
Lateral peaks at sectors 1 and 3 at same axial station → bilateral pair.
Growth bias → axial tip emphasis → distinct "head" and "tail".

### Level 5 — Extremities (natural extension)

More peaks = more appendages. `optimal_appendage_count()` already computes N from thermodynamics.
Each peak generates a sub-mesh. Peaks at different axial stations → legs at different heights.

### Level 6 — Articulation (future)

`detect_joints()` finds qe valleys along each appendage's axial profile.
Low `bond_energy` at valley → flexible joint. High → rigid segment.
Forward kinematics: joint angles from velocity direction + gravity.

### Level 7 — Complex (future)

Extend to 3D field (8×4×4 = 128 nodes) for volumetric body.
Marching cubes over 3D density field → mesh.
Or keep GF1 tubes but with 3D placement from 3D field peaks.

## Invariants

- **INV-M1**: `radial_total(field) == entity.qe` after every diffusion step.
- **INV-M2**: Bilateral symmetry emerges from isotropic init — never forced.
- **INV-M3**: Peak count and position are computed, never assigned.
- **INV-M4**: Joint locations are energy valleys, never hardcoded indices.
- **INV-M5**: All equations are pure functions in `blueprint/equations/`.
- **INV-M6**: Zero labels in production code (`Head`, `Leg`, `Arm`, etc.).

## Dependencies

### Reuses (no changes needed)
- `equations::entity_shape::lateral_offset()` — bilateral positioning math
- `equations::inferred_world_geometry::body_plan::SymmetryMode` — enum
- `equations::internal_field::field_to_radii()` — sqrt scaling pattern
- `geometry_flow::build_flow_mesh_variable_radius()` — tube with per-node radii
- `geometry_flow::merge_meshes()` — combine trunk + appendages
- `topology::generators::slope` — gradient stencil reference

### Modifies
- `batch/arena.rs` — EntitySlot field sizes
- `batch/systems/internal_field.rs` — call 2D diffusion
- `batch/constants.rs` — add radial constants
- `geometry_flow/creature_builder.rs` — radial field → appendage meshes

### Creates
- `blueprint/equations/radial_field.rs` — all 2D radial math

## Testing Strategy

- **Unit**: Every equation function tested independently with edge cases (empty field, single peak, all-equal, conservation).
- **Property**: `radial_total(radial_diffuse(field)) == radial_total(field)` (conservation).
- **Property**: `detect_peaks(uniform_field) == []` (no peaks in uniform).
- **Property**: isotropic init → `field[ax][0] == field[ax][2]` (dorsal == ventral at start).
- **Integration**: Run batch evolution 100 gens with radial field → verify diverse morphologies emerge.
- **Visual**: `evolve_and_view` shows bilateral organisms, not just tubes.
