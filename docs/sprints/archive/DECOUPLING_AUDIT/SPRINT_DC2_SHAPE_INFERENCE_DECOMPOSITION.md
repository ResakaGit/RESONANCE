# DC-2: Entity Shape Inference Decomposition — 15→3×5 Componentes

**Objetivo:** Descomponer `entity_shape_inference_system` (15 componentes, god-system) en 3 sistemas focalizados que respetan la regla de max 5 componentes por query, orquestados por un componente intermedio sin perder la semántica de cache.

**Estado:** PENDIENTE
**Esfuerzo:** M (~4 archivos, ~300 LOC refactor)
**Bloqueado por:** DC-1 ✅
**Desbloquea:** Mantenibilidad de lifecycle/, extensibilidad de morphogenesis pipeline

---

## Hallazgo clave del análisis

`entity_shape_inference_system` corre en **Update** (no FixedUpdate), registrado en `worldgen_plugin.rs:208-215` con `.after(sync_visual_from_sim_system)`. Esto es correcto: la visual sync necesita que V6VisualRoot exista (creado por sync_visual), y los meshes son rendering concerns que van en Update.

`constructal_body_plan_system` corre en **FixedUpdate / Phase::MorphologicalLayer** (morphological_plugin.rs:193). Produce `BodyPlanLayout` que entity_shape_inference lee. Cross-schedule data flow vía componente — ya funciona.

---

## Anatomía exacta del sistema actual (lines 44-196)

```
entity_shape_inference_system(
    commands, meshes,
    query: 15 componentes + With<HasInferredShape>,
    visual_q: (&mut Mesh3d, &mut Transform),
    profile_q: &InferenceProfile,
)

Flow:
  1. [L82-100]  Cache check: signature(fineness, qe_norm, radius, hunger, food_dist, hostile, rugosity, albedo)
                → if ShapeInferred + StableWindow + sig match → SKIP (cache hit)
  2. [L102-135] Sensory modulation: hunger→fineness, hostile→resistance, rugosity→detail, albedo→tint
                → entity_geometry_influence() → GeometryInfluence
  3. [L137-138] Torso mesh: build_flow_spine() → build_flow_mesh()
  4. [L141-177] Organ loop: for each BodyPlanLayout slot → organ_slot_scale() → organ influence → spine → mesh
                → merge_meshes()
  5. [L181-184] Visual sync: mesh_handle → visual_q.get_mut(V6VisualRoot.visual_entity)
  6. [L187-194] Cache writeback: update dependency_signature, insert ShapeInferred
```

---

## Diseño refinado: 3 Sistemas + 1 SparseSet intermedio

### Constraint: entity_shape_inference corre en Update, NO en FixedUpdate

Los 3 nuevos sistemas también corren en Update, encadenados, `.after(sync_visual_from_sim_system)`.

### Sistema 1: `shape_cache_gate_system` (decide rebuild)

**Responsabilidad:** Lee estado mínimo, computa signature, inserta marker `ShapeRebuildNeeded` solo si cache miss.

```rust
/// Gate: inserta ShapeRebuildNeeded si el cache signature cambió.
pub fn shape_cache_gate_system(
    mut commands: Commands,
    query: Query<(
        Entity,                              // 1
        &BaseEnergy,                         // 2
        &SpatialVolume,                      // 3
        Option<&MorphogenesisShapeParams>,   // 4
        Option<&PerformanceCachePolicy>,     // 5
    ), (With<HasInferredShape>, Without<ShapeRebuildNeeded>)>,
    sensory_q: Query<(
        Option<&EnergyAssessment>,
        Option<&SensoryAwareness>,
        Option<&MorphogenesisSurface>,
        Option<&InferredAlbedo>,
    )>,
    inferred_q: Query<&ShapeInferred>,
) {
    for (entity, energy, volume, shape_opt, policy_opt) in &query {
        let fineness = shape_opt.map(|s| s.fineness_ratio()).unwrap_or(FINENESS_DEFAULT);
        let qe_norm = normalized_qe(energy.qe(), VISUAL_QE_REFERENCE);
        let (hunger, food_dist, has_hostile, rugosity, albedo) =
            sensory_q.get(entity).map(|(ea, sa, ms, ia)| {
                // extract_sensory_inputs — stateless helper
            }).unwrap_or_default();

        let new_sig = shape_cache_signature_full(
            fineness, qe_norm, volume.radius(), hunger, food_dist, has_hostile, rugosity, albedo,
        );

        let has_shape = inferred_q.get(entity).is_ok();
        let needs_rebuild = match policy_opt {
            Some(p) if p.enabled && p.scope == CacheScope::StableWindow && has_shape
                => p.dependency_signature != new_sig,
            _ => true,
        };

        if needs_rebuild {
            commands.entity(entity).insert(ShapeRebuildNeeded { signature: new_sig });
        }
    }
}
```

**Query principal:** 5 componentes. Sensory via query secundaria read-only.
**Filtro `Without<ShapeRebuildNeeded>`:** evita re-procesar entidades ya marcadas.

### Pure fn delegada (equations/entity_shape.rs — ya existe parcialmente)

```rust
/// Signature completa incluyendo surface. Compone las dos funciones existentes.
pub fn shape_cache_signature_full(
    fineness: f32, qe_norm: f32, radius: f32,
    hunger: f32, food_dist: f32, has_hostile: bool,
    rugosity: Option<f32>, albedo: Option<f32>,
) -> u16 {
    let base = shape_cache_signature(fineness, qe_norm, radius, hunger, food_dist, has_hostile);
    shape_cache_signature_with_surface(base, rugosity, albedo)
}
```

### Stateless helper (equations/entity_shape.rs)

```rust
/// Extrae inputs sensoriales de componentes opcionales. Defaults seguros.
pub fn extract_sensory_inputs(
    assessment: Option<&EnergyAssessment>,
    awareness: Option<&SensoryAwareness>,
    surface: Option<&MorphogenesisSurface>,
    albedo: Option<&InferredAlbedo>,
) -> (f32, f32, bool, Option<f32>, Option<f32>) {
    let hunger = assessment.map(|e| e.hunger_fraction).unwrap_or(0.0);
    let (food_dist, hostile) = awareness
        .map(|s| (s.food_distance, s.hostile_entity.is_some()))
        .unwrap_or((f32::MAX, false));
    let rug = surface.map(|s| s.rugosity);
    let alb = albedo.map(|a| a.albedo);
    (hunger, food_dist, hostile, rug, alb)
}
```

---

### Sistema 2: `shape_geometry_build_system` (computa mesh)

**Responsabilidad:** Para entidades con `ShapeRebuildNeeded`, computa `GeometryInfluence`, construye torso + órganos, produce mesh final.

```rust
/// Computa mesh GF1 para entidades marcadas con ShapeRebuildNeeded.
pub fn shape_geometry_build_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(
        Entity,                              // 1
        &BaseEnergy,                         // 2
        &OscillatorySignature,               // 3
        &FlowVector,                         // 4
        &MatterCoherence,                    // 5
    ), With<ShapeRebuildNeeded>>,
    volume_q: Query<&SpatialVolume>,
    shape_q: Query<&MorphogenesisShapeParams>,
    sensory_q: Query<(Option<&EnergyAssessment>, Option<&SensoryAwareness>,
                       Option<&MorphogenesisSurface>, Option<&InferredAlbedo>)>,
    body_plan_q: Query<&BodyPlanLayout>,
    profile_q: Query<&InferenceProfile>,
    rebuild_q: Query<&ShapeRebuildNeeded>,
) {
    for (entity, energy, wave, flow, matter) in &query {
        let radius = volume_q.get(entity).map(|v| v.radius()).unwrap_or(1.0);
        let fineness = shape_q.get(entity).ok().map(|s| s.fineness_ratio()).unwrap_or(FINENESS_DEFAULT);
        let (hunger, food_dist, hostile, rugosity, albedo) =
            sensory_q.get(entity).map(|(ea, sa, ms, ia)| extract_sensory_inputs(ea, sa, ms, ia))
                .unwrap_or_default();

        // HOF pipeline: stateless pure fns compuestas
        let influence = compute_entity_influence(
            energy.qe(), wave.frequency_hz(), wave.phase(),
            flow.velocity_2d(), matter.bond_energy_eb(), matter.state(),
            radius, fineness, hunger, food_dist, hostile, rugosity, albedo,
        );

        // Build compound mesh (torso + organs)
        let final_mesh = match body_plan_q.get(entity) {
            Ok(layout) if layout.active_count() > 0 => {
                let mobility = profile_q.get(entity).map(|p| p.mobility_bias).unwrap_or(0.5);
                build_compound_mesh(&influence, layout, mobility)
            }
            _ => build_entity_mesh(&influence),
        };

        let sig = rebuild_q.get(entity).map(|r| r.signature).unwrap_or(0);
        let handle = meshes.add(final_mesh);

        commands.entity(entity).insert(ShapeMeshReady { handle, signature: sig });
        commands.entity(entity).remove::<ShapeRebuildNeeded>();
    }
}
```

**Query principal:** 5 componentes (Entity, BaseEnergy, OscillatorySignature, FlowVector, MatterCoherence).
**Queries secundarias:** read-only, narrow, por entity ID.

### Pure fns (equations/entity_shape.rs — nuevas)

```rust
/// HOF: computa GeometryInfluence desde estado físico + sensorial.
/// Orquesta sensory_modulated_fineness → resistance → tint → detail → influence.
pub fn compute_entity_influence(
    qe: f32, frequency_hz: f32, phase: f32,
    velocity: Vec2, bond_energy: f32, matter_state: MatterState,
    radius: f32, fineness_base: f32,
    hunger: f32, food_dist: f32, has_hostile: bool,
    rugosity: Option<f32>, albedo: Option<f32>,
) -> GeometryInfluence {
    let qe_norm = normalized_qe(qe, VISUAL_QE_REFERENCE);
    let fineness = sensory_modulated_fineness(fineness_base, hunger);
    let resistance = sensory_modulated_resistance(bond_energy, matter_state, has_hostile);
    let tint = frequency_to_tint_rgb(frequency_hz);
    let detail = surface_modulated_detail(entity_lod_detail(qe_norm, radius), rugosity);
    let tint_final = albedo_modulated_tint(tint, albedo);
    entity_geometry_influence(Vec3::ZERO, qe_norm, radius, fineness, resistance,
        Vec3::new(velocity.x, 0.0, velocity.y), tint_final, detail)
}

/// Construye mesh de torso.
pub fn build_entity_mesh(influence: &GeometryInfluence) -> Mesh {
    let spine = build_flow_spine(influence);
    build_flow_mesh(&spine, influence)
}

/// Construye mesh compuesto (torso + organs).
pub fn build_compound_mesh(influence: &GeometryInfluence, layout: &BodyPlanLayout, mobility: f32) -> Mesh {
    let torso = build_entity_mesh(influence);
    let mut all = vec![torso];
    for i in 0..layout.active_count() as usize {
        let (len_f, rad_f) = organ_slot_scale(i, layout.active_count(), mobility);
        if len_f <= 0.0 { continue; }
        let organ_inf = derive_organ_influence(influence, layout, i, len_f, rad_f);
        all.push(build_entity_mesh(&organ_inf));
    }
    merge_meshes(&all)
}

/// Deriva GeometryInfluence para un órgano (sub-mesh) desde la influencia del torso.
fn derive_organ_influence(
    torso: &GeometryInfluence, layout: &BodyPlanLayout,
    slot: usize, len_factor: f32, rad_factor: f32,
) -> GeometryInfluence {
    GeometryInfluence {
        detail: torso.detail * ORGAN_DETAIL_FRACTION,
        energy_direction: layout.direction(slot),
        energy_strength: torso.energy_strength * ORGAN_ENERGY_FRACTION,
        resistance: torso.resistance,
        least_resistance_direction: torso.least_resistance_direction,
        length_budget: torso.length_budget * len_factor,
        max_segments: torso.max_segments,
        radius_base: torso.radius_base * rad_factor,
        start_position: layout.position(slot),
        qe_norm: torso.qe_norm,
        tint_rgb: torso.tint_rgb,
        branch_role: torso.branch_role,
    }
}
```

**Constantes (blueprint/constants/morphogenesis.rs — extender):**
```rust
pub const ORGAN_DETAIL_FRACTION: f32 = 0.7;
pub const ORGAN_ENERGY_FRACTION: f32 = 0.3;
pub const HUNGER_FINENESS_BOOST: f32 = 0.25;
pub const HOSTILE_RESIST_MULT: f32 = 1.35;
```

---

### Sistema 3: `shape_visual_sync_system` (sync mesh a visual entity)

**Responsabilidad:** Toma el mesh pre-construido y lo asigna al visual entity. Mínimo acoplamiento.

```rust
/// Asigna mesh construido al visual entity y actualiza cache.
pub fn shape_visual_sync_system(
    mut commands: Commands,
    query: Query<(
        Entity,                   // 1
        &ShapeMeshReady,          // 2
        &V6VisualRoot,            // 3
        Option<&ShapeInferred>,   // 4
        Option<&mut PerformanceCachePolicy>, // 5
    ), With<HasInferredShape>>,
    mut visual_q: Query<(&mut Mesh3d, &mut Transform), Without<HasInferredShape>>,
) {
    for (entity, ready, visual_root, shape_inferred, policy_opt) in &query {
        // Sync mesh to visual entity
        if let Ok((mut mesh3d, mut tf)) = visual_q.get_mut(visual_root.visual_entity) {
            mesh3d.0 = ready.handle.clone();
            if tf.scale != Vec3::ONE { tf.scale = Vec3::ONE; }
        }

        // Cache writeback
        if let Some(mut policy) = policy_opt {
            if policy.dependency_signature != ready.signature {
                policy.dependency_signature = ready.signature;
            }
        }

        // Insert ShapeInferred marker if first build
        if shape_inferred.is_none() {
            commands.entity(entity).insert(ShapeInferred);
        }

        // Cleanup: remove intermediate
        commands.entity(entity).remove::<ShapeMeshReady>();
    }
}
```

**Query principal:** 5 componentes.

---

## Componentes transitorios

```rust
/// Cache miss marker. Insertado por gate, consumido por geometry build.
#[derive(Component, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct ShapeRebuildNeeded {
    pub signature: u16,
}

/// Mesh listo para sync. Insertado por geometry build, consumido por visual sync.
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct ShapeMeshReady {
    pub handle: Handle<Mesh>,
    pub signature: u16,
}
```

**Lifecycle por tick:**
```
gate → ShapeRebuildNeeded → geometry build → ShapeMeshReady → visual sync → cleanup
       (cache miss only)                    (mesh + handle)              (remove both)
```

---

## Registro (worldgen_plugin.rs:208-215 — reemplazar)

```rust
// ANTES:
app.add_systems(Update, entity_shape_inference_system.after(sync_visual_from_sim_system));

// DESPUÉS:
app.add_systems(Update, (
    shape_cache_gate_system,
    shape_geometry_build_system,
    shape_visual_sync_system,
).chain().after(sync_visual_from_sim_system));
```

---

## Testing TDD (3 capas)

### Capa 1: Unitario — pure fns

```rust
// Tests para compute_entity_influence, sensory_modulated_*, extract_sensory_inputs
// build_entity_mesh, build_compound_mesh, derive_organ_influence
// shape_cache_signature_full

#[test]
fn compute_entity_influence_zero_velocity_uses_default_direction() { ... }
#[test]
fn sensory_modulated_fineness_zero_hunger_passthrough() { ... }
#[test]
fn sensory_modulated_resistance_hostile_multiplied() { ... }
#[test]
fn extract_sensory_inputs_all_none_returns_defaults() { ... }
#[test]
fn derive_organ_influence_scales_correctly() { ... }
#[test]
fn build_compound_mesh_no_organs_equals_torso() { ... }
#[test]
fn shape_cache_signature_full_deterministic() { ... }
```

### Capa 2: Integración — sistemas aislados con MinimalPlugins

```rust
fn run_in_update<S: IntoSystemConfigs<M>, M>(system: S, setup: impl FnOnce(&mut World)) -> World { ... }

#[test]
fn gate_inserts_rebuild_needed_on_first_frame() { ... }
#[test]
fn gate_skips_when_signature_matches_and_shape_inferred() { ... }
#[test]
fn geometry_build_produces_mesh_ready_from_rebuild_needed() { ... }
#[test]
fn geometry_build_removes_rebuild_needed() { ... }
#[test]
fn visual_sync_sets_mesh_on_visual_entity() { ... }
#[test]
fn visual_sync_inserts_shape_inferred_on_first_build() { ... }
#[test]
fn visual_sync_removes_mesh_ready() { ... }
```

### Capa 3: Orquestación — pipeline completo

```rust
/// HOF: ejecuta pipeline de shape inference end-to-end.
fn run_shape_pipeline<S, A>(setup: S, assert_fn: A)
where S: FnOnce(&mut World) -> Entity, A: FnOnce(&World, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.add_systems(Update, (
        shape_cache_gate_system,
        shape_geometry_build_system,
        shape_visual_sync_system,
    ).chain());
    let entity = setup(app.world_mut());
    for _ in 0..3 { app.update(); }
    assert_fn(app.world(), entity);
}

#[test]
fn full_pipeline_produces_shape_inferred() { ... }
#[test]
fn full_pipeline_cache_hit_skips_rebuild() { ... }
#[test]
fn full_pipeline_with_body_plan_produces_compound_mesh() { ... }
```

---

## Axiomas respetados

| Axioma | Cómo se respeta |
|--------|-----------------|
| 1 (Energy) | `qe_norm = normalized_qe(energy.qe(), VISUAL_QE_REFERENCE)` — forma derivada de energía |
| 4 (Dissipation) | No aplica (visual, no physics) |
| 7 (Distance) | `entity_lod_detail(qe_norm, radius)` — detail decae con tamaño |
| 8 (Oscillatory) | `frequency_to_tint_rgb(frequency_hz)` — color derivado de frecuencia |

---

## Criterios de cierre

- [ ] `entity_shape_inference_system` eliminado (0 references)
- [ ] 3 nuevos sistemas registrados en Update con `.chain()`
- [ ] Max 5 componentes en query principal de cada sistema
- [ ] `ShapeRebuildNeeded` y `ShapeMeshReady` son SparseSet
- [ ] Pure fns: `compute_entity_influence`, `build_compound_mesh`, `derive_organ_influence`, `extract_sensory_inputs`
- [ ] Constantes ORGAN_DETAIL_FRACTION, ORGAN_ENERGY_FRACTION en constants/
- [ ] 7+ unit tests, 7+ integration tests, 3+ pipeline tests
- [ ] `cargo test` — 0 failures
- [ ] Bench pre/post: ≤5% regression
