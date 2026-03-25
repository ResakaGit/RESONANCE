# D1: Behavioral Intelligence

**Prioridad**: P0 — Fundamento de toda fauna autónoma
**Phase**: `Phase::Input` (después de will_input_system)
**Dependencias**: L0, L5, L7, SpatialIndex, InferenceProfile
**Systems**: 5

---

## Motivación Científica

La inteligencia comportamental en fauna real emerge de **evaluación de necesidades + percepción + decisión**. No hay "script" — el animal evalúa su estado interno (hambre, miedo, deseo reproductivo) contra estímulos externos (presa cerca, depredador, pareja) y actúa según utilidad.

En Resonance, esto se implementa como **Utility AI**: cada posible acción tiene un score calculado desde el estado de las capas, y la acción con mayor score gana.

---

## Componentes Nuevos

```
src/layers/behavior.rs (NUEVO)
```

### C1: BehaviorIntent (2 fields)
```rust
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BehaviorIntent {
    pub mode: BehaviorMode,
    pub target_entity: Option<Entity>,
}
```

### C2: BehaviorMode (enum, 6 variants)
```rust
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum BehaviorMode {
    Idle,
    Forage { urgency: f32 },
    Hunt { prey: Entity, chase_ticks: u32 },
    Flee { threat: Entity },
    Reproduce,
    Migrate { direction: Vec2 },
}
```

### C3: BehavioralAgent (marker, SparseSet)
```rust
#[derive(Component, Default, Reflect)]
#[component(storage = "SparseSet")]
pub struct BehavioralAgent;
```

### C4: BehaviorCooldown (2 fields)
```rust
#[derive(Component, Reflect, Debug, Clone)]
pub struct BehaviorCooldown {
    pub decision_cooldown: u32,
    pub action_cooldown: u32,
}
```

---

## Ecuaciones Nuevas

```
src/blueprint/equations/behavior/mod.rs (NUEVO)
```

### E1: `utility_forage(deficit_qe, distance_to_food, urgency_bias) -> f32`
Score de utilidad para buscar comida.
```
U_forage = (deficit / max_deficit) × (1 - distance / max_range) × (1 + urgency_bias)
```

### E2: `utility_flee(threat_level, distance_to_threat, resilience) -> f32`
Score de utilidad para huir.
```
U_flee = threat_level × (1 - distance / detection_range) × (1 - resilience × FLEE_RESILIENCE_SCALE)
```

### E3: `utility_hunt(prey_qe, distance, energy_available, mobility_bias) -> f32`
Score de utilidad para cazar.
```
U_hunt = (prey_qe / HUNT_QE_REFERENCE) × (1 - distance / HUNT_MAX_RANGE) × mobility_bias × energy_factor
```

### E4: `utility_reproduce(biomass, viability, maturity_progress) -> f32`
Score de utilidad para reproducirse.
```
U_reproduce = (biomass / threshold) × viability × maturity_progress
```

### E5: `select_best_action(scores: &[f32; 5]) -> usize`
Selecciona la acción con mayor score (deterministic tie-breaking by index).

---

## Constantes Nuevas

```
src/blueprint/constants/behavior.rs (NUEVO)
```

```rust
pub const BEHAVIOR_DECISION_INTERVAL: u32 = 4;        // Decide cada 4 ticks
pub const HUNGER_THRESHOLD_FRACTION: f32 = 0.3;        // <30% buffer → hungry
pub const SATIATED_THRESHOLD_FRACTION: f32 = 0.7;      // >70% buffer → satisfied
pub const PANIC_THRESHOLD: f32 = 0.8;                  // Threat level for panic
pub const MAX_CHASE_TICKS: u32 = 120;                  // Give up chase after 2s
pub const FLEE_RESILIENCE_SCALE: f32 = 0.5;            // Resilience reduces flee urgency
pub const HUNT_QE_REFERENCE: f32 = 500.0;              // Normalization for prey value
pub const HUNT_MAX_RANGE: f32 = 15.0;                  // Max detection for hunting
pub const FORAGE_MAX_RANGE: f32 = 20.0;                // Max detection for foraging
```

---

## Systems (5)

### S1: `behavior_assess_needs_system` (Transformer)
**Phase**: Input, after will_input
**Reads**: BaseEnergy, AlchemicalEngine, NutrientProfile, InferenceProfile
**Writes**: EnergyAssessment (SparseSet cache)
**Run condition**: `any_with_component::<BehavioralAgent>`, every 4 ticks

Evalúa estado interno: hambre, energía, nutrientes.

### S2: `behavior_evaluate_threats_system` (Transformer)
**Phase**: Input, after S1
**Reads**: Transform, SpatialIndex, MobaIdentity, BehavioralAgent
**Writes**: SensoryAwareness (extender existente o nuevo SparseSet)
**Run condition**: `any_with_component::<BehavioralAgent>`, every 4 ticks

Escanea entorno: detecta amenazas y oportunidades.

### S3: `behavior_decision_system` (Transformer)
**Phase**: Input, after S2
**Reads**: EnergyAssessment, SensoryAwareness, InferenceProfile, BehaviorCooldown
**Writes**: BehaviorIntent
**Run condition**: `any_with_component::<BehavioralAgent>`, every 4 ticks

Calcula utility scores, selecciona acción.

### S4: `behavior_will_bridge_system` (Transformer)
**Phase**: Input, after S3
**Reads**: BehaviorIntent, Transform
**Writes**: WillActuator (L7)
**Run condition**: `any_with_component::<BehavioralAgent>`

Traduce BehaviorIntent → WillActuator.movement_intent (el puente entre decisión y física).

```
Idle       → movement_intent = Vec2::ZERO
Forage     → movement_intent = direction_to_food_source
Hunt       → movement_intent = direction_to_prey × SPRINT_FACTOR
Flee       → movement_intent = direction_away_from_threat × PANIC_FACTOR
Reproduce  → movement_intent = direction_to_mate
Migrate    → movement_intent = migration_direction
```

### S5: `behavior_cooldown_tick_system` (Transformer)
**Phase**: Input, before S3
**Reads/Writes**: BehaviorCooldown
**Run condition**: `any_with_component::<BehavioralAgent>`

Decrementa cooldowns.

---

## Registration

```rust
// En SimulationPlugin o nuevo BehaviorPlugin:
app.add_systems(FixedUpdate, (
    behavior_cooldown_tick_system,
    behavior_assess_needs_system,
    behavior_evaluate_threats_system,
    behavior_decision_system,
    behavior_will_bridge_system,
).chain()
 .in_set(Phase::Input)
 .after(InputChannelSet::PlatformWill)
 .run_if(any_with_component::<BehavioralAgent>));
```

---

## Tests

### Ecuaciones
- `utility_forage_zero_deficit_returns_zero`
- `utility_forage_max_deficit_max_proximity_returns_high`
- `utility_flee_no_threat_returns_zero`
- `utility_flee_close_threat_high_level_returns_max`
- `utility_hunt_far_prey_returns_low`
- `select_best_action_deterministic_tiebreak`

### Systems
- `behavior_idle_when_satiated_and_safe`
- `behavior_forage_when_hungry`
- `behavior_flee_overrides_forage_when_panicking`
- `behavior_hunt_when_carnivore_detects_prey`
- `behavior_will_bridge_sets_correct_direction`
