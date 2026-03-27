# Blueprint V7: Worldgen por Composicion Energetica (`worldgen/`)

Si todo es energia, todo lo que VES es una proyeccion de la energia.
Nucleos de energia se propagan por el espacio, la materia se materializa por reglas stateless,
y el aspecto visual emerge de la composicion energetica. Distintos mapas = distintos nucleos.

## Pipeline V7

```mermaid
flowchart LR
    subgraph "1. Nuclei"
        RON["MapConfig<br/>(assets/maps/*.ron)"]
        NUC["EnergyNucleus<br/>freq_hz, emission_rate<br/>propagation_radius, decay"]
    end

    subgraph "2. Campo"
        GRID["EnergyFieldGrid<br/>(grid 2D de EnergyCell)"]
        PROP["Propagation<br/>(diffuse por tick)"]
        DISS["Dissipation<br/>(Segunda Ley)"]
        NF["NutrientFieldGrid<br/>(nutrientes por celda)"]
    end

    subgraph "3. Materializacion"
        RULES["MaterializationRules<br/>(stateless)"]
        WA["WorldArchetype<br/>(Terrain/Flora/Water/...)"]
        MAT["Materialized<br/>(component marker)"]
        EB["ElementBand + DensityClass"]
    end

    subgraph "4. Visual"
        VD["VisualDerivation<br/>(color, scale, emission)"]
        SC["ShapeColorInference<br/>(GF1 + campo)"]
        GM["GrowthMorphology<br/>(organs + mesh)"]
        TM["TerrainVisualMesh"]
        WS["WaterSurface"]
    end

    RON --> NUC
    NUC -->|emision por tick| GRID
    GRID --> PROP --> GRID
    GRID --> DISS --> GRID
    GRID --> NF
    GRID --> RULES
    RULES --> WA --> MAT
    RULES --> EB
    MAT --> VD
    VD --> SC & GM & TM & WS

    style GRID fill:#e74c3c,color:#fff
    style MAT fill:#27ae60,color:#fff
    style VD fill:#3498db,color:#fff
```

## Ciclo de vida del campo

```mermaid
sequenceDiagram
    participant S as Startup
    participant W as Warmup (N ticks)
    participant A as Active (runtime)
    participant U as Update (visual)

    S->>S: load MapConfig RON<br/>spawn EnergyNucleus entities<br/>init EnergyFieldGrid + NutrientFieldGrid
    S->>W: enter Playing/Warmup

    loop WARMUP_TICKS
        W->>W: nuclei emit to grid<br/>propagation (diffuse)<br/>dissipation<br/>materialization delta
    end
    W->>A: enter Active

    loop cada FixedUpdate tick
        A->>A: nuclei emit<br/>propagation<br/>dissipation<br/>materialization delta<br/>eco boundaries<br/>nutrient sync
    end

    A->>U: cada frame (Update)
    U->>U: visual derivation<br/>shape inference<br/>phenology visual<br/>terrain mesh<br/>water surface
```

## Tipos principales

| Tipo | Archivo | Rol |
|------|---------|-----|
| `EnergyNucleus` | nucleus.rs | Fuente: freq, emission, radius, decay |
| `PropagationDecay` | nucleus.rs | InverseSquare / InverseLinear / Flat / Exponential |
| `EnergyFieldGrid` | field_grid.rs | Resource: grid 2D de celdas de energia |
| `EnergyCell` | contracts.rs | Celda: qe total, FrequencyContribution[] |
| `FrequencyContribution` | contracts.rs | freq_hz + intensity por contribucion |
| `Materialized` | contracts.rs | Marker: celda ya materializo entidad |
| `MaterializationResult` | contracts.rs | Resultado: archetype + properties |
| `WorldArchetype` | archetypes.rs | Terrain / Flora / Water / Crystal / ... |
| `ElementBand` | archetypes.rs | Banda elemental dominante |
| `DensityClass` | archetypes.rs | Low / Medium / High |
| `NutrientFieldGrid` | nutrient_field.rs | Grid paralelo de nutrientes |
| `NutrientCell` | nutrient_field.rs | Nutriente disponible + regeneracion |
| `PropagationMode` | propagation_mode.rs | Diffuse / Directional |
| `MapConfig` | map_config.rs | Config de mapa desde RON |
| `CellFieldSnapshot` | cell_field_snapshot.rs | Snapshot de celda para GPU/cache |
| `CellFieldSnapshotCache` | cell_field_snapshot.rs | Cache de snapshots |

## Modulos internos

```
worldgen/
+-- nucleus.rs              -- EnergyNucleus, PropagationDecay
+-- field_grid.rs           -- EnergyFieldGrid (Resource)
+-- contracts.rs            -- EnergyCell, Materialized, visual params
+-- archetypes.rs           -- WorldArchetype, ElementBand, DensityClass
+-- propagation.rs          -- diffusion + dominant frequency resolve
+-- propagation_mode.rs     -- PropagationMode, diffuse system
+-- materialization_rules.rs -- reglas stateless de materializacion
+-- nutrient_field.rs       -- NutrientFieldGrid, bias por frecuencia
+-- map_config.rs           -- MapConfig, load from RON/env
+-- organ_inference.rs      -- attachment points, organ mesh
+-- shape_inference.rs      -- ShapeInferred, GF1 integration
+-- visual_derivation.rs    -- color, scale, emission, opacity
+-- cell_field_snapshot.rs  -- snapshot cache + GPU layout
+-- field_visual_sample.rs  -- muestreo visual por posicion
+-- lod.rs                  -- LOD por distancia
+-- constants.rs            -- FIELD_CELL_SIZE, thresholds
+-- systems/
    +-- startup.rs          -- init grid, spawn nuclei
    +-- prephysics.rs       -- per-tick: propagation, materialization delta
    +-- materialization.rs  -- NucleusFreqTrack, SeasonTransition
    +-- terrain_visual_mesh.rs -- mesh de terreno
    +-- water_surface.rs    -- mesh de agua
    +-- visual.rs           -- derivacion visual (Update)
    +-- phenology_visual.rs -- fenologia estacional
    +-- performance.rs      -- budgets, LOD, cache stats
    +-- materialization_delta.rs -- delta incremental
```

## Dependencias

- `crate::blueprint::equations` — field_color, entity_shape, inferred_world_geometry
- `crate::blueprint::constants` — FIELD_CELL_SIZE, thresholds, morphogenesis
- `crate::layers` — BaseEnergy, SpatialVolume, OscillatorySignature, AmbientPressure
- `crate::eco` — eco_boundaries para clasificacion de zonas
- `crate::topology` — heightmap, drainage para terreno

## Invariantes

- `EnergyFieldGrid` dimensionado en startup, inmutable en tamano despues
- Propagacion conserva energia global (suma de celdas estable modulo dissipation)
- Materializacion stateless: misma celda + misma energia = mismo resultado
- Warmup completo antes de `PlayState::Active`
- Visual derivation en `Update`, no en `FixedUpdate`
- `NutrientFieldGrid` alineado 1:1 con `EnergyFieldGrid`
