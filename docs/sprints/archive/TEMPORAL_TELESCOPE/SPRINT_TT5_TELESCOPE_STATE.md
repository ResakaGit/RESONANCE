# TT-5: Estado del Telescopio

**Objetivo:** Tipos de datos para el estado del Telescopio: `TelescopeState` (controlador), `ReconciliationRecord` (historial), `TelescopeConfig` (parámetros). Lógica de K adaptativo. Todo stateless: las transiciones de estado son funciones puras `(state, event) → state`.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Bajo-Medio (tipos + lógica de máquina de estados funcional)
**Bloqueado por:** TT-1 (RegimeMetrics)
**Desbloquea:** TT-7 (calibration bridge), TT-9 (pipeline)

---

## Entregable

### `src/batch/telescope/mod.rs`

```rust
/// Fase actual del telescopio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelescopePhase {
    Projecting,    // Telescopio tiene estado especulativo, ancla en background
    Reconciling,   // Ancla alcanzó, comparando
    Correcting,    // Aplicando cascada de correcciones
    Idle,          // Sin proyección activa (K=0 o deshabilitado)
}

/// Estado completo del telescopio. Inmutable — transiciones producen nuevo estado.
#[derive(Clone, Debug)]
pub struct TelescopeState {
    pub phase: TelescopePhase,
    pub current_k: u32,                              // K actual (ticks a proyectar)
    pub fork_tick: u64,                               // tick donde se hizo fork
    pub consecutive_perfect: u16,                     // reconciliaciones perfectas seguidas
    pub consecutive_systemic: u16,                    // reconciliaciones sistémicas seguidas
    pub total_reconciliations: u32,                   // contador total
    pub last_metrics: RegimeMetrics,                  // métricas al momento del fork
    pub weights: NormalizerWeights,                   // pesos actuales de normalización
}

/// Registro de una reconciliación (dato de entrenamiento para el puente).
#[derive(Clone, Copy, Debug)]
pub struct ReconciliationRecord {
    pub tick: u64,
    pub k_used: u32,
    pub metrics_at_fork: RegimeMetrics,
    pub diff_class: DiffClass,
    pub mean_qe_error: f32,
    pub affected_fraction: f32,     // affected_count / total_alive
}

/// Config inmutable del telescopio.
#[derive(Clone, Copy, Debug)]
pub struct TelescopeConfig {
    pub k_min: u32,              // floor: 4
    pub k_max: u32,              // ceiling: 1024
    pub k_initial: u32,          // arranque: 16
    pub grow_factor: f32,        // multiplicador tras PERFECT: 1.5
    pub shrink_factor: f32,      // divisor tras SYSTEMIC: 0.5
    pub perfect_streak_to_grow: u16,   // PERFECTs consecutivos para crecer K: 4
}

/// Transición de estado post-reconciliación. Pura: (state, record) → state.
pub fn telescope_after_reconciliation(
    state: &TelescopeState,
    record: &ReconciliationRecord,
    config: &TelescopeConfig,
) -> TelescopeState

/// Calcula K para el próximo fork. Pura: (state, config) → u32.
pub fn next_k(state: &TelescopeState, config: &TelescopeConfig) -> u32
```

---

## Contrato stateless

`TelescopeState` es inmutable semánticamente — las "mutaciones" son funciones `(old_state, event) → new_state`. Esto permite:
- Testear transiciones sin runtime
- Serializar/deserializar estados (checkpoint)
- Comparar estados (property tests)

---

## Preguntas para tests

1. `next_k` tras 4 PERFECT consecutivos → ¿K × 1.5?
2. `next_k` tras 1 SYSTEMIC → ¿K / 2?
3. `next_k` nunca baja de `k_min` (4)
4. `next_k` nunca sube de `k_max` (1024)
5. `telescope_after_reconciliation` con PERFECT → ¿consecutive_perfect incrementa?
6. `telescope_after_reconciliation` con LOCAL → ¿consecutive_perfect resetea a 0?
7. `telescope_after_reconciliation` con SYSTEMIC → ¿consecutive_systemic incrementa?
8. `total_reconciliations` incrementa en cada reconciliación (nunca decrementa)
9. `TelescopeState` con phase=Idle + cualquier reconciliation → ¿sigue Idle?
10. `TelescopeConfig::default()` tiene k_min=4, k_max=1024, k_initial=16

---

## Integración

- **Consume:** `RegimeMetrics`, `NormalizerWeights` (TT-1/TT-3), `DiffClass` (TT-4)
- **Consumido por:** TT-7 (calibration bridge actualiza weights), TT-9 (pipeline usa state)
- **No modifica:** Nada existente
