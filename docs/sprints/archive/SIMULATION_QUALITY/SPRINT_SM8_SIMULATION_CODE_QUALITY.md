# Sprint SM-8 — Refactorizaciones de Calidad en simulation/

**Módulo:** `src/simulation/` (todas las subcarpetas)
**Tipo:** Refactor de code smells, anti-patterns y violaciones SOLID. Zero cambio de comportamiento.
**Onda:** Post SM-2 (estructura ya migrada).
**Estado:** ⏳ Pendiente

## Diagnóstico

Auditoría exhaustiva de 30 archivos en `simulation/`. El módulo `metabolic/` es **ejemplar** (score 9.7/10). Los problemas se concentran en `thermodynamic/`, `reactions.rs`, `input.rs` y `pipeline.rs`.

### Scorecard por subdirectorio

| Subdirectorio | Score | Problema principal |
|--------------|-------|-------------------|
| `metabolic/` | 9.7/10 | Ninguno crítico. Math en equations, guards en todo, 24 tests. |
| `lifecycle/` | 8.5/10 | 2 queries con 7 componentes, inline math menor. |
| `thermodynamic/` | 7/10 | 2 god-systems, magic numbers, SRP violations. |
| Root (raíz) | 6.5/10 | 3 god-systems, inline math, pipeline monolítico, duplicación. |

---

## Hallazgos por categoría

### 1. God-Systems (query >5 component types)

| Sistema | Archivo | Componentes | Líneas |
|---------|---------|-------------|--------|
| `contained_thermal_transfer_system` | `thermodynamic/containment.rs:122` | **13** (6+7 en 2 queries) | 95 |
| `movement_will_drag_system` | `thermodynamic/pre_physics.rs:173` | **9** | 95 |
| `state_transitions_system` | `reactions.rs` | **6** | ~50 |
| `grimoire_cast_intent_system` | `input.rs` | **6** + Window + Camera | ~40 |
| `ability_point_target_pick_system` | `ability_targeting.rs` | **6** + Window + Camera | ~50 |
| `growth_intent_inference_system` | `lifecycle/inference_growth.rs:10` | **7** | 61 |
| `lifecycle_stage_inference_system` | `lifecycle/organ_lifecycle.rs:81` | **7** | 36 |

**Regla violada:** "One system, one transformation" + max 4-5 component types.

### 2. Inline Math (debe ir en equations/)

| Archivo | Línea | Fórmula | Propuesta |
|---------|-------|---------|-----------|
| `grimoire_enqueue.rs` | 120 | `0.5 + 0.02 * spawn_radius` | `equations::projectile_dissipation(radius)` |
| `reactions.rs` | 237 | `eb * w` (weakening) | `equations::bond_weakening(eb, factor)` |
| `lifecycle/inference_growth.rs` | 41 | `base_radius * MAX_FACTOR` | `equations::allometric_max_radius(r, factor)` |
| `lifecycle/inference_growth.rs` | 44 | `qe / REF.max(1.0)` | `equations::normalized_qe(qe, ref)` |
| `lifecycle/allometric_growth.rs` | 52 | `base_radius * MAX_FACTOR` | Misma que arriba (duplicada) |

**Regla violada:** "Math in blueprint/equations. Systems call pure functions, don't inline formulas."

### 3. Magic Numbers (no en constants)

| Archivo | Línea | Valor | Propuesta |
|---------|-------|-------|-----------|
| `thermodynamic/structural_runtime.rs` | 49 | `thermal_load = 0.0` | `STRUCTURAL_DEFAULT_THERMAL_LOAD` o computar |
| `thermodynamic/structural_runtime.rs` | 168 | `0.1` (tension accel) | `TENSION_FIELD_STRENGTH_SCALE` |
| `thermodynamic/sensory.rs` | 397 | `0.5` (entropy drain) | `ATTENTION_ENTROPY_DRAIN_RATE` |
| `bootstrap.rs` | 63-65 | `64, 64, Vec2(-64,-64)` | `DEFAULT_GRID_DIMS`, `DEFAULT_GRID_ORIGIN` |
| `grimoire_enqueue.rs` | 86 | `0.05` (spawn offset) | `PROJECTILE_SPAWN_OFFSET` |
| `ability_targeting.rs` | 86 | `0.05` (spawn offset) | Misma — **duplicada** |
| `grimoire_enqueue.rs` | 27,35,47 | `1e-6` | `DIRECTION_NORMALIZE_EPS` |
| `lifecycle/inference_growth.rs` | 62-63 | `1e-4` (×2) | `GROWTH_INTENT_WRITE_EPS` |
| `lifecycle/evolution_surrogate.rs` | 298 | `0.5` (competition) | `EVOLUTION_DEFAULT_COMPETITION` |
| `input.rs` | 78-83 | `KeyQ=0..KeyR=3` | Resource `KeySlotMapping` |

**Regla violada:** "Constants in constants. Tuning values centralized per module."

### 4. Missing Change Detection Guards

| Archivo | Línea | Mutación sin guard |
|---------|-------|--------------------|
| `thermodynamic/pre_physics.rs` | 283,355 | `transform.translation` |
| `ability_targeting.rs` | 121 | `targeting.active = None` |
| `input.rs` | 102 | `will.set_active_slot()` |
| `reactions.rs` | 237 | `matter.set_bond_energy_eb()` |

**Regla violada:** "Guard change detection. Check equality before mutation."

### 5. SRP Violations (sistema hace >1 cosa)

| Sistema | Archivo | Responsabilidades mezcladas |
|---------|---------|---------------------------|
| `contained_thermal_transfer_system` | `thermodynamic/containment.rs:117` | Overlap area + thermal transfer + drag + energy inject/drain (4 cosas) |
| `movement_will_drag_system` | `thermodynamic/pre_physics.rs:173` | Movement + velocity clamping + drag + position update (4 cosas) |
| `grimoire_cast_intent_system` | `input.rs` | Slot selection + targeting + channeling setup (3 cosas) |
| `catalysis_spatial_filter_system` | `reactions.rs` | Spatial query + sorting + filtering + effect dispatch (4 cosas) |

### 6. Large Functions (>50 LOC)

| Función | Archivo | LOC | Problema |
|---------|---------|-----|---------|
| `register_simulation_pipeline` | `pipeline.rs:36` | **240** | Monolito de registration |
| `movement_will_drag_system` | `thermodynamic/pre_physics.rs` | 95 | God-system |
| `contained_thermal_transfer_system` | `thermodynamic/containment.rs` | 95 | God-system |
| `init_simulation_bootstrap` | `bootstrap.rs` | 74 | Sequential init dump |
| `enqueue_grimoire_cast_intent` | `grimoire_enqueue.rs` | 70 | 9 parámetros |
| `catalysis_math_strategy_system` | `reactions.rs` | 62 | Nested match |
| `growth_intent_inference_system` | `lifecycle/inference_growth.rs` | 61 | Query compleja |

### 7. Duplicated Patterns

| Patrón | Ubicación | Propuesta |
|--------|-----------|-----------|
| `in_state(Playing).and(in_state(Active))` | `pipeline.rs` × 10+ | Extraer a `let run_gameplay = ...` (ya existe línea 54, pero no se reusa en todos los `.add_systems`) |
| Grid init `64, 64, cell_size, Vec2(-64,-64)` | `bootstrap.rs` + `pipeline.rs` | Const o Resource `DefaultGridConfig` |
| Spawn offset `0.05` | `ability_targeting.rs` + `grimoire_enqueue.rs` | Const `PROJECTILE_SPAWN_OFFSET` |
| `base_radius * ALLOMETRIC_MAX_RADIUS_FACTOR` | `inference_growth.rs` + `allometric_growth.rs` | `equations::allometric_max_radius()` |
| Overlay application (3 funciones ~70 LOC) | `thermodynamic/pre_physics.rs` | Función genérica o macro |

---

## Plan de refactorizaciones (por prioridad ROI)

### SM-8A: Extraer magic numbers a constants (Alto ROI, Bajo costo)

**Costo:** ~30 min. **Impacto:** Elimina 12+ magic numbers, mejora tuneabilidad.

1. Crear/extender `blueprint/constants/` con:
   ```rust
   // simulation_defaults.rs
   pub const STRUCTURAL_DEFAULT_THERMAL_LOAD: f32 = 0.0;
   pub const TENSION_FIELD_STRENGTH_SCALE: f32 = 0.1;
   pub const ATTENTION_ENTROPY_DRAIN_RATE: f32 = 0.5;
   pub const PROJECTILE_SPAWN_OFFSET: f32 = 0.05;
   pub const DIRECTION_NORMALIZE_EPS: f32 = 1e-6;
   pub const GROWTH_INTENT_WRITE_EPS: f32 = 1e-4;
   pub const EVOLUTION_DEFAULT_COMPETITION: f32 = 0.5;
   pub const DEFAULT_GRID_DIMS: u32 = 64;
   pub const DEFAULT_GRID_ORIGIN: Vec2 = Vec2::new(-64.0, -64.0);
   ```
2. Reemplazar literales por constantes en los 8 archivos afectados.
3. `cargo test --lib` verde.

### SM-8B: Añadir change detection guards faltantes (Alto ROI, Bajo costo)

**Costo:** ~15 min. **Impacto:** Reduce mutaciones espurias, mejora performance de Bevy change tracking.

1. `thermodynamic/pre_physics.rs:283,355` — guardar translation anterior, comparar antes de mutar.
2. `ability_targeting.rs:121` — `if targeting.active.is_some() { targeting.active = None; }`.
3. `input.rs:102` — `if will.active_slot() != slot { will.set_active_slot(slot); }`.
4. `reactions.rs:237` — `if matter.bond_energy_eb() != new_eb { matter.set_bond_energy_eb(new_eb); }`.

### SM-8C: Extraer inline math a equations/ (Alto ROI, Bajo costo)

**Costo:** ~20 min. **Impacto:** Centraliza 5 fórmulas, elimina duplicación, habilita testing aislado.

1. Crear en `blueprint/equations/`:
   ```rust
   pub fn allometric_max_radius(base_radius: f32, max_factor: f32, min_radius: f32) -> f32
   pub fn normalized_qe(qe: f32, reference: f32) -> f32
   pub fn projectile_dissipation(spawn_radius: f32) -> f32
   pub fn bond_weakening(bond_energy: f32, weakening_factor: f32) -> f32
   ```
2. Reemplazar inline math en `inference_growth.rs`, `allometric_growth.rs`, `grimoire_enqueue.rs`, `reactions.rs`.
3. Tests unitarios para cada función nueva.

### SM-8D: Split god-systems en thermodynamic/ (Medio ROI, Medio costo)

**Costo:** ~1h. **Impacto:** Desacopla 2 sistemas de 95 LOC con 9-13 componentes.

#### D1: Split `contained_thermal_transfer_system` (containment.rs:117, 13 componentes)

Partir en 3 sistemas encadenados:
```rust
// 1. Calcula overlap y decide contacto
pub fn containment_overlap_system(
    query: Query<(&SpatialVolume, &Transform, &Contained)>,
    hosts: Query<(&SpatialVolume, &Transform)>,
) → escribe ContainmentOverlap (componente SparseSet nuevo, o Local)

// 2. Aplica transferencia térmica
pub fn containment_thermal_system(
    query: Query<(&ContainmentOverlap, &mut BaseEnergy, &AmbientPressure)>,
    hosts: Query<&AmbientPressure>,
)

// 3. Aplica drag por inmersión
pub fn containment_drag_system(
    query: Query<(&ContainmentOverlap, &mut FlowVector, &AmbientPressure)>,
)
```
Encadenar: `containment_overlap_system → containment_thermal_system → containment_drag_system`.

#### D2: Split `movement_will_drag_system` (pre_physics.rs:173, 9 componentes)

Partir en 2 sistemas:
```rust
// 1. Aplica voluntad → velocidad (lee WillActuator, escribe FlowVector)
pub fn will_to_velocity_system(
    query: Query<(&WillActuator, &mut FlowVector, &SpatialVolume)>,
)

// 2. Aplica drag + clamp + posición (lee FlowVector, escribe Transform)
pub fn velocity_integration_system(
    query: Query<(&FlowVector, &AmbientPressure, &mut Transform)>,
)
```

### SM-8E: Split pipeline registration (Medio ROI, Bajo costo)

**Costo:** ~30 min. **Impacto:** `register_simulation_pipeline` de 240 LOC → 5 funciones de ~50 LOC.

```rust
pub fn register_simulation_pipeline<S>(app: &mut App, schedule: S) {
    init_simulation_resources(app);
    register_clock_and_input(app, schedule.clone());
    register_thermodynamic_phase(app, schedule.clone());
    register_metabolic_phase(app, schedule.clone());
    register_morphological_phase(app, schedule);
}
```

Extraer también: `let run_gameplay = in_state(GameState::Playing).and(in_state(PlayState::Active));` y reusar en todas las fases (ya existe en línea 54 pero no se pasa a las sub-funciones).

### SM-8F: Reducir query complexity en lifecycle/ (Bajo ROI, Medio costo)

**Costo:** ~45 min. **Impacto:** 2 queries de 7 componentes → 4-5 max.

Opciones:
1. **Extraer helper puro:** La lógica de `infer_lifecycle_transition()` ya es función pura. Pero la query necesita todos los datos. Solución: crear un `LifecycleInput` struct intermedio que se construya en un paso previo, reduciendo el query del sistema principal.
2. **Aceptar excepciones documentadas:** 7 componentes donde 3 son `Option<>` para entidades complejas es borderline aceptable si el sistema es SRP. Documentar con `// NOTE: 7 component types justified — lifecycle reads full entity state`.

### SM-8G: Desacoplar input.rs god-system (Bajo ROI, Medio costo)

**Costo:** ~30 min. **Impacto:** SRP para grimoire input.

Split `grimoire_cast_intent_system` en:
```rust
// 1. Lee keyboard → escribe slot intent
pub fn grimoire_slot_selection_system(...)

// 2. Lee slot intent + spatial → escribe targeting
pub fn grimoire_targeting_system(...)

// 3. Lee targeting → inicia channeling
pub fn grimoire_channeling_start_system(...)
```

---

## Orden de ejecución

| Paso | Refactorización | Costo | ROI | Archivos tocados |
|------|----------------|-------|-----|-----------------|
| 1 | **SM-8A** Magic numbers → constants | 30 min | ★★★★★ | 8 archivos + 1 nuevo |
| 2 | **SM-8B** Change detection guards | 15 min | ★★★★★ | 4 archivos |
| 3 | **SM-8C** Inline math → equations | 20 min | ★★★★☆ | 4 archivos + equations |
| 4 | **SM-8E** Split pipeline registration | 30 min | ★★★★☆ | 1 archivo |
| 5 | **SM-8D** Split god-systems thermo | 1h | ★★★☆☆ | 2 archivos |
| 6 | **SM-8F** Lifecycle query docs | 15 min | ★★☆☆☆ | 2 archivos |
| 7 | **SM-8G** Input SRP split | 30 min | ★★☆☆☆ | 1 archivo |

**Total estimado:** ~3.5h para todo. SM-8A + SM-8B + SM-8C cubren el 80% del valor en ~1h.

---

## Lo que NO hay que tocar

- **`metabolic/`** — Ejemplar. No refactorizar lo que ya es excelente.
- **`lifecycle/competitive_exclusion.rs`** — Limpio, 3 componentes, math en equations.
- **`lifecycle/env_scenario.rs`** — Limpio, guards perfectos.
- **`thermodynamic/osmosis.rs`** — Bien estructurado.
- **`states.rs`, `time_compat.rs`, `player_controlled.rs`** — Mínimos y correctos.

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión tras cada sub-sprint.
- Zero magic numbers en sistemas (grep `\b\d+\.\d+\b` sin match en funciones pub).
- Todo sistema ≤5 component types en query (excepto lifecycle/ documentados).
- Todo `&mut Component` tiene change detection guard.
- Toda fórmula aritmética vive en `blueprint/equations/`.

## Referencias

- `CLAUDE.md` — Coding Rules §1-10, Hard Blocks §1-17
- `docs/sprints/CODE_QUALITY/SPRINT_Q2_MAGIC_NUMBERS.md` — Track relacionado
- `docs/sprints/CODE_QUALITY/SPRINT_Q3_PUB_FIELD_PROTECTION.md` — Track relacionado
- `src/simulation/metabolic/` — Referencia de calidad (score 9.7/10)
