# Sprint R1 — Units and Conservation

## Objetivo

Establecer un contrato único de unidades y garantizar conservación energética en cada tick.

## Rol dentro de SRP

Este sprint crea la base física mínima. Sin R1, determinismo y calibración son difíciles de interpretar porque faltan magnitudes estables.

## Entregables

- Tabla de unidades internas (`qe`, tiempo, temperatura, densidad, tasas).
- Invariantes globales integrados al runtime de tests.
- Reporte de conservación por escenario canónico.

## Tareas

- Consolidar unidades en constantes y documentación.
- Instrumentar chequeos globales:
  - no NaN/Inf,
  - no energía negativa salvo regla explícita,
  - balance entrada/salida/acumulado.
- Agregar tests de conservación en escenarios representativos.

## DoD

- Tests de invariantes pasan.
- Corrida de 10k ticks sin violaciones de conservación.
- Documentación de unidades publicada y referenciada en tests.

## Referencias

- `src/blueprint/constants/`
- `src/blueprint/equations.rs`
- `src/simulation/pipeline.rs`

