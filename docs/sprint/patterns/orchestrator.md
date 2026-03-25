# Patrones del Orquestador

Patrones de diseño para reducir complejidad, mejorar rendimiento, y reducir acoplamiento entre los 32+ systems nuevos.

---

## P-1: Shared Scratch Buffer (Resource Pool Pattern)

**Problema**: N systems hacen spatial queries y cada uno necesita un `Vec<Entity>` temporal.

**Solución**: Un `Local<Vec<Entity>>` por system. Bevy inyecta uno exclusivo por instancia de system. Zero contention, zero allocation después del primer frame.

```rust
pub fn predation_spatial_filter_system(
    predators: Query<(Entity, &Transform, &SpatialVolume), With<TrophicCarnivore>>,
    spatial: Res<SpatialIndex>,
    mut scratch: Local<Vec<Entity>>,
) {
    for (entity, transform, volume) in &predators {
        scratch.clear();
        spatial.query_radius(transform.translation.truncate(), HUNT_RADIUS, &mut scratch);
        // ...
    }
}
```

**Complejidad eliminada**: No hay Resource compartido, no hay Mutex, no hay contention.

---

## P-2: Event Cascade (Producer → Consumer Chain)

**Problema**: Un system detecta una condición → necesita que otro system actúe.

**Solución**: Evento tipado + `.chain()` en registration.

```rust
// Definición
#[derive(Event)]
pub struct HungerEvent {
    pub entity: Entity,
    pub deficit_qe: f32,
}

// Producer (Phase::MetabolicLayer)
pub fn detect_hunger_system(
    query: Query<(Entity, &BaseEnergy, &AlchemicalEngine)>,
    mut events: EventWriter<HungerEvent>,
) {
    for (entity, energy, engine) in &query {
        let threshold = equations::starvation_threshold(engine.max_buffer);
        if energy.qe() < threshold {
            events.write(HungerEvent { entity, deficit_qe: threshold - energy.qe() });
        }
    }
}

// Consumer (Phase::MetabolicLayer, .after(detect_hunger))
pub fn behavior_respond_to_hunger_system(
    mut events: EventReader<HungerEvent>,
    mut wills: Query<&mut BehaviorIntent>,
) {
    for event in events.read() {
        let Ok(mut intent) = wills.get_mut(event.entity) else { continue; };
        if intent.mode != BehaviorMode::Flee {
            intent.mode = BehaviorMode::Forage { urgency: event.deficit_qe };
        }
    }
}

// Registration
app.add_systems(FixedUpdate, (
    detect_hunger_system,
    behavior_respond_to_hunger_system,
).chain().in_set(Phase::MetabolicLayer));
```

**Complejidad eliminada**: Sin acoplamiento directo entre systems. Evento es el contrato.

---

## P-3: Layered Cache Resource (Read-Once, Use-Many)

**Problema**: Múltiples systems necesitan el mismo dato derivado costoso (ej: densidad por celda, temperatura ambiente por zona).

**Solución**: Un system "materializer" escribe un Resource/cache, los demás leen `Res<Cache>`.

```rust
#[derive(Resource, Default)]
pub struct ZoneContextCache {
    pub generation: u32,
    entries: Vec<ZoneContextEntry>,  // Indexed by cell
}

// Materializer (Phase::ThermodynamicLayer, runs ONCE per frame)
pub fn materialize_zone_context_system(
    field: Res<EnergyFieldGrid>,
    terrain: Res<TerrainField>,
    eco: Res<EcoBoundaryField>,
    mut cache: ResMut<ZoneContextCache>,
) {
    if field.generation == cache.generation { return; }  // Skip if unchanged
    cache.generation = field.generation;
    // Recompute entries...
}

// Consumers (any later Phase, Res<ZoneContextCache>)
pub fn behavior_decision_system(
    cache: Res<ZoneContextCache>,
    // ...
) {
    let zone = &cache.entries[cell_index];
    // Use without recomputing
}
```

**Complejidad eliminada**: N consumers × M cells → 1 computation + N lookups. Generation counter evita recomputation innecesaria.

---

## P-4: Cursor Round-Robin (Amortized N² → N per frame)

**Problema**: System necesita comparar cada entidad con vecinos (N² potential).

**Solución**: Cursor Resource que avanza MAX_PER_FRAME por tick. Completa un full sweep en N/MAX_PER_FRAME frames.

```rust
#[derive(Resource, Default)]
pub struct TrophicScanCursor {
    next_index: usize,
}

pub const TROPHIC_SCAN_BUDGET: usize = 64;

pub fn trophic_intake_system(
    query: Query<(Entity, &Transform), With<TrophicConsumer>>,
    mut cursor: ResMut<TrophicScanCursor>,
    // ...
) {
    let entities: Vec<_> = query.iter().collect();
    if entities.is_empty() { return; }

    let start = cursor.next_index.min(entities.len());
    let end = (start + TROPHIC_SCAN_BUDGET).min(entities.len());

    for i in start..end {
        // Process entity[i]
    }

    cursor.next_index = if end >= entities.len() { 0 } else { end };
}
```

**Rendimiento**: Garantiza frame budget constante independiente de population size.

---

## P-5: Component Pair Split (Reduce Query Width)

**Problema**: System necesita leer 6 componentes para tomar decisión.

**Solución**: Dividir en 2 systems: el primero lee 3 y escribe un componente intermedio (SparseSet), el segundo lee 3 + el intermedio.

```rust
// System 1: Evalúa estado energético → escribe EnergyAssessment
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct EnergyAssessment {
    pub deficit: f32,
    pub trend: f32,  // Positive = gaining, negative = losing
}

pub fn assess_energy_system(
    mut commands: Commands,
    query: Query<(Entity, &BaseEnergy, &AlchemicalEngine, &FlowVector)>,
) {
    for (entity, energy, engine, flow) in &query {
        let deficit = equations::energy_deficit(energy.qe(), engine.max_buffer);
        let trend = equations::energy_trend(engine, flow);
        commands.entity(entity).insert(EnergyAssessment { deficit, trend });
    }
}

// System 2: Lee assessment + otros → decide behavior
pub fn behavior_decision_system(
    query: Query<(&EnergyAssessment, &InferenceProfile, &mut BehaviorIntent)>,
) {
    for (assessment, profile, mut intent) in &query {
        // Decision with 3 components instead of 6
    }
}
```

**Complejidad eliminada**: Queries más angostos = más paralelismo del schedule. Intermediate component es SparseSet (transient).

---

## P-6: Enum State Machine (Behavioral FSM sin Trait Objects)

**Problema**: Fauna necesita estados de comportamiento (Idle, Forage, Hunt, Flee, Reproduce).

**Solución**: Enum + match exhaustivo. Sin dyn dispatch. Sin inheritance.

```rust
#[derive(Component, Reflect, Debug, Clone)]
pub struct BehaviorIntent {
    pub mode: BehaviorMode,
    pub target_entity: Option<Entity>,
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BehaviorMode {
    Idle,
    Forage { urgency: f32 },
    Hunt { prey: Entity, chase_ticks: u32 },
    Flee { threat: Entity },
    Reproduce,
    Migrate { direction: Vec2 },
}

// Transition logic in one system — exhaustive match
pub fn behavior_transition_system(
    mut query: Query<(&mut BehaviorIntent, &EnergyAssessment, &SensoryAwareness)>,
) {
    for (mut intent, energy, sensory) in &mut query {
        let new_mode = match &intent.mode {
            BehaviorMode::Idle => {
                if energy.deficit > HUNGER_THRESHOLD {
                    BehaviorMode::Forage { urgency: energy.deficit }
                } else if sensory.threat_nearby {
                    BehaviorMode::Flee { threat: sensory.nearest_threat }
                } else {
                    BehaviorMode::Idle
                }
            }
            BehaviorMode::Forage { urgency } => {
                if sensory.threat_nearby && sensory.threat_level > PANIC_THRESHOLD {
                    BehaviorMode::Flee { threat: sensory.nearest_threat }
                } else if energy.deficit < SATIATED_THRESHOLD {
                    BehaviorMode::Idle
                } else {
                    BehaviorMode::Forage { urgency: energy.deficit }
                }
            }
            BehaviorMode::Hunt { prey, chase_ticks } => {
                if *chase_ticks > MAX_CHASE_TICKS {
                    BehaviorMode::Idle  // Give up
                } else if energy.deficit > EXHAUSTION_THRESHOLD {
                    BehaviorMode::Forage { urgency: energy.deficit }
                } else {
                    BehaviorMode::Hunt { prey: *prey, chase_ticks: chase_ticks + 1 }
                }
            }
            BehaviorMode::Flee { threat } => {
                if !sensory.threat_nearby {
                    BehaviorMode::Idle
                } else {
                    BehaviorMode::Flee { threat: *threat }
                }
            }
            BehaviorMode::Reproduce => {
                BehaviorMode::Idle  // One-shot
            }
            BehaviorMode::Migrate { direction } => {
                BehaviorMode::Migrate { direction: *direction }
            }
        };
        if intent.mode != new_mode {
            intent.mode = new_mode;
        }
    }
}
```

**Complejidad eliminada**: Todo el FSM en un match. Compiler garantiza exhaustividad. Cero allocations.

---

## P-7: Spatial Index Reuse (Zero-Copy Query)

**Problema**: 5+ systems necesitan "find entities near X".

**Solución**: Reusar `SpatialIndex` existente (ya rebuildeado en AtomicLayer).

**Regla**: NUNCA crear un segundo spatial index. Todos los systems post-AtomicLayer leen `Res<SpatialIndex>`.

```
AtomicLayer: update_spatial_index_after_move_system (WRITE)
    ↓
ChemicalLayer+: todos leen Res<SpatialIndex> (READ)
```

---

## P-8: Conditional System (run_if + Marker)

**Problema**: System solo aplica a fauna (no flora, no minerales, no héroes).

**Solución**: Marker component + `With<Marker>` filter + optional `run_if`.

```rust
/// Marca entidades que tienen behavioral AI.
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct BehavioralAgent;

// System solo procesa entidades con BehavioralAgent
pub fn behavior_decision_system(
    query: Query<(&mut BehaviorIntent, &EnergyAssessment), With<BehavioralAgent>>,
) { ... }

// O skip frame entero si no hay fauna (patrón del codebase: in_state)
// NOTA: el codebase NO usa any_with_component. Usa in_state() + PlayState guards.
// Para fauna, usar run condition basada en Resource:
app.add_systems(FixedUpdate, behavior_decision_system
    .in_set(Phase::Input)
    .run_if(resource_exists::<PopulationCensus>));

// Alternativa viable si se quiere filtrar por marker:
// .run_if(|q: Query<(), With<BehavioralAgent>>| !q.is_empty())
```

**Rendimiento**: Zero-cost cuando no hay fauna en el mundo.

---

## P-9: Derivation Cache (SparseSet Intermediate)

**Problema**: Cálculo costoso (ej: organ viability) usado por 3 systems downstream.

**Solución**: SparseSet component como cache. Recalculado solo cuando inputs cambian.

```rust
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct CachedViability {
    pub score: f32,
    pub limiting_factor: u8,
}

pub fn viability_cache_system(
    mut commands: Commands,
    query: Query<(Entity, &BaseEnergy, &NutrientProfile), Changed<BaseEnergy>>,
) {
    for (entity, energy, nutrient) in &query {
        let (score, factor) = equations::metabolic_viability(energy.qe(), nutrient);
        commands.entity(entity).insert(CachedViability { score, limiting_factor: factor });
    }
}
```

**Complejidad eliminada**: Downstream systems leen `Res<CachedViability>` sin recomputar.

---

## P-10: Domain Event Bus (Decouple Cross-Domain)

**Problema**: D2 (Trophic) necesita notificar a D6 (Social) que un miembro de la manada murió.

**Solución**: Usar `DeathEvent` existente. No crear evento directo D2→D6.

```
Regla: Cross-domain communication SOLO via eventos existentes
  (DeathEvent, PhaseTransitionEvent, CollisionEvent, etc.)

Si no hay evento adecuado → crear uno en events.rs
  (NUNCA comunicación directa entre systems de dominios distintos)
```

**Acoplamiento eliminado**: D2 y D6 no se conocen. Solo conocen el evento.
