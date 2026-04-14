# Sprint PP-2: Phototropism — Flora se inclina hacia la luz

**Esfuerzo:** 1 semana
**Bloqueado por:** Nada (independiente)
**Desbloquea:** —

## Contexto

Flora es estática. `WillActuator` solo mueve fauna. Pero el fototropismo no
necesita voluntad — es crecimiento diferencial por gradiente de irradiancia.

## Mecanismo axiomático

```
irradiance_gradient = direction_to_strongest_lux_nucleus
shadow_side_growth = base_growth × (1 + PHOTOTROPISM_SENSITIVITY)
lit_side_growth = base_growth × (1 - PHOTOTROPISM_SENSITIVITY × 0.5)
→ spine tilts toward light (asymmetric radius accumulation)
```

- **Axiom 4:** Lado iluminado disipa más (fotosíntesis activa) → crece menos
- **Axiom 7:** Irradiancia es direccional → gradiente existe

No necesita WillActuator. Es `build_flow_spine` con bias direccional.

## Entregable

1. `irradiance_gradient_direction(entity_pos, lux_nuclei) → Vec2` — pure fn
2. `phototropic_spine_bias(gradient_dir, strength) → Vec3` — bias para GF1 spine
3. Extender `entity_shape_inference_system` — spine tilt proporcional al gradiente
4. Flora con `IrradianceReceiver` se inclina hacia el sol. Sin IrradianceReceiver, vertical.

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `irradiance_gradient_direction` pure fn | `src/blueprint/equations/phototropism.rs` | 4 |
| 2 | `phototropic_spine_bias` pure fn | `src/blueprint/equations/phototropism.rs` | 3 |
| 3 | Leer gradient en shape inference | `src/simulation/lifecycle/entity_shape_inference.rs` | 2 |
| 4 | Pasar bias a `build_flow_spine` | `src/geometry_flow/mod.rs` (GeometryInfluence) | 2 |

## Criterios de aceptación

- [ ] Flora en zona con sol lateral se inclina >15° hacia la fuente
- [ ] Flora en zona sin luz crece vertical (bias = 0)
- [ ] Day/night rotation cambia dirección de inclinación
- [ ] Constante: `PHOTOTROPISM_SENSITIVITY = 1.0 / DENSITY_SCALE = 0.05`
