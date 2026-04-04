# MT-2: Proyección Conservation-Bounded

**Objetivo:** Aplicar las ecuaciones cuánticas de MT-1 dentro de `project_entity` para que: (a) la energía nunca crezca en la proyección (Axioma 4+5), (b) la disipación refleje resonancia solar (Axioma 8). Zero cambio de API — solo cambia la implementación interna.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Bajo (4 líneas cambian en project_entity, rest es tests)
**Bloqueado por:** MT-1 (funciones de conservación y frecuencia)
**Desbloquea:** MT-3 (stack necesita proyecciones conservation-safe para que re-emanación funcione)

---

## Entregable

### Modificación en `src/batch/telescope/projection.rs` — `project_entity`

```rust
// ANTES (actual):
let base_decay = batch_stepping::dissipation_n_ticks(entity.qe, entity.dissipation, k);
projected.qe = project_qe(base_decay, 0.0, metrics, weights, k).max(0.0);

// DESPUÉS (conservation-bounded + frequency-aware):
let effective_rate = frequency_aware_decay_rate(
    entity.dissipation,
    entity.frequency_hz,
    SOLAR_FREQUENCY,
    SOLAR_BANDWIDTH,
    PHOTOSYNTHESIS_EFFICIENCY,
);
let base_decay = batch_stepping::dissipation_n_ticks(entity.qe, effective_rate, k);
let projected_qe = project_qe(base_decay, 0.0, metrics, weights, k);
projected.qe = conservation_bounded_project(entity.qe, base_decay, projected_qe);
```

Cambio neto: 4 líneas. La API pública de `project_entity` no cambia. El contrato se refuerza: `result.qe ≤ input.qe` garantizado.

---

## Contrato reforzado

```
ANTES:  project_entity(entity, ...).qe ≤ entity.qe  (probabilístico — depende de H y trend)
AHORA:  project_entity(entity, ...).qe ≤ entity.qe  (GARANTIZADO — clamp explícito)
        project_entity(entity, ...).qe ≥ base_decay  (no peor que decay puro)
```

---

## Preguntas para tests

### Conservation (Axioma 4+5)
1. Entidad con qe=100, dissipation=0.01, K=50 → ¿projected.qe ≤ 100? (nunca crece)
2. Entidad con qe=100, H=1.0 (persistente), trend positivo → ¿projected.qe ≤ 100? (clamp activo)
3. 20 entidades variadas, K=100 → ¿total_qe(projected) ≤ total_qe(input)? (property test global)
4. K=0 → ¿qe sin cambio? (identity)
5. K=10000 (extremo) → ¿qe ≥ 0.0? (nunca negativo)

### Frequency-Aware Decay (Axioma 8)
6. Entidad con freq=SOLAR_FREQUENCY, dissipation=0.01 → ¿decae MENOS que freq=50 Hz?
7. Entidad con freq=50 Hz (Terra, lejos del sol) → ¿decae igual que antes? (sin subsidio)
8. Dos entidades idénticas excepto freq: la resonante sobrevive más ticks
9. `project_world` con mix de frecuencias → ¿las resonantes retienen más qe?

### Compatibilidad (no rompe tests existentes)
10. Todos los tests existentes de projection.rs siguen pasando
11. `axiom4_dissipation_always_reduces_qe` sigue verde
12. `axiom5_total_qe_never_increases_with_neutral_hurst` sigue verde
13. `project_world_deterministic` sigue verde

---

## Integración

- **Consume:** MT-1 (`conservation_bounded_project`, `frequency_aware_decay_rate`)
- **Consume:** `batch/constants.rs` (`SOLAR_FREQUENCY`, `SOLAR_BANDWIDTH`, `PHOTOSYNTHESIS_EFFICIENCY`)
- **Consumido por:** MT-3 (stack usa project_world con garantía de conservación)
- **Modifica:** `projection.rs` (4 líneas en project_entity)
- **No modifica:** API pública de project_entity, project_world, project_nutrient_grid, project_irradiance_grid
