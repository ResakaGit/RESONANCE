# Track — Simulation Foundations (SF)

**Alineacion:** Los 8 pilares fundamentales del universo + gaps criticos del scorecard (Serialization F, Observability D, Signal Latency parcial).
**Metodologia:** TDD, funciones puras en `blueprint/equations/`, sistemas de una transformacion, zero unsafe.
**Prioridad:** Bloquea experimentacion cientifica. Sin esto no podes reproducir bugs, medir emergencia, ni validar causalidad.

---

## Objetivo del track

Cerrar los 3 gaps tecnicos que impiden que Resonance sea un motor de investigacion ademas de un juego:

1. **Observabilidad externa** — Las metricas existen en memoria pero mueren con el proceso. Conectar el dashboard a CSV/JSON persistente.
2. **Serializacion / Replay** — El game state es efimero. Agregar checkpoint determinista para reproducir bugs, compartir universos, y experimentar.
3. **Latencia de senal** — La propagacion de energia es instantanea (all-at-once). Modelar frente de onda multi-tick para causalidad fisica real.

**Resultado:** Un universo donde podes observar que pasa (CSV), guardar un momento (checkpoint), retomarlo (load), y ver como la informacion viaja (latencia).

---

## Grafo de dependencias

```
SF-1 (Observability: metricas)   SF-2 (Serde: derives + formato)   SF-3 (Latencia: ecuaciones)
  |                                  |                                  |
  v                                  v                                  v
SF-4 (Export: CSV/JSON sistema)  SF-5 (Checkpoint: save/load)       SF-6 (Propagacion: multi-tick)
  |                                  |                                  |
  +----------------------------------+----------------------------------+
                                     |
                                     v
                              SF-7 (Integration: replay + verify)
```

## Ondas de ejecucion

| Onda | Sprints | Que habilita | Estado |
|------|---------|-------------|--------|
| **0** | SF-1, SF-2, SF-3 (paralelo) | Ecuaciones + derives + formato | ✅ Cerrada |
| **A** | SF-4, SF-5, SF-6 (paralelo) | Sistemas: export + checkpoint + propagacion | ⏳ |
| **B** | SF-7 | Integracion: replay determinista + verificacion end-to-end | ⏳ |

## Indice de sprints

Sprint docs eliminados para sprints cerrados.

| Sprint | Descripcion | Modulo principal | Onda | Estado |
|--------|-------------|-----------------|------|--------|
| SF-1 | Metricas expandidas | `blueprint/equations/observability.rs` (6 ecuaciones + 25 tests), `simulation/observability.rs` (SimulationMetricsSnapshot + SimulationEcologySnapshot + metrics_snapshot_system), `blueprint/constants/simulation_foundations.rs` | 0 | ✅ |
| SF-2 | Serde derives + checkpoint | 14 capas + worldgen types con Serialize/Deserialize, `blueprint/checkpoint.rs` (WorldCheckpoint + EntitySnapshot + RON/JSON roundtrip), `bevy serialize` feature | 0 | ✅ |
| SF-3 | Ecuaciones de latencia | `blueprint/equations/signal_propagation.rs` (5 funciones + 4 constantes + 24 tests) | 0 | ✅ |
| [SF-4](SPRINT_SF4_METRICS_EXPORT_SYSTEM.md) | Export CSV/JSON a disco | `simulation/observability.rs` | A | ⏳ |
| [SF-5](SPRINT_SF5_CHECKPOINT_SYSTEM.md) | Save/load checkpoint | `simulation/checkpoint.rs` | A | ⏳ |
| [SF-6](SPRINT_SF6_PROPAGATION_FRONT.md) | Propagacion multi-tick | `worldgen/systems/propagation.rs` | A | ⏳ |
| [SF-7](SPRINT_SF7_INTEGRATION_REPLAY.md) | Replay determinista + verificacion | `tests/`, demo | B | ⏳ |

---

## Paralelismo seguro

| | SF-1 | SF-2 | SF-3 | SF-4 | SF-5 | SF-6 | SF-7 |
|---|---|---|---|---|---|---|---|
| **SF-1** | — | ✅ | ✅ | | | | |
| **SF-2** | ✅ | — | ✅ | | | | |
| **SF-3** | ✅ | ✅ | — | | | | |
| **SF-4** | | | | — | ✅ | ✅ | |
| **SF-5** | | | | ✅ | — | ✅ | |
| **SF-6** | | | | ✅ | ✅ | — | |

SF-1/SF-2/SF-3 son paralelos (Onda 0): archivos distintos, sin overlap.
SF-4/SF-5/SF-6 son paralelos (Onda A): sistemas independientes.
SF-7 depende de todos (Onda B).

---

## Invariantes del track

1. **Determinismo.** Checkpoint load + N ticks = mismo estado que ejecucion original. Zero RNG.
2. **Zero overhead en release sin flag.** Export/checkpoint detras de `#[cfg(feature = "observability")]` o gated por Resource.
3. **Math in equations/.** Metricas derivadas (drift, saturation, diffusion) en `blueprint/equations/`.
4. **Backward compatible.** Mundos sin checkpoint funcionan identico. Propagacion legacy sigue siendo opcion.
5. **No new crates.** Solo `serde` + `ron` + `serde_json` (ya en Cargo.toml).
6. **Max 4 campos por componente/Resource nuevo.**
7. **Phase assignment.** Sistemas nuevos en Phase explicito.

## Contrato de pipeline SF

```
FixedUpdate:
  Phase::MetabolicLayer
    <- [existente] simulation_health_system (observability)
    <- [SF-4] metrics_batch_system (.after simulation_health_system, every 60 ticks)
  Phase::ThermodynamicLayer
    <- [SF-6] diffuse_propagation_front_system (.after propagate_nuclei_system)

PostUpdate (o Startup command):
    <- [SF-5] checkpoint_save_system (on-demand via Resource flag)
    <- [SF-5] checkpoint_load_system (Startup, if checkpoint file exists)
```

---

## Referencias cruzadas

- `docs/extraInfo/digramaFromClaudeOpus.md` — Arquitectura completa + analisis critico
- `src/simulation/observability.rs` — Dashboard existente (SF-1 lo extiende)
- `src/bridge/metrics.rs` — Bridge cache metrics (SF-4 lo integra)
- `src/worldgen/systems/propagation.rs` — Propagacion all-at-once (SF-6 lo reemplaza)
- `src/worldgen/propagation.rs` — `diffusion_transfer()` existente no wired (SF-3 lo formaliza)
