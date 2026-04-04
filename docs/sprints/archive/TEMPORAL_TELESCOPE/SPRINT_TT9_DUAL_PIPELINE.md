# TT-9: Pipeline Dual-Timeline

**Objetivo:** Integrar Ancla + Puente + Telescopio en el pipeline batch. Fork → proyección → simulación background → reconciliación → cascada → feedback. Modo síncrono (test) y paralelo (producción).

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Alto (orquestación, threading, channels)
**Bloqueado por:** TT-6 (projection), TT-7 (calibration), TT-8 (cascade)
**Desbloquea:** TT-10 (activation, dashboard)

---

## Entregable

### Modificación en `src/batch/pipeline.rs`

```rust
/// Ejecuta un tramo dual-timeline: Telescopio proyecta, Ancla simula, reconcilia.
/// Modo síncrono (para tests): secuencial en un thread.
/// Modo paralelo (producción): Ancla en background thread.
pub fn tick_telescope(
    world: &mut SimWorldFlat,
    state: &mut TelescopeState,
    config: &TelescopeConfig,
    cal_config: &CalibrationConfig,
    history: &mut [ReconciliationRecord],  // ring buffer externo
    history_len: &mut usize,
    scratch: &mut ScratchPad,
    parallel: bool,
) -> TelescopeTickResult

/// Resultado de un tramo del telescopio.
#[derive(Clone, Debug)]
pub struct TelescopeTickResult {
    pub k_used: u32,
    pub phase: TelescopePhase,
    pub diff_class: Option<DiffClass>,    // None si todavía projecting
    pub cascade_report: Option<CascadeReport>,
    pub new_weights: NormalizerWeights,
}
```

---

## Flujo del pipeline

```
tick_telescope(world, state, ...):

  1. FORK:
     anchor = world.clone()            // memcpy ~100KB
     metrics = compute_regime_metrics(world, state)

  2. PROJECT (instantáneo, O(1)):
     k = optimal_k(metrics, state.weights, config)
     telescope = project_world(world, metrics, state.weights, k)

  3. SIMULATE (Ancla):
     Modo síncrono:
       for _ in 0..k { tick(&mut anchor, scratch); }
     Modo paralelo:
       spawn thread → for _ in 0..k { tick(&mut anchor, scratch); }
       retorna handle

  4. RECONCILE (cuando Ancla termina):
     diff = world_diff(&anchor, &telescope, threshold)
     record = ReconciliationRecord { tick, k, metrics, diff.class, ... }

  5. CASCADE:
     cascade(&mut telescope, &anchor, &diff, max_hops, attenuation, epsilon)

  6. CALIBRATE:
     new_weights = calibrate(&record, &state.weights, history, cal_config)

  7. COMMIT:
     *world = anchor               // la verdad siempre gana
     state = telescope_after_reconciliation(state, &record, config)
     state.weights = new_weights
     push record to history

  8. RETURN TelescopeTickResult
```

---

## Threading (modo paralelo)

```
Main thread:
  1. Fork + Project (instantáneo)
  2. world = telescope (usuario ve esto)
  3. Spawn Anchor thread

Anchor thread:
  1. Corre k ticks de tick()/tick_fast()
  2. Envía anchor_result por channel

Main thread (cuando recibe):
  1. Diff + Cascade + Calibrate
  2. world = anchor (corrección)
  3. Listo para siguiente fork
```

No usa `Arc<Mutex>`. El Ancla tiene su propia copia de `SimWorldFlat` (owned, move al thread). Comunicación por `std::sync::mpsc::channel` (o equivalente single-producer single-consumer). Sin locks.

---

## Preguntas para tests

1. `tick_telescope` síncrono con K=1 → ¿telescope ≈ anchor? (1 tick de diferencia mínima)
2. `tick_telescope` síncrono con K=64, mundo estable → ¿DiffClass::Perfect?
3. `tick_telescope` síncrono con K=64, mundo caótico → ¿DiffClass::Systemic?
4. Después de tick_telescope, `world` contiene el estado del ANCHOR (no del telescope)
5. `state.weights` se actualizó via calibrate
6. `state.current_k` se ajustó según diff_class
7. `history` contiene el nuevo ReconciliationRecord
8. Modo síncrono y paralelo producen el mismo resultado final (determinismo)
9. Modo paralelo: main thread no bloquea mientras Anchor corre
10. Conservation: world.total_qe antes ≈ world.total_qe después (± disipación de K ticks)

---

## Integración

- **Consume:** TT-3 (normalizers), TT-4 (diff), TT-5 (state), TT-6 (projection), TT-7 (calibration), TT-8 (cascade)
- **Modifica:** `batch/pipeline.rs` (agrega `tick_telescope`)
- **No modifica:** `batch/systems/*.rs` (33 sistemas intactos — Ancla los corre tal cual)
- **Patrón:** Mismo que `tick_fast()` — función que envuelve el tick loop, no reemplaza
