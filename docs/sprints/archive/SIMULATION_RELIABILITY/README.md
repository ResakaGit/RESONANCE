# Track — Simulation Reliability Program (SRP)

Índice maestro del repo: [`../README.md`](../README.md).

**Objetivo:** elevar la confiabilidad del simulador a nivel producción para mundos de juego, con métricas objetivas y gates automáticos.

## Rol del track

SRP no agrega features jugables directamente. Su función es asegurar que toda feature existente o futura se ejecute sobre una base:

- determinista,
- conservativa,
- medible en corridas largas,
- y auditable en CI.

Sin SRP, el crecimiento del motor aumenta deuda de estabilidad.

## Alcance

- Tiempo de simulación (ticks largos, drift, estabilidad).
- Integridad física interna (energía/materia).
- Calidad de aproximaciones (surrogate/cache).
- Observabilidad para diagnóstico rápido.

Fuera de alcance: balance de héroes, UX, contenido narrativo.

## Índice de sprints

| Sprint | Archivo | Foco | Implementación | Estado |
|---|---|---|---|---|
| R1 | [SPRINT_R1_UNITS_AND_CONSERVATION.md](SPRINT_R1_UNITS_AND_CONSERVATION.md) | Contrato de unidades + conservación | `tests/r1_conservation.rs` | ✅ |
| R2 | [SPRINT_R2_DETERMINISM_AND_REPLAY.md](SPRINT_R2_DETERMINISM_AND_REPLAY.md) | Reproducibilidad y replay verificable | `tests/r2_determinism.rs` | ✅ |
| R3 | [SPRINT_R3_MULTI_SCALE_BENCHMARKS.md](SPRINT_R3_MULTI_SCALE_BENCHMARKS.md) | Estabilidad micro/meso/macro | `benches/simulation_scale_bench.rs`, `benches/energy_competition_bench.rs`, `benches/morphogenesis_benchmark.rs` | ✅ |
| R4 | [SPRINT_R4_EMPIRICAL_CALIBRATION.md](SPRINT_R4_EMPIRICAL_CALIBRATION.md) | Calibración con referencias externas | `tests/r4_calibration.rs`, `src/blueprint/equations/calibration.rs` | ✅ |
| R5 | [SPRINT_R5_SENSITIVITY_AND_UNCERTAINTY.md](SPRINT_R5_SENSITIVITY_AND_UNCERTAINTY.md) | Sensibilidad e incertidumbre | `tests/r5_sensitivity.rs`, `src/blueprint/equations/sensitivity.rs` | ✅ |
| R6 | [SPRINT_R6_OBSERVABILITY_TOOLING.md](SPRINT_R6_OBSERVABILITY_TOOLING.md) | Telemetría y diagnóstico | `tests/r6_observability.rs`, `src/blueprint/equations/observability.rs` | ✅ |
| R7 | [SPRINT_R7_MORPH_INFERENCE_ROBUSTNESS.md](SPRINT_R7_MORPH_INFERENCE_ROBUSTNESS.md) | Robustez de inferencia morfológica | `tests/r7_morph_robustness.rs`, `src/blueprint/equations/morph_robustness.rs` | ✅ |
| R8 | [SPRINT_R8_SURROGATE_RELIABILITY.md](SPRINT_R8_SURROGATE_RELIABILITY.md) | Confiabilidad surrogate/exacto | `tests/r8_surrogate.rs`, `src/blueprint/equations/surrogate_error.rs` | ✅ |
| R9 | [SPRINT_R9_CI_RELIABILITY_GATES.md](SPRINT_R9_CI_RELIABILITY_GATES.md) | Gates de confiabilidad en CI | `tests/r9_ci_gates.rs`, `docs/ci/RELIABILITY_GATES.md` | ✅ |

## Orden recomendado

`R1 -> R2 -> R3 -> R4 -> R5 -> R6 -> R7 -> R8 -> R9`

## KPI globales de salida

- Determinismo: misma seed -> mismo hash final.
- Conservación: error acumulado dentro del umbral definido.
- Drift largo: estable en escenarios macro.
- Error surrogate: bajo epsilon pactado vs cómputo exacto.
- Diagnóstico: causa raíz identificable en minutos.

## Referencias

- `docs/sprints/LIVING_ORGAN_INFERENCE/README.md` (track LI cerrado; ver `env_scenario`, `evolution_surrogate` en `src/`)
- `src/simulation/pipeline.rs`
- `src/simulation/evolution_surrogate.rs`

