# TT-6: Motor de Proyección

**Objetivo:** Función pura que toma un `SimWorldFlat` + `NormalizerWeights` + K y produce un `SimWorldFlat` especulativo proyectado K ticks al futuro. Usa solvers analíticos existentes (`macro_analytics`, `exact_cache`) ponderados por los normalizadores.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio-Alto (compone múltiples solvers, debe respetar conservación)
**Bloqueado por:** TT-3 (normalizers), TT-4 (diff types)
**Desbloquea:** TT-9 (pipeline dual)

---

## Entregable

### `src/batch/telescope/projection.rs`

```rust
/// Proyecta un mundo K ticks al futuro usando solvers analíticos + normalizadores.
/// NO modifica `world` — retorna una copia proyectada.
/// Stateless: misma entrada → misma salida.
pub fn project_world(
    world: &SimWorldFlat,
    metrics: &RegimeMetrics,
    weights: &NormalizerWeights,
    k: u32,
) -> SimWorldFlat

/// Proyecta una entidad individual K ticks al futuro.
/// Usa: exponential_decay (qe), allometric_radius (growth), predict_death_ticks (vida).
/// Pondera extrapolación por Hurst weight.
pub fn project_entity(
    entity: &EntitySlot,
    metrics: &RegimeMetrics,
    weights: &NormalizerWeights,
    k: u32,
    dt: f32,
) -> EntitySlot

/// Proyecta el grid de nutrientes K ticks (decaimiento simple + regeneración geológica).
pub fn project_nutrient_grid(
    grid: &[f32],
    k: u32,
    dt: f32,
) -> [f32; GRID_CELLS]

/// Proyecta el grid de irradiancia K ticks (seasonal modulation).
pub fn project_irradiance_grid(
    grid: &[f32],
    tick_id: u64,
    k: u32,
) -> [f32; GRID_CELLS]
```

---

## Contrato stateless

`project_world` clona el mundo (memcpy ~100KB), luego modifica la copia. El mundo original no se toca. No usa `unsafe`. No tiene side effects.

Usa funciones existentes (no las reimplementa):
- `macro_analytics::exponential_decay()` — decaimiento de qe
- `macro_analytics::allometric_radius()` — crecimiento de radio
- `batch_stepping::predict_death_ticks()` — timing de muerte
- `temporal_telescope::project_qe()` — extrapolación Hurst-ponderada

---

## Preguntas para tests

1. `project_world` con K=0 → ¿retorna copia idéntica al input?
2. `project_entity` de entidad aislada con K=100 → ¿qe decae por exponential_decay exacto?
3. `project_entity` que debería morir en tick 50 (via exact_death_tick) con K=100 → ¿alive=false?
4. `project_entity` con growth_bias > 0 y K=50 → ¿radius creció según allometric_radius?
5. `project_world` conserva total_qe ≤ input (Axioma 5: nunca crea energía en la proyección)
6. `project_nutrient_grid` con K=0 → ¿grid sin cambio?
7. `project_irradiance_grid` con K=periodo_estacional → ¿grid ≈ original? (ciclo completo)
8. `project_world` con H=1.0 y trend positivo → ¿total_qe proyectado > actual?
9. `project_world` resultado es determinista (misma entrada → mismo output bit-exacto)
10. Performance: project_world(128 entities) < 100μs

---

## Integración

- **Consume:** `macro_analytics.rs` (solvers), `exact_cache.rs` (caches), TT-3 (normalizers)
- **Consumido por:** TT-9 (pipeline — el telescopio llama a project_world)
- **No modifica:** `macro_analytics.rs`, `exact_cache.rs`, `batch_stepping.rs`
