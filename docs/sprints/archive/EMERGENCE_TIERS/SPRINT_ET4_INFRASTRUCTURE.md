# Sprint ET-4 — Infrastructure: Modificación Persistente del Campo

**Módulo:** `src/simulation/emergence/infrastructure.rs` (nuevo), `src/blueprint/equations/emergence/infrastructure.rs` (nuevo)
**Tipo:** Ecuaciones puras + sistema de modificación de EnergyFieldGrid.
**Tier:** T1-4. **Onda:** B.
**BridgeKind:** `FieldModBridge` — cache Small(32), clave `(cell_idx, tick/decay_period)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: que ya existe

- `worldgen/EnergyFieldGrid` — grid 32×32 mutable con `drain_cell` / `cell_qe`. El target de modificación ya existe.
- ET-3 `CulturalMemory` — grupos con comportamientos compartidos son los que construyen infraestructura colectivamente.
- `blueprint/constants/node_control.rs` — patrón de modificación de campo (GS-6).

**Lo que NO existe:**
1. Modificación persistente del campo por actividad acumulada (no sólo extracción).
2. Amplificación de intake para entidades que usan la infraestructura.
3. Decay de la infraestructura sin mantenimiento (entropía de la estructura).
4. `InfrastructureNode` — marcador de celda modificada.

---

## Objetivo

Entidades que invierten qe en una celda modifican su `EnergyFieldGrid` persistentemente. La modificación amplifica el intake para las entidades que pasen por esa zona. Sin mantenimiento, la infraestructura decae. El mundo recuerda a sus habitantes.

```
dField(x)/dt = Σ E_invested(x) × mod_rate - decay_rate × Field_delta(x)
intake_amplifier(x) = 1 + Field_delta(x) × amplification_factor
```

---

## Responsabilidades

### ET-4A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/infrastructure.rs

/// Delta de campo producido por inversión energética en una celda.
pub fn field_modification_delta(invested_qe: f32, modification_rate: f32) -> f32 {
    invested_qe * modification_rate
}

/// Decay de la modificación acumulada en una celda (entropía de infraestructura).
pub fn field_modification_decay(current_delta: f32, decay_rate: f32) -> f32 {
    current_delta * (1.0 - decay_rate).max(0.0)
}

/// Amplificación de intake para una entidad en una celda con infraestructura.
pub fn infrastructure_intake_amplifier(field_delta: f32, amplification_factor: f32) -> f32 {
    1.0 + (field_delta * amplification_factor).clamp(0.0, MAX_INFRASTRUCTURE_AMPLIFIER)
}

/// ¿Vale la pena invertir en esta celda?
/// expected_uses: cuántas veces se usará la infraestructura.
/// use_benefit: qe ganado por uso.
pub fn infrastructure_roi(
    investment_cost: f32,
    expected_uses: f32,
    use_benefit: f32,
    maintenance_per_tick: f32,
    horizon_ticks: u32,
) -> f32 {
    let total_benefit = expected_uses * use_benefit;
    let total_cost = investment_cost + maintenance_per_tick * horizon_ticks as f32;
    total_benefit - total_cost
}
```

### ET-4B: Tipos

```rust
// src/simulation/emergence/infrastructure.rs

/// Resource: mapa de modificaciones de infraestructura por celda.
/// Vec indexado por cell_idx — igual que EnergyFieldGrid.
#[derive(Resource, Default, Debug)]
pub struct InfrastructureGrid {
    pub modifications: Vec<f32>,  // delta_qe acumulado por celda (init: vec![0.0; 32*32])
    pub decay_rate: f32,
    pub amplification_factor: f32,
}

impl InfrastructureGrid {
    pub fn cell_delta(&self, cell_idx: u32) -> f32 {
        self.modifications.get(cell_idx as usize).copied().unwrap_or(0.0)
    }
    pub fn add_modification(&mut self, cell_idx: u32, delta: f32) {
        if let Some(cell) = self.modifications.get_mut(cell_idx as usize) {
            *cell = (*cell + delta).min(MAX_INFRASTRUCTURE_DELTA);
        }
    }
}

/// Evento: una entidad invierte qe en infraestructura de una celda.
#[derive(Event, Debug, Clone)]
pub struct InfrastructureInvestEvent {
    pub investor: Entity,
    pub cell_idx: u32,
    pub qe_invested: f32,
    pub tick_id: u64,
}
```

### ET-4C: Sistemas

```rust
/// Aplica inversiones de infraestructura y decae modificaciones existentes.
/// Phase::MetabolicLayer — after node_control, before victory_check.
pub fn infrastructure_update_system(
    mut infra: ResMut<InfrastructureGrid>,
    mut events: EventReader<InfrastructureInvestEvent>,
    config: Res<InfrastructureConfig>,
) {
    // Aplicar inversiones este tick
    for ev in events.read() {
        let delta = infra_eq::field_modification_delta(ev.qe_invested, config.modification_rate);
        infra.add_modification(ev.cell_idx, delta);
    }

    // Decay de toda la infraestructura (entropía)
    for cell in infra.modifications.iter_mut() {
        *cell = infra_eq::field_modification_decay(*cell, infra.decay_rate);
    }
}

/// Aplica amplificación de intake a entidades en celdas con infraestructura.
/// Phase::MetabolicLayer — after infrastructure_update_system.
pub fn infrastructure_intake_bonus_system(
    mut agents: Query<(&Transform, &mut AlchemicalEngine)>,
    infra: Res<InfrastructureGrid>,
    field: Res<EnergyFieldGrid>,
    config: Res<InfrastructureConfig>,
) {
    for (transform, mut engine) in &mut agents {
        let cell_idx = field.world_to_cell_idx(transform.translation.x, transform.translation.z);
        let delta = infra.cell_delta(cell_idx);
        if delta < INFRASTRUCTURE_MIN_ACTIVE_DELTA { continue; }
        let amp = infra_eq::infrastructure_intake_amplifier(delta, infra.amplification_factor);
        let boosted = engine.base_intake() * amp;
        if engine.intake() != boosted { engine.set_intake(boosted); }
    }
}
```

### ET-4D: Constantes

```rust
pub struct FieldModBridge;
impl BridgeKind for FieldModBridge {}

pub const INFRA_DEFAULT_MODIFICATION_RATE: f32 = 0.05;
pub const INFRA_DEFAULT_DECAY_RATE: f32 = 0.001;   // lento — infraestructura persiste
pub const INFRA_DEFAULT_AMPLIFICATION_FACTOR: f32 = 0.002;
pub const MAX_INFRASTRUCTURE_DELTA: f32 = 100.0;
pub const MAX_INFRASTRUCTURE_AMPLIFIER: f32 = 2.0; // max 2× intake
pub const INFRASTRUCTURE_MIN_ACTIVE_DELTA: f32 = 0.1;
```

---

## Tacticas

- **InfrastructureGrid como Resource paralelo a EnergyFieldGrid.** No toca la grid existente — la modifica con una capa adicional. El intake bonus se aplica al `AlchemicalEngine`, no al campo.
- **BridgeCache para infrastructure_intake_amplifier.** El `cell_delta` cambia lentamente (decay pequeño). Cache con key `(cell_idx, delta_band)` tiene hit rate alto.
- **Decay garantiza entropía.** Sin mantenimiento, toda infraestructura colapsa. El "olvido" del mundo es físico, no una regla.
- **InfrastructureInvestEvent como puente ET-4 ↔ ET-14 (Institutions).** Las instituciones coordinan inversión colectiva vía este evento. Desacoplamiento limpio.

---

## NO hace

- No genera visuales de infraestructura — eso es GS-7 (VisualHints puede usar `cell_delta`).
- No implementa "construcción" explícita — cualquier inversión de qe en una celda crea infraestructura.
- No trackea quién construyó qué — el campo es impersonal.

---

## Dependencias

- ET-3 `CulturalMemory` — grupos con cultura compartida coordinan inversión.
- `worldgen/EnergyFieldGrid` — referencia espacial para `world_to_cell_idx`.
- `layers/engine.rs::AlchemicalEngine` — receptor del intake bonus.
- `simulation/game_loop.rs::VictoryNucleus` — el núcleo puede recibir infraestructura bonus.

---

## Criterios de Aceptación

### ET-4A
- `field_modification_delta(10.0, 0.05)` → `0.5`.
- `field_modification_decay(100.0, 0.001)` → `99.9`.
- `field_modification_decay(100.0, 1.0)` → `0.0`.
- `infrastructure_intake_amplifier(50.0, 0.002)` → `1.1` (capped at 2.0).
- `infrastructure_roi(10.0, 20.0, 1.0, 0.1, 100)` → `10.0` (break-even).

### ET-4C
- Test: `InfrastructureInvestEvent` → `cell_delta` aumenta.
- Test: N ticks sin inversión → `cell_delta` decae hacia 0.
- Test: entidad en celda con delta > MIN → `AlchemicalEngine.intake()` amplificado.
- Test: entidad en celda sin infraestructura → intake sin cambio.

### General
- `cargo test --lib` sin regresión. `InfrastructureGrid` es Vec inicializado en startup.

---

## Referencias

- ET-3 Cultural Transmission — coordinación de inversión
- `src/worldgen/field_grid.rs` — EnergyFieldGrid
- `docs/sprints/GAMEPLAY_SYSTEMS/SPRINT_GS6_MAP_ENERGY.md` — patrón similar de modificación de campo
- Blueprint §T1-4: "Infrastructure / Field Modification"
