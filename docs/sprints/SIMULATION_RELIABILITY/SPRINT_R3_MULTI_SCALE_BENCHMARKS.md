# Sprint R3 — Multi-scale Benchmarks

## Objetivo

Medir estabilidad y costo computacional del simulador en escalas micro, meso y macro.

## Rol dentro de SRP

Transforma percepciones en evidencia. Define hasta dónde escala el motor antes de degradar fiabilidad o performance.

## Entregables

- Suite de benchmarks por escala.
- Corridas largas con recolección de drift e invariantes.
- Tabla comparativa de rendimiento por escenario.

## Tareas

- Definir fixtures:
  - micro (1-10 entidades),
  - meso (100),
  - macro (1000+).
- Ejecutar corridas largas (100k+ ticks).
- Registrar:
  - drift,
  - consumo CPU,
  - violaciones de invariantes.

## DoD

- Tabla comparativa por escala disponible.
- Drift dentro de umbrales definidos por escenario.
- Métricas reproducibles entre corridas con misma seed.

## Referencias

- `src/simulation/pipeline.rs`
- `src/worldgen/`
- `benches/`

