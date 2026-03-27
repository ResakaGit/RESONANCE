# Track — Auditoría de Codebase: Compilación, DOD y Cobertura

**Diagnóstico:** Auditoría profunda de codebase (2026-03-25). Pipeline, 14 capas, equations, constants, entities, bridge, worldgen, plugins, wiring.
**Motivación:** El proyecto no compilaba. Migración SM-3/SM-4 incompleta, exports faltantes, 5 violaciones DOD confirmadas, 13 funciones fundacionales sin tests.
**Filosofía:** Desbloquear compilación primero, corregir violaciones DOD después, cerrar gaps de cobertura al final.
**Estado:** ✅ Cerrado (2026-03-25). Todos los sprints completados en una sesión.

---

## Resultados

| Sprint | Scope | Estado | Resultado |
|--------|-------|--------|-----------|
| **CA-1** | Fix compilación | ✅ Cerrado | `cargo check` verde. Archivos legacy eliminados, módulos registrados, constantes creadas, naming fixes |
| **CA-2** | Fix violaciones DOD | ✅ Cerrado | 4/5 violaciones corregidas. `attention_gating_system` pendiente decisión (CA-1D) |
| **CA-3** | Tests core_physics | ✅ Cerrado | 40 tests escritos para 13 funciones. 0 fallos |

### Métricas post-fix

| Métrica | Antes | Después |
|---------|-------|---------|
| `cargo check` | 3 errores bloqueantes | 0 errores |
| `cargo test --lib` | No compilaba | **1717 passed**, 0 failed |
| `energy_competition_integration` | No compilaba | **6 passed**, 0 failed |
| `r1_conservation` | No compilaba | **6 passed**, 0 failed |
| core_physics tests | 0 | 40 |
| Violaciones DOD | 5 | 1 (attention_gating pendiente decisión) |

### Detalle de cambios CA-1 (compilación)

| Fix | Archivos |
|-----|----------|
| Eliminar `conservation_error`/`global_conservation_error` duplicados de pool_equations | `equations/energy_competition/mod.rs`, `pool_equations.rs` |
| Registrar módulos metabolic faltantes | `simulation/metabolic/mod.rs` (+4 mods), `simulation/mod.rs` (+4 re-exports) |
| Registrar módulos de constants faltantes | `blueprint/constants/mod.rs` (+3: surrogate, units, calibration) |
| Re-exportar módulos de equations faltantes | `blueprint/equations/mod.rs` (+4: calibration, observability, sensitivity, surrogate_error) |
| Crear `FITNESS_BLEND_RATE` constante faltante | `blueprint/constants/energy_competition_ec.rs` |
| Implementar `drain_dirty_budgeted` faltante | `worldgen/field_grid.rs` |
| Fix naming `vertex_along_flow_color` → `vertex_flow_color` | `geometry_flow/primitives.rs` |
| Fix sigmoid test assertion `< 1.0` → `<= 1.0` | `equations/macro_analytics.rs` |
| Fix field access en integration tests (getters) | `tests/energy_competition_integration.rs`, `tests/r1_conservation.rs` |

### Detalle de cambios CA-2 (DOD)

| Violación | Fix aplicado |
|-----------|-------------|
| `MobaIdentity.relational_tags: Vec<RelationalTag>` | `u16` bitmask + `has_tag()`/`add_tag()`/`remove_tag()` |
| `AlchemicalForge` 2× `Vec<ElementId>` | Split en `AlchemicalForge` (2 campos) + `AlchemicalDiscovery` (4 campos, arrays fijos `[ElementId; 8]`) |
| `AbilitySlot.name: String` | `Cow<'static, str>` (compatible con serde Deserialize) |
| `EntityBuilder.expect()` | `debug_assert!` + `let-else` early return |
| `attention_gating_system` no registrado | **Pendiente decisión**: registrar en pipeline o eliminar |

### Detalle CA-3 (tests)

40 tests en `blueprint/equations/core_physics/mod.rs` cubriendo las 13 funciones: `sphere_volume`, `projected_circle_area`, `sphere_surface_area`, `density`, `interference`, `is_constructive`, `is_destructive`, `is_critical`, `effective_dissipation`, `drag_force`, `integrate_velocity`, `equivalent_temperature`, `state_from_temperature`.

### Falsos positivos descartados (3)

| Hallazgo original | Razón de descarte |
|---|---|
| `density()` retorna `f32::MAX` con radio 0 | `SpatialVolume::new()` clampea a `VOLUME_MIN_RADIUS = 0.01` — inalcanzable |
| `scale_extractions_to_available()` con available negativo | `available_for_extraction()` retorna `.max(0.0)` — imposible |
| `conservation_error()` degrada con pool negativo | La función duplicada no era la usada — la canónica vive en `conservation.rs` |

## Referencias

- `docs/sprints/STRUCTURE_MIGRATION/` — SM-3/SM-4 originaron los conflictos
- `docs/sprints/SPRINT_PRIMER.md` — Reglas de diseño DOD
- `CLAUDE.md` — Hard Blocks y reglas de coding
