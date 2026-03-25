# D2: Trophic & Predation

**Prioridad**: P0 — Cadena alimentaria básica
**Phase**: `Phase::MetabolicLayer` (después de growth_budget)
**Dependencias**: D1 (BehaviorIntent), L0, L5, SpatialIndex, ecology equations
**Systems**: 4

---

## Motivación Científica

Las cadenas tróficas son el motor de la ecología: productores (flora) → herbívoros → carnívoros → descomponedores. Cada nivel transfiere ~10% de la energía al siguiente (ley del 10%, Lindeman 1942).

En Resonance, la transferencia trófica es **transferencia de qe** del consumido al consumidor, modulada por eficiencia de asimilación y resistencia de la presa (bond_energy).

Las ecuaciones ya existen en `equations/ecology/mod.rs`: `trophic_intake_factor`, `trophic_assimilation`, `trophic_maintenance_cost`, `trophic_net_qe_delta`. Solo faltan los systems que las invocan.

---

## Componentes Nuevos

```
src/layers/trophic.rs (NUEVO)
```

### C1: TrophicConsumer (component — extends existing TrophicClass enum)

**NOTA**: `TrophicClass` ya existe en `src/layers/inference.rs:128` como enum con 5 variants:
`PrimaryProducer`, `Herbivore`, `Omnivore`, `Carnivore`, `Detritivore`. Es un enum simple (no Component).

El nuevo component TrophicConsumer REUTILIZA TrophicClass y agrega parámetros de intake:

```rust
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct TrophicConsumer {
    pub class: TrophicClass,   // Reutiliza enum existente
    pub intake_rate: f32,      // qe/s base intake capacity
}
```

### C2: TrophicState (3 fields)
```rust
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct TrophicState {
    pub last_meal_tick: u32,
    pub satiation: f32,         // [0..1] fullness
    pub assimilation_buffer: f32, // qe pending absorption
}
```

---

## Ecuaciones (ya existentes + nuevas)

### Existentes (en `equations/ecology/mod.rs`):
- `trophic_intake_factor(role, medium_density, food_density) -> f32`
- `trophic_assimilation(intake, efficiency) -> f32`
- `trophic_maintenance_cost(mass, temperature_deviation, predation_pressure) -> f32`
- `trophic_net_qe_delta(assimilation, maintenance, competition) -> f32`

### Nuevas (en `equations/trophic/mod.rs`):
- `predation_success_probability(predator_speed, prey_speed, distance, terrain_factor) -> f32`
- `prey_qe_transfer(prey_qe, bond_energy, assimilation_efficiency) -> f32`
- `foraging_intake_from_field(cell_qe, intake_rate, dt) -> f32`

---

## Constantes Nuevas

```
src/blueprint/constants/trophic.rs (NUEVO)
```

```rust
pub const TROPHIC_TRANSFER_EFFICIENCY: f32 = 0.10;   // Lindeman 10% rule
pub const HERBIVORE_ASSIMILATION: f32 = 0.35;         // 35% plant→herbivore
pub const CARNIVORE_ASSIMILATION: f32 = 0.20;         // 20% meat→carnivore
pub const DECOMPOSER_ASSIMILATION: f32 = 0.15;        // 15% decay→decomposer
pub const PREDATION_BASE_SUCCESS: f32 = 0.3;          // 30% base chase success
pub const PREDATION_SPEED_ADVANTAGE_SCALE: f32 = 0.5;
pub const FORAGING_CELL_DRAIN_MAX: f32 = 5.0;         // Max qe/tick from cell
pub const SATIATION_DECAY_RATE: f32 = 0.005;          // Satiation drops per tick
pub const MEAL_SATIATION_GAIN: f32 = 0.3;             // Satiation per successful feed
pub const TROPHIC_SCAN_BUDGET: usize = 64;            // Max spatial queries/frame
```

---

## Systems (4)

### S1: `trophic_satiation_decay_system` (Transformer)
**Phase**: MetabolicLayer
**Reads/Writes**: TrophicState
**Logic**: `satiation -= SATIATION_DECAY_RATE × dt`. Emite HungerEvent si < threshold.

### S2: `trophic_herbivore_forage_system` (Transformer)
**Phase**: MetabolicLayer, after S1
**Reads**: BehaviorIntent (mode == Forage), Transform, TrophicRole (Herbivore|Omnivore)
**Writes**: BaseEnergy (inject), NutrientFieldGrid (drain cell), TrophicState
**Throttle**: Cursor, TROPHIC_SCAN_BUDGET
**Logic**: Si BehaviorMode::Forage → leer celda del NutrientFieldGrid → drenar → inyectar qe × efficiency.

### S3: `trophic_predation_attempt_system` (Transformer + Emitter)
**Phase**: MetabolicLayer, after S2
**Reads**: BehaviorIntent (mode == Hunt), Transform, SpatialIndex, TrophicRole (Carnivore|Omnivore)
**Writes**: BaseEnergy (predator inject, prey drain), TrophicState
**Emits**: PreyConsumedEvent, DeathEvent (if prey dies)
**Throttle**: Cursor, TROPHIC_SCAN_BUDGET
**Logic**:
1. Si BehaviorMode::Hunt → verificar distancia a prey ≤ CAPTURE_RADIUS
2. Calcular `predation_success_probability()`
3. Si éxito → `prey_qe_transfer()` → drain prey, inject predator
4. Si prey.qe < QE_MIN → DeathEvent

### S4: `trophic_decomposer_system` (Transformer)
**Phase**: MetabolicLayer, after S3
**Reads**: DeathEvent (recent), Transform, TrophicRole (Decomposer)
**Writes**: BaseEnergy, NutrientFieldGrid (return nutrients to soil)
**Logic**: Descomponedores absorben qe de cadáveres y devuelven nutrientes al grid.

---

## Eventos Nuevos

```rust
#[derive(Event)]
pub struct HungerEvent {
    pub entity: Entity,
    pub deficit_qe: f32,
}

#[derive(Event)]
pub struct PreyConsumedEvent {
    pub predator: Entity,
    pub prey: Entity,
    pub qe_transferred: f32,
}
```

---

## Tests

### Ecuaciones
- `predation_success_faster_predator_higher_probability`
- `predation_success_far_distance_returns_low`
- `prey_qe_transfer_high_bond_reduces_transfer`
- `foraging_intake_respects_cell_drain_max`
- `trophic_net_qe_delta_maintenance_exceeds_intake_is_negative`

### Systems
- `herbivore_gains_qe_from_nutrient_cell`
- `carnivore_drains_prey_on_capture`
- `prey_dies_when_qe_depleted`
- `decomposer_returns_nutrients_to_grid`
- `satiation_decays_over_time`
