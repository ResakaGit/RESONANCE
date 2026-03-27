# Sprint R2 — Determinism and Replay

## Objetivo

Garantizar reproducibilidad completa por seed y soporte de replay verificable.

## Rol dentro de SRP

Es la capa de confianza causal: permite depurar, comparar ramas y auditar regresiones sin ambigüedad.

## Entregables

- Seed global controlada para corridas de simulación.
- Hash de estado por tick (subset estable).
- Modo replay con validación de hash final.

## Tareas

- Alinear fuentes pseudoaleatorias bajo seed única.
- Definir subset de estado hashable y estable.
- Implementar replay y verificación automática.

## DoD

- 3 corridas idénticas producen mismo hash final.
- Replay reproduce secuencia sin divergencias.
- Diferencias entre ramas se detectan por hash en pipeline de pruebas.

## Referencias

- `src/simulation/pipeline.rs`
- `src/simulation/time_compat.rs`
- `src/worldgen/`

