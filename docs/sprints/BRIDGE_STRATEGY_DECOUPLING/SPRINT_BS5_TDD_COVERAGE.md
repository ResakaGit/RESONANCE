# BS-5: TDD — Tests Unitarios + Integración Tier 1

**Objetivo:** Cerrar los gaps de cobertura de tests identificados en la auditoría. Priorizar axiom-critical systems y ecuaciones sin cobertura.

**Estado:** PENDIENTE
**Esfuerzo:** L (~400 LOC de tests)
**Bloqueado por:** BS-4 (bridges nuevos necesitan tests de integración)
**Desbloquea:** —

---

## Criterio de prioridad

| Tier | Criterio | Ejemplos |
|------|----------|----------|
| **T1** | Axiom-critical, 0 tests | abiogenesis, awakening, reproduction integration |
| **T2** | Ecuaciones sin tests (0 cobertura) | finite_helpers, flux, geometry_flow, lifecycle |
| **T3** | Sistemas con equation tests pero sin integration | emergence ET-2/5/6/9, trophic |
| **T4** | Test naming cleanup, property tests | radial_field renaming, proptest expansion |

Este sprint cubre **T1 + T2**. T3 y T4 son backlog futuro.

---

## T1: Integration Tests — Sistemas Axiom-Critical

### 1. `abiogenesis_system` — Coherence-driven spawn (Axiom 1+8)

```rust
// tests/abiogenesis_integration.rs

/// Spawn ocurre cuando coherence_gain > dissipation_loss en field grid.
#[test]
fn abiogenesis_high_coherence_spawns_entity() {
    // Setup: MinimalPlugins + EnergyFieldGrid con zona de alta coherencia
    // Act: run abiogenesis_system
    // Assert: nueva entidad spawned con BaseEnergy > 0
}

#[test]
fn abiogenesis_low_coherence_no_spawn() {
    // Setup: grid con coherencia uniforme baja
    // Act: run abiogenesis_system
    // Assert: 0 entidades nuevas
}

#[test]
fn abiogenesis_respects_dissipation_threshold() {
    // Setup: coherencia justo debajo del threshold
    // Act: run
    // Assert: no spawn
}

#[test]
fn abiogenesis_spawned_entity_has_valid_layers() {
    // Assert: spawned entity tiene BaseEnergy, SpatialVolume, OscillatorySignature, MatterCoherence
}
```

### 2. `awakening_system` — Inert → BehavioralAgent (Axiom 8)

```rust
// tests/awakening_integration.rs

#[test]
fn awakening_coherence_above_threshold_adds_behavioral_agent() {
    // Setup: entity con coherencia > threshold (axiom-derived)
    // Act: run awakening_system
    // Assert: entity has BehavioralAgent component
}

#[test]
fn awakening_coherence_below_threshold_no_agent() {
    // Setup: entity con coherencia < threshold
    // Act: run
    // Assert: no BehavioralAgent
}

#[test]
fn awakening_already_agent_no_duplicate() {
    // Setup: entity ya tiene BehavioralAgent
    // Act: run
    // Assert: still exactly 1 BehavioralAgent
}
```

### 3. `senescence_death_system` — Gompertz mortality (Axiom 4)

```rust
// tests/senescence_integration.rs

#[test]
fn senescence_young_entity_survives() {
    // Setup: entity con age = 10 ticks
    // Act: run senescence_death_system
    // Assert: entity alive
}

#[test]
fn senescence_old_entity_dies() {
    // Setup: entity con age > max_viable_age
    // Act: run
    // Assert: entity dead (Dead component or despawned)
}

#[test]
fn senescence_gompertz_hazard_increases_with_age() {
    // Property: hazard(age+1) >= hazard(age) para todo age
    // Fuzz: 100 age values
}
```

### 4. `reproduction_spawn_system` — Mutation + Conservation (Axiom 5)

```rust
// tests/reproduction_integration.rs

#[test]
fn reproduction_offspring_qe_leq_parent_drain() {
    // Axiom 5: sum(offspring.qe) <= drained_from_parent
    // Setup: parent con qe=500, capabilities include REPRODUCE
    // Act: run reproduction_spawn_system
    // Assert: offspring.qe <= parent_drained
}

#[test]
fn reproduction_offspring_has_mutated_inference_profile() {
    // Setup: parent con InferenceProfile(growth=0.5, mobility=0.3, ...)
    // Act: run
    // Assert: offspring profile != parent profile (mutation occurred)
}

#[test]
fn reproduction_parent_drained_after_spawn() {
    // Assert: parent.qe decreased by offspring cost
}
```

### 5. `basal_drain_system` — Kleiber scaling (Axiom 4)

```rust
// tests/basal_drain_integration.rs

#[test]
fn basal_drain_reduces_energy_each_tick() {
    // Setup: entity con BaseEnergy(100), SpatialVolume(2.0)
    // Act: run basal_drain_system
    // Assert: energy < 100
}

#[test]
fn basal_drain_larger_entity_drains_more() {
    // Axiom 4 + Kleiber: drain ∝ radius^0.75
    // Setup: entity_small(r=1), entity_large(r=4)
    // Act: run
    // Assert: drain_large > drain_small
}

#[test]
fn basal_drain_zero_radius_drains_zero() {
    // Edge case
}
```

---

## T2: Unit Tests — Ecuaciones sin Cobertura

### `blueprint/equations/finite_helpers.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn finite_non_negative_nan_returns_zero()       { assert_eq!(finite_non_negative(f32::NAN), 0.0); }
    #[test] fn finite_non_negative_neg_inf_returns_zero()    { assert_eq!(finite_non_negative(f32::NEG_INFINITY), 0.0); }
    #[test] fn finite_non_negative_pos_inf_returns_zero()    { assert_eq!(finite_non_negative(f32::INFINITY), 0.0); }
    #[test] fn finite_non_negative_negative_returns_zero()   { assert_eq!(finite_non_negative(-1.0), 0.0); }
    #[test] fn finite_non_negative_zero_returns_zero()       { assert_eq!(finite_non_negative(0.0), 0.0); }
    #[test] fn finite_non_negative_positive_returns_value()  { assert_eq!(finite_non_negative(42.0), 42.0); }

    #[test] fn finite_unit_nan_returns_zero()                { assert_eq!(finite_unit(f32::NAN), 0.0); }
    #[test] fn finite_unit_above_one_clamps()                { assert_eq!(finite_unit(2.0), 1.0); }
    #[test] fn finite_unit_below_zero_clamps()               { assert_eq!(finite_unit(-0.5), 0.0); }
    #[test] fn finite_unit_mid_returns_value()               { assert!((finite_unit(0.5) - 0.5).abs() < 1e-6); }
}
```

### `blueprint/equations/shape_cache.rs` (nuevo, de BS-3)

Cubierto en BS-3.

### `blueprint/equations/metabolic_graph.rs` (nuevo, de BS-3)

Cubierto en BS-3.

### `blueprint/equations/flux/mod.rs`

```rust
// Identificar funciones públicas y testear cada una con:
// - Input zero → output zero o default
// - Input negativo → clamp/zero
// - Input NaN/Inf → safe default
// - Input normal → expected value
// - Boundary values → no panic
```

### `blueprint/equations/field_color/spectrum.rs`

```rust
game_frequency_to_hue_zero_range_returns_zero
game_frequency_to_hue_mid_frequency_returns_mid_hue
game_frequency_to_hue_nan_returns_zero
hsv01_to_linear_rgb_pure_red
hsv01_to_linear_rgb_pure_green
hsv01_to_linear_rgb_nan_saturation_returns_neutral
linear_rgb_from_game_hz_spectrum_known_frequency
```

---

## Test naming cleanup (T4 — backlog, no este sprint)

Tests en `radial_field.rs` que no siguen `<function>_<condition>_<expected>`:

| Actual | Correcto |
|--------|----------|
| `total_uniform` | `radial_total_uniform_field_returns_sum` |
| `total_zeros` | `radial_total_zero_field_returns_zero` |
| `diffuse_conserves` | `diffuse_uniform_field_conserves_total` |
| `diffuse_smooths_spike` | `diffuse_spike_center_smooths_neighbors` |

**NO hacer en este sprint** — backlog para mantener scope controlado.

---

## Property tests expansion (T4 — backlog)

Candidatos para `proptest` en `tests/property_conservation.rs`:

```
// Emergence equations
prop_mutualism_benefit_non_negative(intake: 0..1000, factor: 0..1)
prop_parasitism_drain_leq_host_qe(host_qe: 0..10000, rate: 0..1)
prop_niche_overlap_symmetric(niche_a, niche_b) // overlap(a,b) == overlap(b,a)

// Deterministic RNG
prop_unit_f32_always_in_01(state: u64) // result ∈ [0, 1)
prop_range_f32_in_bounds(state: u64, min: f32, max: f32) // result ∈ [min, max)
prop_gaussian_alignment_reflexive(f: f32, bw: f32) // alignment(f, f, bw) == 1.0
```

**NO hacer en este sprint.**

---

## Archivos nuevos/tocados

| Archivo | Cambio |
|---------|--------|
| `tests/abiogenesis_integration.rs` | **NUEVO** — 4 tests |
| `tests/awakening_integration.rs` | **NUEVO** — 3 tests |
| `tests/senescence_integration.rs` | **NUEVO** — 3 tests |
| `tests/reproduction_integration.rs` | **NUEVO** — 3 tests |
| `tests/basal_drain_integration.rs` | **NUEVO** — 3 tests |
| `src/blueprint/equations/finite_helpers.rs` | + 10 unit tests inline |
| `src/blueprint/equations/flux/mod.rs` | + unit tests inline |
| `src/blueprint/equations/field_color/spectrum.rs` | + unit tests inline |

---

## Checklist pre-merge

- [ ] 16+ integration tests (T1) verdes
- [ ] 20+ unit tests (T2) verdes
- [ ] Test names siguen `<fn>_<condition>_<expected>`
- [ ] Zero mocks (MinimalPlugins + real components)
- [ ] `cargo test` total count > 2500
- [ ] Axiom invariants cubiertos: Conservation (Ax 5), Dissipation (Ax 4), Coherence (Ax 8)
