# Sprint R6 — Observability Tooling

## Objetivo

Reducir tiempo de diagnóstico de incidentes sistémicos con telemetría accionable.

## Rol dentro de SRP

Convierte SRP en operativo diario: sin observabilidad, los demás sprints no escalan en mantenimiento.

## Entregables

- Dashboard de salud de simulación.
- Export estandarizado de corridas.
- Alertas de runtime por umbrales críticos.

## Tareas

- Dashboard con:
  - conservación,
  - drift,
  - saturación,
  - costo por sistema.
- Export a CSV/JSON.
- Alertas en runtime para umbrales críticos.

## DoD

- Diagnóstico de incidente crítico en menos de 5 minutos.
- Export reproducible de corridas de benchmark.
- Dashboard usable en revisión de PRs de sistemas críticos.

## Referencias

- `src/runtime_platform/debug_observability.rs`
- `src/simulation/pipeline.rs`

