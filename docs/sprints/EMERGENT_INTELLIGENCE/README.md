# Track: EMERGENT_INTELLIGENCE — Decisiones y predicción emergentes

Red nerviosa + SelfModel + OtherModelSet → organismos que predicen y planifican.
"Inteligencia" = calidad de predicción × horizonte de planificación.

**Invariante:** Ningún comportamiento inteligente se programa. Emerge de signal quality + memory.

---

## Qué ya existe

| Componente | Archivo | Uso en EI |
|-----------|---------|-----------|
| `SelfModel` (4 fields) | `layers/self_model.rs` | predicted_qe, planning_horizon, self_accuracy, metacog_cost |
| `OtherModelSet` (4 models) | `layers/other_model.rs` | Predicción de vecinos (theory of mind) |
| `theory_of_mind_update_system` | `simulation/emergence/` | Actualiza predicciones de otros |
| `BehaviorIntent` | `layers/behavior.rs` | mode + target_entity |
| `BehavioralAgent` | `layers/behavior.rs` | Marker: tiene AI |
| `cooperation_evaluation_system` | `simulation/cooperation.rs` | Nash alliance detection |

## Sprints (3)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [EI-1](SPRINT_EI1_PREDICTION_QUALITY.md) | Prediction Quality | 1 sem | NS ✅ | `prediction_error()` — mide accuracy de SelfModel |
| [EI-2](SPRINT_EI2_PLANNING_HORIZON.md) | Planning Horizon | 1 sem | EI-1 | `plan_action()` — elige acción que maximiza qe futuro predicho |
| [EI-3](SPRINT_EI3_BATCH_WIRING.md) | Batch Integration | 1 sem | EI-2 | Predicción + planning en batch + intelligence_score observable |

## Arquitectura de archivos

```
src/blueprint/equations/
├── intelligence.rs              ← EI-1/2: prediction_error, plan_action, intelligence_score
src/blueprint/constants/
├── intelligence.rs              ← Constantes: planning_cost, prediction_decay
src/batch/systems/
├── intelligence.rs              ← EI-3: batch system
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 4 | Predecir cuesta energía (metacog_cost en SelfModel). Planificar más lejos cuesta más. |
| 6 | Inteligencia emerge: organismos que predicen mejor sobreviven más. |
| 8 | Predicción usa frequency alignment: predecir entidades con freq similar es más fácil. |

## Constantes derivadas

| Constante | Derivación |
|-----------|-----------|
| `PREDICTION_COST_PER_TICK` | `DISSIPATION_SOLID × 5` — costo de mantener predicción |
| `PLANNING_COST_PER_HORIZON` | `DISSIPATION_SOLID × 10` — costo por tick de horizonte |
| `PREDICTION_DECAY_RATE` | `DISSIPATION_SOLID × 20` — accuracy decae si no se actualiza |
| `INTELLIGENCE_SCORE_SCALE` | `1.0 / KLEIBER_EXPONENT` — normalización |

## Esfuerzo total: ~3 semanas, ~300 LOC, ~35 tests
