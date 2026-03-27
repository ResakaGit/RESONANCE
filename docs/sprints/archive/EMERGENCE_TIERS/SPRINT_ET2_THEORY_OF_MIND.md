# Sprint ET-2 — Theory of Mind: Modelo Interno del Otro

**Módulo:** `src/layers/other_model.rs` (nuevo), `src/blueprint/equations/emergence/other_model.rs` (nuevo)
**Tipo:** Nueva capa (L15) + ecuaciones puras.
**Tier:** T1-2 — Individual Adaptation. **Onda:** A.
**BridgeKind:** `OtherModelBridge` — cache Small(128), clave `(modeler_id, target_id, tick/interval)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: que ya existe

- ET-1 `AssociativeMemory` — recuerda outcomes pasados. ToM requiere ET-1 para calibrar modelos.
- `layers/inference.rs::SensoryAwareness` — lista de entidades detectadas. Son los candidatos a modelar.
- `simulation/behavior.rs::BehaviorMode` — el discriminante que se predice.
- `layers/oscillatory.rs::OscillatorySignature` — observable que permite inferir estado.

**Lo que NO existe:**
1. Modelo interno de la frecuencia/energía/modo de otro agente.
2. Predicción del comportamiento futuro de un competidor.
3. Costo metabólico de mantener modelos de otros.
4. Valor de la deception: emitir señales que descalibren el modelo del rival.

---

## Objetivo

Un agente con ToM puede interceptar flujos de energía antes de que lleguen los competidores. Modela el estado de los rivales y predice sus movimientos. El costo de mantener el modelo se paga con qe; el beneficio es qe capturado por anticipación.

```
model_accuracy = 1 - |predicted_state - actual_state| / max_deviation
deception_value = E[misprediction_benefit] - E[false_signal_cost]
net_model_value = E[energy_intercepted] - model_maintenance_cost
```

---

## Responsabilidades

### ET-2A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/other_model.rs

/// Precisión del modelo: 1 = predicción perfecta, 0 = completamente errado.
/// predicted_freq: qué frecuencia predijo el modelador.
/// actual_freq: frecuencia real del target este tick.
pub fn model_accuracy(predicted_freq: f32, actual_freq: f32, max_freq_deviation: f32) -> f32 {
    let error = (predicted_freq - actual_freq).abs();
    (1.0 - error / max_freq_deviation.max(f32::EPSILON)).clamp(0.0, 1.0)
}

/// Actualiza la predicción del modelo con un error observado.
/// Gradiente de aprendizaje: learning_rate × error → ajuste de la predicción.
pub fn update_prediction(
    current_prediction: f32,
    actual_value: f32,
    learning_rate: f32,
) -> f32 {
    current_prediction + learning_rate * (actual_value - current_prediction)
}

/// Costo de mantener un modelo de otro agente. Escala con complejidad del modelo.
pub fn model_maintenance_cost(accuracy: f32, base_cost: f32) -> f32 {
    // Modelos más precisos son más caros de mantener
    base_cost * (1.0 + accuracy)
}

/// Valor de la deception: cuánta energía gana A si B tiene un modelo incorrecto de A.
/// misprediction_magnitude: cuán equivocado está B sobre A.
pub fn deception_value(
    misprediction_magnitude: f32,
    energy_at_stake: f32,
    false_signal_cost: f32,
) -> f32 {
    misprediction_magnitude * energy_at_stake - false_signal_cost
}

/// ¿Vale mantener el modelo? Retorna true si el modelo es energéticamente rentable.
pub fn is_model_worth_maintaining(
    expected_interception: f32,
    maintenance_cost: f32,
) -> bool {
    expected_interception > maintenance_cost
}
```

### ET-2B: Componente

```rust
// src/layers/other_model.rs

/// Un modelo interno de otro agente. Dato inmutable en runtime — no Component directo.
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct OtherModel {
    pub target_id:    u32,   // WorldEntityId del agente modelado
    pub predicted_freq: f32, // frecuencia predicha del target
    pub accuracy:     f32,   // [0,1] precisión histórica del modelo
    pub update_cost:  f32,   // qe gastado en actualizar este tick
}

/// Capa T1-2: OtherModelSet — conjunto de modelos internos de otros agentes.
/// Array fijo — max 4 campos, max MAX_MODELS modelos activos.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct OtherModelSet {
    pub models:          [OtherModel; MAX_MODELS],
    pub model_count:     u8,
    pub update_interval: u8,    // ticks entre actualizaciones (evita re-calcular cada tick)
    pub base_model_cost: f32,   // qe/tick por modelo mantenido
}

pub const MAX_MODELS: usize = 4;
```

### ET-2C: Sistema

```rust
/// Actualiza modelos internos de otros agentes. Corre cada update_interval ticks.
/// Phase::Input, in_set(EmergenceTier1Set), after associative_memory_update_system.
pub fn theory_of_mind_update_system(
    mut modelers: Query<(
        &WorldEntityId,
        &mut OtherModelSet,
        &mut BaseEnergy,
        &SensoryAwareness,
    ), With<BehavioralAgent>>,
    targets: Query<(&WorldEntityId, &OscillatorySignature, &BaseEnergy)>,
    clock: Res<SimulationClock>,
    config: Res<OtherModelConfig>,
) {
    for (self_id, mut model_set, mut energy, awareness) in &mut modelers {
        // Sólo actualizar cada N ticks
        if clock.tick_id % model_set.update_interval as u64 != 0 { continue; }

        let mut total_cost = 0.0f32;

        for i in 0..model_set.model_count as usize {
            let model = &model_set.models[i];
            // Buscar el target en el mundo
            let Some((_, osc, _)) = targets.iter()
                .find(|(id, _, _)| id.0 == model.target_id) else { continue; };

            let actual_freq = osc.frequency_hz();
            let accuracy = other_model_eq::model_accuracy(
                model.predicted_freq, actual_freq, config.max_freq_deviation,
            );
            let new_pred = other_model_eq::update_prediction(
                model.predicted_freq, actual_freq, config.learning_rate,
            );
            let cost = other_model_eq::model_maintenance_cost(accuracy, model_set.base_model_cost);

            model_set.models[i].accuracy = accuracy;
            model_set.models[i].predicted_freq = new_pred;
            model_set.models[i].update_cost = cost;
            total_cost += cost;
        }

        // Poda de modelos no rentables
        let mut count = model_set.model_count as usize;
        let mut i = 0;
        while i < count {
            let m = &model_set.models[i];
            if !other_model_eq::is_model_worth_maintaining(
                m.accuracy * config.base_interception_value,
                m.update_cost,
            ) {
                model_set.models[i] = model_set.models[count - 1];
                model_set.models[count - 1] = OtherModel::default();
                count -= 1;
            } else { i += 1; }
        }
        model_set.model_count = count as u8;

        // Drain de qe
        let new_qe = (energy.qe() - total_cost).max(0.0);
        if energy.qe() != new_qe { energy.set_qe(new_qe); }
    }
}
```

### ET-2D: Constantes y BridgeKind

```rust
// src/bridge/config.rs
pub struct OtherModelBridge;
impl BridgeKind for OtherModelBridge {}

// src/blueprint/constants/emergence/other_model.rs
pub const OTHER_MODEL_DEFAULT_UPDATE_INTERVAL: u8 = 5;   // actualizar cada 5 ticks
pub const OTHER_MODEL_DEFAULT_BASE_COST: f32 = 0.2;       // qe/tick por modelo
pub const OTHER_MODEL_LEARNING_RATE: f32 = 0.1;
pub const OTHER_MODEL_MAX_FREQ_DEVIATION: f32 = 500.0;    // Hz — banda máxima
pub const OTHER_MODEL_BASE_INTERCEPTION_VALUE: f32 = 5.0; // qe esperado por uso del modelo

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct OtherModelConfig {
    pub learning_rate: f32,
    pub max_freq_deviation: f32,
    pub base_interception_value: f32,
}
```

---

## Tacticas

- **update_interval como throttle.** `update_interval = 5` → el sistema corre 1/5 de los ticks. Drástica reducción de carga. El BridgeCache cubre el 4/5 de los ticks que no actualizan.
- **BridgeCache clave temporal.** `(modeler_id, target_id, tick_id / update_interval)` — misma clave durante todo el intervalo. Cache hit = 80% de los ticks.
- **Deception como señal física.** Un agente puede emitir frecuencia falsa modificando su `OscillatorySignature` temporalmente. El costo es el drift de homeostasis (L12 ya existe). ET-2 no necesita implementar deception — emerge del sistema de frecuencias existente.
- **Poda por rentabilidad.** Modelos de agentes que ya no están en rango sensorial pierden rentabilidad y son podados automáticamente.

---

## NO hace

- No implementa modelos de grupos (sólo individuos). Coaliciones = GS-4 + ET-8.
- No modifica OscillatorySignature directamente — la deception emerge de Homeostasis.
- No predice el intent (WillActuator) — sólo predice estado observable (freq, qe).

---

## Dependencias

- ET-1 `AssociativeMemory` — calibración de modelos desde experiencias pasadas.
- `layers/oscillatory.rs::OscillatorySignature` — el observable que se predice.
- `layers/inference.rs::SensoryAwareness` — candidatos a modelar.
- `bridge/config.rs` — `OtherModelBridge`.

---

## Criterios de Aceptación

### ET-2A (Ecuaciones)
- `model_accuracy(440.0, 440.0, 500.0)` → `1.0`.
- `model_accuracy(440.0, 940.0, 500.0)` → `0.0`.
- `model_accuracy(440.0, 540.0, 500.0)` → `0.8`.
- `update_prediction(440.0, 500.0, 0.1)` → `446.0`.
- `is_model_worth_maintaining(5.0, 3.0)` → `true`.
- `is_model_worth_maintaining(1.0, 3.0)` → `false`.
- Determinismo: mismas entradas → mismo resultado.

### ET-2C (Sistema)
- Test (MinimalPlugins): modelo con accuracy alta y target frecuente → modelo persiste.
- Test: modelo con `expected_interception < maintenance_cost` → podado.
- Test: sistema no corre en ticks donde `tick_id % interval != 0`.
- Test: qe drain correcto tras actualización.

### General
- `cargo test --lib` sin regresión. Sin Vec/Box en componentes.

---

## Referencias

- ET-1 `AssociativeMemory` — fundación de calibración
- `src/layers/inference.rs` — SensoryAwareness
- `src/bridge/config.rs` — patrón BridgeKind
- Blueprint §T1-2: "Theory of Mind", ecuaciones de prediction_accuracy
