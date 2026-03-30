# Sprint MC-1 — Cell Adhesion: cuándo dos células se unen

**Módulo:** `src/blueprint/equations/cell_adhesion.rs` (nuevo)
**Constantes:** `src/blueprint/constants/multicellular.rs` (nuevo)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente

---

## Objetivo

Ecuación pura que determina si dos entidades se unen formando un enlace estructural.
Basada en: proximidad espacial (Axiom 7) × frequency alignment (Axiom 8).
El enlace tiene costo energético (Axiom 4).

## Diseño

### `adhesion_affinity(freq_a, freq_b, distance, radius_a, radius_b) → f32`

```rust
/// Affinity ∈ [0, 1]. > ADHESION_THRESHOLD → bond forms.
///
/// Axiom 7: decays with distance. Axiom 8: frequency match required.
/// Axiom 4: only forms if both cells have enough qe to pay bond cost.
pub fn adhesion_affinity(
    freq_a: f32, freq_b: f32,
    distance: f32,
    radius_a: f32, radius_b: f32,
) -> f32 {
    let contact_dist = radius_a + radius_b;
    if distance > contact_dist * 2.0 { return 0.0; } // too far (Axiom 7)
    let proximity = 1.0 - (distance / (contact_dist * 2.0)).clamp(0.0, 1.0);
    let freq_align = gaussian_alignment(freq_a, freq_b, ADHESION_FREQ_BANDWIDTH);
    proximity * freq_align // both must be high for adhesion
}
```

### `bond_strength(affinity, qe_a, qe_b) → f32`

```rust
/// Strength of the structural bond. Axiom 1: bond energy from qe.
pub fn bond_strength(affinity: f32, qe_a: f32, qe_b: f32) -> f32 {
    affinity * qe_a.min(qe_b).sqrt() * BOND_STRENGTH_SCALE
}
```

### `adhesion_cost(bond_strength) → f32`

```rust
/// Energy cost to maintain bond per tick. Axiom 4.
pub fn adhesion_cost(strength: f32) -> f32 {
    strength * ADHESION_COST
}
```

## Tests

### Contrato
- `affinity_zero_when_far` — distance > 2× contact → 0
- `affinity_one_when_touching_same_freq` — distance=0, same freq → ~1.0
- `affinity_in_unit` — always ∈ [0, 1]
- `bond_strength_positive` — affinity > 0 → strength > 0
- `adhesion_cost_positive` — strength > 0 → cost > 0

### Lógica (Axioms)
- `affinity_decays_with_distance` — farther = lower (Axiom 7)
- `affinity_decays_with_freq_diff` — different freq = lower (Axiom 8)
- `bond_stronger_with_more_energy` — higher qe = stronger bond (Axiom 1)
- `cost_proportional_to_strength` — strong bond costs more (Axiom 4)

### Errores
- `nan_inputs_safe` — NaN freq/distance → 0 affinity
- `zero_radius_no_panic` — radius=0 → handled

## Criterios de aceptación

- 3 funciones puras, zero heap, zero Bevy.
- Axiom 7 (distance) + Axiom 8 (frequency) en affinity.
- Cost derivado de DISSIPATION_SOLID.
- 12+ tests.
