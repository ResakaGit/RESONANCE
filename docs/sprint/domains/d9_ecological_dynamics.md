# D9: Ecological Dynamics

**Prioridad**: P2
**Phase**: `Phase::MetabolicLayer` (después de trophic)
**Dependencias**: D2 (trophic events), D7 (offspring events), NutrientField, EnergyFieldGrid
**Systems**: 3

---

## Motivación Científica

La dinámica ecológica gobierna poblaciones a escala macro:

1. **Capacidad de carga (K)**: Recursos finitos → población max. Lotka-Volterra: dN/dt = rN(1 - N/K)
2. **Sucesión ecológica**: Comunidades cambian con el tiempo. Pioneros (r-strategy) → climax (K-strategy)
3. **Census y monitoring**: Sin conteo no hay retroalimentación

En Resonance, K emerge naturalmente del NutrientFieldGrid (recursos finitos) + competitive_exclusion. Pero sin census explícito, no hay forma de modular tasas de reproducción/abiogenesis según presión poblacional.

---

## Componentes Nuevos

### C1: PopulationCensus (Resource, no Component)
```rust
#[derive(Resource, Default)]
pub struct PopulationCensus {
    pub total_entities: u32,
    pub by_trophic_role: [u32; 5],   // Producer, Herbivore, Carnivore, Decomposer, Omnivore
    pub by_zone: Vec<u32>,           // Per grid cell
    pub generation: u32,
}
```

---

## Ecuaciones Nuevas

```
src/blueprint/equations/ecology_dynamics/mod.rs (NUEVO)
```

### E1: `carrying_capacity(cell_qe: f32, nutrient_total: f32, cell_size: f32) -> u32`
```
K = floor(cell_qe × nutrient_total / (ENTITY_MIN_QE × cell_size²))
```
Capacidad proporcional a energía × nutrientes disponibles.

### E2: `reproduction_pressure(local_population: u32, carrying_capacity: u32) -> f32`
```
pressure = 1.0 - (local_population as f32 / carrying_capacity as f32).min(1.0)
```
0 = at capacity (no reproduction), 1 = empty (max reproduction).

### E3: `succession_stage(time_since_disturbance: u32, dominant_trophic: TrophicRole) -> SuccessionStage`
```
enum SuccessionStage { Pioneer, Early, Mid, Climax }
// Pioneer: mostly producers, low diversity
// Early: herbivores arrive
// Mid: carnivores establish
// Climax: stable equilibrium
```

### E4: `abiogenesis_modulated_threshold(base: f32, reproduction_pressure: f32) -> f32`
```
threshold = base / (1.0 + reproduction_pressure × ABIOGENESIS_PRESSURE_SCALE)
```
Cuando hay espacio → más fácil que surja vida espontáneamente.

---

## Constantes

```rust
pub const CENSUS_INTERVAL: u32 = 30;                 // Count every 30 ticks (0.5s)
pub const CARRYING_CAPACITY_QE_FACTOR: f32 = 10.0;   // qe per entity slot
pub const ABIOGENESIS_PRESSURE_SCALE: f32 = 2.0;     // How much pressure boosts abiogenesis
pub const SUCCESSION_PIONEER_TICKS: u32 = 300;        // 5s before herbivores
pub const SUCCESSION_EARLY_TICKS: u32 = 1200;         // 20s before carnivores
pub const SUCCESSION_MID_TICKS: u32 = 3600;           // 60s before climax
```

---

## Systems (3)

### S1: `ecology_census_system` (Transformer)
**Phase**: MetabolicLayer
**Reads**: TrophicRole, Transform, EnergyFieldGrid
**Writes**: PopulationCensus (Resource)
**Run condition**: Every CENSUS_INTERVAL ticks
**Logic**:
1. Count all entities by trophic role
2. Count per-cell populations
3. Update generation counter

### S2: `ecology_carrying_capacity_system` (Transformer)
**Phase**: MetabolicLayer, after S1
**Reads**: PopulationCensus, EnergyFieldGrid, NutrientFieldGrid
**Writes**: ReproductionPressure (new Resource per-cell)
**Run condition**: Every CENSUS_INTERVAL ticks
**Logic**:
1. Per cell: compute K from cell_qe × nutrients
2. Per cell: compute reproduction_pressure
3. This pressure modulates D7's reproduction guard and abiogenesis threshold

### S3: `ecology_succession_system` (Transformer)
**Phase**: MetabolicLayer, after S2
**Reads**: PopulationCensus, SimulationClock
**Writes**: SuccessionState (Resource)
**Run condition**: Every 60 ticks
**Logic**:
1. Track dominant trophic role per zone
2. Compute succession stage from time + composition
3. Modulate abiogenesis profiles (Pioneer → more producers, Climax → balanced)

---

## Tests

- `carrying_capacity_zero_nutrients_returns_zero`
- `carrying_capacity_high_qe_high_nutrients_returns_many`
- `reproduction_pressure_at_capacity_is_zero`
- `reproduction_pressure_empty_cell_is_one`
- `succession_starts_as_pioneer`
- `succession_reaches_climax_after_threshold`
- `abiogenesis_threshold_lower_when_empty`
