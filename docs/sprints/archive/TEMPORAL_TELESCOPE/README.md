# Track: TEMPORAL_TELESCOPE — Ejecución Especulativa Dual-Timeline

**Objetivo:** Implementar un sistema de doble línea temporal (Ancla + Telescopio) que proyecta el futuro instantáneamente mientras la simulación real corre en background. El Ancla produce la verdad; el Telescopio la anticipa. Un puente de calibración convierte los resultados del Ancla en pesos para los normalizadores del Telescopio.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Bloqueado por:** Nada (usa batch/, equations/, exact_cache existentes, no los modifica)
**Desbloquea:** Simulaciones de escala geológica, fast-forward interactivo, análisis de régimen dinámico
**ADR:** `docs/arquitectura/ADR/ADR-015-temporal-telescope.md`

---

## Arquitectura en una frase

```
Telescopio (proyecta) ←── Puente de Calibración ←── Ancla (simula)
     │                          │                         │
     │ Instantáneo              │ Stateless converter     │ Tick-a-tick
     │ Normalizadores × K       │ DiffReport → Pesos      │ Ground truth
     │ O(1) analítico           │ O(entidades_afectadas)  │ O(N×ticks)
```

**Tres componentes, tres responsabilidades:**
- **Ancla:** Corre la simulación completa. No sabe que el Telescopio existe. Produce `SimWorldFlat` + métricas.
- **Puente de Calibración:** Compara Ancla vs Telescopio. Convierte diffs en calibraciones. Stateless: `(DiffReport, Weights) → Weights`.
- **Telescopio:** Proyecta el futuro con solvers analíticos + pesos del puente. No modifica al Ancla.

---

## Principio de diseño: FULL STATELESS entre componentes

```
Cada sprint produce funciones puras:
  fn(inputs) → outputs

Composición:
  Ancla produce → DiffReport (datos puros)
  Puente recibe → (DiffReport, NormalizerWeights) → NormalizerWeights
  Telescopio recibe → (SimWorldFlat, NormalizerWeights, K) → SimWorldFlat

Ningún componente guarda referencia al otro.
Ningún componente muta estado del otro.
Los datos fluyen en una sola dirección: Ancla → Puente → Telescopio.
```

---

## Casos de Uso

### CU-1: Fast-forward interactivo
El usuario pide avanzar 1000 ticks. El Telescopio proyecta instantáneamente. El usuario ve el resultado. El Ancla corre en background. Cuando alcanza, reconcilia. Si hay diferencia, las entidades afectadas se corrigen con un breve highlight visual.

### CU-2: Análisis de régimen dinámico
Un investigador quiere saber si el ecosistema está en estasis, pre-transición, o transición. Los normalizadores (Hurst, Fisher, ρ₁, λ_max) se exponen como métricas en el dashboard — son outputs científicos independientes del time-skip.

### CU-3: Simulación de escala geológica
Correr 1,000,000 de ticks para ver evolución a largo plazo. El Telescopio proyecta agresivamente (K=1024) durante estasis. El Ancla confirma. Durante transiciones, K baja automáticamente. Resultado: horas de simulación en minutos.

### CU-4: Calibración de fármacos
Experimentos de resistencia tumoral (cancer_therapy, pathway_inhibitor) necesitan miles de generaciones. El Telescopio acelera las fases de tratamiento estable. El Ancla garantiza que los eventos de resistencia emergente no se pierdan.

### CU-5: Validación de proyección
Un desarrollador quiere verificar que el Telescopio proyecta bien. Corre ambas timelines en modo síncrono (sin paralelismo). Compara tick a tick. El puente genera estadísticas de precisión por régimen.

---

## Archivos que se CREAN (todo nuevo)

```
src/blueprint/equations/temporal_telescope.rs    (TT-1, TT-2, TT-3: math pura)
src/blueprint/constants/temporal_telescope.rs    (TT-1: umbrales derivados)
src/batch/telescope/mod.rs                       (TT-5: TelescopeState, tipos)
src/batch/telescope/diff.rs                      (TT-4: diff engine)
src/batch/telescope/projection.rs                (TT-6: projection engine)
src/batch/telescope/calibration_bridge.rs        (TT-7: puente de calibración)
src/batch/telescope/cascade.rs                   (TT-8: cascade propagator)
```

## Archivos que se MODIFICAN (mínimo)

```
src/batch/mod.rs                                 (1 línea: pub mod telescope;)
src/batch/pipeline.rs                            (TT-9: fork + reconciliation point)
src/batch/arena.rs                               (TT-4: impl diff en SimWorldFlat)
src/runtime_platform/dashboard_bridge.rs         (TT-10: exponer métricas)
src/simulation/emergence/geological_lod.rs       (TT-10: wire to telescope K)
src/simulation/emergence/multiscale.rs           (TT-10: feed into normalizers)
```

## Archivos que NO se modifican

```
src/blueprint/equations/macro_analytics.rs       (Telescopio lo usa, no lo cambia)
src/blueprint/equations/exact_cache.rs           (Telescopio lo usa, no lo cambia)
src/blueprint/equations/batch_stepping.rs        (predict_death_ticks(), is_isolated() — usa)
src/batch/systems/*.rs                           (33 sistemas — Ancla los corre tal cual)
src/layers/*.rs                                  (sin componentes nuevos)
src/blueprint/equations/derived_thresholds.rs    (constantes fundamentales intactas)
src/sim_world.rs                                 (INV-7 intacto — Ancla es simulación completa)
```

---

## 10 Sprints

| Sprint | Título | Entregable | Dependencias |
|--------|--------|------------|--------------|
| [TT-1](SPRINT_TT1_SLIDING_STATISTICS.md) | Estadísticas de ventana deslizante | `sliding_variance`, `sliding_autocorrelation_lag1`, `shannon_entropy`, `fisher_information` + constantes | — |
| [TT-2](SPRINT_TT2_HURST_DFA.md) | Exponente de Hurst via DFA | `hurst_dfa(window, min_box, max_box) → f32` | — |
| [TT-3](SPRINT_TT3_PROJECTION_NORMALIZERS.md) | Normalizadores de proyección | `project_qe`, `project_population`, `event_density`, `confidence_horizon` | TT-1, TT-2 (solo tipos, no runtime) |
| [TT-4](SPRINT_TT4_DIFF_ENGINE.md) | Motor de diff | `entity_diff`, `world_diff`, `DiffReport`, `DiffClass` | — |
| [TT-5](SPRINT_TT5_TELESCOPE_STATE.md) | Estado del telescopio | `TelescopeState`, `ReconciliationRecord`, `NormalizerWeights`, adaptive K | TT-1 (tipos) |
| [TT-6](SPRINT_TT6_PROJECTION_ENGINE.md) | Motor de proyección | `project_world(world, weights, K) → SimWorldFlat` | TT-3, TT-4 |
| [TT-7](SPRINT_TT7_CALIBRATION_BRIDGE.md) | Puente de calibración | `calibrate(diff, weights, history) → NormalizerWeights` | TT-4, TT-5 |
| [TT-8](SPRINT_TT8_CASCADE_PROPAGATOR.md) | Propagador de cascada | `cascade(world, diff, spatial_radii) → CascadeReport` | TT-4 |
| [TT-9](SPRINT_TT9_DUAL_PIPELINE.md) | Pipeline dual-timeline | Fork + anchor thread + reconciliation + channel | TT-6, TT-7, TT-8 |
| [TT-10](SPRINT_TT10_ACTIVATION.md) | Activación y dashboard | Wire GeologicalLOD, MultiscaleSignalGrid, dashboard metrics | TT-9 |

---

## Grafo de dependencias

```
TT-1 (statistics) ──┐
                     ├──→ TT-3 (normalizers) ──→ TT-6 (projection) ──┐
TT-2 (Hurst DFA) ───┘                                                │
                                                                      ├──→ TT-9 (pipeline) ──→ TT-10 (activation)
TT-4 (diff engine) ─────→ TT-7 (calibration bridge) ─────────────────┤
         │                                                            │
         └───────────────→ TT-8 (cascade propagator) ─────────────────┘

TT-5 (telescope state) ──→ TT-7, TT-9

Paralelos: {TT-1, TT-2, TT-4, TT-5} pueden ejecutarse simultáneamente.
```

---

## Criterios de cierre del track

- [x] 9 archivos nuevos en `src/blueprint/equations/`, `src/blueprint/constants/`, `src/batch/telescope/`
- [x] Cada archivo: funciones puras + `#[cfg(test)] mod tests` con ≥5 tests (179 tests totales)
- [x] `cargo test --lib` verde (3298 tests, 0 warnings, 0 failures)
- [x] 0 `unsafe`, 0 `async`, 0 `Arc<Mutex>`, 0 `unwrap()` en código nuevo
- [x] Todas las funciones nuevas son stateless: `fn(inputs) → outputs`
- [x] Flujo de datos unidireccional: Ancla → Puente → Telescopio (nunca al revés)
- [x] Constantes derivadas de los 4 fundamentals donde sea posible (calibración honesta donde no)
- [x] Dual pipeline corre en modo síncrono (test); paralelo preparado (TT-9 pipeline.rs)
- [x] GeologicalLOD y MultiscaleSignalGrid wiring via funciones puras (activation.rs)
- [x] TelescopeSummary expone: régimen, K, precisión, frecuencia de corrección, H, ρ₁, F, λ_max
- [x] Axiomas 4 (disipación), 5 (conservación), 7 (atenuación) verificados con property tests
- [x] 0 hardcoded values — todos extraídos a constantes en `blueprint/constants/temporal_telescope.rs`
- [x] Código duplicado centralizado (`neighbors_within_radius` en `batch_stepping.rs`)
