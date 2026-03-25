# Sprint R8 — Surrogate Reliability

## Objetivo

Validar que el surrogate/cache evolutivo sea fiable frente al cómputo exacto bajo presupuesto limitado.

## Rol dentro de SRP

Permite escalar simulación de escenarios sin sacrificar confiabilidad de decisiones.

## Entregables

- Medición formal de error surrogate/exacto.
- Epsilon por métrica documentado.
- Política de refine bajo incertidumbre alta.
- Verificación de convergencia top-K.

## Tareas

- Medir error surrogate vs exacto en muestra control.
- Definir epsilon máximo por métrica.
- Implementar refine adaptativo.
- Verificar convergencia estable de top-K.

## DoD

- Error surrogate dentro de epsilon pactado.
- Convergencia top-K estable en fixtures definidos.
- Métricas de hit/miss del cache disponibles para análisis.

## Referencias

- `src/simulation/evolution_surrogate.rs`
- `src/bridge/`
- `docs/sprints/LIVING_ORGAN_INFERENCE/README.md` (LI9 cerrado en `evolution_surrogate.rs`)

