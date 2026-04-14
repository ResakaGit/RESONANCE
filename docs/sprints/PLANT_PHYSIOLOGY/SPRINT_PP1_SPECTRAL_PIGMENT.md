# Sprint PP-1: Spectral Pigmentation — Color emerge de frecuencia × densidad

**ADR:** [ADR-034](../../arquitectura/ADR/ADR-034-spectral-absorption-model.md)
**Esfuerzo:** 1 semana
**Bloqueado por:** PP-0
**Desbloquea:** PP-8 (color como señal perceptible)

## Contexto

Color = `frequency_to_tint_rgb(entity_freq)`. Decorativo. Todos los órganos
del mismo color. No hay relación con la física de absorción.

## Principio agnóstico

El color de un órgano es la frecuencia que **no absorbe**. La frecuencia de
absorción se deriva de la frecuencia del órgano, que se deriva de su densidad:

```
organ_freq = entity_freq × (organ_density / entity_density)
reflected_freq = solar_freq - organ_freq
color = frequency_to_tint_rgb(reflected_freq)
```

Un órgano denso tiene frecuencia cercana al entity → absorbe banda amplia → oscuro.
Un órgano de baja densidad tiene frecuencia desplazada → refleja más → color vivo.
**No hay tabla Leaf=verde, Petal=rojo.** El color sale de la física.

## Entregable

1. `ReflectedSpectrum { reflected_freq_hz: f32 }` — SparseSet component
2. `organ_frequency(entity_freq, organ_density, entity_density) → f32` — pure fn
3. `reflected_frequency(solar_freq, absorption_freq) → f32` — pure fn
4. `spectral_tint_rgb(reflected_freq, albedo) → [f32; 3]` — pure fn
5. `entity_shape_inference` usa `reflected_freq` per-organ para colorear sub-meshes

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `ReflectedSpectrum` component | `src/layers/reflected_spectrum.rs` | 1 |
| 2 | 3 pure fns espectrales agnósticas | `src/blueprint/equations/spectral_absorption.rs` | 8 |
| 3 | System que computa reflected_freq | Extender `albedo_inference_system` | 2 integration |
| 4 | Shape inference lee reflected_freq per-organ | `src/simulation/lifecycle/entity_shape_inference.rs` | 2 integration |
| 5 | Register component + type | `src/layers/mod.rs`, `src/plugins/layers_plugin.rs` | — |

## Criterios de aceptación

- [ ] Órgano denso (bond_energy alta) → color oscuro/marrón (absorción amplia)
- [ ] Órgano de baja densidad → color vivo (absorción estrecha, reflejo amplio)
- [ ] Órgano que cambia de densidad (madura) → cambia de color automáticamente
- [ ] Órgano sin irradiancia (subterráneo) → sin reflected_freq (sin color)
- [ ] Ninguna referencia a OrganRole en las ecuaciones espectrales
- [ ] `cargo test` pasa, zero regresiones en color existente
