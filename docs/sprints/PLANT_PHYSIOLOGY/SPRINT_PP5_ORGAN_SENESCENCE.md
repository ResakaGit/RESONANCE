# Sprint PP-5: Organ Senescence — Marchitamiento por estado material, no por rol

**ADR:** [ADR-033](../../arquitectura/ADR/ADR-033-organ-sub-pools.md)
**Esfuerzo:** 1 semana
**Bloqueado por:** PP-0 (necesita OrganSlot con estado físico)
**Desbloquea:** —

## Contexto

Hoy, `Declining` = muerte del entity completo. No hay pérdida gradual de órganos.

## Principio agnóstico

El coeficiente de senescencia de un órgano se deriva de su estado material,
no de su rol. Un órgano gaseoso (baja densidad) envejece 16× más rápido que
un órgano sólido (alta densidad). Nadie le dice qué es — su física lo determina.

```
density = organ.qe / organ.volume
matter_state = state_from_density(density)
senescence_coeff = dissipation_rate(matter_state)

// Rates from 4 fundamentals:
// SOLID  → 0.005/tick (duro, resiste)
// LIQUID → 0.02/tick
// GAS    → 0.08/tick  (efímero, se marchita)
// PLASMA → 0.25/tick

organ_qe(t) = organ_qe(0) × exp(-senescence_coeff × age_organ)
organ muere cuando organ_qe < ORGAN_DEATH_THRESHOLD
```

- **Axiom 4:** Cada órgano disipa a la tasa de su matter_state
- **Axiom 2:** Cuando organ muere, su qe se devuelve al entity pool
- **Axiom 6:** Qué muere primero emerge de densidad, no se programa

## Entregable

1. `organ_senescence_rate(density) → f32` — pure fn, `density → matter_state → dissipation`
2. `organ_alive(organ_qe, threshold) → bool` — pure fn
3. `organ_senescence_system` — drain per-organ, remove dead organs del manifest
4. Dead organ → `scale_factor → 0` en shape inference (mesh shrinks gradualmente)

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `organ_senescence_rate(density)` pure fn | `src/blueprint/equations/organ_energy.rs` | 6 |
| 2 | `organ_alive` pure fn | `src/blueprint/equations/organ_energy.rs` | 3 |
| 3 | `organ_senescence_system` | `src/simulation/lifecycle/organ_lifecycle.rs` | 4 integration |
| 4 | Shape inference: dead organ → scale 0 | `src/simulation/lifecycle/entity_shape_inference.rs` | 2 |
| 5 | Pool return: dead organ qe → entity qe | `src/simulation/metabolic/organ_distribution.rs` | 2 |

## Criterios de aceptación

- [ ] Órgano GAS-density muere ~16× más rápido que órgano SOLID-density
- [ ] Entity sobrevive mientras al menos 1 órgano denso viva
- [ ] Dead organ qe se devuelve al pool (conservación)
- [ ] Visualmente: órganos de baja densidad shrink y desaparecen primero
- [ ] Ninguna referencia a OrganRole en la lógica de senescencia
