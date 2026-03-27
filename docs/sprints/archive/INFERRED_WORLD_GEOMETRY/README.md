# Track — Geometría de Mundo Inferida (Inferred World Geometry)

**Blueprint:** `docs/design/INFERRED_WORLD_GEOMETRY.md`
**Arquitectura:** `docs/arquitectura/blueprint_inferred_world_geometry.md`
**Alineación:** filosofía "todo es energía" + "math in equations/" + "one system, one transformation" + stateless-first del proyecto.
**Metodología:** TDD, funciones puras en `blueprint/equations/inferred_world_geometry/`, sistemas de una transformación, vertex color inferido (no texturas artísticas).

---

## Objetivo del track

Completar el pipeline visual del mundo: conectar las piezas que **ya existen** (terrain mesher, organ inference, visual derivation) con las que **faltan** (terrain colors, body plan bilateral, water, atmosphere dinámica). El resultado es un mundo a resolución N64 donde todo es inferido.

**Resultado jugable:** un mundo con montañas donde bond_energy es alta, valles donde hay líquido, organismos cuyas extremidades están posicionadas bilateralmente, agua que cubre las celdas líquidas, y un sol/niebla/bloom inferidos del estado global. Todo a ~6000 polígonos/escena.

---

## Principio fundamental

> La geometría no se diseña. Se proyecta. Si el campo de energía define qué existe, las ecuaciones de inferencia definen cómo se ve.

---

## Auditoría de lo existente

| Pieza | Estado | Ubicación | Qué falta |
|-------|--------|-----------|-----------|
| **Organ positioning (flora)** | ✅ Funciona end-to-end | `worldgen/inference/organ.rs` | Simetría bilateral, cache |
| **Organ primitivas** | ✅ 4 tipos | `geometry_flow/primitives.rs` | — |
| **GF1 branching** | ✅ Recursivo | `geometry_flow/branching.rs` | — |
| **Terrain heightmap data** | ✅ Completo | `topology/terrain_field.rs` | — |
| **Terrain mesh builder** | ✅ Testeado | `topology/terrain_mesher.rs` | Wiring a ECS |
| **Terrain vertex colors** | ⚠ Solo `neutral_flat()` | `topology/terrain_mesher.rs` | Cruce V7 → colors |
| **Terrain mesh en ECS** | ❌ No wired | — | Sistema que llame `generate_terrain_mesh()` |
| **Body plan bilateral** | ❌ Solo spine 1D | — | Offset lateral + simetría |
| **Water surface** | ❌ No existe | — | Detección + mesh |
| **Atmosphere** | ⚠ Luz hardcoded | `scenario_isolation/mod.rs` | Inferencia dinámica |

---

## Grafo de dependencias

```
IWG-1 (Body Plan: simetría)     IWG-3 (Terrain: cruce visual V7)
  │                                  │
  ▼                                  ▼
IWG-2 (Body Plan: cache+wiring) IWG-4 (Terrain: wiring ECS)        IWG-6 (Atmosphere: inferencia)
                                     │                               │
                                     ▼                               │
                                 IWG-5 (Water Surface)               │
                                     │                               │
                                     └──────────┬───────────────────┘
                                                ▼
                                          IWG-7 (Integration Demo)
```

## Ondas de ejecución

**Estado: TRACK CERRADO — IWG-1 a IWG-7 implementados en código (2026-03-25).**

Sprint docs eliminados. Evidencia por sprint:

| Sprint | Descripción | Módulo principal | Estado |
|--------|-------------|-----------------|--------|
| IWG-1 | Body Plan: Simetría + Escala | `blueprint/equations/inferred_world_geometry/body_plan.rs`, `blueprint/constants/inferred_world_geometry.rs` | ✅ |
| IWG-2 | Body Plan: Cache + Wiring | `layers/body_plan_layout.rs`, `worldgen/inference/organ.rs` (`build_organ_mesh_with_layout`, `assemble_body_plan`) | ✅ |
| IWG-3 | Terrain: Cruce Visual V7 | `blueprint/equations/inferred_world_geometry/terrain_visuals.rs` | ✅ |
| IWG-4 | Terrain: Wiring ECS | `worldgen/systems/terrain_visual_mesh.rs`, registrado en `WorldgenPlugin` | ✅ |
| IWG-5 | Water Surface | `worldgen/systems/water_surface.rs`, registrado en `WorldgenPlugin` | ✅ |
| IWG-6 | Atmosphere: Sol/Fog/Bloom | `blueprint/equations/inferred_world_geometry/atmosphere.rs`, `simulation/metabolic/atmosphere_inference.rs`, registrado en `WorldgenPlugin` (`register_atmosphere_pipeline`) | ✅ |
| IWG-7 | Integration Demo | `world/demos/inferred_world.rs`, `assets/maps/inferred_world.ron` | ✅ |

Demo: `RESONANCE_MAP=inferred_world cargo run`

---

## Paralelismo seguro

| | IWG-1 | IWG-2 | IWG-3 | IWG-4 | IWG-5 | IWG-6 | IWG-7 |
|---|---|---|---|---|---|---|---|
| **IWG-1** | — | | ✅ | | | ✅ | |
| **IWG-3** | ✅ | | — | | | ✅ | |
| **IWG-2** | | — | | ✅ | | ✅ | |
| **IWG-4** | | ✅ | | — | | ✅ | |
| **IWG-6** | ✅ | ✅ | ✅ | ✅ | | — | |

IWG-1 y IWG-3 son paralelos (Onda 0): archivos distintos, sin overlap.
IWG-2, IWG-4 y IWG-6 son paralelos (Onda A): sistemas independientes.
IWG-5 depende de IWG-4 (terrain height como referencia).

---

## Invariantes del track

1. **Math in equations/.** Ecuaciones nuevas en `blueprint/equations/inferred_world_geometry/`.
2. **Max 4 campos por componente.** `BodyPlanLayout` = 4 campos.
3. **SparseSet para todo nuevo.**
4. **Guard change detection.** `if old != new`.
5. **Sin RNG.** Determinista.
6. **Backward compatible.** Flora existente sin cambios. Mundos sin V7 → sin terrain mesh.
7. **Phase assignment.** Inferencia → `Phase::MorphologicalLayer`. Sync → `Update`.
8. **Reutiliza existente.** `generate_terrain_mesh()`, `organ_attachment_points()`, `organ_orientation()`, `build_organ_mesh()` no se reescriben — se extienden o cableman.
9. **Mesh válida.** positions == normals == uvs == colors. Indices in range.
10. **Separación FixedUpdate/Update.** Inferencia determinista nunca toca `Mesh3d`.

## Contrato de pipeline IWG

```
FixedUpdate:
  Phase::MetabolicLayer
    ← [existente] metabolic_graph_step, entropy, pools, trophic, etc.
  Phase::MorphologicalLayer
    ← [existente] shape_optimization, albedo_inference, surface_rugosity, allometric_growth
    ← terrain_mesh_generation_system  (lee Res<TerrainField> + Res<EnergyFieldGrid>)
    ← water_surface_system            (.after terrain_mesh_generation)
    ← atmosphere_inference_system     (independiente, cada 30 ticks)

Update:
    ← [existente] shape_color_inference_system (ahora con BodyPlanLayout si existe)
    ← terrain_mesh_sync_system
    ← water_mesh_sync_system
    ← atmosphere_sync_system
```

Nota: `body_plan_assembly` se integra dentro de `shape_color_inference_system` (IWG-2C) vía el parámetro `layout: Option<&BodyPlanLayout>` en `build_organ_mesh()`.

---

## Ejemplo motivador: Mundo inferido

```
EnergyFieldGrid 32×32 + TerrainField 32×32:

  Celda (16,16): Terra-band, bond=1200, Solid, slope=0.1
    → terrain altitude = 4.2 (montaña — ya en TerrainField)
    → terrain color = [0.45, 0.38, 0.28, 1.0] (marrón — IWG-3)
    → generate_terrain_mesh() → mesh con Y=4.2, color marrón — (existente + IWG-4 wiring)

  Celda (24,16): Aqua-band, bond=200, Liquid
    → terrain altitude = 0.3 (valle — ya en TerrainField)
    → water_surface_height = 1.2 (IWG-5)
    → water color = deep blue — (IWG-5)

  Organismo cuadrúpedo en (10,10):
    OrganManifest: [Core, Stem, Limb×4, Sensory, Thorn×2]
    count_limbs = 4 → infer_symmetry_mode → Bilateral
    organ_attachment_points (existente) → posiciones en spine
    + lateral_offset (IWG-1B) → patas a ±X del spine
    → BodyPlanLayout cache (IWG-2A)
    → build_organ_mesh(layout: Some) → cuadrúpedo con cuernos

  Atmósfera:
    avg_qe_norm = 0.6 → bloom = 0.12
    world_radius = 50.0 → fog_start = 30, fog_end = 60
    latitude = 0.4 → sun at ~23°

→ Mundo completo inferido. Cero assets artísticos.
```

---

## Referencias cruzadas

- `docs/design/INFERRED_WORLD_GEOMETRY.md` — Blueprint teórico
- `docs/arquitectura/blueprint_inferred_world_geometry.md` — Contrato (8 secciones)
- `src/worldgen/inference/organ.rs` — Pipeline de órganos existente (se extiende)
- `src/topology/terrain_mesher.rs` — `generate_terrain_mesh()` existente (se cablea)
- `src/runtime_platform/scenario_isolation/mod.rs` — Luz hardcoded (se migra)
- `docs/sprints/MORPHOGENESIS_INFERENCE/README.md` — Track MG (prerequisito)
- `docs/sprints/GEOMETRY_FLOW/README.md` — GF1 (primitivas reutilizadas)
- `docs/design/TERRAIN_MESHER.md` — Blueprint del terrain mesher
