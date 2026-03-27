# Blueprint: Topology (Terrain)

**Modulo:** `src/topology/`
**Rol:** Sustrato topologico procedural — heightmap, slope, drainage, clasificacion. NOT a layer — Resource hermano del campo energetico
**Diseno:** `docs/design/TOPOLOGY.md`

---

## 1. Idea central

El terreno es un **Resource** (`TerrainField`) alineado al `EnergyFieldGrid`. Se genera una vez al startup con funciones puras (noise -> slope -> drainage -> classification -> erosion) y puede mutar en runtime via `TerrainMutationEvent`.

No es capa ECS: las entidades no "tienen" terreno, lo consultan por posicion.

---

## 2. Pipeline de generacion

```mermaid
flowchart LR
    N[NoiseParams] --> H[generate_heightmap]
    H --> NM[normalize_heightmap]
    NM --> FP[fill_pits]
    FP --> S[derive_slope_aspect]
    S --> FD[compute_flow_direction]
    FD --> FA[compute_flow_accumulation]
    FA --> ER[erode_hydraulic]
    ER --> CL[classify_all]
    CL --> TF[TerrainField<br>Resource]
```

---

## 3. TerrainField (Resource)

Grid SoA con arrays paralelos por celda:

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `altitude` | `Vec<f32>` | Altura normalizada |
| `slope` | `Vec<f32>` | Pendiente local |
| `aspect` | `Vec<f32>` | Orientacion de pendiente (rad) |
| `drainage` | `Vec<Vec2>` | Direccion de flujo |
| `drainage_accumulation` | `Vec<f32>` | Acumulacion de caudal |
| `terrain_type` | `Vec<TerrainType>` | Clasificacion geometrica |
| `generation` | `u32` | Contador de version (invalidacion de caches) |

Coordenadas: row-major `y * width + x`, misma convencion que `EnergyFieldGrid`.

---

## 4. Tipos

| Tipo | Archivo | Rol |
|------|---------|-----|
| `TerrainField` | `terrain_field.rs` | Resource principal — grid SoA |
| `TerrainConfig` | `config.rs` | Parametros de generacion (noise, erosion, umbrales) |
| `TerrainType` | `contracts.rs` | Peak / Ridge / Slope / Valley / Plain / Riverbed / Basin / Cliff / Plateau |
| `DrainageClass` | `contracts.rs` | Dry / Moist / Wet / River (umbrales en constants) |
| `TerrainSample` | `contracts.rs` | Snapshot Copy de una celda |
| `TerrainMutationEvent` | `mutations.rs` | Mutacion runtime (region + tipo) |

---

## 5. Generadores (funciones puras)

| Modulo | Funcion | Entrada | Salida |
|--------|---------|---------|--------|
| `noise` | `generate_heightmap` | NoiseParams, seed | Vec<f32> |
| `slope` | `derive_slope_aspect` | altitude[] | slope[], aspect[] |
| `drainage` | `fill_pits` -> `compute_flow_direction` -> `compute_flow_accumulation` | altitude[] | drainage[], accumulation[] |
| `hydraulics` | `erode_hydraulic` | altitude[], ErosionParams | altitude[] modificado |
| `classifier` | `classify_all` | slope[], accum[], thresholds | TerrainType[] |

---

## 6. Modulacion del campo energetico

Funciones en `topology::functions` modifican parametros del campo segun terreno:

- `modulate_emission(altitude)` — emision escala con altura
- `modulate_diffusion(slope)` — difusion escala con pendiente
- `modulate_decay(drainage_accum)` — decay escala con humedad

---

## 7. Mutaciones runtime

`TerrainMutationEvent` -> `apply_mutation()` -> `rederive_region(DirtyRegion)` — recalcula slope/drainage/classification solo en la region afectada. Incrementa `generation` para invalidar caches downstream.

---

## 8. Invariantes

1. `TerrainField` y `EnergyFieldGrid` comparten `width`, `height`, `cell_size`, `origin`
2. `world_to_cell()` y `cell_to_world()` son inversas (centro de celda)
3. Todas las funciones de generacion son stateless y deterministas
4. `TerrainType::default()` es `Plain`
