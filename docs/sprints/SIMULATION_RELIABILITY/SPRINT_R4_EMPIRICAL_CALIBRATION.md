# Sprint R4 — Empirical Calibration

## Objetivo

Calibrar parámetros críticos con referencias externas para mejorar plausibilidad y estabilidad de largo plazo.

## Rol dentro de SRP

Conecta el modelo interno con una referencia observable. Reduce deriva de tuning arbitrario.

## Entregables

- Dataset de referencia versionado.
- Mapeo explícito de unidades externas a unidades internas.
- Parámetros calibrados + snapshots golden para regresión.

## Tareas

- Definir dataset de referencia (rangos plausibles por variable).
- Mapear unidades externas al dominio del motor.
- Ajustar parámetros críticos:
  - intake,
  - maintenance,
  - growth,
  - decay.
- Guardar snapshots golden para regresión.

## DoD

- Error medio por métrica clave por debajo del objetivo pactado.
- Golden tests de calibración pasan.
- Cambios de parámetros quedan trazados con justificación.

## Referencias

- `src/blueprint/equations.rs`
- `src/blueprint/constants/`
- `docs/design/BLUEPRINT.md`

