# Blueprint — Capa 15: Inferencia Táctica

**Versión:** 1.0
**Depende de:** L0 (BaseEnergy), L2 (OscillatorySignature), L5 (AlchemicalEngine), L7 (WillActuator)
**Estado:** Diseño

---

## 1. Pregunta Energética

> **¿Cómo decide la energía su próxima transformación?**

La Capa 15 generaliza el patrón `InferenceProfile → Intent → Reducer` (ya funcionando para crecimiento) a **todas las conductas**: movimiento, ramificación, depredación. No es una AI con árboles de decisión — es una función pura que mapea estímulos + perfil → intención.

---

## 2. Validación 5-Test (DESIGNING.md)

| Test | Resultado | Justificación |
|------|-----------|---------------|
| 1. ¿Responde una pregunta sobre energía? | ✅ | "¿Hacia dónde se transforma esta energía?" No es derivable de otras capas. |
| 2. ¿Se ubica en el árbol de dependencias? | ✅ | Nivel 3: depende de L0, L2, L5, L7. Produce intents que L7 (WillActuator) ejecuta. |
| 3. ¿Tipo A o B? | Tipo A | Propiedad de la entidad (no es una entidad independiente). |
| 4. ¿Obedece la segunda ley? | ✅ | Los intents son SparseSet transient — se remueven si las condiciones cambian. La energía gastada en decidir se disipa. |
| 5. ¿La interferencia lo afecta? | ✅ | Interferencia constructiva amplifica estímulos → decisiones más agresivas. Destructiva los suprime → parálisis/huida. |

---

## 3. Lo que YA Existe (base de esta capa)

```
layers/inference.rs         → InferenceProfile (4 campos), CapabilitySet (u8 flags), GrowthIntent (SparseSet)
simulation/inference_growth.rs → growth_intent_inference_system (InferenceProfile → GrowthIntent)
simulation/allometric_growth.rs → allometric_growth_system (GrowthIntent → SpatialVolume)
simulation/sensory.rs       → SensoryProfile, ArtefactoReceptor, AttentionGrid, attention_convergence_system
blueprint/equations.rs      → inferred_growth_delta() (modula delta con bias + resilience + qe_norm)
```

El patrón Inference → Intent → Reducer **ya funciona para crecimiento**. Esta capa lo extiende a movimiento, ramificación y depredación.

---

## 4. Componentes Nuevos

### 4.1 MotionIntent (SparseSet, transient)

```rust
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct MotionIntent {
    pub direction: Vec2,          // Dirección deseada (normalizada)
    pub urgency: f32,             // [0,1] → modula fuerza en WillActuator
    pub confidence: f32,          // [0,1] → calidad de la decisión
}
```

**Reducer:** `motion_intent_reducer_system` → escribe `WillActuator.movement_intent` si `CapabilitySet::MOVE`.

### 4.2 BranchIntent (SparseSet, transient)

```rust
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct BranchIntent {
    pub preferred_direction: Vec3, // Hacia dónde ramificar
    pub budget_fraction: f32,      // [0,1] de biomasa a invertir
    pub confidence: f32,           // [0,1]
}
```

**Reducer:** `branch_intent_reducer_system` → modula `GeometryInfluence.energy_direction` en el siguiente rebuild GF1.

### 4.3 Ecuación Pura: `infer_motion_intent`

```rust
/// Stateless: estímulos + perfil → intención de movimiento.
pub fn infer_motion_intent(
    energy_gradient: Vec2,      // ∇E del campo (hacia mayor energía)
    threat_direction: Vec2,     // Dirección de amenaza (interferencia destructiva)
    mobility_bias: f32,         // InferenceProfile.mobility_bias
    resilience: f32,            // InferenceProfile.resilience
    qe_norm: f32,               // Energía normalizada [0,1]
) -> (Vec2, f32) {              // (dirección, urgencia)
    // Alta resiliencia → resiste amenazas (no huye)
    // Alta movilidad → responde más rápido al gradiente
    // Baja energía → urgencia sube (busca energía)
    let flee_weight = (1.0 - resilience) * threat_direction.length();
    let seek_weight = mobility_bias * (1.0 - qe_norm);
    let combined = energy_gradient * seek_weight - threat_direction * flee_weight;
    let urgency = combined.length().min(1.0);
    let direction = if urgency > 0.01 { combined.normalize() } else { Vec2::ZERO };
    (direction, urgency)
}
```

### 4.4 Ecuación Pura: `infer_branch_intent`

```rust
/// Stateless: biomasa + gradiente + perfil → intención de ramificación.
pub fn infer_branch_intent(
    biomass_available: f32,
    energy_gradient: Vec3,      // Dirección de mayor energía (3D)
    branching_bias: f32,        // InferenceProfile.branching_bias
    branch_threshold: f32,      // Constante: biomasa mínima para ramificar
) -> (Vec3, f32) {              // (dirección, fracción_de_biomasa)
    if biomass_available < branch_threshold { return (Vec3::ZERO, 0.0); }
    let fraction = branching_bias * (biomass_available - branch_threshold).min(1.0);
    (energy_gradient.normalize_or_zero(), fraction)
}
```

---

## 5. Sistemas

### 5.1 `tactical_inference_system` (Phase::MorphologicalLayer)

Una sola función pura que infiere TODOS los intents para una entidad:

```rust
Query<(
    Entity,
    &InferenceProfile,
    &CapabilitySet,
    &BaseEnergy,
    &Transform,
    Option<&GrowthBudget>,
    Option<&FlowVector>,
)>
```

- Si `can_grow` + `GrowthBudget` → ya cubierto por `growth_intent_inference_system`
- Si `can_move` → calcula `infer_motion_intent()` → inserta `MotionIntent`
- Si `can_branch` + `biomass > threshold` → calcula `infer_branch_intent()` → inserta `BranchIntent`

### 5.2 `motion_intent_reducer_system` (Phase::MorphologicalLayer, después de inference)

```rust
Query<(&MotionIntent, &CapabilitySet, &mut WillActuator)>
```

Escribe `will.movement_intent = intent.direction * intent.urgency`.

### 5.3 `branch_intent_reducer_system` (Update, antes de shape_color_inference)

Modifica `GeometryInfluence.energy_direction` para influir la dirección de la próxima rama GF1.

---

## 6. Tabla de Comportamiento Emergente por Perfil

| InferenceProfile | Comportamiento |
|---|---|
| `growth=1.0, mobility=0.0, branch=0.8, resilience=0.9` | **Árbol**: crece alto, ramifica mucho, no se mueve, resiste daño |
| `growth=0.6, mobility=0.0, branch=0.3, resilience=0.4` | **Hierba**: crece moderado, pocas ramas, frágil |
| `growth=0.3, mobility=0.8, branch=0.0, resilience=0.6` | **Animal**: crece poco, se mueve mucho, no ramifica |
| `growth=0.5, mobility=0.5, branch=0.7, resilience=0.3` | **Híbrido planta-animal**: crece, se mueve, ramifica, frágil |
| `growth=0.0, mobility=1.0, branch=0.0, resilience=0.9` | **Depredador**: no crece, se mueve rápido, resiste |

**Sin tags.** Mismo pipeline, diferente vector de 4 pesos.

---

## 7. Coherencia cursor/rules

| Regla | Cumplimiento |
|---|---|
| Max 4 campos | ✅ MotionIntent=3, BranchIntent=3, InferenceProfile=4 (ya existe) |
| SparseSet transient | ✅ MotionIntent, BranchIntent |
| Math en equations.rs | ✅ `infer_motion_intent()`, `infer_branch_intent()` |
| No god-systems | ✅ Inference y reducers son sistemas separados |
| Changed<T> | ✅ Reducers usan `With<MotionIntent>` (presence filter) |
| No valores derivados | ✅ Intents son transient, no persistidos |
| Phase assignment | ✅ Todo en MorphologicalLayer |

---

## 8. NO Hace

- No crea un "cerebro" o behavior tree — es una función pura.
- No almacena memoria ni historial de decisiones.
- No agrega crates (no ML, no GOAP).
- No modifica InferenceProfile en runtime (es inmutable, como bond_energy).
- No reemplaza WillActuator — lo alimenta con intents.
- 5-7 parámetros maestros, no 40.

---

## 9. Referencias

- `DESIGNING.md` — 5-Test para capas nuevas
- `src/layers/inference.rs` — InferenceProfile, CapabilitySet, GrowthIntent (existentes)
- `src/simulation/inference_growth.rs` — Patrón inference → reducer (existente)
- `src/simulation/sensory.rs` — AttentionGrid, transducción (existente)
- `src/blueprint/equations.rs` — `inferred_growth_delta()` (existente)
