# D8: Morphological Adaptation

**Prioridad**: P2
**Phase**: `Phase::MorphologicalLayer`
**Dependencias**: D4 (Homeostasis), organ_inference, lifecycle, morphogenesis equations
**Systems**: 3

---

## Motivación Científica

La adaptación morfológica es la respuesta fenotípica a presión ambiental:

- **Regla de Bergmann**: Animales en climas fríos son más grandes (menos superficie relativa → menos pérdida de calor)
- **Regla de Allen**: Extremidades más cortas en climas fríos
- **Ley constructal (Bejan)**: La forma optimiza el flujo de energía
- **Plasticidad fenotípica**: Dentro de una vida, la forma responde al uso (huesos más densos bajo carga)

En Resonance, el MetabolicGraph + MorphogenesisShapeParams + OrganManifest ya implementan gran parte de esto. Lo que falta es el **feedback loop**: condiciones ambientales → presión → modificación de InferenceProfile → re-inferencia de órganos.

---

## Ecuaciones Nuevas

```
src/blueprint/equations/morpho_adaptation/mod.rs (NUEVO)
```

### E1: `bergmann_radius_pressure(t_env: f32, t_target: f32, current_radius: f32) -> f32`
```
thermal_stress = (t_target - t_env).max(0.0) / t_target
pressure = thermal_stress × BERGMANN_GROWTH_SCALE
// Positive pressure = should grow larger
```

### E2: `allen_appendage_pressure(t_env: f32, t_target: f32) -> f32`
```
cold_stress = (t_target - t_env).max(0.0) / t_target
pressure = -cold_stress × ALLEN_LIMB_REDUCTION_SCALE
// Negative = reduce limb/fin scale
```

### E3: `use_driven_bone_density(load_history: f32, current_bond_energy: f32) -> f32`
```
target_bond = current_bond + (load_history - HOMEOSTATIC_LOAD) × WOLFF_ADAPTATION_RATE
```
Ley de Wolff: hueso se adapta a la carga.

---

## Systems (3)

### S1: `morphology_environmental_pressure_system` (Transformer)
**Phase**: MorphologicalLayer, before organ inference chain
**Reads**: Transform, EnergyFieldGrid (local conditions), BaseEnergy, SpatialVolume, MatterCoherence
**Writes**: InferenceProfile (adjust biases based on env pressure)
**Run condition**: Every 16 ticks
**Logic**:
1. Compute t_env from local cell
2. bergmann_pressure → nudge growth_bias (larger in cold)
3. allen_pressure → nudge branching_bias (less branching in cold)
4. Apply small deltas (max ADAPTATION_RATE per tick) to InferenceProfile
5. Change detection guard on each field

### S2: `morphology_use_adaptation_system` (Transformer)
**Phase**: MorphologicalLayer, after S1
**Reads**: FlowVector (speed history proxy), MatterCoherence, BehaviorMode (if hunting/fleeing)
**Writes**: MatterCoherence (bond_energy: increase with sustained use)
**Run condition**: Every 16 ticks
**Logic**: Entities que se mueven mucho → bond_energy sube (Wolff). Entities sedentarias → bond_energy baja.

### S3: `morphology_organ_rebalance_system` (Emitter)
**Phase**: MorphologicalLayer, after S2
**Reads**: InferenceProfile (Changed), OrganManifest
**Writes**: PendingGrowthMorphRebuild (trigger re-inference of organs)
**Logic**: Cuando InferenceProfile cambia significativamente → flag para que organ_inference re-calcule el manifest.

---

## Tests

- `bergmann_cold_increases_growth_bias`
- `bergmann_hot_no_pressure`
- `allen_cold_reduces_branching`
- `wolff_running_entity_increases_bond`
- `wolff_sedentary_decreases_bond`
- `organ_rebalance_triggered_on_profile_change`
