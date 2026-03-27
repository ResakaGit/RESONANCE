# Sprint ET-13 — Geological Time LOD: Física Comprimida para Escalas Geológicas

**Módulo:** `src/simulation/emergence/geological_lod.rs` (nuevo), `src/blueprint/equations/emergence/geological_lod.rs` (nuevo)
**Tipo:** Ecuaciones puras + sistema de LOD temporal.
**Tier:** T3-4. **Onda:** B.
**BridgeKind:** `LODPhysicsBridge` — cache Small(16), clave `(lod_level, tick_compression)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

En escalas geológicas (millones de ticks), simular entidad por entidad es imposible. El LOD temporal agrega entidades en poblaciones, simplifica la física y comprime múltiples ticks en uno. Cuando el LOD vuelve a la resolución completa, las poblaciones se desagregan con varianza apropiada.

```
LOD 0 (full): cada entidad individual, cada tick
LOD 1 (compressed): grupos de 8, tick_compression=10
LOD 2 (population): regiones de 32, tick_compression=100
LOD 3 (geological): mapa completo, tick_compression=1000
```

---

## Responsabilidades

### ET-13A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/geological_lod.rs

/// Nivel de LOD óptimo dado el número de entidades y el horizonte temporal.
pub fn optimal_lod_level(
    entity_count: u32,
    tick_horizon: u32,
    performance_budget: f32,   // entidades×ticks que podemos simular por frame
) -> u8 {
    let required_work = entity_count as f32 * tick_horizon as f32;
    if required_work <= performance_budget { return 0; }
    let compression_needed = required_work / performance_budget;
    match compression_needed as u32 {
        0..=9   => 0,
        10..=99 => 1,
        100..=999 => 2,
        _         => 3,
    }
}

/// Física simplificada para LOD > 0: sólo energía media del grupo.
/// population_qe: suma de qe de todas las entidades del grupo.
/// group_size: número de entidades.
pub fn compressed_physics_step(
    population_qe: f32,
    mean_intake: f32,
    mean_dissipation: f32,
    tick_compression: u32,
) -> f32 {
    let net_per_tick = mean_intake - mean_dissipation;
    (population_qe + net_per_tick * tick_compression as f32).max(0.0)
}

/// Varianza del grupo para desagregar con ruido apropiado.
pub fn population_variance(mean_qe: f32, variance_factor: f32, group_size: u32) -> f32 {
    if group_size == 0 { return 0.0; }
    mean_qe * variance_factor / (group_size as f32).sqrt()
}

/// Qe asignado a una entidad al desagregar una población (con varianza).
/// entity_seed: u32 deterministico por entidad para varianza reproducible.
pub fn desegregated_qe(mean_qe: f32, variance: f32, entity_seed: u32) -> f32 {
    // Ruido pseudo-aleatorio deterministico usando LCG
    let noise = (entity_seed.wrapping_mul(1664525).wrapping_add(1013904223)) as f32
        / u32::MAX as f32;
    let centered_noise = noise * 2.0 - 1.0;  // [-1, 1]
    (mean_qe + centered_noise * variance).max(0.0)
}

/// Tasa de extinción simplificada para LOD alto.
pub fn population_extinction_rate(
    mean_qe: f32,
    dissipation_rate: f32,
    environmental_stress: f32,
) -> f32 {
    if mean_qe <= 0.0 { return 1.0; }
    (dissipation_rate + environmental_stress) / mean_qe
}
```

### ET-13B: Tipos

```rust
// src/simulation/emergence/geological_lod.rs

/// Resource: estado del LOD temporal global.
#[derive(Resource, Debug)]
pub struct GeologicalLODState {
    pub current_lod:        u8,     // 0=full, 1=compressed×10, 2=×100, 3=×1000
    pub tick_compression:   u32,    // ticks simulados por "super-tick" de LOD
    pub performance_budget: f32,    // entidades×ticks/frame máximo
    pub aggregate_groups:   Vec<PopulationGroup>,  // activo sólo en LOD > 0
}

#[derive(Debug, Clone)]
pub struct PopulationGroup {
    pub group_id:     u32,
    pub mean_qe:      f32,
    pub entity_count: u16,
    pub region_idx:   u8,   // índice de región 8×8 (de MultiscaleSignalGrid)
}

/// Marker: entidad actualmente comprimida en un grupo LOD.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct LODCompressed {
    pub group_id:    u32,
    pub lod_level:   u8,
    pub enter_tick:  u64,
    pub seed:        u32,  // para desagregación determinista
}
```

### ET-13C: Sistemas

```rust
/// Evalúa si se debe cambiar el nivel de LOD según carga actual.
/// Phase::ThermodynamicLayer — primera fase, ajusta antes de simular.
pub fn geological_lod_controller(
    entity_count: Query<(), With<BaseEnergy>>,
    mut lod_state: ResMut<GeologicalLODState>,
    config: Res<GeologicalLODConfig>,
    clock: Res<SimulationClock>,
) {
    if clock.tick_id % 100 != 0 { return; }  // evalúa cada 100 ticks
    let n = entity_count.iter().count() as u32;
    let new_lod = geological_lod_eq::optimal_lod_level(
        n, config.planning_horizon, lod_state.performance_budget,
    );
    if lod_state.current_lod != new_lod { lod_state.current_lod = new_lod; }
}

/// Agrega entidades en grupos de población cuando LOD > 0.
/// Phase::MetabolicLayer — after all individual metabolism.
pub fn population_aggregation_system(
    mut commands: Commands,
    agents: Query<(Entity, &BaseEnergy, &Transform), Without<LODCompressed>>,
    mut lod_state: ResMut<GeologicalLODState>,
    field: Res<EnergyFieldGrid>,
    clock: Res<SimulationClock>,
) {
    if lod_state.current_lod == 0 { return; }

    lod_state.aggregate_groups.clear();
    let mut regions: [PopulationGroup; 64] = core::array::from_fn(|i| PopulationGroup {
        group_id: i as u32, mean_qe: 0.0, entity_count: 0, region_idx: i as u8,
    });

    for (entity, energy, transform) in &agents {
        let cell_idx = field.world_to_cell_idx(transform.translation.x, transform.translation.z);
        let region = MultiscaleSignalGrid::cell_to_region(cell_idx);
        regions[region].mean_qe    += energy.qe();
        regions[region].entity_count += 1;
        commands.entity(entity).insert(LODCompressed {
            group_id:   region as u32,
            lod_level:  lod_state.current_lod,
            enter_tick: clock.tick_id,
            seed:       entity.index(),
        });
    }

    for r in &regions {
        if r.entity_count > 0 {
            let mut group = r.clone();
            if r.entity_count > 0 { group.mean_qe /= r.entity_count as f32; }
            lod_state.aggregate_groups.push(group);
        }
    }
}

/// Simula física comprimida sobre grupos de población en LOD activo.
/// Phase::MetabolicLayer — after population_aggregation_system.
pub fn compressed_population_physics_system(
    mut lod_state: ResMut<GeologicalLODState>,
    ms: Res<MultiscaleSignalGrid>,
    config: Res<GeologicalLODConfig>,
    clock: Res<SimulationClock>,
) {
    if lod_state.current_lod == 0 { return; }
    let compression = lod_state.tick_compression;
    for group in lod_state.aggregate_groups.iter_mut() {
        let env_qe = ms.regional_at(group.region_idx as usize);
        let mean_intake = env_qe * config.mean_intake_factor;
        let new_qe = geological_lod_eq::compressed_physics_step(
            group.mean_qe * group.entity_count as f32,
            mean_intake,
            config.mean_dissipation,
            compression,
        ) / group.entity_count.max(1) as f32;
        group.mean_qe = new_qe;
    }
}
```

### ET-13D: Constantes

```rust
pub struct LODPhysicsBridge;
impl BridgeKind for LODPhysicsBridge {}

pub const LOD_DEFAULT_PERFORMANCE_BUDGET: f32 = 100_000.0;  // entidades×ticks/frame
pub const LOD_PLANNING_HORIZON:           u32 = 1000;
pub const LOD_MEAN_INTAKE_FACTOR:         f32 = 0.1;
pub const LOD_MEAN_DISSIPATION:           f32 = 0.05;
pub const LOD_TICK_COMPRESSIONS: [u32; 4] = [1, 10, 100, 1000];
```

---

## Tacticas

- **LOD dinámico por presión.** Si hay 10k entidades con horizonte 1000 = 10M ops/frame — inaceptable. LOD 3 reduce a 10k/frame. El `optimal_lod_level` se ajusta solo.
- **`LODCompressed` como SparseSet marker.** Las entidades comprimidas no ejecutan sus sistemas normales — el marker es suficiente para excluirlas con `Without<LODCompressed>` en otros queries.
- **Desagregación determinista.** `entity.index()` como seed garantiza que la misma entidad recibe el mismo qe al desagregar — determinismo preservado (INV-4).
- **LODPhysicsBridge cachea `compressed_physics_step`.** Para grupos con mismo `(mean_qe_band, compression)`, el resultado es idéntico. Cache Small(16) — pocos LOD levels.

---

## NO hace

- No modifica LOD per-entidad — el LOD es global (toda la simulación).
- No implementa streaming de chunks — el mapa es 32×32 fijo.
- No implementa LOD visual — eso es GS-7 VisualContract.

---

## Dependencias

- ET-11 `MultiscaleSignalGrid` — señal regional para la física comprimida.
- ET-12 `TectonicState` — los tectonic_ticks también se comprimen en LOD alto.
- `worldgen/field_grid.rs::EnergyFieldGrid` — referencia espacial.

---

## Criterios de Aceptación

- `optimal_lod_level(100, 100, 100_000.0)` → `0` (bajo presupuesto).
- `optimal_lod_level(10000, 1000, 100_000.0)` → `2` (×100 compresión).
- `compressed_physics_step(1000.0, 10.0, 5.0, 100)` → `1500.0`.
- `desegregated_qe(100.0, 10.0, 42)` → determinista, en rango `[80, 120]`.
- Test: LOD > 0 → entidades tienen `LODCompressed` component.
- Test: grupo con mean_qe=0 → extinction_rate=1.0.
- Test: misma seed → misma desagregación (determinismo).
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-11 Multiscale Information — señal regional fuente
- ET-12 Continental Drift — tectónica comprimida en LOD alto
- Blueprint §T3-4: "Geological Time LOD", population compression
