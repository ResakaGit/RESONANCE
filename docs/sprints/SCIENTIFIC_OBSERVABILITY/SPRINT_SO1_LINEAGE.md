# SO-1: Lineage Tracking

**Objetivo:** Cada genoma sabe quién fue su padre y en qué generación nació. Sin esto, no hay árboles filogenéticos, no hay análisis de especiación, no hay papers publicables.

**Estado:** PENDIENTE
**Esfuerzo:** S (~60 LOC)
**Bloqueado por:** —

---

## Diseño

### Nuevo tipo: `LineageId`

```rust
// src/batch/lineage.rs (NUEVO)

/// Identificador único de linaje. Determinista: hash(parent_id, child_index, generation).
/// Deterministic lineage identifier: hash(parent_id, child_index, generation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineageId(pub u64);

impl LineageId {
    /// Genera id para entidad sin padre (abiogénesis / seed initial).
    pub fn root(seed: u64, slot_index: u8) -> Self {
        Self(crate::blueprint::equations::determinism::hash_f32_slice(
            &[seed as f32, slot_index as f32],
        ))
    }

    /// Genera id para offspring.
    pub fn child(parent: LineageId, child_index: u8, generation: u32) -> Self {
        Self(crate::blueprint::equations::determinism::hash_f32_slice(
            &[parent.0 as f32, child_index as f32, generation as f32],
        ))
    }
}
```

### Extensión de `GenomeBlob`

Añadir metadata de lineage al GenomeBlob sin romper el formato binario existente:

```rust
// src/batch/lineage.rs

/// Genome + metadata de linaje. El GenomeBlob puro sigue intacto (22 bytes).
/// La metadata es optional y no se serializa en el formato legacy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackedGenome {
    pub genome:      GenomeBlob,
    pub lineage_id:  LineageId,
    pub parent_id:   Option<LineageId>,
    pub birth_gen:   u32,
}
```

**NO se modifica GenomeBlob.** TrackedGenome es un wrapper que añade metadata sin tocar el formato existente de 22 bytes.

### Integración en `GeneticHarness`

En `repopulate()`, al crear offspring:

```rust
let child_lineage = LineageId::child(parent.lineage_id, child_idx, generation);
let tracked = TrackedGenome {
    genome: child_genome,
    lineage_id: child_lineage,
    parent_id: Some(parent.lineage_id),
    birth_gen: generation,
};
```

---

## Constantes

Ninguna nueva. `LineageId` se genera por hash determinista (misma función que el PRNG existente: `determinism::hash_f32_slice`).

---

## Tests (TDD)

```
lineage_id_root_deterministic_same_inputs
lineage_id_root_different_slots_differ
lineage_id_child_deterministic_same_inputs
lineage_id_child_different_parents_differ
lineage_id_child_different_generations_differ
tracked_genome_preserves_genome_blob_bitwise
tracked_genome_root_has_no_parent
tracked_genome_child_has_parent
```

---

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/batch/lineage.rs` | **NUEVO** — LineageId, TrackedGenome |
| `src/batch/mod.rs` | + `pub mod lineage` + re-exports |
| `src/batch/harness.rs` | + tracking en repopulate (wrap GenomeBlob → TrackedGenome) |

---

## Invariantes

- `LineageId` es determinista: mismo seed + mismo parent + mismo index → mismo id
- `GenomeBlob` no se modifica (backward compat con save/load existente)
- `TrackedGenome` es Copy (stack-allocated, zero heap)
- El tracking es opt-in: `GeneticHarness` puede correr con o sin lineage
