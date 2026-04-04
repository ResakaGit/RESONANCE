# TT-7: Puente de Calibración

**Objetivo:** Función pura que convierte los resultados de la reconciliación (DiffReport + historial) en nuevos pesos para los normalizadores del Telescopio. Es el mecanismo de feedback: Ancla → Puente → Telescopio. Stateless: `(diff, weights, history) → weights`.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (lógica de calibración empírica, no ML)
**Bloqueado por:** TT-4 (DiffReport), TT-5 (TelescopeState, ReconciliationRecord)
**Desbloquea:** TT-9 (pipeline usa weights actualizados)

---

## Entregable

### `src/batch/telescope/calibration_bridge.rs`

```rust
/// Calibra los pesos de los normalizadores basándose en el resultado de la reconciliación.
/// Stateless: (diff, current_weights, recent_history) → new_weights.
///
/// Lógica:
///   PERFECT → mantener o relajar pesos (el telescopio acertó)
///   LOCAL   → identificar qué métrica debió haber alertado, ajustar ese peso
///   SYSTEMIC → ajustar todos los pesos conservadoramente
pub fn calibrate(
    record: &ReconciliationRecord,
    current_weights: &NormalizerWeights,
    history: &[ReconciliationRecord],
    config: &CalibrationConfig,
) -> NormalizerWeights

/// Identifica qué normalizador falló cuando la proyección fue LOCAL/SYSTEMIC.
/// Heurística: la métrica que más cambió entre fork y reconciliation es la culpable.
pub fn identify_weak_normalizer(
    metrics_at_fork: &RegimeMetrics,
    diff: &DiffReport,
) -> NormalizerDimension

/// Dimensiones de los normalizadores (para identificar cuál ajustar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizerDimension {
    Hurst,
    Inertia,
    Fisher,
    Horizon,
    EventDensity,
    Entropy,
}

/// Config del puente de calibración.
#[derive(Clone, Copy, Debug)]
pub struct CalibrationConfig {
    pub learning_rate: f32,         // cuánto ajustar por reconciliación (default 0.1)
    pub min_history_for_adjust: u16, // mínimo de records antes de ajustar (default 8)
    pub weight_floor: f32,          // peso mínimo (default 0.1 — nunca ignorar una métrica)
    pub weight_ceiling: f32,        // peso máximo (default 5.0 — nunca sobreponderar)
}
```

---

## Contrato stateless

`calibrate` recibe datos inmutables y retorna datos nuevos. `history` es `&[ReconciliationRecord]` — el caller decide cuántos records pasar (últimos 256 del ring buffer). El puente NO guarda estado propio — todo lo que necesita viene como parámetro.

---

## Preguntas para tests

1. `calibrate` con PERFECT → ¿weights sin cambio significativo (Δ < 0.01)?
2. `calibrate` con SYSTEMIC → ¿todos los weights se multiplican por (1 - learning_rate)?
3. `calibrate` con LOCAL + Hurst identificado como débil → ¿hurst_weight sube?
4. `calibrate` nunca produce weights < weight_floor (0.1)
5. `calibrate` nunca produce weights > weight_ceiling (5.0)
6. `calibrate` con history vacío → ¿retorna current_weights sin cambio?
7. `calibrate` con history < min_history_for_adjust → ¿retorna current_weights?
8. `identify_weak_normalizer` cuando Fisher estaba alto al fork y diff es LOCAL → ¿Fisher?
9. `identify_weak_normalizer` cuando event_rate era alto → ¿EventDensity?
10. Convergencia: 100 calibrations con diffs aleatorios → ¿weights se estabilizan? (no divergen)

---

## Integración

- **Consume:** TT-4 (`DiffReport`), TT-5 (`ReconciliationRecord`, `NormalizerWeights`)
- **Consumido por:** TT-9 (pipeline actualiza weights tras reconciliación)
- **No modifica:** Nada existente
- **Flujo:** Ancla produce DiffReport → Puente convierte en NormalizerWeights → Telescopio los usa
