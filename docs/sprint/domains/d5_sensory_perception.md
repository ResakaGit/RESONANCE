# D5: Sensory & Perception

**Prioridad**: P2
**Phase**: `Phase::Input` (before D1 behavior decision)
**Dependencias**: L2 (OscillatorySignature), SpatialIndex, EcoField
**Systems**: 3

---

## Motivación Científica

La percepción sensorial determina qué información tiene el animal para tomar decisiones. Tres modalidades dominantes en ecología:

1. **Visual**: Detectar entidades por distancia + línea de visión (ya parcial en fog_of_war)
2. **Química/Olfato**: Detectar frecuencias (en Resonance: OscillatorySignature) a distancia sin LOS
3. **Vibración/Sonido**: Detectar movimiento (FlowVector ≠ 0) por perturbación del campo

En Resonance, la "frecuencia" es la modalidad universal de detección. Un depredador detecta la frecuencia de la presa; la presa detecta la frecuencia del depredador.

---

## Componentes Nuevos

```
src/layers/sensory.rs (extender existente o nuevo)
```

### C1: SensoryAwareness (4 fields)
```rust
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct SensoryAwareness {
    pub nearest_threat: Option<Entity>,
    pub nearest_food: Option<Entity>,
    pub threat_level: f32,
    pub food_proximity: f32,
}
```

### C2: ThreatMemory (2 fields)
```rust
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct ThreatMemory {
    pub last_threat_position: Vec2,
    pub ticks_since_seen: u32,
}
```

---

## Ecuaciones Nuevas

```
src/blueprint/equations/sensory/mod.rs (NUEVO)
```

### E1: `frequency_detection_range(sensitivity: f32, emitter_qe: f32, noise_floor: f32) -> f32`
```
range = sensitivity × sqrt(emitter_qe / noise_floor)
```
Más energía emite → más lejos se detecta. Más sensibilidad → más rango.

### E2: `threat_level_assessment(entity_qe: f32, entity_speed: f32, is_predator: bool, distance: f32) -> f32`
```
threat = (qe / REFERENCE_QE) × (1 + speed × SPEED_THREAT_SCALE) × predator_factor / (1 + distance)
```

### E3: `food_attractiveness(entity_qe: f32, distance: f32, hunger: f32) -> f32`
```
attractiveness = (qe × hunger) / (1 + distance²)
```

---

## Systems (3)

### S1: `sensory_frequency_scan_system` (Transformer)
**Phase**: Input, before behavior_assess_needs
**Reads**: Transform, OscillatorySignature, SpatialIndex, InferenceProfile (SENSE capability)
**Writes**: SensoryAwareness
**Throttle**: Cursor, 128/frame
**Logic**:
1. Query SpatialIndex in `frequency_detection_range()`
2. For each detected: classify as threat or food by TrophicRole + Faction
3. Compute threat_level, food_proximity
4. Store nearest of each

### S2: `sensory_threat_memory_system` (Transformer)
**Phase**: Input, after S1
**Reads**: SensoryAwareness, Transform
**Writes**: ThreatMemory
**Logic**:
1. If threat detected → update position, reset ticks_since_seen
2. If no threat → increment ticks_since_seen
3. If ticks_since_seen > MEMORY_DECAY_TICKS → clear memory

### S3: `sensory_awareness_event_system` (Emitter)
**Phase**: Input, after S2
**Reads**: SensoryAwareness (Changed)
**Emits**: ThreatDetectedEvent
**Logic**: Si threat_level cruza PANIC_THRESHOLD → emit event para D1.

---

## Tests

- `detection_range_scales_with_qe`
- `detection_range_zero_sensitivity_returns_zero`
- `threat_level_predator_at_close_range_is_high`
- `food_attractiveness_scales_with_hunger`
- `threat_memory_decays_over_time`
