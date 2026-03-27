# Sprint AI-3 — Capability Thresholds from Density + Coherence

**Módulo:** `src/blueprint/equations/abiogenesis/axiomatic.rs`, `src/blueprint/equations/awakening.rs`
**Tipo:** Refactor ecuaciones puras
**Eje axiomático:** Axiom 1 (energy = state), Axiom 4 (dissipation), Axiom 8 (coherence)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** AI-2 (density thresholds)
**Esfuerzo:** Bajo (~30min)

---

## Qué existe hoy

```rust
// En axiomatic.rs — hardcoded:
const MOVE_DENSITY_MIN: f32 = 50.0;          // ARBITRARY
const MOVE_DENSITY_MAX: f32 = 600.0;         // ARBITRARY
const SENSE_COHERENCE_MIN: f32 = 0.4;        // ARBITRARY
const BRANCH_QE_MIN: f32 = 30.0;             // ARBITRARY

// En awakening.rs — duplicado:
pub const AWAKENING_THRESHOLD: f32 = 0.3;    // EMPIRICALLY-CALIBRATED
pub const AWAKENING_MIN_QE: f32 = 20.0;      // Duplicate of SELF_SUSTAINING_QE_MIN
```

## Derivación axiomática

- **MOVE:** Requiere density en régimen líquido (puede fluir) pero no gas (demasiado difuso).
  `move_min = liquid_threshold × 0.5`, `move_max = gas_threshold × 1.5`
- **SENSE:** Requiere coherencia sobre el noise floor de disipación.
  `sense_min = DISSIPATION_SOLID / (DISSIPATION_SOLID + bandwidth_factor)`
- **BRANCH:** Requiere energía para dividirse y que ambas mitades sobrevivan.
  `branch_qe = self_sustaining_qe_min × 2`
- **AWAKENING_THRESHOLD:** Break-even algebraico donde coherencia = 2× disipación.
  `threshold = 1/3` (de la fórmula del potencial)
- **AWAKENING_MIN_QE:** = `self_sustaining_qe_min()` — eliminar duplicado

## Tareas

1. Reemplazar 4 constantes en `axiomatic.rs` con `derived_thresholds::*()`.
2. En `awakening.rs`: importar `spawn_potential_threshold()` y `self_sustaining_qe_min()` de `derived_thresholds`. Eliminar `AWAKENING_THRESHOLD` y `AWAKENING_MIN_QE`.
3. Actualizar tests.

## Criterio de cierre

- `AWAKENING_MIN_QE` eliminado (usar `derived_thresholds::self_sustaining_qe_min()`)
- `AXIOMATIC_SPAWN_THRESHOLD` y `AWAKENING_THRESHOLD` consolidados en `spawn_potential_threshold()`
- Capability tests pasan: entities gain MOVE cuando density está en rango derivado
