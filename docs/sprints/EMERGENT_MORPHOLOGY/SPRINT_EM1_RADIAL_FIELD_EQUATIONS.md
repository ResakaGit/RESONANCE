# Sprint EM-1 — Radial Field Equations: 2D Diffusion + Peak Detection

**Modulo:** `src/blueprint/equations/radial_field.rs` (nuevo)
**Tipo:** Ecuaciones puras. Zero Bevy. Zero side effects.
**Onda:** Sin bloqueantes.
**Estado:** ⏳ Pendiente

---

## Contexto

El campo interno actual (`internal_field.rs`) es 1D: 8 nodos axiales.
Solo puede producir variación de grosor a lo largo de un eje.
Para bilateral symmetry y extremidades necesitamos un campo **2D radial**:
8 axiales × 4 radiales = 32 nodos.

---

## Objetivo

Crear `radial_field.rs` con todas las ecuaciones puras para:
- Difusión 2D (axial + radial neighbors)
- Detección de picos (local maxima)
- Gradiente direccional por nodo
- Aspect ratio de picos (bulbo vs tubo)
- Distribución de qe desde genome
- Detección de joints (valles cross-sectional)

---

## Responsabilidades

### EM-1A: Tipos y constantes

```rust
pub const AXIAL_NODES: usize = 8;
pub const RADIAL_SECTORS: usize = 4;
pub type RadialField = [[f32; RADIAL_SECTORS]; AXIAL_NODES];

pub const PEAK_THRESHOLD_FACTOR: f32 = 1.8;
pub const APPENDAGE_QE_MIN: f32 = 0.5;
pub const JOINT_FLEX_THRESHOLD: f32 = 5.0;
pub const MAX_PEAKS: usize = 8;
```

### EM-1B: radial_total + radial_diffuse

```rust
/// Sum all 32 nodes.
pub fn radial_total(field: &RadialField) -> f32;

/// 2D diffusion: axial neighbors (i±1, same sector) + radial neighbors (same i, sector±1 mod 4).
/// Conservation: sum before == sum after.
/// Axiom 7: adjacent only. Axiom 4: dissipation in transfer.
pub fn radial_diffuse(field: &RadialField, conductivity: f32, dt: f32) -> RadialField;
```

Radial neighbors wrap: sector 3 ↔ sector 0 (torus topology on radial axis).

### EM-1C: detect_peaks

```rust
/// Find nodes where qe > mean × PEAK_THRESHOLD_FACTOR AND qe > all 4 neighbors.
/// Returns up to MAX_PEAKS (axial_idx, radial_idx, qe_value).
/// Axiom 6: peaks emerge from diffusion, not programmed.
pub fn detect_peaks(field: &RadialField, threshold_factor: f32) -> [(u8, u8, f32); MAX_PEAKS];
```

A peak is a node higher than its 4 neighbors (axial±1, radial±1) AND above threshold.

### EM-1D: gradient_at + peak_aspect_ratio

```rust
/// Gradient vector (axial_component, radial_component) at node.
/// Axiom 7: from adjacent neighbors only.
pub fn gradient_at(field: &RadialField, ax: usize, rad: usize) -> (f32, f32);

/// Aspect ratio: axial extent / radial extent of peak region.
/// High → elongated (tube). Low → compact (bulb).
pub fn peak_aspect_ratio(field: &RadialField, ax: u8, rad: u8) -> f32;
```

### EM-1E: distribute_to_radial

```rust
/// Genome → 2D field. Initialized from center (isotropic → bilateral emerges).
/// growth → axial tips. resilience → axial center. branching → lateral sectors.
/// Axiom 6: bilateral symmetry from isotropic init, not forced mirroring.
pub fn distribute_to_radial(
    total_qe: f32, growth: f32, resilience: f32, branching: f32,
) -> RadialField;
```

Key: `branching` distributes energy to sectors 1 and 3 (lateral).
But it does NOT mirror — it adds to all laterals equally.
If the field is initialized symmetrically, diffusion preserves symmetry.
If mutation breaks a bias, asymmetry emerges naturally.

### EM-1F: detect_joints

```rust
/// Find axial stations where ALL radial sectors have qe below threshold.
/// A cross-sectional valley = natural segmentation/joint.
/// Returns (axial_idx, min_qe_at_station).
pub fn detect_joints(
    field: &RadialField, bond_profile: &[f32; AXIAL_NODES],
) -> [(u8, f32); AXIAL_NODES];
```

### EM-1G: radial_to_axial_radii + radial_freq_entrain

```rust
/// Average across radial sectors → per-station radius for trunk mesh.
pub fn radial_to_axial_radii(
    field: &RadialField, base_radius: f32, min_ratio: f32, max_ratio: f32,
) -> [f32; AXIAL_NODES];

/// Frequency entrainment across 2D neighbors.
pub fn radial_freq_entrain(
    freq: &RadialField, coupling: f32, dt: f32,
) -> RadialField;
```

---

## NO hace

- No modifica EntitySlot — eso es EM-2.
- No modifica creature_builder — eso es EM-3.
- No detecta joints articulados — eso es EM-4.
- No importa Bevy ni batch.

---

## Dependencias

- `crate::blueprint::equations::internal_field` — reusa `NODE_COUNT` pattern, `field_total` concept.
- Ninguna dependencia nueva.

---

## Criterios de aceptacion

### Conservation
- `radial_total(radial_diffuse(field, k, dt)) == radial_total(field)` para todo field válido.
- `radial_total(distribute_to_radial(100.0, g, r, b)) == 100.0`.

### Peak detection
- Uniform field → 0 peaks.
- Single spike → 1 peak at spike location.
- Bilateral spikes (sectors 1 and 3, same axial) → 2 peaks.

### Symmetry emergence
- `distribute_to_radial(qe, g, r, b)`: sectors 1 and 3 have equal qe (isotropic init).
- After N diffusion steps on isotropic init: sectors 1 and 3 remain equal.

### Gradient
- Flat field → gradient (0, 0) everywhere.
- Single spike → gradient points away from spike at neighbors.

### Joints
- Uniform field → 0 joints.
- Field with axial valley (all sectors low at station 4) → joint at station 4.

### General
- `cargo test --lib` sin regresión.
- Zero `use bevy::` en el archivo.
- Todas las funciones son `pub fn(inputs) -> output`. Sin `&mut`. Sin side effects.

---

## Referencias

- `docs/design/EMERGENT_MORPHOLOGY.md` — design doc completo
- `src/blueprint/equations/internal_field.rs` — patrón 1D a extender
- `src/topology/generators/slope.rs` — gradient stencil de referencia
