# Sprint PP-3: Phenology Wiring — Floración estacional

**Esfuerzo:** 0.5 semana
**Bloqueado por:** Nada (independiente)
**Desbloquea:** —

## Contexto

El módulo `blueprint/equations/phenology/` existe con ecuaciones de estacionalidad
pero no está wired al lifecycle stage. Hoy, flora florece en cuanto tiene energía
suficiente — sin importar la estación.

## Entregable

Wire el guard estacional al transition `Mature → Reproductive`:

```
can_transition_to_reproductive =
    growth_progress >= 0.7
    && biomass >= 1.5
    && viability >= 1.2
    && irradiance >= PHENOLOGY_BLOOM_THRESHOLD  ← NUEVO
```

El threshold se deriva de `year_period_ticks` y `axial_tilt` del mapa.
En invierno, irradiancia < threshold → flora no entra en Reproductive.
En primavera, irradiancia sube → floración.

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | Derivar `PHENOLOGY_BLOOM_THRESHOLD` | `src/blueprint/constants/plant_physiology.rs` | 2 |
| 2 | Guard en `infer_lifecycle_stage` | `src/blueprint/equations/lifecycle/mod.rs` | 4 |
| 3 | Pasar `IrradianceReceiver` al lifecycle inference | `src/simulation/lifecycle/organ_lifecycle.rs` | 2 |

## Criterios de aceptación

- [ ] Flora NO entra en Reproductive con irradiance < threshold
- [ ] Flora SÍ entra cuando irradiance sube (primavera)
- [ ] Mapas sin year_period (sin estaciones) → threshold = 0 (siempre florece, backward compatible)
- [ ] `PHENOLOGY_BLOOM_THRESHOLD = DISSIPATION_LIQUID × DENSITY_SCALE = 0.4`
