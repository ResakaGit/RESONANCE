# TT-4: Motor de Diff

**Objetivo:** Funciones puras que comparan dos `SimWorldFlat` y producen un `DiffReport` clasificado (PERFECT / LOCAL / SYSTEMIC). El diff es la señal que alimenta al puente de calibración y al cascade propagator.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (comparación campo a campo + clasificación)
**Bloqueado por:** —
**Desbloquea:** TT-7 (calibration bridge), TT-8 (cascade propagator)

---

## Entregables

### 1. `src/batch/telescope/diff.rs`

```rust
/// Diff por entidad entre dos EntitySlot.
#[derive(Clone, Copy, Debug, Default)]
pub struct EntityDiff {
    pub index: usize,
    pub qe_delta: f32,          // anchor.qe - telescope.qe
    pub pos_delta_sq: f32,      // distancia² entre posiciones
    pub freq_delta: f32,        // |anchor.freq - telescope.freq|
    pub alive_mismatch: bool,   // uno vivo y otro muerto
}

/// Clasificación del diff global.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffClass {
    Perfect,   // diff < 0.5% en todas las entidades
    Local,     // 1-10% de entidades difieren > 2%
    Systemic,  // >10% de entidades difieren
}

/// Reporte completo del diff entre Ancla y Telescopio.
#[derive(Clone, Debug)]
pub struct DiffReport {
    pub class: DiffClass,
    pub entity_diffs: [EntityDiff; 128],  // MAX_ENTITIES, stack-allocated
    pub affected_count: u16,               // entidades con diff > threshold
    pub max_qe_delta: f32,                 // mayor diff absoluta de qe
    pub alive_mismatches: u16,             // entidades vivas en uno pero no el otro
    pub mean_qe_error: f32,                // error medio normalizado
}

/// Compara dos SimWorldFlat. Stateless: no muta ninguno.
pub fn world_diff(anchor: &SimWorldFlat, telescope: &SimWorldFlat, threshold_pct: f32) -> DiffReport

/// Compara un EntitySlot entre anchor y telescope.
pub fn entity_diff(anchor: &EntitySlot, telescope: &EntitySlot) -> EntityDiff

/// Clasifica un DiffReport según umbrales.
pub fn classify_diff(affected_count: u16, total_alive: u16, alive_mismatches: u16) -> DiffClass
```

### 2. En `src/batch/arena.rs` (modificación mínima)

Agregar método delegado a `SimWorldFlat`:

```rust
impl SimWorldFlat {
    /// Diff con otro mundo. Delega a telescope::diff::world_diff.
    pub fn diff(&self, other: &SimWorldFlat) -> DiffReport {
        crate::batch::telescope::diff::world_diff(self, other, DIFF_THRESHOLD_PCT)
    }
}
```

---

## Contrato stateless

`world_diff` recibe dos `&SimWorldFlat` (read-only) y retorna `DiffReport` (owned). No muta nada. `entity_diffs` es array fijo `[EntityDiff; 128]` — stack-allocated, sin Vec, sin heap.

---

## Preguntas para tests

1. `world_diff` de dos mundos idénticos → ¿DiffClass::Perfect, affected_count=0?
2. `world_diff` donde 1 entidad tiene qe distinto por 5% → ¿DiffClass::Local, affected_count=1?
3. `world_diff` donde 50% de entidades difieren → ¿DiffClass::Systemic?
4. `entity_diff` con misma entidad → ¿todos los deltas = 0.0, alive_mismatch = false?
5. `entity_diff` donde anchor está muerta y telescope viva → ¿alive_mismatch = true?
6. `world_diff` respeta solo entidades alive (no compara slots muertos en ambos)
7. `mean_qe_error` es media de |delta| / max(qe_anchor, 1.0) sobre entidades vivas
8. `max_qe_delta` es el mayor |delta| absoluto (no relativo)
9. Performance: diff de 128 entidades < 10μs (es O(N) simple loop)
10. `classify_diff` con 0 affected → Perfect, 5/100 → Local, 15/100 → Systemic

---

## Integración

- **Consume:** `SimWorldFlat`, `EntitySlot` (de `batch/arena.rs`)
- **Consumido por:** TT-7 (calibration bridge), TT-8 (cascade), TT-9 (pipeline reconciliation)
- **Modifica:** `batch/arena.rs` (1 método delegado), `batch/mod.rs` (1 línea: `pub mod telescope;`)
