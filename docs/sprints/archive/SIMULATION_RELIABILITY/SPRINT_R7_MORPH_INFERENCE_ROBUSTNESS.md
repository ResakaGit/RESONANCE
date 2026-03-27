# Sprint R7 — Morph Inference Robustness

## Objetivo

Garantizar que toda inferencia morfológica esté respaldada por viabilidad energética y reglas estables de transición.

## Rol dentro de SRP

Es el puente entre confiabilidad física y legibilidad visual: evita morfologías “gratis” o erráticas.

## Entregables

- Suite de pruebas de quiebre por entorno.
- Histéresis aplicada en cambios de órgano.
- Auditoría de costos energéticos por órgano.

## Tareas

- Tests de quiebre:
  - abundante,
  - hostil,
  - extremo.
- Aplicar histéresis para evitar flicker.
- Verificar costo energético por órgano inferido.
- Auditar no aparición de órganos sin costo.

## DoD

- 0 órganos sin costo en escenarios auditados.
- Transiciones estables sin oscilación espuria.
- Regresión visual funcional estable en `organ_inference`.

## Referencias

- `src/worldgen/organ_inference.rs`
- `src/layers/organ.rs`
- `src/blueprint/equations.rs`

