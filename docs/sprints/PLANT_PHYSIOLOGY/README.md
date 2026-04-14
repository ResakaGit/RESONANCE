# Track: PLANT_PHYSIOLOGY — Propiedades materiales emergentes de flujos de energía

Pigmentos, tejidos, volátiles, tropismos y estacionalidad derivan de los 8 axiomas.
Ninguna propiedad material se programa — todas emergen de cómo la energía fluye,
se absorbe, se disipa y se redistribuye dentro de un organismo.

**Invariante:** Toda propiedad material es una consecuencia observable de `qe + frequency + density + dissipation`. Zero lookup tables. Zero `if role == X`. Los sistemas son agnósticos — funcionan con cualquier entidad que tenga las propiedades físicas requeridas.

---

## Principio fundamental

> "El color de un órgano no es un atributo — es la frecuencia que no pudo absorber.
> Su longevidad no es un parámetro — es la tasa de disipación de su estado material.
> Su fragancia no es una feature — es la energía que le sobra y no puede retener."

Cada órgano es un **packet de energía con estado físico** (`qe + volume + bond_energy`).
De su densidad se deriva su matter_state. De su matter_state se derivan dissipation,
senescence, volatility, color, prioridad. No hay tabla de roles → comportamientos.

---

## Fundación (ADR-033)

| Sprint | Nombre | Esfuerzo | Entregable | Estado |
|--------|--------|----------|------------|--------|
| [PP-0](SPRINT_PP0_ORGAN_SUBPOOLS.md) | Organ Sub-Pools | 1 sem | `OrganSlot { qe, volume, bond_energy }`, distribución por densidad, pool invariant | Diseñado |

## Onda A — Propiedades visuales (ADR-034)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [PP-1](SPRINT_PP1_SPECTRAL_PIGMENT.md) | Spectral Pigmentation | 1 sem | PP-0 | `ReflectedSpectrum`, color per-organ desde `organ_freq × density` |
| [PP-2](SPRINT_PP2_PHOTOTROPISM.md) | Phototropism | 1 sem | — | GF1 spine sigue gradiente de irradiancia |
| [PP-3](SPRINT_PP3_PHENOLOGY.md) | Phenology Wiring | 0.5 sem | — | Wire módulo phenology existente → lifecycle stage guard estacional |

## Onda B — Propiedades físicas

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [PP-4](SPRINT_PP4_TISSUE_CURVATURE.md) | Tissue Curvature | 1.5 sem | PP-0 | Crecimiento diferencial por gradiente de nutrientes → curvatura |
| [PP-5](SPRINT_PP5_ORGAN_SENESCENCE.md) | Organ Senescence | 1 sem | PP-0 | Gompertz per-organ, `dissipation(matter_state)` como coeff |
| [PP-6](SPRINT_PP6_VOLATILE_EMISSION.md) | Volatile Emission | 1 sem | PP-0 | Órgano gaseoso con overflow → emisión al grid (ADR-035) |

## Onda C — Propiedades estructurales

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [PP-7](SPRINT_PP7_ROOT_DIFFERENTIATION.md) | Subterranean Differentiation | 1 sem | PP-0 | Constructal underground por gradiente de nutrientes |
| [PP-8](SPRINT_PP8_POLLINATION.md) | Cross-Transfer | 2 sem | PP-6 | EnergyTag mediado por entidad móvil, reproducción cruzada |

## Dependency chain

```
PP-0 (organ sub-pools) ──┬── PP-1 (pigment) ─────────────┐
                          ├── PP-4 (curvature)             │
                          ├── PP-5 (organ senescence)      ├── PP-8 (cross-transfer)
                          └── PP-6 (volatile) ─────────────┘
                               │
PP-2 (phototropism) ───────────┤ (independiente)
PP-3 (phenology) ──────────────┘ (independiente)
PP-7 (subterranean) ── PP-0
```

## Arquitectura de archivos

```
src/blueprint/
├── equations/
│   ├── organ_energy.rs              ← PP-0: organ_density, organ_priority, distribute, enforce
│   ├── spectral_absorption.rs       ← PP-1: organ_frequency, reflected_frequency, spectral_tint
│   ├── tissue_growth.rs             ← PP-4: differential_growth_rate, curvature_from_gradient
│   ├── volatile_emission.rs         ← PP-6: can_emit, emission_rate, volatile_decay, perceive
│   ├── subterranean_morphology.rs   ← PP-7: nutrient_gradient, constructal_branch_count
│   ├── cross_transfer.rs            ← PP-8: transfer_compatibility, mix_profiles
│   └── phototropism.rs              ← PP-2: irradiance_gradient, phototropic_spine_bias
src/layers/
│   ├── organ.rs                     ← PP-0: OrganSlot { qe, volume, bond_energy }
│   ├── reflected_spectrum.rs        ← PP-1: ReflectedSpectrum (SparseSet)
│   └── energy_tag.rs                ← PP-8: EnergyTag (SparseSet, transient)
src/simulation/
├── metabolic/
│   ├── organ_distribution.rs        ← PP-0: redistribution system
│   ├── volatile_emission.rs         ← PP-6: emission system
│   └── morphogenesis.rs             ← PP-1: extend albedo_inference
├── lifecycle/
│   ├── organ_lifecycle.rs           ← PP-5: per-organ Gompertz
│   ├── entity_shape_inference.rs    ← PP-4: asymmetric GF1 rings
│   └── morpho_adaptation.rs         ← PP-2: irradiance → spine tilt
├── thermodynamic/
│   └── sensory.rs                   ← PP-6: volatile grid reading
└── reproduction/
    └── cross_transfer.rs            ← PP-8: deposit, transfer, decay systems
src/geometry_flow/
│   └── mod.rs                       ← PP-4: asymmetric ring radius
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 1 | Todo es energía. Pigmento = qe reflejado. Volátil = qe emitido. Curvatura = qe redistribuido. |
| 2 | Pool invariant: `sum(organ_qe) ≤ entity_qe`. Enforced por PP-0. |
| 3 | Órganos compiten por energía. Los menos densos pierden primero bajo estrés. |
| 4 | Cada órgano disipa a la tasa de su matter_state. Volátiles decaen a tasa GAS. |
| 5 | Energía nunca se crea. Pigmento refleja lo no-absorbido, no genera color. |
| 6 | **Clave.** Todo emerge de estado físico. Zero `if role == X`. Zero lookup tables. |
| 7 | Nutrient flux atenúa con distancia → curvatura. Volátil atenúa → alcance. |
| 8 | Absorción espectral, percepción de volátil, compatibilidad de transfer — frequency-selective. |

## Constantes derivadas (todas de las 4 fundamentales, ninguna de roles)

| Constante | Derivación |
|-----------|-----------|
| `GAS_DENSITY_THRESHOLD` | `gas_density_threshold()` en `derived_thresholds.rs` — umbral para emisión |
| `VOLATILE_EFFICIENCY` | `1.0 - DISSIPATION_GAS` = 0.92 |
| `VOLATILE_DECAY_RATE` | `DISSIPATION_GAS` = 0.08/tick |
| `CURVATURE_NUTRIENT_RATIO` | `DISSIPATION_LIQUID / DISSIPATION_SOLID` = 4.0 |
| `PHOTOTROPISM_SENSITIVITY` | `1.0 / DENSITY_SCALE` = 0.05 |
| `CONCENTRATION_THRESHOLD` | `DISSIPATION_LIQUID × DENSITY_SCALE` = 0.4 |
| `TRANSFER_THRESHOLD` | 0.5 (alignment mínimo, derivable de `1 - DISSIPATION_PLASMA`) |
| `TAG_LIFETIME` | `1.0 / DISSIPATION_LIQUID` ≈ 50 ticks |
| `ORGAN_DEATH_THRESHOLD` | `DISSIPATION_SOLID × DENSITY_SCALE` = 0.1 qe |

## ADRs relacionadas

| ADR | Tema | Sprints |
|-----|------|---------|
| [ADR-033](../../arquitectura/ADR/ADR-033-organ-sub-pools.md) | Organ Sub-Pools: energía per-organ, distribución por densidad | PP-0, PP-5 |
| [ADR-034](../../arquitectura/ADR/ADR-034-spectral-absorption-model.md) | Spectral Absorption: pigmentación desde `organ_freq × density` | PP-1 |
| [ADR-035](../../arquitectura/ADR/ADR-035-volatile-field-protocol.md) | Volatile Field: emisión por densidad < GAS_THRESHOLD | PP-6, PP-8 |

## Paralelismo seguro

| | PP-1 | PP-2 | PP-3 | PP-4 | PP-5 | PP-6 | PP-7 | PP-8 |
|--|------|------|------|------|------|------|------|------|
| PP-1 | — | ✅ | ✅ | | | | | |
| PP-2 | ✅ | — | ✅ | ✅ | ✅ | ✅ | ✅ | |
| PP-3 | ✅ | ✅ | — | ✅ | ✅ | ✅ | ✅ | |
| PP-4 | | ✅ | ✅ | — | ✅ | | | |
| PP-5 | | ✅ | ✅ | ✅ | — | | | |
| PP-6 | | ✅ | ✅ | | | — | | |
| PP-7 | | ✅ | ✅ | | | | — | |
| PP-8 | | | | | | | | — |

PP-2 y PP-3 son totalmente independientes — pueden ir en paralelo con cualquier otro sprint.

## Invariantes del track

1. **Zero lookup tables.** Ningún mapeo `OrganRole → propiedad`. Toda propiedad se deriva de estado físico.
2. **Agnóstico.** Las ecuaciones no saben qué es un pétalo, una raíz, o un tallo. Solo ven `qe + volume + bond_energy + frequency`.
3. **Stateless equations.** Toda fn en `blueprint/equations/` es `(inputs) → output`. Sin side effects.
4. **Cache-friendly.** `OrganSlot` es 12 bytes (3 × f32). Inline array de 12 slots = 144 bytes. Cache line.
5. **Pool invariant siempre.** `sum(organ_qe) ≤ entity_qe` verificado cada tick.
6. **Dissipation siempre.** Emitir cuesta. Mantener cuesta. Envejecer cuesta. Nada gratis.
7. **SparseSet para transient.** `ReflectedSpectrum`, `EnergyTag` — solo cuando aplica.
8. **Phase assignment.** Cada system nuevo tiene phase explícito en pipeline.

## Esfuerzo total: ~10 semanas, ~1500 LOC, ~120 tests
