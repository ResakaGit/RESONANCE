# Sprint PP-7: Subterranean Differentiation — Morfología underground por gradiente

**Esfuerzo:** 1 semana
**Bloqueado por:** PP-0
**Desbloquea:** —

## Contexto

Los órganos subterráneos son todos tubos idénticos. En la naturaleza, la
morfología underground responde al gradiente de nutrientes del suelo.

## Principio agnóstico

No se dice "raíz pivotante vs fibrosa". Cualquier órgano cuya posición
(BodyPlanLayout) esté debajo del plano del terreno se orienta y ramifica
según el gradiente de nutrientes:

```
nutrient_gradient = direction_to_max_nutrient(grid, organ_pos)
gradient_strength = |nutrient_at_deep - nutrient_at_surface|

// Constructal: minimizar costo de transporte = distancia × count
optimal_topology = constructal_branch_count(gradient_strength, organ_qe)

if gradient_strength > CONCENTRATION_THRESHOLD:
    → 1 estructura larga (minimiza distancia al recurso concentrado, Axiom 7)
else:
    → N estructuras cortas (maximiza cobertura superficial)
```

Es la misma optimización constructal que `constructal_body_plan` ya aplica
a appendages de fauna — extendida al subsuelo.

## Entregable

1. `nutrient_gradient_direction(grid, pos) → (Vec2, f32)` — pure fn
2. `constructal_branch_count(gradient_strength, available_qe) → (count, length)` — pure fn
3. Organ inference: órganos subterráneos ramifican según gradient
4. Shape inference: orientación GravityDown biaseada por gradient direction

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | 2 pure fns | `src/blueprint/equations/subterranean_morphology.rs` | 6 |
| 2 | Organ inference integra gradient | `src/blueprint/equations/organ_inference/mod.rs` | 4 |
| 3 | Shape inference: orientación underground | `src/simulation/lifecycle/entity_shape_inference.rs` | 2 |

## Criterios de aceptación

- [ ] Nutrientes concentrados profundo: 1 estructura larga
- [ ] Nutrientes dispersos superficiales: N estructuras cortas
- [ ] `CONCENTRATION_THRESHOLD = DISSIPATION_LIQUID × DENSITY_SCALE = 0.4`
- [ ] Constructal: minimiza energía de transporte
- [ ] Ninguna referencia a "Root" en las ecuaciones — aplica a cualquier órgano underground
