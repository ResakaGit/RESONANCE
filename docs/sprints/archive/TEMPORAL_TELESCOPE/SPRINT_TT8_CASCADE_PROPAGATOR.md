# TT-8: Propagador de Cascada

**Objetivo:** Función pura que aplica correcciones locales al estado del Telescopio a partir de un `DiffReport`. Las correcciones se propagan a vecinos con atenuación (Axioma 7). La propagación es finita (damped) — muere en 2-3 hops.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (propagación espacial + atenuación)
**Bloqueado por:** TT-4 (DiffReport)
**Desbloquea:** TT-9 (pipeline reconciliation)

---

## Entregable

### `src/batch/telescope/cascade.rs`

```rust
/// Reporte de cascada aplicada.
#[derive(Clone, Debug)]
pub struct CascadeReport {
    pub entities_corrected: u16,     // directamente del diff
    pub entities_cascaded: u16,      // corregidos por propagación
    pub total_affected: u16,         // corrected + cascaded
    pub max_hops: u8,                // profundidad máxima alcanzada
    pub total_qe_correction: f32,    // |Σ Δqe| aplicado
}

/// Aplica correcciones del DiffReport al mundo especulativo.
/// Para diffs LOCAL: corrige entidades afectadas + propaga a vecinos con atenuación.
/// Para diffs SYSTEMIC: memcpy completo del anchor (no cascade).
/// Para diffs PERFECT: no-op.
///
/// Stateless: no muta `anchor`. Muta `telescope` in-place (es la copia descartable).
pub fn cascade(
    telescope: &mut SimWorldFlat,
    anchor: &SimWorldFlat,
    diff: &DiffReport,
    max_hops: u8,
    attenuation_per_hop: f32,
    correction_epsilon: f32,
) -> CascadeReport

/// Encuentra vecinos de una entidad dentro del radio de interacción.
/// Retorna slice de índices. Stack-allocated (max 128 neighbors).
pub fn neighbors_within_radius(
    positions: &[[f32; 2]],
    alive_mask: u128,
    entity_idx: usize,
    radius_sq: f32,
) -> ([usize; 128], usize)
```

---

## Contrato

`cascade` muta `telescope` in-place — es la copia especulativa (descartable). `anchor` es read-only (la verdad). Para SYSTEMIC: `*telescope = anchor.clone()` (memcpy ~100KB). Para PERFECT: retorna `CascadeReport` con todos los contadores en 0.

`neighbors_within_radius` usa array fijo `[usize; 128]` — sin heap, sin Vec. El segundo valor del retorno es el conteo de vecinos encontrados.

---

## Algoritmo de cascada (LOCAL)

```
1. Para cada entidad en diff con |qe_delta| > threshold:
   a. Corregir: telescope[i].qe = anchor[i].qe
   b. Corregir: telescope[i].position = anchor[i].position (si pos_delta > ε)
   c. Marcar como "corrected"

2. Para cada entidad corrected:
   a. Buscar vecinos dentro de ISOLATION_RANGE_SQ
   b. Para cada vecino no-corrected:
      correction = qe_delta_original × attenuation_per_hop
      Si |correction| > correction_epsilon:
        telescope[neighbor].qe += correction
        Marcar como "cascaded"
   c. Repetir para vecinos de cascaded (hop 2, 3, ... hasta max_hops)
   d. Atenuación se acumula: hop_k correction = original × attenuation^k
```

---

## Preguntas para tests

1. `cascade` con DiffClass::Perfect → ¿CascadeReport con todo en 0, telescope sin cambio?
2. `cascade` con DiffClass::Systemic → ¿telescope = anchor completo, total_affected = all?
3. `cascade` con 1 entidad diferente, sin vecinos → ¿entities_corrected=1, cascaded=0?
4. `cascade` con 1 entidad diferente, 3 vecinos → ¿cascaded=3 (si correction > ε)?
5. `cascade` con attenuation=0.1, correction=10.0 → ¿hop 1: 1.0, hop 2: 0.1, hop 3: stop (< ε)?
6. `cascade` con max_hops=1 → ¿nunca propaga más allá del vecino directo?
7. `cascade` con alive_mismatch (anchor muerta, telescope viva) → ¿telescope entidad marcada dead?
8. `neighbors_within_radius` con entidad aislada → ¿count = 0?
9. `neighbors_within_radius` con 3 entidades en rango → ¿count = 3, indices correctos?
10. `cascade` conservación: |Σ corrections| ≤ |Σ diffs| (nunca amplifica, solo distribuye)

---

## Integración

- **Consume:** TT-4 (`DiffReport`), `SimWorldFlat`, `ISOLATION_RANGE_SQ` (de `batch/constants.rs`)
- **Consumido por:** TT-9 (pipeline aplica cascade tras reconciliación)
- **No modifica:** `batch/constants.rs`, `batch/arena.rs` (usa, no cambia)
