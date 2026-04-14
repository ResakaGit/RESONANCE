# Sprint PP-6: Volatile Emission — Cualquier órgano gaseoso con overflow emite

**ADR:** [ADR-035](../../arquitectura/ADR/ADR-035-volatile-field-protocol.md)
**Esfuerzo:** 1 semana
**Bloqueado por:** PP-0 (necesita OrganSlot con estado físico)
**Desbloquea:** PP-8 (pollination)

## Contexto

No existe señalización química entre entidades. Un volátil es energía emitida
por un órgano al NutrientFieldGrid — mecánicamente idéntico a un núcleo temporal.

## Principio agnóstico

No se dice "Petal emite fragancia". La condición de emisión es puramente física:

```
density = organ.qe / organ.volume
matter_state = state_from_density(density)
maintenance = organ.qe × dissipation_rate(matter_state)
overflow = organ.qe - maintenance

can_emit = overflow > 0 AND density < GAS_DENSITY_THRESHOLD

emission = overflow × DISSIPATION_GAS × VOLATILE_EFFICIENCY
field[cell].volatile_signal += emission × 1/(1 + d²)    ← Axiom 7
field[cell].volatile_freq = organ_frequency(organ)       ← Axiom 8
organ.qe -= emission / VOLATILE_EFFICIENCY               ← Axiom 4

// Decay every tick:
field[cell].volatile_signal *= (1 - DISSIPATION_GAS)     ← efímero
```

Un órgano denso (tallo) no emite porque `density > GAS_THRESHOLD`.
Un órgano gaseoso con exceso de energía emite naturalmente. Emerge.

## Entregable

1. `volatile_signal` + `volatile_freq` campos en NutrientFieldGrid
2. `can_emit(organ_density, gas_threshold) → bool` — pure fn
3. `emission_rate(overflow_qe, dissipation, efficiency) → f32` — pure fn
4. `volatile_decay(signal, rate) → f32` — pure fn
5. `perceive_volatile(signal, freq, sensor_freq, bandwidth) → f32` — pure fn
6. `volatile_emission_system` — ChemicalLayer
7. `volatile_decay_system` — ThermodynamicLayer (pre-physics)

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | Agregar campos al grid | `src/batch/scratch.rs` | 2 |
| 2 | 4 pure fns | `src/blueprint/equations/volatile_emission.rs` | 10 |
| 3 | Emission system | `src/simulation/metabolic/volatile_emission.rs` | 3 integration |
| 4 | Decay system | `src/simulation/thermodynamic/pre_physics.rs` | 2 |
| 5 | Sensory reads volatile | `src/simulation/thermodynamic/sensory.rs` | 2 |

## Criterios de aceptación

- [ ] Órgano con density < GAS_THRESHOLD y overflow > 0 emite; otros no
- [ ] Signal decae 50% en ~8 ticks (`(1-0.08)^8 ≈ 0.51`)
- [ ] Entidad con SENSE + freq alineada percibe el volátil
- [ ] Entidad con freq desalineada no lo percibe
- [ ] Emission cost drena organ_qe — entidad pobre no emite
- [ ] Ninguna referencia a OrganRole en emisión/percepción
