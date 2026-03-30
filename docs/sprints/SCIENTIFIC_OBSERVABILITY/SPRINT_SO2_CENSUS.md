# SO-2: Population Census (Per-Generation Snapshot)

**Objetivo:** Capturar el estado completo de la población al final de cada generación. Sin esto, solo tenemos agregados (mean fitness) — no distribuciones, no outliers, no trayectorias individuales.

**Estado:** PENDIENTE
**Esfuerzo:** M (~150 LOC)
**Bloqueado por:** SO-1 (lineage ids para vincular snapshots entre generaciones)

---

## Diseño

### `EntitySnapshot` — estado puntual de una entidad

```rust
// src/batch/census.rs (NUEVO)

/// Estado inmutable de una entidad al final de una evaluación.
/// Immutable entity state at the end of an evaluation.
///
/// Stack-allocated, Copy. Capturable sin alloc si se pre-aloca el buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntitySnapshot {
    pub lineage_id:    LineageId,
    pub world_index:   u16,
    pub slot_index:    u8,
    pub archetype:     u8,        // 0=inert, 1=flora, 2=fauna
    pub alive:         bool,
    pub qe:            f32,
    pub radius:        f32,
    pub frequency_hz:  f32,
    pub growth_bias:   f32,
    pub mobility_bias: f32,
    pub branching_bias:f32,
    pub resilience:    f32,
    pub trophic_class: u8,
    pub age_ticks:     u16,
}
```

**Nota:** >4 campos, pero esto NO es un ECS Component — es un struct de datos para export. La regla de 4 campos aplica a Components, no a DTOs de análisis.

### `PopulationCensus` — snapshot de toda la generación

```rust
/// Censo completo de una generación. Contiene N_worlds × M_entities snapshots.
/// Complete census of one generation.
#[derive(Clone, Debug)]
pub struct PopulationCensus {
    pub generation: u32,
    pub snapshots:  Vec<EntitySnapshot>,  // heap: ~64 bytes × N_worlds × 64_slots
}

impl PopulationCensus {
    /// Captura el censo desde un WorldBatch post-evaluación.
    pub fn capture(generation: u32, batch: &WorldBatch, lineages: &[TrackedGenome]) -> Self {
        let mut snapshots = Vec::with_capacity(batch.world_count() * 64);
        for (wi, world) in batch.worlds().iter().enumerate() {
            for (si, slot) in world.entities().iter().enumerate() {
                let lineage_id = lineages.get(wi * 64 + si)
                    .map(|t| t.lineage_id)
                    .unwrap_or(LineageId::root(0, si as u8));
                snapshots.push(EntitySnapshot::from_slot(
                    slot, lineage_id, wi as u16, si as u8,
                ));
            }
        }
        Self { generation, snapshots }
    }

    /// Solo entidades vivas.
    pub fn alive(&self) -> impl Iterator<Item = &EntitySnapshot> {
        self.snapshots.iter().filter(|s| s.alive)
    }

    /// Distribución de un campo (HOF: recibe accessor).
    pub fn distribution<F: Fn(&EntitySnapshot) -> f32>(&self, f: F) -> Vec<f32> {
        self.alive().map(f).collect()
    }
}
```

### `EntitySnapshot::from_slot` — pure function

```rust
impl EntitySnapshot {
    /// Extrae snapshot desde un EntitySlot del batch simulator.
    pub fn from_slot(slot: &EntitySlot, lineage_id: LineageId, world: u16, idx: u8) -> Self {
        Self {
            lineage_id,
            world_index: world,
            slot_index: idx,
            archetype:     slot.archetype,
            alive:         slot.alive(),
            qe:            slot.qe,
            radius:        slot.radius,
            frequency_hz:  slot.frequency_hz,
            growth_bias:   slot.growth_bias,
            mobility_bias: slot.mobility_bias,
            branching_bias:slot.branching_bias,
            resilience:    slot.resilience,
            trophic_class: slot.trophic_class,
            age_ticks:     slot.age_ticks,
        }
    }
}
```

### Integración en `GeneticHarness`

Nuevo campo opt-in:

```rust
pub struct GeneticHarness {
    // ... existente ...
    pub census_history: Option<Vec<PopulationCensus>>,  // None = no tracking (default)
}
```

Habilitación:
```rust
harness.enable_census();  // activa capture por generación
```

---

## HOF: `distribution()` como patrón composable

```rust
// Distribución de fitness:
let fitness_dist = census.distribution(|s| s.qe);

// Distribución de tamaño:
let size_dist = census.distribution(|s| s.radius);

// Distribución de frecuencia (solo fauna):
let freq_dist: Vec<f32> = census.alive()
    .filter(|s| s.archetype == 2)
    .map(|s| s.frequency_hz)
    .collect();
```

Este patrón HOF permite extraer CUALQUIER distribución sin código nuevo.

---

## Tests

```
// EntitySnapshot
snapshot_from_slot_preserves_all_fields
snapshot_from_dead_slot_alive_false
snapshot_from_alive_slot_alive_true

// PopulationCensus
census_capture_counts_all_entities
census_alive_filters_dead
census_distribution_returns_values_for_alive_only
census_distribution_empty_if_all_dead

// HOF composition
census_distribution_qe_returns_energies
census_distribution_radius_returns_radii
census_filter_by_archetype_works

// Integration
harness_with_census_captures_per_generation
harness_without_census_has_none_history
census_count_matches_world_count_times_slots
```

---

## Memoria

- `EntitySnapshot` = ~48 bytes (Copy, stack)
- 100 worlds × 64 slots = 6,400 snapshots/gen
- 500 generaciones × 6,400 = 3.2M snapshots
- Total: ~150 MB para un run completo
- Opt-in: solo se aloca si `enable_census()` llamado

---

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/batch/census.rs` | **NUEVO** — EntitySnapshot, PopulationCensus |
| `src/batch/mod.rs` | + `pub mod census` |
| `src/batch/harness.rs` | + `census_history: Option<Vec<PopulationCensus>>`, + `enable_census()` |
