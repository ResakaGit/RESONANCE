# Reliability Gates — Resonance SRP

Gates automáticos ejecutados por `cargo test` en cada PR.

## Gates críticos (bloquean merge)

| Gate | Test | Umbral | Sprint |
|------|------|--------|--------|
| Conservación energética | `r1_conservation` | error < CONSERVATION_ERROR_TOLERANCE (1e-3) | R1 |
| Determinismo | `r2_determinism` | hash idéntico con misma seed | R2 |
| Calibración golden | `r4_calibration` | todos los rangos nominales plausibles | R4 |
| Error surrogate | `r8_surrogate` | error < SURROGATE_FITNESS_EPSILON (5%) | R8 |
| Configuración gates | `r9_ci_gates` | todos los umbrales en rango válido | R9 |

## Ejecución local

```bash
# Todos los gates de confiabilidad:
cargo test --test r1_conservation --test r2_determinism --test r4_calibration --test r8_surrogate --test r9_ci_gates

# Gate rápido (solo umbrales):
cargo test --test r9_ci_gates
```

## Modificar umbrales

Los umbrales viven en:
- `src/blueprint/constants/units.rs` — `CONSERVATION_ERROR_TOLERANCE`
- `src/blueprint/constants/energy_competition_ec.rs` — `POOL_CONSERVATION_EPSILON`
- `src/blueprint/constants/surrogate.rs` — `SURROGATE_*` thresholds

Cambiar un umbral requiere:
1. Justificación en el PR
2. Re-ejecución de todos los gates afectados
3. Revisión explícita del cambio en review

## Qué verifica cada gate

### r1_conservation
Tres invariantes sobre el pool de energía EC:
- `qe` nunca es NaN o Inf tras 1000 ticks
- `conservation_error < CONSERVATION_ERROR_TOLERANCE` en cada tick con ledger
- `qe >= 0` bajo extracción agresiva (100 ticks)

### r2_determinism
Misma configuración inicial → mismo snapshot bit a bit:
- Dos corridas de 200 ticks → hashes iguales
- Tres corridas de 1000 ticks → hashes iguales
- Control negativo: distinto entity count → hashes distintos

### r4_calibration
Golden values para rangos nominales del sistema:
- Todos los parámetros de calibración dentro de rangos plausibles

### r8_surrogate
Precisión del surrogate model contra el exacto:
- Relative error `<= SURROGATE_FITNESS_EPSILON` (5%)
- Cache hit rate `>= SURROGATE_MIN_HIT_RATE` (70%)
- Top-K convergence con `SURROGATE_TOP_K_EPSILON` (1%)

### r9_ci_gates
Validación de la configuración de los propios gates:
- `CONSERVATION_ERROR_TOLERANCE` en [1e-6, 0.1]
- `POOL_CONSERVATION_EPSILON == CONSERVATION_ERROR_TOLERANCE`
- `SURROGATE_FITNESS_EPSILON` en [0.001, 0.5]
- `SURROGATE_MIN_HIT_RATE` en [0.5, 1.0]
- Comportamiento correcto de las funciones gate en boundary y overshoot
