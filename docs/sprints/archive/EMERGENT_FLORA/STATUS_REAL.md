# Estado Real — Flora Emergente (auditoría 2026-03-23)

Los sprints FL1-FL4 documentados como "pendientes" ya estaban implementados. Esta auditoría refleja el estado real del código.

---

## Tabla Maestra: Implementado vs Pendiente

### Ecuaciones Puras (blueprint/equations.rs) — 100% ✅

| Función | Línea | Tests | Sprint |
|---------|-------|-------|--------|
| `irradiance_at_distance_sq()` | ✅ | ✅ | FL1 |
| `photosynthetic_yield()` | ✅ | ✅ | FL1 |
| `photosynthetic_growth_bonus()` | ✅ | ✅ | FL3 |
| `liebig_growth_budget()` | ✅ | ✅ | FL3 |
| `genetic_efficiency_for_element()` | ✅ | ✅ | FL3 |
| `growth_size_feedback()` | ✅ | ✅ | FL4 |
| `inferred_growth_delta()` | ✅ | ✅ | FL4 |
| `energy_gradient_2d()` | ✅ | ✅ | FL4 |
| `shape_inferred_direction()` | ✅ | ✅ | FL4 |
| `branch_budget()` | ✅ | ✅ | FL4 |
| `osmotic_pressure_delta()` | ✅ | ✅ | TL1 |
| `osmotic_permeability()` | ✅ | ✅ | TL1 |
| `nutrient_depletion_scale()` | ✅ | ✅ | FL2 |
| `nutrient_return_scale()` | ✅ | ✅ | FL2 |

### Componentes ECS (layers/) — 100% ✅

| Componente | Archivo | Campos | Storage | Tests |
|-----------|---------|--------|---------|-------|
| `InferenceProfile` | layers/inference.rs | 4 | Table | ✅ |
| `CapabilitySet` | layers/inference.rs | 1 (u8) | Table | ✅ |
| `GrowthIntent` | layers/inference.rs | 3 | SparseSet | ✅ |
| `NutrientProfile` | layers/nutrient.rs | 4 | Table | ✅ |
| `GrowthBudget` | layers/growth.rs | 3 | SparseSet | ✅ |
| `AllometricRadiusAnchor` | layers/growth.rs | 1 | Table | ✅ |
| `IrradianceReceiver` | layers/irradiance.rs | 2 | SparseSet | ✅ |
| `ShapeInferred` | worldgen/shape_inference.rs | 0 | SparseSet | ✅ |

### Sistemas de Orquestación — 100% ✅ (registrados en pipeline)

| Sistema | Archivo | Fase | Registrado | Tests |
|---------|---------|------|------------|-------|
| `irradiance_update_system` | photosynthesis.rs | ThermodynamicLayer | ✅ pipeline.rs | ✅ 6 tests |
| `photosynthetic_contribution_system` | photosynthesis.rs | ChemicalLayer | ✅ reactions.rs | ✅ 1 test |
| `nutrient_uptake_system` | nutrient_uptake.rs | ChemicalLayer | ✅ reactions.rs | ✅ 4 tests |
| `nutrient_regen_system` | nutrient_uptake.rs | ChemicalLayer | ✅ reactions.rs | ✅ |
| `nutrient_depletion_system` | nutrient_uptake.rs | ChemicalLayer | ✅ reactions.rs | ✅ |
| `nutrient_return_on_death_system` | nutrient_uptake.rs | ChemicalLayer | ✅ reactions.rs | ✅ |
| `osmotic_diffusion_system` | osmosis.rs | ChemicalLayer | ✅ reactions.rs | ✅ 7 tests |
| `growth_budget_system` | growth_budget.rs | MetabolicLayer | ✅ pipeline.rs | ✅ 7 tests |
| `growth_intent_inference_system` | inference_growth.rs | MorphologicalLayer | ✅ pipeline.rs | ✅ 4 tests |
| `cleanup_orphan_growth_intent_system` | inference_growth.rs | MorphologicalLayer | ✅ pipeline.rs | ✅ 1 test |
| `allometric_growth_system` | allometric_growth.rs | MorphologicalLayer | ✅ pipeline.rs | ✅ 2 tests |
| `attention_convergence_system` | sensory.rs | ThermodynamicLayer | ✅ pipeline.rs | — |
| `shape_color_inference_system` | shape_inference.rs | Update | ✅ pipeline.rs | ✅ |
| `growth_morphology_system` | shape_inference.rs | Update | ✅ pipeline.rs | ✅ |

### Infraestructura — 100% ✅

| Módulo | Estado |
|--------|--------|
| `NutrientFieldGrid` (worldgen/nutrient_field.rs) | ✅ Implementado con seed + regen |
| `EnergyFieldGrid` (worldgen/field_grid.rs) | ✅ Con neighbors4, world_pos, dirty tracking |
| GF1 spine engine (geometry_flow/mod.rs) | ✅ 288 líneas |
| Recursive branching (geometry_flow/branching.rs) | ✅ 207 líneas |
| AttentionGrid (simulation/sensory.rs) | ✅ Grid + convergence |
| BridgeCache<OsmosisBridge> | ✅ Integrado |
| EntityBuilder (.nutrient, .growth_budget, .irradiance) | ✅ |

---

## LO QUE FALTA (4 archivos)

| # | Archivo | Tipo | Esfuerzo | Descripción |
|---|---------|------|----------|-------------|
| 1 | `assets/elements/flora.ron` | Contenido | ~15 líneas | ElementDef para Flora (freq=85Hz, bond=800, electro=2.8) |
| 2 | `src/entities/archetypes.rs` | Código | ~30 líneas | `spawn_flora_seed()`, `spawn_rosa()`, `spawn_oak()` usando EntityBuilder |
| 3 | `assets/maps/flora_demo.ron` | Contenido | ~25 líneas | Mapa 32×32 con 3 nuclei (terra, lux, aqua) |
| 4 | `src/world/flora_demo.rs` | Código | ~40 líneas | Startup system que spawnea 5 plantas con diferentes perfiles |

### Items opcionales (no bloqueantes)

| # | Archivo | Tipo | Descripción |
|---|---------|------|-------------|
| 5 | `simulation/sensory.rs:145-146` | Fix | Reemplazar mock values (local_hz=500, local_energy=50) con lectura real del EnergyFieldGrid |
| 6 | `simulation/pipeline.rs` | Registro | Registrar `attention_gating_system` en Phase::ThermodynamicLayer |
| 7 | `layers/inference.rs` | Extensión L15 | `MotionIntent`, `BranchIntent` (futuros, no necesarios para planta estática) |
| 8 | `blueprint/equations.rs` | Extensión L15 | `infer_motion_intent()`, `infer_branch_intent()` |
| 9 | `simulation/tactical_inference.rs` | Sistema L15 | `tactical_inference_system` + `motion_intent_reducer` + `branch_intent_reducer` |

---

## Pipeline Completo Actual (verificado en código)

```
FixedUpdate (30 Hz):
  SimulationClockSet → advance_clock, bridge_phase_tick

  Phase::Input → grimoire_cast, ability_targeting, element_layer2

  Phase::ThermodynamicLayer
    ├─ terrain_config, climate_config, climate_tick
    ├─ attention_convergence_system          ← SENSORY
    ├─ containment, structural_constraint
    ├─ thermal_transfer, resonance_link
    ├─ engine_processing
    ├─ irradiance_update_system              ← FL1 ✅
    └─ perception_system

  Phase::AtomicLayer → physics (movement, collision, drag)

  Phase::ChemicalLayer
    ├─ catalysis_scan, catalysis_apply
    ├─ osmotic_diffusion_system              ← TL1 ✅
    ├─ nutrient_regen_system                 ← FL2 ✅
    ├─ nutrient_uptake_system                ← FL2 ✅
    ├─ photosynthetic_contribution_system    ← FL1 ✅
    ├─ nutrient_depletion_system             ← FL2 ✅
    └─ nutrient_return_on_death_system       ← FL2 ✅

  Phase::MetabolicLayer
    ├─ growth_budget_system                  ← FL3 ✅
    └─ faction_identity_system

  Phase::MorphologicalLayer
    ├─ cleanup_orphan_growth_intent
    ├─ growth_intent_inference_system        ← FL3b ✅
    ├─ allometric_growth_system              ← FL4 ✅
    └─ bridge_metrics_collect

Update (vsync):
    ├─ visual_derivation (changed, missing, sync)
    ├─ shape_color_inference_system          ← GF1 mesh ✅
    └─ growth_morphology_system              ← mesh rebuild ✅
```

---

## Resumen

**De los 6 sprints documentados como pendientes, 4 ya están completos en código con tests.**

| Sprint | Doc Estado | Código Real | Tests |
|--------|-----------|-------------|-------|
| FL1 Irradiance | ⏳ Pendiente | ✅ **Implementado** | 6 tests |
| FL2 Nutrient Uptake | ⏳ Pendiente | ✅ **Implementado** | 4 tests |
| FL3 Growth Budget | ⏳ Pendiente | ✅ **Implementado** | 7 tests |
| FL4 Allometric Growth | ⏳ Pendiente | ✅ **Implementado** | 2 tests + 4 inference |
| FL5 Flora Signature | ⏳ Pendiente | ❌ **Falta** | — |
| FL6 Rosa Demo | ⏳ Pendiente | ❌ **Falta** | — |

**Para tener la rosa simulada: faltan 4 archivos (~110 líneas de código).**
