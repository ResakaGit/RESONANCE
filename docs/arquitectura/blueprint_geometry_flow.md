# Blueprint: Geometry Flow (motor stateless)

Referencia de contrato para el track [`docs/sprints/GEOMETRY_FLOW/README.md`](../sprints/GEOMETRY_FLOW/README.md) y el cableado EPI3 en código (`worldgen::gf1_field_linear_rgb_qe_at_position`, `field_visual_sample`; track EPI cerrado — [`ENERGY_PARTS_INFERENCE/README.md`](../sprints/ENERGY_PARTS_INFERENCE/README.md)).  
Template base: [`00_contratos_glosario.md`](00_contratos_glosario.md).

## 1) Propósito y frontera

- **Qué resuelve:** generación **pura** de spine + malla 3D (flujo / flora visual) desde un DTO `GeometryInfluence`, con LOD `detail ∈ [0,1]` y color por vértice inferido sin texturas de estado.
- **Qué no resuelve:** simulación ECS, propagación V7, física, almacenamiento de assets de imagen, ni decisión de gameplay de planta.

## 2) Superficie pública (contrato)

- **Rust:** `crate::geometry_flow` — `GeometryInfluence`, `SpineNode`, `build_flow_spine`, `build_flow_mesh`, `vertex_along_flow_color`.
- **EPI3 (tinte por nodo/rama):** `build_flow_spine_painted` — el llamador inyecta `FnMut(Vec3, &GeometryInfluence) -> ([f32; 3], f32)` para RGB + `qe_norm` por punto (p. ej. vía `worldgen::gf1_field_linear_rgb_qe_at_position`); `GeometryInfluence` puede llevar `branch_role` para modular sin `match` por especie en ecuaciones.
- **Matemática:** `crate::blueprint::equations` — `flow_push_along_tangent`, `flow_maintain_straight_segment`, `flow_steered_tangent` (y constante opcional `FLOW_STEER_ON_BREAK` si aplica).
- **Eventos / resources:** ninguno **dentro** del núcleo; el inyector (otro módulo) puede usar `Res`/`Query`.

## 3) Invariantes y precondiciones

- `detail` se interpreta clamped a \([0,1]\) en la API pública.
- Direcciones de entrada finitas; degeneradas → fallback documentado (sin panic).
- `max_segments ≥ 4` (mínimo operativo del spine en GF1).

## 4) Comportamiento runtime

- **Fase:** fuera del motor — típicamente `Update` o tras snapshot de render.
- **Orden:** después de que exista el paquete inyectado; antes de render del `Mesh`.
- **Determinismo:** sin RNG en `geometry_flow` ni en las ecuaciones de flow spine citadas.
- **Side-effects:** solo los que imponga el llamador al insertar `Mesh` en `Assets<Mesh>`.

## 5) Implementación y trade-offs

- **DoD:** buffers planos para posiciones, normales, UV, color, índices.
- **Costo vs valor:** tubo simple alrededor del spine; ramificación L-system queda para sprints posteriores.
- **Límite:** triángulos acotados por `detail` y techos numéricos explícitos.

## 6) Fallas y observabilidad

- **NaN/Inf:** precondición del llamador; el núcleo puede usar `normalize_or_zero` y clamps.
- **LOD cero:** `detail = 0` aún debe producir geometría mínima válida (mínimo segmentos/anillos).

## 7) Checklist de atomicidad

- ¿Una responsabilidad principal? **Sí** — solo geometría desde DTO.
- ¿Acopla más de un dominio? **No** en el núcleo; el inyector acopla ECS/campo/almanaque.
