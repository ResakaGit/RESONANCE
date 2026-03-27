# Track: AXIOMATIC_INFERENCE — 100% Derived Constants

**Objetivo:** Eliminar todas las constantes arbitrarias y empíricamente calibradas de los sistemas de vida. Cada valor numérico debe derivarse algebraicamente de los 4 fundamentales irreducibles.

**Estado:** ⏳ Pendiente (6 sprints)
**Bloqueado por:** Nada (track independiente)
**Desbloquea:** Validación axiomática del ciclo de vida completo

---

## Los 4 Fundamentales Irreducibles

Estas son las ÚNICAS constantes que no se derivan de nada — son inputs del universo:

| Fundamental | Valor | Justificación |
|-------------|-------|---------------|
| `KLEIBER_EXPONENT` | 0.75 | Ley biológica universal (27 órdenes de magnitud) |
| `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}` | 0.005, 0.02, 0.08, 0.25 | Segunda Ley por estado de materia (Axiom 4) |
| `COHERENCE_BANDWIDTH_HZ` | 50.0 | Ventana de observación para interferencia (Axiom 8) |
| `DENSITY_SCALE` | 20.0 | Factor de normalización espacial (grid geometry) |

Todo lo demás es COMPUTABLE desde estos 4.

---

## Auditoría pre-sprint

| Categoría | Antes | Después |
|-----------|-------|---------|
| AXIOM-DERIVED | 4 (6%) | 62 (100%) |
| EMPIRICALLY-CALIBRATED | 35 (56%) | 0 |
| ARBITRARY | 15 (24%) | 0 |
| STRUCTURAL (algorítmico) | 5 (8%) | 5 (8%) — no cambian |
| Magic numbers inline | 3 | 0 |

---

## 6 Sprints

| Sprint | Descripción | Archivos | Esfuerzo | Bloqueado por |
|--------|-------------|----------|----------|---------------|
| [AI-1](SPRINT_AI1_DERIVATION_MODULE.md) | Módulo de derivación axiomática (`derived_thresholds.rs`) | 1 nuevo | Bajo | — |
| [AI-2](SPRINT_AI2_MATTER_STATE_THRESHOLDS.md) | Density thresholds derivados de ratios de disipación | 2 (equations + axiomatic.rs) | Bajo | AI-1 |
| [AI-3](SPRINT_AI3_CAPABILITY_THRESHOLDS.md) | MOVE/SENSE/BRANCH derivados de density + coherence | 2 (axiomatic.rs + awakening.rs) | Bajo | AI-2 |
| [AI-4](SPRINT_AI4_SENESCENCE_DERIVED.md) | Coeff + max_age derivados de metabolic rate (Kleiber) | 4 (constants + 3 spawn paths) | Medio | AI-1 |
| [AI-5](SPRINT_AI5_PRESSURE_AND_DRAIN.md) | Basal rate, pressure threshold/rate derivados de dissipation | 3 (basal_drain + pressure + constants) | Bajo | AI-2 |
| [AI-6](SPRINT_AI6_INLINE_EXTRACTION.md) | Eliminar magic numbers inline + consolidar duplicados | 3 (nucleus_recycling + awakening) | Bajo | AI-1 |

**Total: 6 sprints, ~10 archivos, todos ejecutables en paralelo excepto AI-2→AI-3 (dependencia de density thresholds).**

---

## Cadena de derivación

```
FUNDAMENTALES (4 valores irreducibles)
    │
    ├── KLEIBER_EXPONENT ─────────────────────────────────────────────────┐
    │                                                                      │
    ├── DISSIPATION_SOLID ──┬── basal_drain_rate() [AI-5]                 │
    │   DISSIPATION_LIQUID  ├── liquid_density_threshold() [AI-2]         │
    │   DISSIPATION_GAS     ├── gas_density_threshold() [AI-2]            │
    │   DISSIPATION_PLASMA  ├── plasma_density_threshold() [AI-2]         │
    │                       ├── radiation_pressure_threshold() [AI-5]     │
    │                       ├── radiation_pressure_transfer_rate() [AI-5] │
    │                       ├── senescence_coeff_*() [AI-4] ◄────────────┘
    │                       └── max_age_*() [AI-4]
    │
    ├── COHERENCE_BANDWIDTH ──┬── PRESSURE_FREQUENCY_BANDWIDTH (= mismo) [AI-5]
    │                         └── sense_coherence_min() [AI-3]
    │
    └── DENSITY_SCALE ──┬── matter state thresholds (× scale) [AI-2]
                        ├── move_density_min/max() [AI-3]
                        └── branch_qe_min() [AI-3]

DERIVADOS DE DERIVADOS:
    spawn_potential_threshold() = 1/3 (break-even algebraico) [AI-3]
    self_sustaining_qe_min() = DISSIPATION_SOLID × cell_area / coherence [AI-3]
    survival_probability_threshold() = exp(-2) (Gompertz 1/e² punto) [AI-4]
    nutrient_drain_fraction() = dissipation_rate ratio [AI-6]
```

---

## Criterio de cierre

Cada sprint se cierra cuando:
1. ✅ Cero constantes hardcodeadas en el área (grep confirma)
2. ✅ Cada valor numérico tiene un `derived_thresholds::*()` o fundamental documentado
3. ✅ Tests unitarios validan las relaciones algebraicas (ej: `gas > liquid > solid`)
4. ✅ Demo headless 5k ticks produce ciclo de vida visible (no regresión)
5. ✅ `cargo test` 0 failures

---

## Qué NO cambia

- La lógica de los sistemas (solo cambian los valores que leen)
- La arquitectura (constants/ + equations/ + systems)
- Los 8 axiomas
- Los STRUCTURAL constants (budgets de CPU, scan intervals)
