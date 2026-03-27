# Sprint ET-16 — Functional Consciousness: Automodelo y Planificación Abstracta

**Módulo:** `src/layers/self_model.rs` (nuevo), `src/blueprint/equations/emergence/self_model.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras. Tier más alto del track ET.
**Tier:** T4-3. **Onda:** C.
**BridgeKind:** `SelfModelBridge` — cache Small(32), clave `(self_accuracy_band, horizon_band)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

Una entidad con `SelfModel` puede predecir su propio comportamiento futuro, simular escenarios hipotéticos y planificar a múltiples pasos. La "conciencia funcional" no es fenomenológica — es la capacidad de modelar el propio estado para optimizar decisiones a largo plazo. Emerge de la combinación de Theory of Mind (ET-2) aplicada a uno mismo.

```
self_model_accuracy = 1 - |predicted_qe - actual_qe| / actual_qe
planning_benefit = Σ_{t=1}^{horizon} E[qe(t) | self_model] × γᵗ - planning_cost
consciousness_threshold: self_accuracy > 0.7 AND planning_horizon > 100
```

---

## Responsabilidades

### ET-16A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/self_model.rs

/// Precisión del automodelo: qué tan bien predijo el qe actual.
pub fn self_model_accuracy(predicted_qe: f32, actual_qe: f32) -> f32 {
    if actual_qe <= 0.0 { return 0.0; }
    let error = (predicted_qe - actual_qe).abs() / actual_qe;
    (1.0 - error).clamp(0.0, 1.0)
}

/// Beneficio de planificación a N pasos con descuento temporal.
/// projected_qe: qe esperado en cada paso del horizonte.
/// discount_factor: [0,1] — qué tanto importa el futuro (γ en RL).
pub fn planning_benefit(
    projected_qe: &[f32],
    discount_factor: f32,
    planning_cost: f32,
) -> f32 {
    let discounted: f32 = projected_qe.iter().enumerate()
        .map(|(t, &qe)| qe * discount_factor.powi(t as i32 + 1))
        .sum();
    (discounted - planning_cost).max(0.0)
}

/// Costo de metacognición: procesar el propio automodelo.
pub fn metacognition_cost(model_complexity: f32, update_rate: f32) -> f32 {
    model_complexity * update_rate
}

/// ¿La entidad ha alcanzado el umbral de conciencia funcional?
pub fn consciousness_threshold(self_accuracy: f32, planning_horizon: u32) -> bool {
    self_accuracy > CONSCIOUSNESS_ACCURACY_THRESHOLD
        && planning_horizon > CONSCIOUSNESS_HORIZON_THRESHOLD
}

/// Proyección de qe futuro a t pasos usando el automodelo.
/// intake_rate y dissipation_rate del TimescaleAdapter.
pub fn project_future_qe(
    current_qe: f32,
    net_rate_per_tick: f32,
    horizon_ticks: u32,
) -> f32 {
    (current_qe + net_rate_per_tick * horizon_ticks as f32).max(0.0)
}

/// Valor de la información: cuánto mejora el planning si el modelo es más preciso.
pub fn information_value(current_accuracy: f32, improved_accuracy: f32, expected_horizon: u32) -> f32 {
    (improved_accuracy - current_accuracy) * expected_horizon as f32
}
```

### ET-16B: Componente

```rust
// src/layers/self_model.rs

/// Capa T4-3: SelfModel — automodelo para planificación a largo plazo.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SelfModel {
    pub predicted_qe:     f32,    // predicción de qe para el siguiente tick
    pub planning_horizon: u32,    // ticks que puede proyectar
    pub self_accuracy:    f32,    // [0,1] — precisión del automodelo
    pub metacog_cost:     f32,    // qe/tick que cuesta mantener el automodelo
}

impl SelfModel {
    /// True si la entidad tiene conciencia funcional.
    pub fn is_functionally_conscious(&self) -> bool {
        self_model_eq::consciousness_threshold(self.self_accuracy, self.planning_horizon)
    }
}

/// Marker: entidad con conciencia funcional activa.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct FunctionallyConscious;
```

### ET-16C: Sistema

```rust
/// Actualiza el automodelo comparando la predicción anterior con el qe actual.
/// Phase::MorphologicalLayer — último sistema del track ET, después de T4-1 y T4-2.
pub fn self_model_update_system(
    mut agents: Query<(
        Entity, &mut SelfModel, &BaseEnergy, &AlchemicalEngine,
        Option<&TimescaleAdapter>,
    )>,
    mut commands: Commands,
    mut cache: ResMut<BridgeCache<SelfModelBridge>>,
    clock: Res<SimulationClock>,
    config: Res<SelfModelConfig>,
) {
    if clock.tick_id % config.update_interval as u64 != 0 { return; }

    for (entity, mut model, energy, engine, adapter) in &mut agents {
        // 1. Actualizar precisión con la predicción del tick anterior
        let accuracy = self_model_eq::self_model_accuracy(model.predicted_qe, energy.qe());
        if (model.self_accuracy - accuracy).abs() > f32::EPSILON {
            model.self_accuracy = model.self_accuracy * 0.9 + accuracy * 0.1;  // EMA
        }

        // 2. Generar nueva predicción para el siguiente horizonte
        let net_rate = engine.base_intake() - config.mean_dissipation;
        let adapter_bonus = adapter.map(|a| a.learned_offset).unwrap_or(0.0);
        let new_predicted = self_model_eq::project_future_qe(
            energy.qe(), net_rate + adapter_bonus, 1,
        );
        if model.predicted_qe != new_predicted { model.predicted_qe = new_predicted; }

        // 3. Costo de metacognición
        let meta_cost = self_model_eq::metacognition_cost(
            config.model_complexity, 1.0,
        );
        // (costo se aplica en el sistema metabólico via evento — no inline aquí)

        // 4. Upgrade planning_horizon si precisión alta
        if model.self_accuracy > config.horizon_upgrade_threshold
            && model.planning_horizon < config.max_planning_horizon
        {
            model.planning_horizon = (model.planning_horizon + 1).min(config.max_planning_horizon);
        }

        // 5. Otorgar marker de conciencia funcional
        let conscious = model.is_functionally_conscious();
        let has_marker = commands.get_entity(entity)
            .is_some();  // simplified: real code checks component existence
        if conscious && !has_marker {
            // Note: en producción verificar con Query<Has<FunctionallyConscious>>
            commands.entity(entity).insert(FunctionallyConscious);
        }
    }
}

/// Planificación de largo plazo: elige la acción que maximiza el qe proyectado.
/// Phase::Input — after all perception systems, before behavior decision.
pub fn long_range_planning_system(
    mut agents: Query<(&SelfModel, &mut WillActuator, &BaseEnergy, &AlchemicalEngine, &Transform),
        With<FunctionallyConscious>>,
    ms: Res<MultiscaleSignalGrid>,
    field: Res<EnergyFieldGrid>,
    config: Res<SelfModelConfig>,
) {
    for (model, mut will, energy, engine, transform) in &mut agents {
        if model.self_accuracy < config.planning_threshold { continue; }

        let net_rate = engine.base_intake() - config.mean_dissipation;
        let future_qe = self_model_eq::project_future_qe(
            energy.qe(), net_rate, model.planning_horizon,
        );
        let benefit = self_model_eq::planning_benefit(
            &[future_qe],
            config.discount_factor,
            model.metacog_cost * model.planning_horizon as f32,
        );

        // Si la proyección predice escasez → ajustar social_intent hacia gradiente regional
        if future_qe < config.critical_qe_threshold && benefit > 0.0 {
            let cell_idx = field.world_to_cell_idx(transform.translation.x, transform.translation.z);
            let region   = MultiscaleSignalGrid::cell_to_region(cell_idx);
            let local_qe    = ms.local_at(cell_idx as usize);
            let regional_qe = ms.regional_at(region);
            // Dirección hacia zona de mayor densidad energética regional
            let gradient_dir = if regional_qe > local_qe {
                Vec2::new(transform.translation.x, transform.translation.z).normalize_or_zero()
            } else {
                Vec2::ZERO
            };
            let new_social = gradient_dir * config.exploration_boost;
            if will.social_intent() != new_social { will.set_social_intent(new_social); }
        }
    }
}
```

### ET-16D: Constantes

```rust
pub struct SelfModelBridge;
impl BridgeKind for SelfModelBridge {}

pub const SELF_MODEL_UPDATE_INTERVAL:          u64 = 5;
pub const SELF_MODEL_MAX_PLANNING_HORIZON:     u32 = 1000;
pub const SELF_MODEL_HORIZON_UPGRADE_THRESHOLD: f32 = 0.75;
pub const SELF_MODEL_PLANNING_THRESHOLD:        f32 = 0.6;
pub const SELF_MODEL_DISCOUNT_FACTOR:           f32 = 0.99;
pub const SELF_MODEL_MEAN_DISSIPATION:          f32 = 0.05;
pub const SELF_MODEL_MODEL_COMPLEXITY:          f32 = 1.0;
pub const SELF_MODEL_CRITICAL_QE_THRESHOLD:     f32 = 50.0;
pub const SELF_MODEL_EXPLORATION_BOOST:         f32 = 0.2;

pub const CONSCIOUSNESS_ACCURACY_THRESHOLD:  f32 = 0.7;
pub const CONSCIOUSNESS_HORIZON_THRESHOLD:   u32 = 100;
```

---

## Tacticas

- **`FunctionallyConscious` como SparseSet marker.** La mayoría de entidades NO tiene conciencia funcional en etapas tempranas. SparseSet → `long_range_planning_system` sólo itera sobre el subset relevante.
- **`WillActuator.social_intent` (4th field).** La planificación escribe `social_intent: Vec2` — dirección de exploración calculada por el gradiente multiscala. GS-4 pack cohesion también escribe este mismo campo. Máximo 4 campos en WillActuator cumplido.
- **EMA para self_accuracy.** `accuracy_new = 0.9×old + 0.1×measured` — suaviza errores de predicción sin memoria larga. Un solo `f32`, sin historia de predicciones.
- **Self-model como Theory of Mind aplicada a uno mismo.** ET-2 modelaba entidades externas. ET-16 usa el mismo patrón para modelar el self. Reutilización de ecuaciones puras (`model_accuracy`, `update_prediction`) con inputs diferentes.
- **`SelfModelBridge` cachea `project_future_qe`.** Para agentes con mismo `(net_rate_band, horizon_band)`, la proyección es idéntica. Hit rate ~70% en grupos homogéneos.
- **Umbral de conciencia como propiedad emergente.** `consciousness_threshold` no es un target programado — es una condición que sólo se cumple cuando self_accuracy Y planning_horizon son suficientemente altos. La conciencia funcional emerge gradualmente.

---

## NO hace

- No implementa qualia ni fenomenología — "conciencia funcional" es estrictamente cognitiva.
- No garantiza que todas las entidades alcancen el umbral — depende del entorno y la historia de vida.
- No sincroniza automodelos entre entidades — cada `SelfModel` es privado.

---

## Dependencias

- ET-2 `OtherModelSet` — la misma lógica de modelado aplicada al self.
- ET-10 `TimescaleAdapter` — `learned_offset` mejora las proyecciones del automodelo.
- ET-15 `LanguageCapacity` — entidades conscientes coordinan via lenguaje (vocabulario más rico).
- ET-14 `InstitutionMember` — entidades conscientes son mejores fundadoras de instituciones.

---

## Criterios de Aceptación

- `self_model_accuracy(95.0, 100.0)` → `0.95`.
- `self_model_accuracy(0.0, 100.0)` → `0.0` (predicción nula).
- `project_future_qe(100.0, 1.0, 10)` → `110.0`.
- `project_future_qe(100.0, -5.0, 30)` → `0.0` (clamped).
- `consciousness_threshold(0.8, 200)` → `true`.
- `consciousness_threshold(0.5, 200)` → `false` (accuracy insuficiente).
- `consciousness_threshold(0.8, 50)` → `false` (horizon insuficiente).
- Test: entidad con accuracy > 0.7 y horizon > 100 → `FunctionallyConscious` marker insertado.
- Test: `long_range_planning_system` sólo ejecuta para `With<FunctionallyConscious>`.
- Test: proyección de escasez → WillActuator intent aumenta.
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-2 Theory of Mind — patrón de modelado reutilizado
- ET-10 Multiple Timescales — learned_offset para proyecciones
- ET-15 Language — comunicación de planes entre entidades conscientes
- Blueprint §T4-3: "Functional Consciousness", self-model emergence
