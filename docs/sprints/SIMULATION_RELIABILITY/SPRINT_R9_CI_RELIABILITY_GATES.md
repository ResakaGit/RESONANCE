# Sprint R9 — CI Reliability Gates

## Objetivo

Enforzar confiabilidad con gates automáticos de merge.

## Rol dentro de SRP

Cierra el programa: convierte buenas prácticas en política técnica obligatoria.

## Entregables

- Pipeline CI con gates de confiabilidad críticos.
- Umbrales de corte versionados.
- Reporte por PR consumible por reviewers.

## Tareas

- Integrar en CI:
  - determinismo,
  - conservación,
  - benchmarks largos,
  - regresión de calibración,
  - error surrogate.
- Definir umbrales de corte por gate.
- Bloquear merge en fallo crítico.

## DoD

- CI falla ante violación crítica de confiabilidad.
- Reporte por PR visible y legible.
- Cambios de umbral requieren revisión explícita.

## Referencias

- `.github/workflows/` o pipeline equivalente
- `docs/sprints/SIMULATION_RELIABILITY/README.md`

