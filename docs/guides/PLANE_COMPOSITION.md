# Composición del plano de juego (Resonance)

Qué se apila en el **plano horizontal** de la sim y cómo se relaciona con las **capas ECS**, el **terreno** (ríos) y los **bordes lógicos**.

## 1. Capas de datos (de abajo hacia arriba)

| Capa | Qué es | Dónde vive | Rol |
|------|--------|------------|-----|
| **Campo energético** | `EnergyCell` denso (qe, Hz, fase, materia derivada…) | `EnergyFieldGrid` (resource) | Fuente de verdad escalar del worldgen; propagación / disipación. |
| **Terreno** | Altitud, pendiente, drenaje, `TerrainType` | `TerrainField` (resource, mismo tamaño que el grid) | Modula fricción de movimiento (`terrain_effects_system` en cadena `Phase::AtomicLayer`, `simulation/physics.rs`) y clasifica **Riverbed** / valle / etc. (`classify_terrain` / `classify_all` en `src/topology/generators/classifier.rs`). **No** es collider de malla 3D. |
| **Celdas materializadas** | Entidades con capas 0–4+ (`BaseEnergy`, `SpatialVolume`, …) + `Materialized` | ECS (spawn por celda) | Puente grid → entidades; color vía `EnergyVisual` + almanac. |
| **Héroe** | Mismo stack de capas + `PlayerControlled` + `WillActuator` | ECS | Input → `FlowVector` → integración en plano sim (XZ en full3d). |
| **Eco fronteras** | `BoundaryMarker` por celda, contextos por zona | `EcoBoundaryField` | `ContextLookup::context_at` para presión/viscosidad/reactividad; **Void** fuera del grid o en banda de margen (ver §3). |

Los **elementos** en el plano (Terra / Ignis / …) aparecen como **frecuencia dominante + almanac** en cada celda del grid y en las firmas de entidades materializadas; no hay una segunda grilla “de elementos” aparte del campo.

## 2. Ríos (terreno, no textura aparte)

Los **ríos** son celdas con `TerrainType::Riverbed`: alto acumulado de flujo hídrico simulado + umbrales en `TerrainConfig.classification.river_accumulation`. Se generan con el pipeline en `topology/generators` (ruido → erosión → drenaje → `classify_all`).

- Config global por defecto: `assets/terrain_config.ron`.
- Demo con más relieve/cauces: `RESONANCE_TERRAIN_CONFIG=terrain_presets/river_plateau_demo.ron` (ver comentarios en ese RON).
- Mapa de escena: `assets/maps/demo_river_plateau.ron` + `RESONANCE_MAP=demo_river_plateau`.

## 3. Bordes lógicos y “vacío”

- **Fuera del rectángulo del grid:** `context_at_inner` devuelve `void_context_response()` (presión/viscosidad/reactividad en cero, `ZoneClass::Void`) — ya existía.
- **Banda interior opcional:** `playfield_margin_cells` en el mapa RON crea un **anillo de celdas** en el borde del grid tratadas igual que vacío para contexto y con el **clamp del jugador** reducido a la zona interior (misma semántica que “no gameplay” en el borde).

**Nota — chunk 16×16:** el proyecto usa chunks de **16×16 celdas** para dirty flags de materialización (`FIELD_GRID_CHUNK_SIZE`), no como frontera de contexto. El margen lógico actual es **por celda en el borde del mundo**, no automáticamente en cada frontera de chunk interno.

## 4. Referencias

- Flujo demo general: [DEMO_FLOW.md](./DEMO_FLOW.md)
- Eco: `src/eco/context_lookup.rs` (`void_context_response`, `EcoPlayfieldMargin`)
- Mapas: `src/worldgen/map_config.rs` (`playfield_margin_cells`)
- Terreno: `src/worldgen/systems/terrain.rs`, `src/topology/generators/classifier.rs`
