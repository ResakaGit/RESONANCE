# Geometría de Mundo Inferida (Inferred World Geometry)

## 1) Propósito y frontera

**Qué resuelve.** Genera toda la geometría visible del mundo — terreno, agua, body plans de fauna, atmósfera — como inferencia de la simulación energética. Ningún asset artístico; cada vértice, color y altura emerge de ecuaciones puras alimentadas por el estado de las 14 capas ECS.

**Qué no resuelve.**
- No reemplaza GF1 (branching/spine) — lo usa como primitiva.
- No modifica la inferencia morfológica existente (MG-1→MG-7) — la extiende con posicionamiento espacial.
- No implementa UI, HUD ni menús.
- No implementa animación skeletal — las entidades se mueven por `FlowVector`, no por huesos.
- No gestiona networking ni multiplayer.

---

## 2) Superficie pública (contrato)

### Tipos exportados

| Tipo | Módulo | Storage | Campos | Propósito |
|------|--------|---------|--------|-----------|
| `BodyPlanLayout` | `layers/body_plan_layout.rs` | SparseSet | 4 | Cache de posiciones/direcciones de órganos inferidas del DAG |
| `SymmetryMode` | `layers/body_plan_layout.rs` | — (enum) | — | Bilateral / Radial / Asymmetric |
| `TerrainChunkMesh` | `layers/terrain_visual.rs` | SparseSet | 3 | Handle de mesh + dirty flag + chunk coords |
| `WaterSurfaceMesh` | `layers/water_visual.rs` | SparseSet | 3 | Handle de mesh + water height + wave params |
| `AtmosphereState` | Resource | — | 4 | sun_dir, fog_start, fog_end, bloom_intensity |

### Funciones puras (blueprint/equations/inferred_world_geometry/)

| Función | Input | Output | Archivo |
|---------|-------|--------|---------|
| `body_plan_organ_position` | role, index, count, core_radius, symmetry | `Vec3` | `body_plan.rs` |
| `infer_symmetry_mode` | limb_count | `SymmetryMode` | `body_plan.rs` |
| `allometric_organ_scale` | role, biomass, organ_count | `f32` | `body_plan.rs` |
| `body_plan_organ_direction` | role, position, core_pos, flow_dir | `Vec3` | `body_plan.rs` |
| `optimal_appendage_count` | radius, fineness, density, velocity, viscosity, limb_l, limb_r, max | `u8` | `entity_shape.rs` |
| `organ_slot_scale` | slot_index, active_count, mobility_bias | `(f32, f32)` | `entity_shape.rs` |
| `projected_area_with_limbs` | radius, count, limb_l, limb_r | `f32` | `entity_shape.rs` |
| `terrain_height_from_energy` | bond_energy, density, matter_state | `f32` | `terrain_mesh.rs` |
| `terrain_vertex_color` | element_id, qe_norm, matter_state | `[f32; 3]` | `terrain_mesh.rs` |
| `build_terrain_chunk_mesh` | heights, colors, chunk_size, cell_size | `Mesh` | `terrain_mesh.rs` |
| `water_surface_height` | liquid_cells_heights, terrain_height | `f32` | `water_surface.rs` |
| `build_water_mesh` | bounds, height, subdivisions, depth_colors | `Mesh` | `water_surface.rs` |
| `inferred_sun_direction` | latitude, time_angle | `Vec3` | `atmosphere.rs` |
| `inferred_fog_params` | world_radius, avg_density, canopy_factor | `(f32, f32)` | `atmosphere.rs` |
| `inferred_bloom_intensity` | avg_qe_norm | `f32` | `atmosphere.rs` |

### Sistemas

| Sistema | Phase | Lee | Escribe | Orden |
|---------|-------|-----|---------|-------|
| `body_plan_assembly_system` | MorphologicalLayer | OrganManifest, MetabolicGraph, SpatialVolume, FlowVector | BodyPlanLayout | `.after(allometric_growth_system)` |
| `constructal_body_plan_system` | MorphologicalLayer | L1, L3, L6, MorphShapeParams, InferenceProfile, CapabilitySet | BodyPlanLayout | `.after(albedo_inference_system)` |
| `entity_shape_inference_system` | Update | L0-L4, L2, BodyPlanLayout, MorphSurface, InferredAlbedo | Mesh3d (compound) | `.after(sync_visual)` |
| `terrain_mesh_generation_system` | MorphologicalLayer | EnergyFieldGrid (Res), TopologyData (Res) | TerrainChunkMesh | — |
| `water_surface_system` | MorphologicalLayer | EnergyFieldGrid (Res), TerrainChunkMesh | WaterSurfaceMesh | `.after(terrain_mesh_generation_system)` |
| `atmosphere_inference_system` | MorphologicalLayer | EnergyFieldGrid (Res), SimulationClock (Res) | AtmosphereState (ResMut) | — |
| `terrain_mesh_sync_system` | Update | TerrainChunkMesh | Mesh3d, MeshMaterial3d | — |
| `water_mesh_sync_system` | Update | WaterSurfaceMesh | Mesh3d, MeshMaterial3d | — |
| `atmosphere_sync_system` | Update | AtmosphereState (Res) | DirectionalLight, FogSettings, BloomSettings | — |

### Eventos

Ninguno nuevo. Los sistemas leen estado de componentes/recursos existentes.

---

## 3) Invariantes y precondiciones

1. **Determinismo.** Misma `EnergyFieldGrid` + mismo `OrganManifest` → misma geometría. Sin RNG en ninguna ecuación IWG.
2. **Conservación de polígonos.** Cada sistema respeta un budget configurable (`MAX_TERRAIN_TRIS`, `MAX_WATER_TRIS`, `MAX_BODY_PLAN_TRIS`). Nunca se excede sin warning.
3. **Backward compatible.** Entidades sin `MetabolicGraph` → no reciben `BodyPlanLayout`. Mundos sin `EnergyFieldGrid` → no generan terrain mesh.
4. **Max 4 campos.** Todo componente nuevo cumple la regla.
5. **SparseSet.** Todos los componentes nuevos son transient/sparse — solo entidades que lo necesitan.
6. **Guard change detection.** Todo sistema verifica `if old != new` antes de mutar. Terrain mesh solo se regenera si el chunk está dirty.
7. **Mesh válida.** Toda mesh generada tiene positions, normals, uvs, colors, indices sincronizados. No hay índices fuera de rango.

---

## 4) Comportamiento runtime

```
FixedUpdate:
  Phase::MetabolicLayer
    ← [existente] metabolic_graph_step_system, entropy systems, pool systems, etc.
  Phase::MorphologicalLayer
    ← [existente] shape_optimization, albedo_inference, surface_rugosity, allometric_growth
    ← constructal_body_plan_system   (.after albedo_inference — MOVE + L6 entities)
    ← body_plan_assembly_system      (.after allometric_growth — worldgen entities)
    ← terrain_mesh_generation_system  (independiente, lee Res<EnergyFieldGrid>)
    ← water_surface_system            (.after terrain_mesh_generation)
    ← atmosphere_inference_system     (independiente, lee Res)

Update:
    ← terrain_mesh_sync_system        (lee TerrainChunkMesh → escribe Mesh3d)
    ← water_mesh_sync_system          (lee WaterSurfaceMesh → escribe Mesh3d)
    ← atmosphere_sync_system          (lee AtmosphereState → escribe Light/Fog/Bloom)
```

**Frecuencia de actualización:**
- Body plan: cada tick (entidades cambian por crecimiento).
- Terrain: solo cuando chunk dirty (evento de worldgen o mutación topológica).
- Water: solo cuando terrain cambia o liquid cells cambian.
- Atmosphere: cada N ticks (lento, parámetros globales).

**Side-effects:**
- Los sistemas de `FixedUpdate` solo escriben componentes/recursos. No spawnan/despawnan entidades.
- Los sistemas de `Update` insertan/actualizan `Mesh3d` y `MeshMaterial3d` en entidades que ya tienen el componente IWG correspondiente.

---

## 5) Implementación y trade-offs

### Terrain mesh: chunked vs monolítico

**Elegido: chunked** (grid dividido en chunks de 8×8 o 16×16 celdas). Cada chunk es una entidad con su mesh.

**Costo:** más entidades, más draw calls.
**Valor:** actualización incremental (solo chunks dirty), LOD por distancia (Far = low-res, Near = full-res), frustum culling natural.

### Body plan: constructal inference + compound mesh

**Elegido: constructal optimization + compound GF1 mesh.**

Two paths to BodyPlanLayout:
1. **Constructal** (entities with L6 AmbientPressure + MOVE): `optimal_appendage_count()` minimizes `drag × thrust_efficiency + maintenance` → N limbs. Organ proportions from `organ_slot_scale(slot, count, mobility_bias)` — front/rear asymmetry emerges from mobility.
2. **Fallback** (entities without L6): hardcoded bilateral quadruped via `bilateral_quadruped_attachments()`.

**Compound mesh assembly** in `entity_shape_inference_system`:
- Torso = main GF1 tube (build_flow_spine + build_flow_mesh)
- Per organ slot: sub-influence (position from layout, scaled length/radius) → sub-spine → sub-mesh
- `merge_meshes([torso, organs...])` → single Mesh3d

**Costo:** no hay animación articulada (no hay joints/bones).
**Valor:** cero dependencias nuevas. Shapes emerge from energy composition (mobility_bias → primate-like arms, low mobility → equal quadruped legs). Organisms change form by constructal optimizer, not by template.

### Water: vertex shader vs CPU displacement

**Elegido: vertex shader WGSL** para ondulación.

**Costo:** un shader custom (mínimo, ~15 líneas WGSL).
**Valor:** animación fluida sin costo CPU. El mesh se genera una vez y el shader lo ondula cada frame.

### Atmosphere: resource vs per-entity

**Elegido: `AtmosphereState` como `Resource`** global.

**Costo:** una sola atmósfera por mundo (no hay biomas con luz diferente).
**Valor:** simplicidad. Iteración futura puede hacer per-zone con un `Local<AtmosphereState>` o componente en zona.

---

## 6) Fallas y observabilidad

| Modo de falla | Detección | Fallback |
|---------------|-----------|----------|
| Body plan produce órganos solapados | `debug_assert!` de distancia mínima entre posiciones | Distribución radial equidistante como fallback |
| Terrain height NaN/Inf | Guard clamp en `terrain_height_from_energy` | Retorna `BASE_HEIGHT` |
| Water surface en celdas no-líquidas | Filter `matter_state == Liquid` estricto | No genera mesh |
| Atmosphere con qe_norm = 0 | Guard en `inferred_bloom_intensity` | Bloom = 0.0, luz mínima |
| Mesh con >budget polígonos | Counter en mesh builder | Trunca subdivisiones; warn en log |

**Observabilidad:**
- `DebugPlugin` puede mostrar gizmos de body plan (esferas en posiciones de órganos).
- Terrain chunks con wireframe overlay en modo debug.
- AtmosphereState imprimible vía inspector de Bevy.

---

## 7) Checklist de atomicidad

- **¿Una responsabilidad principal?** Sí: convertir estado energético simulado en geometría visible.
- **¿Acopla más de un dominio?** Lee de worldgen (EnergyFieldGrid), layers (14 capas), topology (heights). Escribe geometría. El acoplamiento es de lectura, no de escritura bidireccional.
- **¿Debería dividirse?** Los 4 subsistemas (body plan, terrain, water, atmosphere) son independientes entre sí. Podrían ser tracks separados, pero comparten el mismo principio y pipeline stage. Mantenerlos juntos facilita la demo de integración.

---

## 8) Referencias cruzadas

- `docs/design/INFERRED_WORLD_GEOMETRY.md` — Blueprint teórico completo
- `docs/design/MORPHOGENESIS.md` — Inferencia morfológica (base teórica)
- `docs/design/V7.md` — Campo de energía procedural
- `docs/design/TERRAIN_MESHER.md` — Diseño previo de terrain mesh
- `docs/arquitectura/blueprint_morphogenesis_inference.md` — MetabolicGraph, EntropyLedger
- `docs/arquitectura/blueprint_geometry_flow.md` — GF1 primitivas y branching
- `docs/arquitectura/blueprint_v7.md` — EnergyFieldGrid, materialización
- `docs/arquitectura/blueprint_layers.md` — 14 capas ECS
- `docs/sprints/INFERRED_WORLD_GEOMETRY/README.md` — Track de implementación
- `src/geometry_flow/primitives.rs` — build_organ_primitive(), 4 primitivas
- `src/worldgen/systems/visual.rs` — visual derivation pipeline
- `src/layers/organ.rs` — OrganManifest, OrganRole, GeometryPrimitive
