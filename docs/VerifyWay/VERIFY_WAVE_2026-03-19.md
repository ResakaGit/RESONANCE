# Verify Wave - 2026-03-19

## Resultado ejecutivo

- Estado global: `BLOCK` (4/5 auditores) y `WARN` (1/5).
- Accion recomendada: cerrar hallazgos criticos de determinismo, DoD y frontera hexagonal antes de avanzar sprint.
- Alcance auditado: `docs/design/V2.md` a `docs/design/V6.md` contra implementacion actual en `src/`.

## Tabla consolidada de hallazgos

| Auditor | Severidad | Hallazgo | Referencias (paths) | Estrategia de fix |
|---|---|---|---|---|
| V6 Core | Critical | Simulacion core puede correr en `Update` variable (no fixed-step por defecto) | `Cargo.toml`, `src/plugins/simulation_plugin.rs`, `src/v6/simulation_tick/mod.rs` | Forzar pipeline de simulacion en `FixedUpdate`; dejar `Update` para visual/debug. |
| V6 Core | Critical | Orden de colisiones legacy no estable por iteracion de `HashMap` | `src/v6/spatial_index_backend/mod.rs`, `src/world/space.rs`, `src/simulation/physics.rs` | Reemplazar por pares canonicos ordenados (`candidate_pairs`) o ordenar `(entity_a, entity_b)` antes de resolver. |
| V6 Core | High | Input de simulacion depende de camara mutada en `Update` | `src/v6/intent_projection_3d/mod.rs`, `src/v6/camera_controller_3d/mod.rs`, `src/plugins/simulation_plugin.rs` | Desacoplar base de proyeccion de estado visual; snapshot determinista en `FixedUpdate`. |
| Evolutivo V2-V4 | Critical | Contrato V3 incompleto: habilidades siguen acopladas a `frequency` en runtime | `docs/design/V3.md`, `src/layers/will.rs`, `src/simulation/input.rs` | Migrar output de habilidades a `element_id` E2E y resolver frecuencia via almanac. |
| Evolutivo V2-V4 | High | SSOT de interferencia roto: catálisis recalcula formula inline | `docs/design/V2.md`, `src/simulation/reactions.rs`, `src/layers/oscillatory.rs` | Centralizar formula en helper/ops unico y cubrir equivalencia con test de regresion. |
| Evolutivo V2-V4 | Medium | Gap de contenido V3: assets elementales incompletos vs objetivo blueprint | `docs/design/V3.md`, `assets/elements/*.ron` | Completar dataset y agregar test de cobertura de bandas/elementos. |
| ECS/DoD | Critical | Component bloat: `ResonanceOverlay` supera limite DoD (>4 fields) | `src/layers/link.rs` | Split en overlays ortogonales o mover parte de estado derivado a recursos/eventos frame-local. |
| ECS/DoD | High | `movement_system` con query ancha y responsabilidad mixta | `src/simulation/physics.rs` | Partir en sistemas: `will_to_force` y `integrate_kinematics` con queries minimas. |
| ECS/DoD | High | `catalysis_resolution_system` mezcla calculo, spawn y cleanup | `src/simulation/reactions.rs` | Separar en 2-3 systems encadenados en `Phase::Reactions` (una transformacion principal por system). |
| ECS/DoD | High | Mutacion de gameplay en fase Input (consumo/spawn) | `src/simulation/input.rs`, `src/plugins/simulation_plugin.rs` | Dejar Input como captura/intencion; mover consumo/spawn a fase de simulacion. |
| Determinismo | Critical | Catálisis usa `time.elapsed_secs()` (wall-clock) | `src/simulation/reactions.rs` | Usar tiempo discreto de simulacion (`tick_id` + `fixed_dt`/`Time<Fixed>`). |
| Determinismo | High | Capa 11 acumula en estructuras con orden no canonico | `src/simulation/structural_runtime.rs`, `src/world/space.rs` | Ordenar por `Entity::to_bits()` y acumular en estructura ordenada (`BTreeMap`/`Vec` sorted). |
| Determinismo | Medium | Tie-break no determinista de host dominante en containment | `src/simulation/containment.rs` | Definir desempate estable cuando `priority` empata (menor `Entity::to_bits()`). |
| Hex Boundary 2D/3D | High | `render_bridge_3d` lee capas de dominio live en vez de snapshot | `src/v6/render_bridge_3d/mod.rs` | Introducir `VisualStateSnapshot` post-simulacion y hacer bridge solo contra snapshot. |
| Hex Boundary 2D/3D | Medium | `intent_projection_3d` muta Capa 7 directamente | `src/v6/intent_projection_3d/mod.rs` | Separar proyeccion (adapter output) de aplicacion a dominio (system de simulacion). |
| Hex Boundary 2D/3D | Low | Acople leve de adapter en plugin de simulacion | `src/plugins/simulation_plugin.rs` | Mover wiring de adapters V6 al composition root/plugin de integracion. |

## Matriz de estrategia de remediacion

| Prioridad | Objetivo | Cambios minimos recomendados | Resultado esperado |
|---|---|---|---|
| P0 | Blindar determinismo del core | `FixedUpdate` obligatorio + orden canonic de colisiones + tiempo discreto en reacciones | Reproducibilidad entre corridas/maquinas. |
| P0 | Cumplir DoD/ECS en hot path | Split de `ResonanceOverlay`, `movement_system`, `catalysis_resolution_system` | Menor acople, queries mas chicas, menor riesgo de regresion. |
| P1 | Cerrar frontera hexagonal V6 | Snapshot de salida para render + separar projection/apply en intención 3D | Adapter reemplazable, dominio aislado. |
| P1 | Cerrar deuda evolutiva V2/V3 | Migracion E2E a `ElementId` + SSOT unico de interferencia | Consistencia matematica y semantica de habilidades. |
| P2 | Evidencia de calidad | Agregar tests de determinismo, invariantes DoD y contratos de fase | Gate de CI alineado a blueprint. |

## Suite minima de tests sugerida

- `determinism_replay_same_seed_same_state`: hash de estado por tick N=30.
- `collision_order_invariant`: mismo resultado con distinto orden de insercion.
- `reactions_use_sim_clock`: prohibir dependencia de `elapsed_secs` en logica core.
- `input_phase_no_gameplay_mutation`: Input no drena engine ni spawnea entidades.
- `component_field_budget_guard`: falla si un `Component` supera 4 fields.
- `hex_boundary_render_snapshot_only`: bridge visual no lee capas core directo.

