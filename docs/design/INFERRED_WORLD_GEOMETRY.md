# BLUEPRINT — Geometría de Mundo Inferida (Inferred World Geometry)

---

## 0. Resumen Ejecutivo

Resonance infiere forma, color y superficie de organismos desde termodinámica (MG-1→MG-7). Pero el **mundo mismo** — terreno, agua, atmósfera, body plans de fauna — todavía no usa este pipeline. Este blueprint extiende la inferencia a todo lo visible: un planeta entero renderizado a resolución N64 (~300-800 polígonos por entidad) donde cada vértice, color y altura emerge de la simulación energética.

**Principio rector:** si el albedo de un organismo se infiere de su balance radiativo, entonces la altura del terreno se infiere de su densidad de enlace, el color del agua de su estado de materia, y la posición de las extremidades de un animal del DAG metabólico que las conecta. No hay assets artísticos — hay ecuaciones.

**Relación con Resonance:** este blueprint cierra la brecha entre simulación (que ya funciona end-to-end) y presencia visual. Completa el pipeline: `EnergyFieldGrid` → `TerrainMesh`, `OrganManifest` → `BodyPlanLayout` → `CompoundMesh`, `MatterState::Liquid` → `WaterSurface`, `WorldState` → `Atmosphere`.

---

## 1. Auditoría: Qué Tenemos vs Qué Falta

### 1.1 Geometría de Organismos

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| Primitivas geométricas (Tube, FlatSurface, PetalFan, Bulb) | ✅ Implementado | `geometry_flow/primitives.rs` |
| Branching recursivo (GF1) | ✅ Implementado | `geometry_flow/branching.rs` |
| OrganManifest (12 roles, lifecycle) | ✅ Implementado | `layers/organ.rs` |
| build_organ_primitive() dispatch | ✅ Implementado | `geometry_flow/primitives.rs` |
| Vertex color por energía | ✅ Implementado | `geometry_flow/mod.rs` |
| Posicionamiento de órganos en spine (flora) | ✅ Implementado | `worldgen/inference/organ.rs` |
| organ_attachment_points() (Apical/Basal/Distributed/Full) | ✅ Implementado | `worldgen/inference/organ.rs` |
| organ_orientation() (GravityDown/Outward/AlongTangent) | ✅ Implementado | `worldgen/inference/organ.rs` |
| build_organ_mesh() → Mesh3d pipeline end-to-end | ✅ Implementado | `worldgen/inference/organ.rs` + `shape.rs` |
| **Simetría bilateral/radial inferida (offset lateral)** | ❌ Falta | — |
| **Body plan cache (BodyPlanLayout component)** | ❌ Falta | — |
| **Escala alométrica WBE 3/4** | ❌ Falta | — |

### 1.2 Terreno

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| EnergyFieldGrid (V7) | ✅ Implementado | `worldgen/field_grid.rs` |
| Visual derivation (energy → color, scale, opacity) | ✅ Implementado | `worldgen/systems/visual.rs` |
| Topology (heightmap, drainage, erosion) | ✅ Implementado | `topology/` |
| TerrainField resource (altitude, slope, drainage, terrain_type) | ✅ Implementado | `topology/terrain_field.rs` |
| `generate_terrain_mesh()` (heightmap → Mesh con smooth normals) | ✅ Implementado, testeado | `topology/terrain_mesher.rs` |
| `TerrainVisuals` struct (vertex_colors SoA) | ✅ Implementado | `topology/terrain_mesher.rs` |
| **Cruce visual V7 → TerrainVisuals (colores reales)** | ❌ Falta — solo `neutral_flat()` | — |
| **Sistema ECS que llame `generate_terrain_mesh()` e inserte Mesh3d** | ❌ Falta — solo en tests | — |

### 1.3 Agua

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| MatterState (Solid, Liquid, Gas, Plasma) | ✅ Implementado | `layers/matter.rs` |
| Detección de celdas líquidas en field grid | ✅ Parcial | `worldgen/` (implícito en materialización) |
| **Water surface mesh** | ❌ Falta | — |
| **Wave vertex shader** | ❌ Falta | — |

### 1.4 Atmósfera

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| DirectionalLight (estática) | ✅ Hardcoded | `runtime_platform/scenario_isolation/mod.rs` (20k lux, pos fija) |
| FogSettings, BloomSettings | ✅ Disponible en Bevy, no usado | — |
| AmbientPressure (viscosity, delta_qe) | ✅ Implementado | `layers/ambient_pressure.rs` |
| **Iluminación inferida del estado del mundo** | ❌ Falta | — |
| **Fog inferido de radio/densidad** | ❌ Falta | — |
| **Bloom inferido de energía promedio** | ❌ Falta | — |

---

## 2. Arquitectura de 4 Capas

```
Capa 1: ECUACIONES PURAS (blueprint/equations/inferred_world_geometry/)
  ├── body_plan.rs        → posicionamiento, simetría, proporciones
  ├── terrain_mesh.rs     → height_from_energy, vertex_color, mesh builder
  ├── water_surface.rs    → detección, height, wave displacement
  └── atmosphere.rs       → sun direction, fog, ambient, bloom

Capa 2: COMPONENTES (layers/)
  ├── body_plan_layout.rs → BodyPlanLayout (organ positions cache)
  ├── terrain_visual.rs   → TerrainChunkMesh (mesh handle + dirty flag)
  └── water_visual.rs     → WaterSurfaceMesh (mesh handle + params)

Capa 3: SISTEMAS (simulation/)
  ├── body_plan_assembly_system      → Phase::MorphologicalLayer
  ├── terrain_mesh_generation_system → Phase::MorphologicalLayer
  ├── water_surface_system           → Phase::MorphologicalLayer (.after terrain)
  └── atmosphere_inference_system    → Phase::MorphologicalLayer

Capa 4: RENDERING BRIDGE (Update)
  ├── terrain_mesh_sync_system       → Update (inserta/actualiza Mesh3d)
  ├── water_mesh_sync_system         → Update (inserta/actualiza Mesh3d + material)
  └── atmosphere_sync_system         → Update (actualiza luces + fog)
```

**Separación FixedUpdate / Update:** la inferencia (capas 1-3) corre en `FixedUpdate` determinista. La sincronización visual (capa 4) corre en `Update` no-determinista, leyendo los componentes inferidos.

---

## 3. Especificación: Body Plan Assembler

### 3.1 Problema

`OrganManifest` dice *qué* órganos tiene un organismo. `build_organ_primitive()` sabe generar la mesh de *cada* órgano. Pero nadie dice *dónde* va cada órgano en el espacio 3D. Actualmente la rosa usa GF1 branching que posiciona órganos a lo largo de un spine — funciona para flora pero no para fauna.

### 3.2 Solución: Inferencia de Posición desde DAG

El `MetabolicGraph` ya define la **topología** del organismo: qué órgano conecta con qué. Un Limb conectado a Core vía una arista con alto `flow_rate` → extremidad principal. Un Sensory conectado al final de la cadena → cabeza (punto más alejado del Core en el DAG).

**Reglas de posicionamiento:**

1. **Core** → origen (0, 0, 0) relativo a la entidad
2. **Stem** → eje principal (Y+ por defecto, modulado por `energy_direction`)
3. **Limbs** → distribuidos simétricamente alrededor del Stem
   - `count` par → simetría bilateral (pares opuestos en plano XZ)
   - `count` impar → simetría radial (equidistantes en TAU/count)
4. **Sensory** → extremo del Stem más alejado del Core (arriba/adelante)
5. **Root** → extremo opuesto al Sensory (abajo/atrás)
6. **Leaf/Petal/Thorn** → distribuidos a lo largo del Stem (filotaxis)
7. **Shell** → envuelve el Core (offset radial = core_radius × 1.2)
8. **Fin** → plano lateral al FlowVector dominante
9. **Fruit/Bud** → nodos terminales del DAG (puntas de ramas)

**Proporciones alométricas:**

```
organ_length = core_radius × ROLE_LENGTH_RATIO[role] × scale_factor
organ_radius = organ_length / ROLE_FINENESS[role]
```

Los ratios dependen del role y son constantes tunables. La escala absoluta viene de `SpatialVolume.radius`.

### 3.3 Simetría

```rust
pub enum SymmetryMode {
    Bilateral,   // Limb.count par → pares opuestos (vertebrados)
    Radial,      // Limb.count impar o >4 → equidistante (estrellas, medusas)
    Asymmetric,  // Limb.count == 1 → sin simetría impuesta (caracoles)
}
```

La simetría se **infiere** del conteo de Limbs, no se diseña. Un organismo con 4 Limbs → bilateral (cuadrúpedo). Con 5 → radial (estrella de mar). Con 1 → asimétrico.

### 3.4 Componente

```rust
/// Cache de posiciones de órganos inferidas del DAG metabólico.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct BodyPlanLayout {
    /// Posiciones relativas al Core, indexadas por OrganSpec index.
    positions: [Vec3; MAX_ORGANS_PER_ENTITY],
    /// Direcciones de cada órgano (normal outward).
    directions: [Vec3; MAX_ORGANS_PER_ENTITY],
    /// Simetría inferida.
    symmetry: SymmetryMode,
    /// Cantidad de órganos activos (match con OrganManifest.len()).
    active_count: u8,
}
```

4 campos. SparseSet (solo entidades con MetabolicGraph + OrganManifest).

---

## 4. Especificación: Terrain Mesh

### 4.1 Problema

V7 genera un `EnergyFieldGrid` donde cada celda tiene qe, frecuencia, densidad, bond_energy, matter_state. El sistema visual actual (`visual.rs`) mapea esto a color/scale por entidad — pero no genera un **mesh de terreno continuo**.

### 4.2 Solución: Heightmap Mesh desde Campo Energético

```
height(x, z) = f(bond_energy, density, matter_state)
color(x, z)  = visual_derivation(element_id, qe_norm)
```

**Función de altura:**

```
height = BASE_HEIGHT
       + bond_energy × BOND_HEIGHT_SCALE          (roca = alto, gas = bajo)
       × density_factor(density)                    (denso = sólido = elevado)
       × state_modifier(matter_state)               (Solid=1.0, Liquid=0.3, Gas=0.1)
```

- Terreno sólido con alta energía de enlace → montañas
- Líquido con baja densidad → valles/cuencas
- Gas → plano o depresión

**Vertex color:** reutiliza el pipeline `EnergyVisual` existente — ya mapea (element_id, qe_norm, matter_state) → sRGB.

**Mesh:** heightmap estándar — grid de vértices con Y = height, triangulado en quads, normals por face-average. Chunked para LOD.

### 4.3 Budget de Polígonos (Mario 64 Reference)

Mario 64 renderizaba escenas completas con ~3000-6000 polígonos totales. Target:

| Elemento | Polys | Método |
|----------|-------|--------|
| Terreno (32×32 grid) | ~2000 | Heightmap mesh |
| Agua | ~200 | Plano subdividido |
| Flora ×10 | ~2000 | GF1 branching existente |
| Fauna ×5 | ~2000 | Body plan assembler |
| **Total** | **~6200** | Dentro del budget N64 |

---

## 5. Especificación: Water Surface

### 5.1 Detección

Recorrer `EnergyFieldGrid`. Celdas donde `matter_state == Liquid` → candidatas para agua. La altura del agua = promedio de `terrain_height` de las celdas líquidas vecinas.

### 5.2 Mesh

Plano subdividido (8×8 a 16×16) en la región líquida. Vertex Y = `water_height`. Color = azul modulado por profundidad (`water_height - terrain_height`).

### 5.3 Animación (Opcional)

Vertex shader WGSL: desplazamiento sinusoidal.

```wgsl
let wave = sin(position.x * WAVE_FREQ + globals.time * WAVE_SPEED) * WAVE_AMPLITUDE;
out.position.y += wave;
```

---

## 6. Especificación: Atmospheric Inference

### 6.1 Sol

```
sun_direction = normalize(cos(latitude) * cos(time_angle), sin(latitude), cos(latitude) * sin(time_angle))
sun_intensity = BASE_INTENSITY × max(0, sun_direction.y)
```

El ángulo solar se infiere de la posición en el mundo y un reloj de simulación (si existe) o se fija al mediodía.

### 6.2 Fog

```
fog_start = world_radius × FOG_START_RATIO
fog_end   = world_radius × FOG_END_RATIO
fog_color = sky_color × (1 - avg_canopy_density)
```

Inferido del tamaño del mundo y la densidad promedio de canopy.

### 6.3 Bloom

```
bloom_intensity = avg_qe_norm × BLOOM_QE_SCALE
```

Más energía promedio en el mundo → más bloom. Mundos energéticos brillan; mundos muertos son mates.

---

## 7. Presupuesto Visual vs Mario 64

| Aspecto | Mario 64 | Resonance IWG |
|---------|----------|---------------|
| Polys/personaje | 300-800 | 200-800 (body plan assembler) |
| Polys/árbol | 12-50 | 200-2000 (GF1, configurable por LOD) |
| Texturas | 32×32 px tiles | Vertex color inferido (equivalente) |
| Iluminación | 1 directional + ambient | 1 directional + ambient + fog (inferidos) |
| Agua | Plano texturado | Plano con wave shader |
| Terreno | Heightmap pre-baked | Heightmap inferido de energía |
| Total polys/escena | 3000-6000 | 4000-8000 (configurable) |
| FPS target | 30 (N64) | 60 (PC) |

**La diferencia clave:** en Mario 64 un artista modeló cada árbol, cada colina, cada textura. En Resonance IWG, **la misma simulación que determina qué criatura sobrevive también determina cómo se ve el mundo**.

---

## 8. Riesgos y Mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Body plan assembler produce poses incoherentes | Visual roto | Fallback a distribución radial simple; tests de overlap |
| Terrain mesh demasiado uniforme | Visualmente aburrido | Noise de alta frecuencia sobre height function; rugosity del terreno |
| Water detection produce islas falsas | Glitches visuales | Flood-fill para conectar regiones líquidas; mínimo de celdas contiguas |
| Performance con muchas entidades | <60fps | LOD agresivo: Far=congelar mesh, Mid=update cada N ticks, Near=cada tick |
| Atmosphere demasiado simple | No se nota | Iterar post-integración; es la pieza menos crítica |

---

## 9. Relación con Tracks Existentes

| Track | Relación |
|-------|----------|
| MORPHOGENESIS_INFERENCE (MG) | IWG extiende MG: body plan usa MetabolicGraph de MG-2, shape params de MG-4 |
| GEOMETRY_FLOW (GF) | IWG reutiliza primitivas GF1; body plan ensambla lo que GF1 genera |
| ENERGY_PARTS_INFERENCE (EPI) | IWG terrain usa visual derivation de EPI |
| BLUEPRINT_V7 | IWG terrain lee EnergyFieldGrid de V7 |
| THERMODYNAMIC_LADDER (TL) | Las ecuaciones de IWG respetan la escalera termodinámica |
| ECOSYSTEM_AUTOPOIESIS (EA) | Body plan se activa para entidades spawneadas por EA |

---

## 10. Referencias

- Bejan, A. (1997) — *Constructal Law*: shape optimization from thermodynamic constraints
- West, Brown & Enquist (1997) — *WBE Scaling*: allometric proportions from metabolic networks
- Thompson, D'Arcy W. (1917) — *On Growth and Form*: morphology as physical solution
- Turing, A. (1952) — *Chemical basis of morphogenesis*: pattern from reaction-diffusion
- `docs/design/MORPHOGENESIS.md` — blueprint de inferencia morfológica
- `docs/design/V7.md` — campo de energía procedural
- `docs/design/TERRAIN_MESHER.md` — diseño previo de mallas de terreno (si existe)
- `docs/design/GEOMETRY_DEFORMATION_ENGINE.md` — motor de deformación geométrica
