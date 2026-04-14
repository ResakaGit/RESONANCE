# Sprint PP-4: Tissue Curvature — Pétalos curvados por crecimiento diferencial

**Esfuerzo:** 1.5 semanas
**Bloqueado por:** PP-0 (necesita organ_qe para gradiente)
**Desbloquea:** —

## Contexto

PetalFan es geometría plana radial. En la naturaleza, la curvatura de pétalos
y hojas viene de crecimiento diferencial: la cara que recibe más nutrientes
crece más rápido → curvatura.

## Mecanismo axiomático

```
nutrient_flux(inner) = organ_qe × attenuation(distance_to_stem)  ← Axiom 7
nutrient_flux(outer) = organ_qe × attenuation(distance_to_stem + organ_length)
growth_ratio = nutrient_flux(inner) / nutrient_flux(outer)
curvature = log(growth_ratio) × CURVATURE_SCALE
```

- **Axiom 7:** Distancia al tallo atenúa nutrientes → gradiente natural
- **Axiom 4:** Disipación diferencial inner/outer
- **Axiom 1:** Curvatura = redistribución de qe, no fuerza externa

## Entregable

1. `differential_growth_rate(organ_qe, distance_inner, distance_outer) → f32` — pure fn
2. `curvature_from_gradient(growth_ratio) → f32` — pure fn
3. Modificar `build_flow_mesh` en GF1: ring radius varía por cara (inner > outer)
4. Pétalos se curvan hacia afuera, hojas se curvan hacia abajo (gravedad + nutrientes)

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | 2 pure fns | `src/blueprint/equations/tissue_growth.rs` | 6 |
| 2 | `GeometryInfluence` + campo `curvature: f32` | `src/geometry_flow/mod.rs` | 2 |
| 3 | Ring radius asimétrico en `build_flow_mesh` | `src/geometry_flow/mod.rs` | 4 |
| 4 | Shape inference pasa curvature al influence | `src/simulation/lifecycle/entity_shape_inference.rs` | 2 |
| 5 | Constantes | `src/blueprint/constants/plant_physiology.rs` | 2 |

## Criterios de aceptación

- [ ] Pétalo con alto organ_qe se curva hacia afuera (curvature > 0)
- [ ] Hoja con bajo organ_qe queda plana (curvature ≈ 0)
- [ ] Curvatura 0 cuando growth_ratio = 1 (sin gradiente)
- [ ] `CURVATURE_NUTRIENT_RATIO = DISSIPATION_LIQUID / DISSIPATION_SOLID = 4.0`
