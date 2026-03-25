# Sprint Q8 — Aislamiento Estricto de Geometría y Color (Refactor DoD)

**Módulos:** `src/worldgen/materialization_rules.rs`, `src/worldgen/systems/visual.rs`, `src/geometry_flow/`, `src/topology/terrain_field.rs`
**Tipo:** Refactorización de Arquitectura (Modelo Yanagi / Stateless)
**Onda:** C (Continuación de Q7, para alinea VRAM/CPU caché)
**Severidad:** ALTA (Coherencia arquitectónica de rendering)

---

## Objetivo
Traducir a código limpio las correcciones en `TERRAIN_MESHER.md` y el contrato V7 de materialización/visual (`docs/design/V7.md`, `visual_derivation` / `systems/visual`). Consiste extirpar del código cualquier acoplamiento cromático dentro de los motores de generación de polígonos/mallas. El motor geométrico debe ser 100% `Stateless` y `Agnóstico de Color`. Todo color de las "cosas" o terreno se debe aplicar indirectamente desde el ECS hacia `MeshMaterial3d` o búferes SoA visuales, preservando el determinismo.

## Tareas de código

### 1. Refactor Topológico (Terrain Mesher)
- [ ] Auditar/Refactorizar la función que procesa la grilla del suelo. Garantizar que la `Altitud (Y)` provenga **exclusiva y estrictamente** del `TerrainField`. 
- [ ] Eliminar del `terrain_mesher` toda inyección de "energía = elevación" impuesta por versiones antiguas. Si hay código que multiplica `Energy` para elevar Y, purgarlo.
- [ ] Implementar la estructura `TerrainVisuals` para suministrar de forma asíncrona los *vertex colors*, cruzándolos con V7 antes de montar el `Mesh` final en Bevy.

### 2. Purificar rules de materialización (V7 / `materialization_rules`)
- [ ] En `src/worldgen/materialization_rules.rs`, revisar la traza de `materialize_cell`.
- [ ] Extraer por completo la resolución/cálculo de `Color / Tint` si reside dentro de la fase que determina la *Forma* (`WorldArchetype`). La materialización dicta "*qué es*", no "*de qué color se pinta*".

### 3. Sistema visual y color (V7 / `visual.rs` + `visual_derivation`)
- [ ] En `src/worldgen/systems/visual.rs` y `src/worldgen/visual_derivation.rs`, validar que `EnergyVisual` es el canal de transporte único.
- [ ] Validar que la sincronía visual a Bevy (el sistema que hace `.insert(Sprite { color })` o similar) sea totalmente ortogonal. 

### 4. Geometry Flow (`flower_demo`)
- [ ] Verificar brevemente `build_flow_mesh` en `src/geometry_flow/`. Validar que `vertex_along_flow_color` funciona de manera puramente funcional con la inyección desde `GeometryInfluence`.

## Criterios de Aceptación
- Un `grep` de llamadas que modifiquen Y/Altitud basada en lógica V7 dentro de generadores de mallas base devuelve 0 matches. Mismas garantías para color dentro de lógicas `Solid/Liquid/Gas` del arquetipo.
- El proyecto compila. `cargo run --bin resonance --features="demo"` o equivalente y la demo `RESONANCE_MAP=flower_demo cargo run` ejecutan establemente.
- Los tests en los pipelines pasaron (`cargo test` completo).
