# Sprint Blueprint V7 — backlog

Alineado a `docs/design/V7.md`. **Pendiente operativo:**

| Sprint | Archivo | Qué falta |
|--------|---------|-----------|
| **06** | `SPRINT_06_V7_MATERIALIZATION_SYSTEM.md` | Materialización incremental dedicada (delta/cola) desacoplada del wiring provisional en `simulation_plugin`. |
| **07** | `SPRINT_07_V7_WORLDGEN_PLUGIN.md` | `plugins/worldgen_plugin` — registrar worldgen sin inflar `SimulationPlugin`. |
| **14** | `SPRINT_14_V7_QUANTIZED_COLOR_ENGINE.md` | Paletas GPU + cuantización + WGSL (`docs/design/QUANTIZED_COLOR_ENGINE.md`). |

**Cerrados (01–05, 08–13):** implementados en `src/worldgen/` y subsistemas asociados. Los `SPRINT_*` de esos números se **eliminaron** del repo al cerrar el track; el contrato vivo está en `docs/design/V7.md`, `blueprint_v7.md` y el código.

## Dependencias rápidas (solo pendientes)

- **06** antes o en paralelo con refactors de cola de materialización (coherente con **07**).
- **14** depende de derivación visual estable (**05** hecho) + LOD (**13** hecho).

## Referencias

- `docs/design/V7.md`
- `docs/arquitectura/blueprint_v7.md`
