# Arquitectura — Escalera de Complejidad Termodinámica

## Posición en el Motor

La Escalera de Complejidad Termodinámica es el **modelo conceptual** que organiza las 14 capas ECS y todos los sistemas de Resonance en 5 niveles de abstracción energética. No es un módulo nuevo — es la lente que unifica lo que ya existe y señala lo que falta.

```
EXISTENTE (95-100%)                    NUEVO (sprints TL1-TL6)
──────────────────                     ──────────────────────
Capa 1: BaseEnergy, FlowVector,       + IrradianceReceiver
        density(), drag_force(),       + irradiance_propagation_system
        state_from_temperature()

Capa 2: AlchemicalAlmanac,            (sin cambios — activar campos
        OscillatorySignature,           electronegativity, ionization_ev
        MatterCoherence, purity()       en ecuaciones existentes)

Capa 3: catalysis_result(),           + osmotic_pressure_delta()
        thermal_transfer(),            + osmotic_diffusion_system
        interference()

Capa 4: AlchemicalEngine (buffer),    + NutrientProfile component
        will_force()                   + GrowthBudget component
                                       + liebig_growth_budget()
                                       + photosynthetic_yield()
                                       + nutrient_uptake_system
                                       + growth_budget_system

Capa 5: geometry_flow (GF1),          + recursive_branch_spine()
        shape_inference,               + branch_budget()
        quantized_color (Sprint 14),   + growth_morphology_system
        GF2 (blueprint ready)          + allometric_intake()
```

## Coherencia con la Filosofía del Repositorio

### Principio: Easy vs Simple

La Escalera prioriza **simple** (cada capa aislada, sin enredos) sobre **easy** (un god-system que haga todo). El costo inicial de separar osmosis de catalysis rinde cuando un diseñador quiere activar osmosis sin catalysis para un bioma acuático.

### Principio: Layered ECS (no Hexagonal)

La Escalera NO introduce ports/adapters. Los nuevos componentes (`NutrientProfile`, `GrowthBudget`, `IrradianceReceiver`) son datos puros en capas ortogonales. Los nuevos sistemas son transformaciones enfocadas registradas en Phases existentes.

### Principio: DOD Strict

- **Máximo 4 campos** por componente: NutrientProfile tiene 4, GrowthBudget tiene 3.
- **Un sistema, una transformación**: `nutrient_uptake_system` sólo escribe `NutrientProfile`.
- **Sin derived storage**: `GrowthBudget.biomass_available` se computa cada tick, no se almacena como snapshot.
- **SparseSet** para `GrowthBudget` e `IrradianceReceiver` (transient — sólo entidades "vivas").

### Principio: Yanagi Aesthetic

Las ecuaciones nuevas siguen la densidad funcional:
```rust
// liebig: el mínimo de los 4 nutrientes × eficiencia genética
let (budget, limiter) = liebig_growth_budget(c, n, p, w, eff);
```

## Reutilización del Stack de Rendimiento

### BridgeCache<B> (11 tipos existentes)

Cada nueva ecuación costosa se envuelve en un BridgeCache:

```
BridgeOsmosis     → osmotic_pressure_delta(Δconc, permeability)
BridgeGrowth      → liebig_growth_budget(C, N, P, W, eff)
BridgePhotosynth  → photosynthetic_yield(photons, water, carbon, temp)
```

Patrón idéntico a `BridgeDensity`, `BridgeThermal`, etc. Normalización [0,1] → cache hit cuando el input no cambió significativamente.

### LOD Near/Mid/Far (Sprint 13)

| Banda | Termodinámica (existente) | Metabólica (nuevo) | Morfológica (nuevo) |
|-------|---------------------------|--------------------|--------------------|
| Near | Propagation cada tick | Growth budget cada tick | Mesh regen on Changed |
| Mid | Propagation cada 4 ticks | Growth budget cada 8 ticks | Mesh congelado |
| Far | No propagation | No growth computation | No mesh (sprite fallback) |

### Quantized Color (Sprint 14)

El `PaletteRegistry` ya tiene paletas por elemento. Las nuevas capas modulan el `enorm` input:

```
enorm_visual = qe / QE_REFERENCE                          // existente
enorm_health = growth_budget / MAX_GROWTH_BUDGET           // nuevo
enorm_final  = enorm_visual * (0.5 + 0.5 * enorm_health)  // modulación
```

Entidad con alto `qe` pero bajo `growth_budget` → color desaturado (marchita).
Mismo `quantized_palette_index()`, mismo ρ LOD, cero cambios en el shader pipeline.

## Mapeo de las 14 Capas ECS a la Escalera

| ECS Layer | Escalera | Rol en la Escalera |
|-----------|----------|-------------------|
| L0 BaseEnergy | C1 | Input fundamental — qe disponible |
| L1 SpatialVolume | C1 | density = qe / volume |
| L2 OscillatorySignature | C2 | Identidad elemental (frecuencia = elemento) |
| L3 FlowVector | C1 | Transporte de energía en el espacio |
| L4 MatterCoherence | C2 | Retención (bond_energy), fase (state) |
| L5 AlchemicalEngine | C4 | Buffer metabólico (ATP proxy) |
| L6 AmbientPressure | C1 | Condición ambiental inyectada |
| L7 WillActuator | C4 | Dirección metabólica (intención = propósito) |
| L8 AlchemicalInjector | C3 | Energía de activación inyectada |
| L9 MobaIdentity | — | Gameplay, no termodinámica |
| L10 ResonanceLink | C3 | Modulador de reacciones (buff/debuff) |
| L11 TensionField | C5 | Vector ambiental (fuerza a distancia → tropismo) |
| L12 Homeostasis | C4 | Adaptación frecuencial (feedback metabólico) |
| L13 StructuralLink | C5 | Constraint mecánico (raíces, ramas) |

**Nuevos componentes propuestos (no son nuevas capas ECS):**
- `NutrientProfile` → C4 (4 campos, no necesita nueva capa — es extensión de C4)
- `GrowthBudget` → C4 (SparseSet, marker-like)
- `IrradianceReceiver` → C1 (SparseSet, extensión del campo)

Estos **no pasan el 5-Test** para nueva capa ECS (son derivables de capas existentes + grid), por lo que se agregan como componentes auxiliares, no como L14/L15/L16.

## Diagrama de Dependencias entre Sprints

```
                    ┌──────────┐     ┌──────────┐
                    │ TL1      │     │ TL2      │
                    │ Ósmosis  │     │ Nutrient │
                    │ (C3)     │     │ Field    │
                    └────┬─────┘     └────┬─────┘
                         │                │
                    ┌────▼────────────────▼─────┐
                    │ TL3                       │
                    │ Growth Budget + Liebig    │
                    │ (C4)                      │
                    └────┬─────────────────┬────┘
                         │                 │
                    ┌────▼─────┐     ┌─────▼────┐
                    │ TL4      │     │ TL5      │
                    │ Photo-   │     │ Recursive│
                    │ synthesis│     │ Branching│
                    │ (C4)     │     │ (C5)     │
                    └──────────┘     └─────┬────┘
                                           │
                                     ┌─────▼────┐
                                     │ TL6      │
                                     │ Allometry│
                                     │ (C4→C5)  │
                                     └──────────┘
```

## Referencias

- `docs/design/THERMODYNAMIC_LADDER.md` — Spec completa
- `docs/sprints/THERMODYNAMIC_LADDER/` — Sprints de implementación
- `.cursor/rules/architect.mdc` — Pattern: Layered ECS
- `.cursor/rules/ecs-strict-dod.mdc` — DOD enforcer
- `.cursor/rules/easy-vs-simple-pragmatism.mdc` — Pragmatismo contextual
