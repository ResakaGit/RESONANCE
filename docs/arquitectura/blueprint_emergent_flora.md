# Módulo: Flora Emergente — Orquestación de Simulación Termodinámica

Blueprint de arquitectura para la implementación de flora emergente sin condicionales.
Fuentes:
- `design/EMERGENT_FLORA.md`
- `design/THERMODYNAMIC_LADDER.md`
- `sprints/EMERGENT_FLORA/`

## 1) Frontera y Responsabilidad

- **Qué Resuelve:** Conectar los módulos existentes (irradiancia, nutrientes, crecimiento, geometría) en una tubería de sistemas ECS que produce flora con comportamiento emergente: fototropismo, competencia por nutrientes, ramificación por biomasa, marchitamiento por estrés.
- **Qué NO Resuelve:** No define nuevos componentes (todos existen). No modifica el motor de geometría (GF1/branching). No implementa gameplay directo (spawning de flora es decisión del worldgen/level designer).
- **Naturaleza:** Capa de orquestación — 4 sistemas nuevos que conectan componentes existentes.

## 2) Posición en la Arquitectura

```
┌──────────────────────────────────────────────────────────┐
│                    PLUGINS (wiring)                       │
├──────────────────────────────────────────────────────────┤
│ ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│ │ irradiance  │→ │ nutrient     │→ │ growth_budget   │  │ ← NUEVOS (FL1-FL3)
│ │ propagation │  │ uptake       │  │ orchestration   │  │
│ └─────────────┘  └──────────────┘  └────────┬────────┘  │
│                                              │           │
│ ┌─────────────────┐  ┌─────────────────────┐│           │
│ │ allometric      │← │ growth_morphology   ││ EXISTENTE │
│ │ growth (FL4)    │  │ (shape_inference)   │←           │
│ └─────────────────┘  └─────────────────────┘            │
├──────────────────────────────────────────────────────────┤
│                 COMPONENTS (14 layers)                    │
│  NutrientProfile  GrowthBudget  IrradianceReceiver       │
│  SpatialVolume    BaseEnergy    MatterCoherence           │
├──────────────────────────────────────────────────────────┤
│                  BLUEPRINT (equations.rs)                 │
│  photosynthetic_yield  liebig_growth_budget               │
│  irradiance_at_distance  growth_size_feedback             │
│  energy_gradient_2d  branch_budget                        │
└──────────────────────────────────────────────────────────┘
```

## 3) Coherencia con cursor/rules

### 3.1 ecs-strict-dod.mdc — Cumplimiento Total

**Cero componentes nuevos.** Los 4 sistemas operan sobre componentes existentes:
- `IrradianceReceiver` (2 campos, SparseSet) — lectura de fuentes Lux, escritura de photon_density
- `NutrientProfile` (4 campos, Table) — lectura de grid, escritura de C/N/P/water
- `GrowthBudget` (3 campos, SparseSet) — lectura de nutrientes+irradiance, escritura de biomass
- `SpatialVolume` (1 campo) — lectura de budget, escritura de radius

**Un sistema = una transformación:**
| Sistema | Lee | Escribe | Transformación |
|---------|-----|---------|----------------|
| irradiance_propagation | Transform, BaseEnergy, OscillatorySignature | IrradianceReceiver | Acumula fotones |
| nutrient_uptake | Transform, NutrientFieldGrid | NutrientProfile | Absorbe nutrientes del suelo |
| growth_budget | NutrientProfile, IrradianceReceiver, MatterCoherence | GrowthBudget | Calcula biomasa disponible |
| allometric_growth | GrowthBudget, AllometricRadiusAnchor | SpatialVolume | Crece el radio |

**Guard change detection en todos:**
```rust
// Patrón: solo escribe si el valor cambió
let new_val = equations::liebig_growth_budget(...);
if growth.biomass_available != new_val { growth.biomass_available = new_val; }
```

### 3.2 architect.mdc — Vertical Slice Pattern

La flora es un **vertical slice** que cruza:
- `layers/` — 6 componentes (ya existen)
- `blueprint/equations.rs` — 12 funciones puras (ya existen)
- `blueprint/constants/` — ~8 constantes nuevas (shard por dominio)
- `simulation/` — 4 sistemas nuevos
- `worldgen/shape_inference.rs` — bridge visual (ya existe)
- `geometry_flow/` — GF1 + branching (ya existe)
- `entities/builder.rs` — spawn con `.nutrient()`, `.growth_budget()` (ya existe)

**No crea módulos nuevos.** Los sistemas se agregan a archivos existentes o a un único archivo `simulation/flora.rs`.

### 3.3 tech-stack.mdc — Sin Violaciones

- Bevy 0.15: queries con tuple destructuring, `in_set(Phase::X)`, `Changed<T>`
- Sin crates: usa solo `bevy::prelude` + `crate::blueprint::equations`
- Sin unsafe, sin async, sin HashMap en hot path
- Grid indexado por posición (O(1) lookup)

### 3.4 easy-vs-simple-pragmatism.mdc

**Decisión arquitectónica:** 4 sistemas simples vs 1 sistema "flora_lifecycle" complejo.

**Elegimos simple:**
- Cada sistema es testeable aisladamente
- Changed<T> chain elimina trabajo innecesario
- Reordenar/desactivar un sistema no rompe los otros
- Un god-system "flora_lifecycle" violaría DOD y sería imposible de cachear

**Costo:** 4 registros en pipeline.rs en vez de 1. Aceptable.

## 4) Reutilización del Stack de Rendimiento

### 4.1 BridgeCache

Dos bridges opcionales (no bloqueantes para FL1-FL4):
- `BridgeIrradiance`: cachea `irradiance_at_distance_sq` para pares fuente-receptor
- `BridgeGrowth`: cachea `liebig_growth_budget` para cuartetos de nutrientes

### 4.2 LOD Integration

Los sistemas respetan el LOD existente:

| Sistema | Near | Mid | Far |
|---------|------|-----|-----|
| irradiance_propagation | ✅ cada tick | ✅ cada 4 ticks | ❌ skip |
| nutrient_uptake | ✅ cada tick | ✅ cada 4 ticks | ❌ skip |
| growth_budget | ✅ Changed<T> | ✅ Changed<T> | ❌ frozen |
| allometric_growth | ✅ Changed<T> | ✅ Changed<T> | ❌ frozen |
| growth_morphology | ✅ 16/frame | ✅ 4/frame | ❌ dormant |

**Entidades Far:** mesh congelado, cero CPU. Al entrar a Mid, Changed<T> dispara recálculo.

### 4.3 Quantized Color

Vertex colors del mesh GF1 usan `palette_tint_rgb()` → `PaletteRegistry` → `quantized_palette_index()`. Sin cambios necesarios.

## 5) Mapeo de Fases del Pipeline

```
FixedUpdate (paso fijo):
  ┌─ SimulationClockSet ─┐
  │  advance_clock        │
  │  bridge_phase_tick    │
  └───────────────────────┘
  ┌─ Phase::ThermodynamicLayer ──────────────────────┐
  │  ... worldgen, eco, containment, motor ...      │
  │  irradiance_update_system  ← fotosíntesis (FL1)  │
  │  perception_system                                │
  └───────────────────────────────────────────────────┘
  ┌─ Phase::AtomicLayer ─────────────────────────────┐
  │  física / colisión / movimiento                   │
  └───────────────────────────────────────────────────┘
  ┌─ Phase::ChemicalLayer ───────────────────────────┐
  │  nutrient_uptake_system, photosynthetic_*, ...   │  ← FL2 / reacciones
  └───────────────────────────────────────────────────┘
  ┌─ Phase::MetabolicLayer ────────────────────────────┐
  │  fog, nucleus death notify, faction_identity       │
  │  growth_budget_system  ← FL3                     │
  └───────────────────────────────────────────────────┘
  ┌─ Phase::MorphologicalLayer ──────────────────────┐
  │  inference_growth*, allometric_growth_system (FL4)│
  │  bridge_metrics_collect_system                    │
  └───────────────────────────────────────────────────┘

Update:
  ┌─ Visual Derivation ──────────────────────────────┐
  │  ...                                              │
  │  growth_morphology_system  (EXISTENTE)            │
  │  shape_color_inference_system  (EXISTENTE)        │
  └───────────────────────────────────────────────────┘
```

## 6) Dependencias entre Sprints

```
FL1 (Irradiance) ──┐
                    ├──► FL3 (Growth Budget) ──► FL4 (Allometric Growth)
FL2 (Nutrient)  ───┘         │
                         FL5 (Firma Flora) ──► FL6 (Rosa Demo)
```

- **Onda A (independientes):** FL1 + FL2 en paralelo
- **Onda B (síntesis):** FL3 (requiere A)
- **Onda C (morfología):** FL4 (requiere B)
- **Onda D (validación):** FL5 + FL6 (requiere C)
