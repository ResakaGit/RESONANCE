# Modulo: Ecosystem Autopoiesis — Ecosistema Auto-formado por Inferencia

Blueprint de arquitectura para la transicion de spawn manual a ecosistema emergente autopoietico.

## 1) Frontera y Responsabilidad

- **Que Resuelve:** Cerrar el ciclo de vida completo: nacimiento espontaneo (abiogenesis), crecimiento (existente), reproduccion, competencia y muerte. Con estos 4 sistemas nuevos + 1 fix, el ecosistema se auto-forma sin spawn manual.
- **Que NO Resuelve:** No define fauna (L15 Tactical Inference), ni gameplay MOBA (habilidades, facciones). No modifica el pipeline existente de crecimiento (FL1-FL4).
- **Naturaleza:** 4 sistemas nuevos + 6 ecuaciones puras + 1 fix sensorial + 1 elemento.

## 2) Posicion en la Arquitectura

```
┌──────────────────────────────────────────────────────────┐
│                    PIPELINE (FixedUpdate)                  │
├──────────────────────────────────────────────────────────┤
│                                                            │
│  Phase::ThermodynamicLayer                                 │
│    ├─ attention_convergence ← EA3 FIX (real grid values)  │
│    └─ irradiance_update (FL1 ✅)                          │
│                                                            │
│  Phase::ChemicalLayer                                      │
│    ├─ nutrient_uptake (FL2 ✅)                            │
│    ├─ competitive_exclusion ← EA7 NEW                      │
│    └─ osmotic_diffusion (TL1 ✅)                          │
│                                                            │
│  Phase::MetabolicLayer                                     │
│    ├─ growth_budget (FL3 ✅)                              │
│    └─ metabolic_stress_death ← EA4 NEW                     │
│                                                            │
│  Phase::MorphologicalLayer                                 │
│    ├─ growth_intent_inference (FL4 ✅)                    │
│    ├─ allometric_growth (FL4 ✅)                          │
│    ├─ reproduction ← EA6 NEW                               │
│    └─ abiogenesis ← EA5 NEW                                │
│                                                            │
├──────────────────────────────────────────────────────────┤
│                 EQUATIONS (blueprint/equations.rs)          │
│  starvation_threshold()      metabolic_viability()         │
│  abiogenesis_potential()     abiogenesis_profile()         │
│  can_reproduce()             mutate_bias()                 │
│  competition_energy_drain()                                │
├──────────────────────────────────────────────────────────┤
│                 COMPONENTS (layers/)                        │
│  Existentes: BaseEnergy, InferenceProfile, CapabilitySet,  │
│  GrowthBudget, SpatialVolume, AllometricRadiusAnchor,      │
│  NutrientProfile, MatterCoherence                          │
│  Nuevo: ReproductionCooldown (SparseSet, transient)        │
├──────────────────────────────────────────────────────────┤
│                 EVENTS (existentes)                         │
│  DeathEvent (Dissipation) ← metabolic_stress emite         │
│  faction_identity_system ← despawn consume                 │
│  nutrient_return_on_death ← devuelve al grid               │
└──────────────────────────────────────────────────────────┘
```

**EA8 (fenología):** no va en `FixedUpdate`; `phenology_visual_apply_system` está en la cadena `Update` de `register_visual_derivation_pipeline` (`simulation/pipeline.rs`), después de `visual_derivation_*` y antes de `shape_color_inference_system`.

## 3) Ciclo de Vida Emergente

```
 EnergyFieldGrid + NutrientFieldGrid
         │ condiciones (qe, hz, water)
         ▼
   ┌─────────────┐
   │ ABIOGENESIS  │ EA5: spawn espontaneo
   └──────┬──────┘
          │ entidad nueva
          ▼
   ┌─────────────┐
   │ GROWTH      │ FL1-FL4 existente: irradiance → nutrient → budget → allometric
   └──────┬──────┘
          │ radius crece
          ▼
   ┌──────────────┐        ┌──────────────────┐
   │ REPRODUCTION │ EA6    │ COMPETITION      │ EA7
   │ (si biomasa  │───────►│ (drain por       │
   │  suficiente) │        │  densidad local) │
   └──────┬───────┘        └────────┬─────────┘
          │ semilla                  │ qe baja
          ▼                         ▼
   ┌─────────────┐         ┌──────────────┐
   │ nueva planta│         │ DEATH        │ EA4
   │ (hereda +   │         │ (qe < umbral │
   │  muta perfil│         │  → DeathEvent│
   └─────────────┘         └──────┬───────┘
                                   │ nutrient_return_on_death
                                   ▼
                           NutrientFieldGrid (recicla)
```

## 4) Coherencia con cursor/rules

### 4.1 ecs-strict-dod.mdc
- **1 componente nuevo:** `ReproductionCooldown` (1 campo u32, SparseSet transient). Cumple max 4 campos.
- **4 sistemas nuevos:** Cada uno lee 2-3 componentes, escribe 1. Ningun god-system.
- **Guard change detection:** `competitive_exclusion` verifica `new_qe != energy.qe` antes de mutar.

### 4.2 architect.mdc
- **Vertical slice pattern:** Ecuaciones en equations.rs, constantes inline (modulo-local), sistemas en simulation/, componente en layers/.
- **No crea modulos de infraestructura nuevos.** Solo archivos de sistema.

### 4.3 tech-stack.mdc
- Bevy 0.15: queries con tuple destructuring, `.in_set(Phase::X)`, `Changed<T>`, `EventWriter<DeathEvent>`.
- Sin crates externos.
- Sin unsafe, sin async.

### 4.4 easy-vs-simple-pragmatism.mdc
- **4 sistemas simples** vs 1 "ecosystem_lifecycle" complejo. Cada uno testeable aisladamente.
- El Vec temporal en `competitive_exclusion` (cell_counts) es O(cells), no O(entities^2).

## 5) Dependencias entre Sprints

```
EA1 (flora.ron)  ──► EA2 (spawn_rosa) ──► EA6 (reproduction)
                                                │
EA3 (sensory fix)                               │
                                                │
EA4 (death) ──► EA5 (abiogenesis) ──────────────┤
            ──► EA7 (competition) ──────────────┘

EA8 (fenología visual) ──► worldgen: PhenologyVisualParams + phenology_visual_apply_system
     (mezcla young/mature en lineal; puras phenology_* en equations.rs; alineado con EAC/EPI2)
```

**Ecosistema minimo viable:** EA1 + EA4 + EA5 = campo genera flora que muere.
**Ecosistema auto-sostenible:** + EA6 = poblacion se reproduce.
**Ecosistema competitivo:** + EA7 = exclusion selectiva, sucesion ecologica.
**Lectura brote→madurez:** EA8 + datos `phenology` en almanaque (sin estados nombrados en gameplay).

## 6) Invariantes del Ecosistema

1. **Poblacion acotada:** `MAX_ABIOGENESIS_PER_FRAME` + `MAX_REPRODUCTIONS_PER_FRAME` + competencia + muerte = equilibrio dinamico.
2. **Energia conservada:** Spawn consume qe del campo/padre. Muerte retorna nutrientes al grid. No se crea energia de la nada.
3. **Sin etiquetas taxonomicas:** Ningun sistema pregunta `if is_plant`. Todo emerge de InferenceProfile + CapabilitySet + equations.
4. **Determinismo:** Sin `rand()`. Mutacion usa `entity.index()` como semilla determinista.
