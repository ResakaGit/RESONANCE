# Sprints — Geometry Flow (motor stateless flora / flujo)

Backlog del repo: [`../README.md`](../README.md) (aquí solo **GF2** pendiente).

## Origen

- Adaptación de la blueprint informal [`TERRAIN_MESHER.md`](../../design/TERRAIN_MESHER.md) a la filosofía Resonance (DoD, hex boundary, sin simulación en la capa de forma).
- Alineado con [`PLANT_SIMULATION.md`](../../../PLANT_SIMULATION.md): la **planta** vive en ECS + campo V7; este track solo **da geometría** a partir de un paquete inyectado.

## Principio filosófico

> **Solo forma.** El núcleo es **stateless**: mismas entradas → misma malla; sin historial interno ni `Query` dentro del módulo `geometry_flow`.

Invariantes:

- **Hex boundary:** quien arma `GeometryInfluence` es la última capa antes del motor (demo, bridge extendido, o sistema de snapshot); el motor **no** lee `BaseEnergy` ni `EnergyFieldGrid` directamente.
- **Texturas stateless:** UV y color por vértice = funciones puras del paquete + posición paramétrica a lo largo del eje; **sin** `Handle<Image>` en el núcleo.
- **LOD explícito:** `detail ∈ [0,1]` acota segmentos del spine y vértices por anillo (más cerca de 1 → más polígonos).

## Supersedencia

La blueprint informal **grid → altura → mesh** queda **supersedida para flora / flow** por este sprint. El trozo terreno/campo escalar puede fusionarse después con topología (T*) si se desea un único doc de “heightmap meshing”.

## Grafo de dependencias

```text
ECS + V7 (sim)  →  inyector (arma GeometryInfluence)  →  geometry_flow (spine + mesh)  →  Bevy Mesh / materiales (capa render)
```

## Índice de sprints

| Sprint | Archivo | Estado |
|--------|---------|--------|
| GF1 | — | **Cerrado** en `src/geometry_flow/`; el doc de sprint se eliminó. Contrato: `blueprint_geometry_flow.md`, `GEOMETRY_DEFORMATION_ENGINE.md` (contexto). |
| GF2 | [SPRINT_GF2_GEOMETRY_DEFORMATION_ENGINE.md](SPRINT_GF2_GEOMETRY_DEFORMATION_ENGINE.md) | ⏳ Pendiente — deformación post-branching |

**Demo jugable:** `RESONANCE_MAP=flower_demo cargo run` — tallo + pétalos + sépalos (`geometry_flow`) + pistilo; ver `docs/DEMO_FLOW.md`.

## Arquitectura de referencia

- Contrato de módulo (glosario): [`docs/arquitectura/blueprint_geometry_flow.md`](../../arquitectura/blueprint_geometry_flow.md)
- Ecuaciones de decisión recto vs curva: `crate::blueprint::equations` (bloque flow spine)

## Riesgos / trade-offs

- **“Crecimiento real”:** en GF1 es **morfología procedural + LOD**, no L-system completo ni botánica detallada salvo que un sprint posterior lo acote.
- **Costo CPU:** `detail → 1` sube triángulos rápido; GF1 fija techos (`max_segments`, anillos máximos) y tests de monotonía.
