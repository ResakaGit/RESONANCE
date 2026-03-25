# Blueprint — Escalera de Complejidad Termodinámica

**Versión:** 1.0
**Depende de:** V7 (design), GEOMETRY_DEFORMATION_ENGINE, QUANTIZED_COLOR_ENGINE
**Estado:** Diseño aprobado — implementación pendiente

---

## 1. Resumen

La Escalera de Complejidad Termodinámica organiza toda la lógica del motor en **5 capas de abstracción** que transforman energía bruta en forma visible. Cada capa es una **tubería stateless** que recibe un payload inyectable y devuelve un resultado puro. La capa N sólo consume el output de la capa N-1.

```
         ┌─────────────────────────────┐
Capa 5   │  MORFOLÓGICA / ESTRUCTURAL  │  Mesh 3D, Transform, Vertex Color
         ├─────────────────────────────┤
Capa 4   │  METABÓLICA / CELULAR       │  GrowthBudget, NutrientProfile
         ├─────────────────────────────┤
Capa 3   │  REACCIONES QUÍMICAS        │  Catalysis, OsmoticTransfer, Equilibrium
         ├─────────────────────────────┤
Capa 2   │  ATÓMICA / ELEMENTAL        │  ElementDef, MatterCoherence, Purity
         ├─────────────────────────────┤
Capa 1   │  TERMODINÁMICA BASE         │  BaseEnergy, FlowVector, Temperature, Phase
         └─────────────────────────────┘
```

**Principio rector:** Una planta no "sabe" que es planta. El Almanac define sus parámetros; el campo energético inyecta sus condiciones; las 5 capas producen su forma. Si un hechizo destruye el Nitrógeno local, la Capa 2 reporta "0 N", la Capa 3 falla sus reacciones, la Capa 4 colapsa el presupuesto de crecimiento, y la Capa 5 marchita la geometría. Sin `if hechizo then matar_planta`.

---

## 2. Mapeo a la Arquitectura Existente

### 2.1 Capa 1 — Termodinámica Base (95% existente)

**Responsabilidad:** Calcular flujo, disipación y estado de excitación en el espacio.

| Concepto | Existe | Módulo | Función/Componente |
|----------|--------|--------|--------------------|
| Energía fundamental (qe) | ✅ | `layers/energy.rs` | `BaseEnergy` |
| Volumen espacial | ✅ | `layers/volume.rs` | `SpatialVolume` |
| Densidad | ✅ | `equations.rs` | `density(qe, radius)` |
| Temperatura equivalente | ✅ | `equations.rs` | `equivalent_temperature(density)` |
| Transiciones de fase | ✅ | `equations.rs` | `state_from_temperature(temp, bond_energy)` |
| Disipación efectiva | ✅ | `equations.rs` | `effective_dissipation(base, velocity, friction)` |
| Arrastre | ✅ | `equations.rs` | `drag_force(viscosity, density, velocity)` |
| Integración velocidad | ✅ | `equations.rs` | `integrate_velocity(v, force, qe, dt)` |
| Transporte de fotones | ❌ | — | Necesita: irradiance field |
| Nucleación | ❌ | — | Necesita: cristalización espontánea |

**Falta:** Transporte de fotones como campo inyectable (no sólo `visibility` pasiva). La irradiancia se puede modelar como un canal adicional del `EnergyFieldGrid` — un `f32` de densidad de fotones por celda que las fuentes Lux emiten y la distancia atenúa.

### 2.2 Capa 2 — Atómica / Elemental (100% existente)

**Responsabilidad:** Definir la capacidad de retención y los límites de los materiales.

| Concepto | Existe | Módulo | Función/Componente |
|----------|--------|--------|--------------------|
| Tabla periódica alquímica | ✅ | `blueprint/almanac.rs` | `AlchemicalAlmanac`, `ElementDef` |
| Frecuencia como identidad | ✅ | `layers/oscillatory.rs` | `OscillatorySignature` |
| Pureza / mezcla | ✅ | `almanac.rs` | `purity(freq)`, `contains(freq)` |
| Bond energy (retención) | ✅ | `layers/coherence.rs` | `MatterCoherence.bond_energy_eb` |
| Conductividad térmica | ✅ | `layers/coherence.rs` | `MatterCoherence.thermal_conductivity` |
| Electronegatividad | ✅ | `almanac.rs` | `ElementDef.electronegativity` |
| Ionización | ✅ | `almanac.rs` | `ElementDef.ionization_ev` |
| Valencia / estados redox | ❌ | — | Usable vía `electronegativity` existente |
| Solubilidad | ❌ | — | Modelable como `interference` + `bond_energy` |

**Completamente cubierta.** Los campos `electronegativity` y `ionization_ev` del `ElementDef` existen pero aún no tienen ecuaciones que los consuman. Los sprints TL1-TL2 los activan.

### 2.3 Capa 3 — Reacciones Químicas (90% existente)

**Responsabilidad:** Transferencias de energía cuando moléculas interactúan.

| Concepto | Existe | Módulo | Función/Componente |
|----------|--------|--------|--------------------|
| Catálisis | ✅ | `simulation/reactions.rs` | `catalysis_scan_system`, `catalysis_apply_system` |
| Interferencia constructiva/destructiva | ✅ | `equations.rs` | `interference()`, `is_constructive()` |
| Transferencia térmica (3 canales) | ✅ | `equations.rs` | `thermal_transfer()` — Surface/Immersed/Radiated |
| Contención host-guest | ✅ | `simulation/containment.rs` | `containment_system` |
| Ósmosis / difusión | ❌ | — | **Sprint TL1** |
| Equilibrio reversible | ❌ | — | Modelable con `interference` bidireccional |
| Cinética enzimática | ❌ | — | Modelable como catálisis con tercer cuerpo |

**Falta clave:** Ósmosis. El gradiente de concentración entre celdas adyacentes impulsa transferencia de `qe` y frecuencia. La ecuación es análoga a `thermal_transfer` pero con `electronegativity` como driving force.

### 2.4 Capa 4 — Metabólica / Celular (70% existente)

**Responsabilidad:** Usar moléculas complejas para mantener orden frente a la entropía.

| Concepto | Existe | Módulo | Función/Componente |
|----------|--------|--------|--------------------|
| Buffer energético (ATP) | ✅ | `layers/engine.rs` | `AlchemicalEngine` (buffer, valves) |
| Casting de habilidades | ✅ | `layers/will.rs` | `AbilitySlot`, `Grimoire` |
| Fuerza motriz | ✅ | `equations.rs` | `will_force(intent, buffer, max)` |
| Overlay térmico | ✅ | `layers/link.rs` | `ResonanceThermalOverlay` |
| Presupuesto de crecimiento | ❌ | — | **Sprint TL3** (`GrowthBudget` component) |
| Ley del Mínimo de Liebig | ❌ | — | **Sprint TL3** (ecuación pura) |
| Campo de nutrientes | ❌ | — | **Sprint TL2** (extensión de `EnergyFieldGrid`) |
| Rendimiento fotosintético | ❌ | — | **Sprint TL4** (ecuación pura) |
| Alometría | ❌ | — | **Sprint TL6** |

**Falta clave:** El presupuesto de crecimiento (`GrowthBudget`) que sintetiza las capas inferiores en un único escalar que la Capa 5 consume. Liebig's Law selecciona el nutriente limitante.

### 2.5 Capa 5 — Morfológica / Estructural (85% existente)

**Responsabilidad:** Traducir presupuesto metabólico en forma 3D.

| Concepto | Existe | Módulo | Función/Componente |
|----------|--------|--------|--------------------|
| Gradiente energético 2D | ✅ | `equations.rs` | `energy_gradient_2d()` |
| Dirección de crecimiento | ✅ | `equations.rs` | `shape_inferred_direction()` |
| Spine GF1 (L-system) | ✅ | `geometry_flow/mod.rs` | `build_flow_spine()`, `build_flow_mesh()` |
| Deformación GF2 (tensores) | ⏳ | Blueprint listo | `deform_spine()` — pendiente |
| Color cuantizado LOD | ✅ | `rendering/quantized_color/` | `quantized_palette_index()`, paletas |
| Shape inference | ✅ | `worldgen/shape_inference.rs` | `shape_color_inference_system` |
| Branching recursivo | ❌ | — | **Sprint TL5** |
| Dominancia apical | ❌ | — | **Sprint TL5** |
| Penetración de raíces | ❌ | — | **Sprint TL5** (opcional) |

---

## 3. Flujo de Datos Stateless (Pipeline Completo)

```
                          INYECCIÓN
                             │
  ┌──────────────────────────▼──────────────────────────┐
  │ CAPA 1: Termodinámica Base                          │
  │   Input:  Temperatura, Densidad de fotones, Entropía│
  │   Math:   density(), equivalent_temperature(),      │
  │           state_from_temperature()                   │
  │   Output: Potencial energético (qe, T, phase)       │
  └──────────────────────────┬──────────────────────────┘
                             │
  ┌──────────────────────────▼──────────────────────────┐
  │ CAPA 2: Atómica / Elemental                         │
  │   Input:  Potencial (C1) + ElementDef del Almanac   │
  │   Math:   purity(), bond_energy, electronegativity  │
  │   Output: Moléculas simples + estabilidad térmica    │
  └──────────────────────────┬──────────────────────────┘
                             │
  ┌──────────────────────────▼──────────────────────────┐
  │ CAPA 3: Reacciones Químicas                         │
  │   Input:  Moléculas (C2) + Energía de activación    │
  │   Math:   catalysis_result(), thermal_transfer(),   │
  │           osmotic_pressure_delta() [NUEVO]           │
  │   Output: Moléculas complejas + ΔE liberada/absorb  │
  └──────────────────────────┬──────────────────────────┘
                             │
  ┌──────────────────────────▼──────────────────────────┐
  │ CAPA 4: Metabólica / Celular                        │
  │   Input:  Moléculas complejas (C3) + Código genético│
  │   Math:   liebig_growth_budget(), photosynthetic_   │
  │           yield(), allometric_intake() [NUEVOS]      │
  │   Output: GrowthBudget (biomasa) + Desechos         │
  └──────────────────────────┬──────────────────────────┘
                             │
  ┌──────────────────────────▼──────────────────────────┐
  │ CAPA 5: Morfológica / Estructural                   │
  │   Input:  GrowthBudget (C4) + Vectores ambientales  │
  │   Math:   build_flow_spine(), build_flow_mesh(),    │
  │           recursive_branch_spine() [NUEVO]           │
  │   Output: Mesh 3D + Transform (forma final)         │
  └─────────────────────────────────────────────────────┘
```

---

## 4. Reutilización del Rendimiento Existente

### 4.1 BridgeCache (Sprint B1-B10)

El patrón `BridgeCache<B>` normaliza inputs y cachea resultados de ecuaciones costosas. Cada nueva ecuación de la escalera puede reutilizarlo:

| Ecuación nueva | BridgeCache type | Normalización |
|---------------|------------------|---------------|
| `osmotic_pressure_delta` | `BridgeOsmosis` | electronegativity → [0,1], Δconcentration → [0,1] |
| `liebig_growth_budget` | `BridgeGrowth` | nutrient ratios → [0,1], temperature → [0,1] |
| `photosynthetic_yield` | `BridgePhoto` | irradiance → [0,1], CO₂ proxy → [0,1] |
| `allometric_intake` | Inline (O(1)) | volume^(2/3) — demasiado barato para cachear |

### 4.2 LOD / Presupuesto por Frame (Sprint 13)

El sistema LOD Near/Mid/Far ya controla:
- **Materialización:** cull_distance decide qué celdas spawneamos
- **Visual derivation:** `max_visual_derivation_per_frame` limita recálculos
- **Shape inference:** `SHAPE_INF_MAX_PER_FRAME` limita meshes nuevos

Las nuevas capas **reutilizan el mismo patrón**:
- `max_growth_budget_per_frame` — presupuesto de cálculos de crecimiento
- `max_osmosis_per_frame` — presupuesto de difusión osmótica
- Far entities: growth budget = 0 (dormidas, no gastan CPU)

### 4.3 Color Cuantizado (Sprint 14)

El `PaletteRegistry` ya genera paletas por elemento. Las nuevas capas lo extienden:
- **Health palette:** GrowthBudget alto → verde vibrante, bajo → amarillo/marrón
- **Nutrient deficiency:** Liebig limiting factor → desaturación visual
- **La misma función** `quantized_palette_index(enorm, rho, n_max)` determina el color
- **El mismo ρ** (precision factor) reduce resolución cromática con la distancia

### 4.4 Changed<T> Filter (Sprint G8)

Todas las capas nuevas usan `Changed<T>` para skip-when-unchanged:
- `Changed<BaseEnergy>` → recalcular sólo si la energía cambió
- `Changed<NutrientProfile>` → recalcular growth budget sólo si nutrientes cambiaron
- `Changed<GrowthBudget>` → regenerar mesh sólo si el presupuesto cambió

---

## 5. Nuevos Componentes ECS

### 5.1 NutrientProfile (Capa 4)

```rust
#[derive(Component, Reflect, Debug, Clone)]
pub struct NutrientProfile {
    pub carbon_norm: f32,      // [0,1] — proxy CO₂ / materia orgánica
    pub nitrogen_norm: f32,    // [0,1] — proxy N disponible
    pub phosphorus_norm: f32,  // [0,1] — proxy P disponible
    pub water_norm: f32,       // [0,1] — proxy H₂O disponible
}
```

**4 campos** (máximo por componente). Normalizados [0,1] para compatibilidad con BridgeCache.

### 5.2 GrowthBudget (Capa 4)

```rust
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct GrowthBudget {
    pub biomass_available: f32,  // Unidades de crecimiento disponible
    pub limiting_factor: u8,     // 0=none, 1=C, 2=N, 3=P, 4=H₂O
    pub efficiency: f32,         // [0,1] — multiplicador genético (Almanac)
}
```

**3 campos.** SparseSet porque se añade/remueve según si la entidad es "viva" o inerte.

### 5.3 IrradianceReceiver (Capa 1 extensión)

```rust
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct IrradianceReceiver {
    pub photon_density: f32,    // Densidad de fotones recibida [0, ∞)
    pub absorbed_fraction: f32, // Fracción absorbida [0,1] (depende de visibility)
}
```

**2 campos.** SparseSet — sólo entidades en zona iluminada.

---

## 6. Nuevas Ecuaciones (Pure Math)

Todas en `blueprint/equations.rs`, testadas sin ECS.

### 6.1 Ósmosis

```
osmotic_pressure_delta(concentration_a, concentration_b, membrane_permeability) -> f32
  = (concentration_a - concentration_b) * membrane_permeability
```

Análoga a `thermal_transfer` pero con concentración en vez de temperatura.

### 6.2 Liebig's Law

```
liebig_growth_budget(carbon, nitrogen, phosphorus, water, genetic_efficiency) -> (f32, u8)
  = (min(C, N, P, H₂O) * genetic_efficiency, index_of_minimum)
```

El mínimo determina el presupuesto. El índice identifica el factor limitante (para visual feedback).

### 6.3 Rendimiento Fotosintético

```
photosynthetic_yield(photon_density, water_norm, carbon_norm, temperature) -> f32
  = photon_density * min(water, carbon) * temperature_efficiency_curve(T)
```

Curva de eficiencia: campana gaussiana centrada en temperatura óptima del elemento.

### 6.4 Alometría

```
allometric_intake(base_intake, volume) -> f32
  = base_intake * volume^(2/3)
```

Ley de Kleiber: metabolismo escala con superficie, no volumen.

### 6.5 Branch Budget (Morfología)

```
branch_budget(growth_budget, depth, max_depth) -> u32
  = floor(growth_budget * (1 - depth/max_depth)^2)
```

Branches permitidos decrece cuadráticamente con profundidad (dominancia apical).

---

## 7. Nuevos Sistemas

| Sistema | Fase | Input | Output | Presupuesto |
|---------|------|-------|--------|-------------|
| `irradiance_propagation_system` | PrePhysics | Nuclei Lux + grid | `IrradianceReceiver` | Por chunk (LOD) |
| `osmotic_diffusion_system` | Reactions | Grid vecinos + electronegativity | ΔBaseEnergy, Δfrequency | `max_osmosis_per_frame` |
| `nutrient_uptake_system` | Reactions | Grid nutrients + Materialized | `NutrientProfile` | Changed<BaseEnergy> filter |
| `growth_budget_system` | PostPhysics | NutrientProfile + IrradianceReceiver + Almanac | `GrowthBudget` | Changed<NutrientProfile> |
| `growth_morphology_system` | Update | GrowthBudget + GeometryInfluence | Mesh3d update | `max_growth_morph_per_frame` |

---

## 8. Lo que NO Hace Este Blueprint

- No cambia las 14 capas ECS existentes — las extiende con 3 componentes nuevos.
- No reemplaza `catalysis_result()` — la ósmosis es una reacción paralela, no un reemplazo.
- No implementa biología molecular real — los "nutrientes" son proxies normalizados [0,1].
- No requiere cambios en el Almanac — usa `electronegativity`, `ionization_ev` y `bond_energy` que ya existen pero estaban sin consumir.
- No modifica el sistema de spawning — el `EntityBuilder` recibe los nuevos componentes opcionalmente.
- No introduce trait objects ni herencia — todo es composición ECS pura.

---

## 9. Demostración de Flexibilidad (El Test del Alquimista)

> Un hechizo destruye el Nitrógeno en el suelo.

| Capa | Qué pasa | Sistema responsable |
|------|----------|---------------------|
| C2 | `NutrientProfile.nitrogen_norm` → 0.0 (celda afectada) | `nutrient_uptake_system` (detecta N=0 en grid) |
| C3 | Ósmosis intenta importar N de vecinos (gradiente) | `osmotic_diffusion_system` |
| C4 | `liebig_growth_budget(C, 0, P, W)` → budget ≈ 0 | `growth_budget_system` |
| C5 | `shape_inferred_length(0, ...)` → longitud mínima | `growth_morphology_system` |
| Visual | Palette index bajo → color desaturado/marrón | `quantized_palette_index` (ya existente) |

**Cero condicionales específicos.** El marchitamiento emerge de las matemáticas.

---

## 10. Dependencias y Orden de Implementación

```
Sprint TL1 (Ósmosis)       ─┐
Sprint TL2 (Nutrientes)    ─┼─► Sprint TL3 (Growth Budget) ─► Sprint TL5 (Branching)
Sprint TL4 (Fotosíntesis)  ─┘                                       │
                                                              Sprint TL6 (Alometría)
```

TL1 y TL2 son independientes entre sí. TL3 depende de ambos. TL4 puede ser paralelo a TL3. TL5 depende de TL3 + GF2. TL6 es incremental sobre TL3.

---

## 11. Criterios de Éxito Global

1. `cargo test` pasa con todas las capas activadas.
2. Demo default muestra entidades que crecen, se marchitan y responden a cambios de nutrientes.
3. El test del Alquimista funciona sin `if` específicos.
4. Performance: <2ms adicionales por frame en grid 64×64 (budget-limited).
5. BridgeCache hit ratio > 85% en estado estacionario para las nuevas ecuaciones.
6. Cero regresión en tests de Sprint V7, GF1, Sprint 14.

---

## Referencias

- `docs/design/V7.md` — V7 energy field
- `docs/design/GEOMETRY_DEFORMATION_ENGINE.md` — GF2 tensors
- `docs/design/QUANTIZED_COLOR_ENGINE.md` — Sprint 14
- `docs/design/BRIDGE_OPTIMIZER.md` — BridgeCache pattern
- `docs/design/ECO_BOUNDARIES.md` — Eco zones
- `.cursor/rules/architect.mdc` — Layered ECS pattern
- `.cursor/rules/ecs-strict-dod.mdc` — DOD enforcer
