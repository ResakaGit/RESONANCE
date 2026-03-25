# D3: Locomotion Energy Cost

**Prioridad**: P1
**Phase**: `Phase::AtomicLayer` (con physics)
**Dependencias**: L3 (FlowVector), L5 (AlchemicalEngine), L6 (AmbientPressure), topology
**Systems**: 3

---

## Motivación Científica

En biología, la locomoción es el mayor gasto energético después del metabolismo basal. Un animal que corre gasta energía proporcional a masa × velocidad² (energía cinética). El terreno modula: cuesta arriba cuesta más, agua resiste más, hielo es inestable.

Actualmente, `movement_will_drag_system` aplica arrastre al velocity pero NO drena energía del buffer (L5). Moverse es "gratis" — anti-termodinámico.

---

## Ecuaciones Nuevas

```
src/blueprint/equations/locomotion/mod.rs (NUEVO)
```

### E1: `locomotion_energy_cost(mass: f32, speed: f32, terrain_factor: f32) -> f32`
```
E_locomotion = KINETIC_FACTOR × mass × speed² × terrain_factor
```
Donde mass = density × volume = qe / (4/3 π r³) × (4/3 π r³) ≈ qe (simplificación: qe IS mass-energy).

### E2: `terrain_locomotion_factor(slope: f32, viscosity: f32, matter_state: MatterState) -> f32`
```
f_terrain = (1 + slope × SLOPE_COST_SCALE) × viscosity × state_multiplier
  state_multiplier: Solid=1.0, Liquid=1.5, Gas=0.8, Plasma=2.0
```

### E3: `stamina_recovery_rate(current_buffer: f32, max_buffer: f32) -> f32`
```
recovery = BASE_RECOVERY × (current_buffer / max_buffer)²
```
Más vacío el tanque → más lento recupera.

---

## Constantes

```
src/blueprint/constants/locomotion.rs (NUEVO)
```

```rust
pub const LOCOMOTION_KINETIC_FACTOR: f32 = 0.002;    // qe per unit of speed²
pub const SLOPE_COST_SCALE: f32 = 1.5;                // Cuesta arriba ×2.5 a 45°
pub const LOCOMOTION_MIN_SPEED_THRESHOLD: f32 = 0.1;  // Below this, no cost
pub const STAMINA_BASE_RECOVERY: f32 = 0.5;           // qe/s base recovery
pub const SPRINT_COST_MULTIPLIER: f32 = 3.0;          // Sprint costs 3× normal
```

---

## Systems (3)

### S1: `locomotion_energy_drain_system` (Transformer)
**Phase**: AtomicLayer, after movement_will_drag_system
**Reads**: FlowVector, AlchemicalEngine, SpatialVolume, BaseEnergy
**Writes**: AlchemicalEngine (drain buffer), BaseEnergy (drain if buffer empty)
**Logic**:
1. `speed = flow.velocity().length()`
2. If speed < LOCOMOTION_MIN_SPEED_THRESHOLD → skip
3. `cost = locomotion_energy_cost(energy.qe(), speed, terrain_factor)`
4. `engine.consume(cost × dt)` — drain from buffer first
5. If buffer empty → drain from BaseEnergy (emergency metabolic burn)

### S2: `locomotion_terrain_modulation_system` (Transformer)
**Phase**: AtomicLayer, before S1
**Reads**: Transform, TerrainField (Res), AmbientPressure (optional)
**Writes**: Local terrain factor cache
**Logic**: Calcula terrain_factor desde slope + viscosity + matter_state de la celda.

### S3: `locomotion_exhaustion_system` (Emitter)
**Phase**: AtomicLayer, after S1
**Reads**: AlchemicalEngine, FlowVector, BehavioralAgent
**Emits**: (modifica BehaviorCooldown → forces rest)
**Logic**: Si buffer < 5% AND speed > 0 → force Idle for N ticks.

---

## Tests

- `locomotion_cost_zero_speed_returns_zero`
- `locomotion_cost_doubles_with_sqrt2_speed`
- `terrain_factor_uphill_costs_more`
- `terrain_factor_liquid_higher_than_solid`
- `exhaustion_forces_idle_when_buffer_empty`
