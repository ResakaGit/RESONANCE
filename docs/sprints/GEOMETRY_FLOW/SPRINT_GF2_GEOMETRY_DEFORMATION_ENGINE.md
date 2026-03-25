# Sprint GF2 — Motor de Deformación Termodinámica por Tensores

**Módulo (implementación):** `src/geometry_flow/deformation/`
**Tipo:** Motor stateless — tensores de energía → deformación de spines y meshes.
**Onda:** B — Depende de GF1 (spine base), Sprint 08 (EnergyVisual), Almanac.
**Estado:** ⏳ Pendiente

## Objetivo

Implementar la deformación procedural de geometría de entidades basada en tensores físicos:
- `T_energia` (fototropismo: la entidad se orienta hacia su fuente de energía dominante)
- `T_gravedad` (gravitropismo: la energía acumulada ES la masa; más energía → más curvatura)
- `T_resistencia` (bond_energy del material: resiste la deformación)

Una entidad existe en un plano vacío. Son los valores inyectados del contexto los que definen su
tendencia de forma. Una rosa se define solo por sus parámetros de Almanac y los tensores de entrada.

---

## Sub-sprint GF2A: Tensores Puras (CPU, funciones puras)

- Módulo `geometry_flow/deformation/tensors.rs`.
- `deformation_delta(tangent, t_energy, t_gravity, bond_energy) -> Vec3` — branchless.
- `calculate_tropism_vector(absorbed_energy, bond_energy, energy_direction, gravity_scale) -> (Vec3, Vec3)`.
- Estas funciones son el núcleo de toda la física. Sin ECS, sin estado.

## Sub-sprint GF2B: Deformación del Spine (CPU, funciones puras)

- Módulo `geometry_flow/deformation/deformation.rs`.
- `deform_spine(payload: &DeformationPayload) -> Vec<SpineNode>`.
  - Itera segmento a segmento del spine (GF1).
  - En cada segmento: calcula `deformation_delta` y acumula el giro angularmente (curvatura parabólica).
  - Los segmentos de la BASE del spine tienen `weight = 0.0` (anclados), los del extremo tienen `weight = 1.0`.
- `apply_spine_to_mesh(base_mesh: &Mesh, deformed_spine: &[SpineNode]) -> Vec<[f32; 3]>`.
- `deformation_fingerprint(payload: &DeformationPayload) -> u64`.

## Sub-sprint GF2C: Caché por Rangos de Oscilación (CPU, Resource)

- Módulo `geometry_flow/deformation/cache.rs`.
- `GeometryDeformationCache` (Resource) — parallel array al grid (igual que `MaterializationCellCache`).
- **Cache hit:** tensor actual ∈ `[tensor_min, tensor_max]` → reutilizar `deformed_spine`.
- **Cache miss:** recalcular + slide de rango → `tensor_min/max` migran hacia el nuevo equilibrio.
- **Degradación:** `range_width_factor *= 0.9999` por frame. Material envejece → rangos más estrechos.
- **Miss empuja al extremo opuesto:** si el tensor supera `tensor_max`, el rango sube. El miss es
  una señal de evolución del estado material, no un error.

## Sub-sprint GF2D: Sistema ECS (CPU, Update)

- Sistema `geometry_deformation_system` en `Update`, después de `visual_derivation_system`.
- Query: `Materialized + BaseEnergy + OscillatorySignature + Handle<Mesh>`.
- Para cada entidad:
  1. Leer `energy_direction` de `EnergyFieldGrid` (celda de la entidad).
  2. Leer `bond_energy` del `AlchemicalAlmanac` vía `WorldArchetype`.
  3. Construir `DeformationPayload`.
  4. Consultar `GeometryDeformationCache` → HIT o MISS.
  5. Si MISS: `deform_spine` + `apply_spine_to_mesh` + actualizar `Assets<Mesh>`.
  6. Actualizar rango en caché.

---

## Tácticas

- **El peso del vértice lo define el mesh base (Vertex Paint, o índice normalizado del spine).**
  No mandamos geometría nueva; deformamos posiciones existentes. El modelador/generador (GF1) define
  qué vértices son base (peso 0) y cuáles son punta (peso 1).
- **Curvatura parabólica, no lineal.** El `deformation_delta` se acumula con `weight * weight`
  (cuadrático), no lineal. Produce curvatura orgánica natural.
- **Sin gravedad = rama crece como el padre.** Si `gravity_scale = 0.0`, el tensor de gravedad
  desaparece. La rama lateral que nace horizontal se mantendrá horizontal (sin masa que la curve).
- **Caché dimensionada igual que EnergyFieldGrid.** `sync_deformation_cache_len_system` mantiene
  el array paralelo al grid. Lookup O(1) por índice de celda.
- **La frecuencia de onda dicta el ancho del rango inicial.** Alta frecuencia (`oscillation_hz` alto)
  → el material oscila rápido → rangos más anchos inicialmente. Baja frecuencia → rangos estrechos
  (árbol lento y estable).
- **Diagnóstico de ratio hit/miss.** En estado estacionario, `hits / (hits + misses) > 0.90` es el
  objetivo. Si cae por debajo, los rangos son demasiado estrechos → aumentar `range_width_factor` base.

## NO hace

- No genera el spine base (eso es GF1).
- No colorea el mesh (eso es Sprint 14).
- No genera nueva topología de vértices (no puede hacer crecer ramas que no existían en el mesh base).
- No modifica `EnergyFieldGrid` ni el Almanac.
- No duplica `MaterializationCellCache` (esa cachea arquetipo de celda, esta cachea postura de entidad).
- No inventa bandas LOD propias (usa `oscillation_hz` por entidad, no las bandas Near/Mid/Far del grid).

## Demarcación con Sprint 13 y sistemas existentes

| Sistema existente | Sprint GF2 | Relación |
|-------------------|------------|----------|
| `MaterializationCellCache` | `GeometryDeformationCache` | Ortogonales: forma de celda ≠ postura de entidad |
| LOD Near/Mid/Far | `oscillation_hz` de entidad | LOD del grid ≠ frecuencia de deformación |
| `GF1 build_flow_spine` | `deform_spine(base_spine)` | GF2 es etapa 2 del spine: deformación post-generación |
| Sprint 14 (Color Cuantizado) | Visualiza el mesh deformado | GF2 deforma. Sprint 14 pinta. Mismo mesh, dos pasadas |

## Dependencias

- `GF1: build_flow_spine, SpineNode` — spine base no deformado.
- `Sprint 08: EnergyVisual` — coexistencia (Sprint GF2 no reemplaza EnergyVisual en 2D).
- `Sprint 13: WorldgenPerfSettings, WorldgenLodContext` — las entidades dentro del cull_distance son las que reciben deformación.
- `AlchemicalAlmanac: bond_energy` por `WorldArchetype` — resistencia del material.
- `EnergyFieldGrid` — dirección e intensidad del campo en la celda de la entidad.
- `bevy::asset::Assets<Mesh>` — para mutar las posiciones de vértices del mesh en runtime.

---

## Criterios de aceptación

### GF2A (Tensores)
- Test: `deformation_delta` con `bond_energy` muy alto → delta ≈ Vec3::ZERO.
- Test: `deformation_delta` con `bond_energy` muy bajo → delta ≈ `t_energy + t_gravity` (normalizado).
- Test: sin `t_energy` y sin gravedad → delta = Vec3::ZERO (sin fuerza → sin cambio).
- Test: `calculate_tropism_vector` es branchless — sin `if` en implementación.

### GF2B (Spine)
- Test: `deform_spine` con `gravity_scale = 0.0` y `energy_direction = tangente_actual` → spine sin cambio.
- Test: `deform_spine` con `gravity_scale = 1.0` → los segmentos del tope se curvan hacia abajo.
- Test: `deformed_spine.len() == base_spine.len()` siempre.
- Test: mismo payload → mismo spine (determinismo).

### GF2C (Caché)
- Test: segundo llamado con mismo payload → CACHE HIT (no recalcula).
- Test: payload con tensor fuera del rango → CACHE MISS → rango deslizado al nuevo centro.
- Test: después de N frames, `range_width_factor` decrece (degradación de material).
- Test: hit_ratio > 0.90 con 100 frames de tensor estable.

### GF2D (Sistema ECS)
- Test integración: entidad spawn con `BaseEnergy` alta en celda con `Lux` apuntando al norte →
  mesh resultante tiene vértices superiores desplazados hacia el norte.
- Test: `cargo test` pasa sin romper tests de GF1, Sprint 14 ni Sprint 08.

## Referencias

- `docs/design/GEOMETRY_DEFORMATION_ENGINE.md`
- `docs/arquitectura/blueprint_geometry_deformation.md`
- `docs/sprints/GEOMETRY_FLOW/README.md` (GF1 cerrado en código)
- `docs/sprints/BLUEPRINT_V7/SPRINT_14_V7_QUANTIZED_COLOR_ENGINE.md`
- `docs/design/TOPOLOGY.md`
