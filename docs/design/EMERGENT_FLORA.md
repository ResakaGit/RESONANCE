# Blueprint — Flora Emergente: De Energía a Forma Sin Condicionales

**Versión:** 1.0
**Depende de:** THERMODYNAMIC_LADDER, GEOMETRY_DEFORMATION_ENGINE, QUANTIZED_COLOR_ENGINE
**Estado:** Diseño — implementación pendiente

---

## 1. Resumen

Este blueprint define la **capa de orquestación** que conecta los módulos existentes (NutrientFieldGrid, GrowthBudget, IrradianceReceiver, GF1+branching, shape inference) en una tubería que produce flora emergente. Una Rosa no "sabe" que es rosa. Su firma elemental (frecuencia, bond_energy, electronegativity) combinada con las condiciones ambientales (luz, nutrientes, temperatura) determina su forma, tamaño, dirección de crecimiento y estado de salud — sin un solo `if is_plant`.

**Principio rector:** El fototropismo es GRATIS. El gradiente de energía del campo V7 apunta hacia las fuentes de luz. El spine GF1 sigue ese gradiente. La planta crece hacia la luz sin programarlo.

```
Lux nucleus emite qe a ~1000 Hz
  → IrradianceReceiver acumula photon_density
    → photosynthetic_yield(photons, water, C, temp) produce energía
      → NutrientProfile se actualiza desde NutrientFieldGrid
        → liebig_growth_budget(C, N, P, water, eff) determina biomasa
          → GrowthBudget.biomass_available crece
            → allometric_intake escala con radio (Kleiber)
              → energy_gradient_2d → spine apunta hacia la luz
                → build_flow_spine sigue gradiente
                  → branching.rs ramifica si biomasa > umbral
                    → Mesh 3D visible, vertex colors cuantizados
```

---

## 2. Validación de Coherencia con cursor/rules

### 2.1 ecs-strict-dod.mdc

| Regla | Cumplimiento | Evidencia |
|-------|-------------|-----------|
| Max 4 campos/componente | ✅ | NutrientProfile=4, GrowthBudget=3, IrradianceReceiver=2 |
| SparseSet para transient | ✅ | GrowthBudget, IrradianceReceiver, ShapeInferred |
| Un sistema, una transformación | ✅ | 6 sistemas separados, cada uno con una mutación |
| Math en equations.rs | ✅ | 12 funciones puras ya existentes |
| No god-systems | ✅ | Máximo 4 tipos de componente por query |
| No valores derivados almacenados | ✅ | Densidad, temperatura, eficiencia: computados al vuelo |
| Changed<T> guards | ✅ | Todos los sistemas usan changed detection |
| No trait objects | ✅ | Composición pura de componentes |

### 2.2 architect.mdc

| Regla | Cumplimiento | Evidencia |
|-------|-------------|-----------|
| Layered ECS (no hexagonal) | ✅ | Plugins → Systems → Events → Components → Blueprint |
| Vertical slices | ✅ | Flora es slice vertical: layers + systems + equations |
| Phase ordering | ✅ | Irradiance→PrePhysics, NutrientUptake→Physics, Growth→PostPhysics |
| Event-driven cross-phase | ✅ | Changed<T> propaga entre fases sin eventos explícitos |
| Blueprint separation | ✅ | Toda la math en equations.rs |

### 2.3 tech-stack.mdc

| Regla | Cumplimiento |
|-------|-------------|
| Bevy 0.15 | ✅ |
| No crates externos | ✅ |
| No unsafe | ✅ |
| No async | ✅ |
| No HashMap hot path | ✅ — usa grid indexado |
| No String en components | ✅ |

### 2.4 easy-vs-simple-pragmatism.mdc

**Contexto:** Core de simulación → requiere simplicidad (capas desacopladas).
**Decisión:** Cada sistema es una transformación atómica. La complejidad emergente viene de la composición, no de lógica condicional.

---

## 3. Módulos Existentes (Inventario)

### 3.1 Ecuaciones Puras (equations.rs) — 100% existentes

| Función | Propósito | Sprint que la activa |
|---------|-----------|---------------------|
| `photosynthetic_yield(photon, water, C, temp)` | Fotosíntesis | FL1 |
| `irradiance_at_distance(emission, dist, decay)` | Atenuación lumínica | FL1 |
| `irradiance_at_distance_sq(emission, dist_sq, decay)` | Variante optimizada | FL1 |
| `photosynthetic_growth_bonus(photon, absorbed)` | Bonus a growth budget | FL3 |
| `liebig_growth_budget(C, N, P, water, eff)` | Ley de Liebig | FL3 |
| `genetic_efficiency_for_element(bond_e, electro)` | Eficiencia genética | FL3 |
| `growth_size_feedback(budget, r, r_max)` | Feedback alométrico | FL4 |
| `allometric_intake(intake, radius)` | Kleiber intake | FL4 |
| `energy_gradient_2d(left, right, down, up)` | Gradiente para fototropismo | FL4 |
| `shape_inferred_direction(gradient, blend)` | Dirección de crecimiento | FL4 |
| `branch_budget(growth, depth, max_depth)` | Dominancia apical | FL4 |
| `branch_attenuation_values(...)` | Cascada morfológica | FL4 |

### 3.2 Componentes ECS — 100% existentes

| Componente | Campos | Storage | Módulo |
|-----------|--------|---------|--------|
| `NutrientProfile` | 4 (C, N, P, water) | Table | `layers/nutrient.rs` |
| `GrowthBudget` | 3 (biomass, limiter, efficiency) | SparseSet | `layers/growth.rs` |
| `IrradianceReceiver` | 2 (photon_density, absorbed_fraction) | SparseSet | `layers/irradiance.rs` |
| `AllometricRadiusAnchor` | 1 (base_radius) | Table | `layers/growth.rs` |
| `ShapeInferred` | 0 (marker) | SparseSet | `worldgen/shape_inference.rs` |
| `PendingGrowthMorphRebuild` | 0 (marker) | SparseSet | `worldgen/shape_inference.rs` |

### 3.3 Infraestructura — 100% existente

| Módulo | Propósito |
|--------|-----------|
| `NutrientFieldGrid` | Grid paralelo a EnergyFieldGrid con C/N/P/water por celda |
| `EnergyFieldGrid` | Grid de energía V7 con neighbors4(), world_pos() |
| `GeometryInfluence` | Paquete inyectable para GF1 spine |
| `build_flow_spine()` | Genera polyline siguiendo energía |
| `build_flow_mesh()` | Triangula tube alrededor de spine |
| `build_branched_tree()` | Árbol recursivo con atenuación |
| `flatten_tree_to_mesh()` | Combina ramas en un solo Mesh |
| `shape_color_inference_system` | Bridge worldgen → GF1 (ya registrado en pipeline) |
| `growth_morphology_system` | Regenera mesh en Changed<GrowthBudget> |
| `PaletteRegistry` | Color cuantizado Sprint 14 |
| `EntityBuilder` | API fluent con `.nutrient()`, `.growth_budget()`, `.volume()` |

### 3.4 Sistemas de Orquestación — FALTANTES (objetivo de este track)

| Sistema | Fase | Input | Output | Sprint |
|---------|------|-------|--------|--------|
| `irradiance_propagation_system` | PrePhysics | Lux nuclei, EnergyFieldGrid | IrradianceReceiver.photon_density | FL1 |
| `nutrient_uptake_system` | Physics | NutrientFieldGrid, Transform | NutrientProfile (C, N, P, water) | FL2 |
| `growth_budget_system` | PostPhysics | NutrientProfile, IrradianceReceiver | GrowthBudget (biomass, limiter) | FL3 |
| `allometric_growth_system` | PostPhysics | GrowthBudget, SpatialVolume | SpatialVolume.radius | FL4 |

---

## 4. Firma de una Rosa: Definición Emergente

Una Rosa no tiene tipo "Rosa". Es una composición de capas cuyas propiedades producen comportamiento rosa-like:

### 4.1 Firma Elemental (ElementDef en Almanac)

```ron
ElementDef(
    name: "Flora",
    symbol: "Fl",
    atomic_number: 0,
    frequency_hz: 85.0,          // Banda Terra baja (~75-100 Hz)
    freq_band: (70.0, 110.0),    // Estabilidad amplia (adaptable)
    bond_energy: 800.0,          // Baja (flexible, no roca)
    conductivity: 0.05,          // Muy baja (aislante vegetal)
    visibility: 0.6,             // Moderada
    matter_state: Solid,         // Nace sólida
    electronegativity: 2.8,      // Alta (avidez por nutrientes)
    ionization_ev: 3.0,          // Baja (fácil de ionizar / quemar)
    color: (0.18, 0.55, 0.12),  // Verde base
    is_compound: false,
)
```

### 4.2 Composición de Capas (spawn)

```rust
EntityBuilder::new()
    .energy(50.0)                                      // L0: qe inicial modesto
    .volume(0.08)                                      // L1: radio pequeño (semilla)
    .oscillatory(85.0, 0.0)                            // L2: banda Flora
    .flow(Vec2::ZERO, DEFAULT_DISSIPATION_RATE)        // L3: inmóvil al inicio
    .coherence(MatterState::Solid, 800.0, 0.05)       // L4: sólida, flexible
    .nutrient(0.3, 0.2, 0.15, 0.5)                    // L9: nutrientes iniciales
    .growth_budget(0.0, 3, 0.0)                        // GrowthBudget: sin biomasa, water-limited
    .irradiance()                                      // IrradianceReceiver: listo para fotones
    .spawn(commands)
```

### 4.3 Comportamiento Emergente Esperado

| Condición Ambiental | Resultado Emergente | Mecanismo |
|---------------------|---------------------|-----------|
| Lux nucleus cercano | Crece hacia la luz | energy_gradient_2d → spine direction |
| Lux lejano o ausente | No crece / se marchita | photon_density ≈ 0 → photosynthetic_yield = 0 → budget = 0 |
| Suelo rico en N/P | Crece rápido, más ramas | liebig no limita → biomasa alta → branch_budget > threshold |
| Suelo pobre en N | Crece lento, pocas ramas | N es limitante → biomasa baja |
| Temperatura alta (plasma) | Muere | state → Plasma → dissipation_mult = 3.0 → qe drena |
| Temperatura baja (sólido denso) | Latencia | growth_size_feedback → 0 (radio = 0 → sin crecimiento) |
| Competencia (otra planta cerca) | Ambas crecen menos | NutrientFieldGrid se agota localmente |
| Hechizo de fuego (Ignis) | Arde | Interferencia destructiva → bond_energy baja → qe drena |
| Agua abundante | Crece más (water no limita) | liebig: min(C,N,P,water) sube |

### 4.4 Fototropismo: El Mecanismo Exacto

```
1. EnergyFieldGrid tiene celdas con qe acumulado.
2. Un nucleus Lux inyecta qe alto en su radio.
3. energy_gradient_2d(left, right, down, up) calcula ∇E.
4. El gradiente apunta HACIA la fuente Lux (mayor concentración).
5. shape_inferred_direction(gradient, blend) → Vec3 con sesgo horizontal.
6. GF1 build_flow_spine() sigue esa dirección:
   - energy_push = dot(energy_direction, tangent)
   - Si push ≥ resistance → segmento recto (hacia la luz)
   - Si push < resistance → curva hacia least_resistance_dir
7. El resultado: la planta CURVA hacia la fuente de luz.
   Sin "if light then rotate". La geometría emerge de la física.
```

---

## 5. Constantes Nuevas (`blueprint/constants/`)

```rust
// ── Flora: Irradiance Propagation ──
/// Radio máximo de propagación de fotones desde una fuente Lux.
pub const FLORA_IRRADIANCE_MAX_RADIUS: f32 = 25.0;

/// Presupuesto de entidades actualizadas por frame para irradiancia.
pub const MAX_IRRADIANCE_SOURCES_PER_FRAME: u32 = 16;

// ── Flora: Nutrient Uptake ──
/// Radio de absorción de nutrientes del grid (mundo → entidad).
pub const NUTRIENT_UPTAKE_RADIUS_CELLS: u32 = 1;

/// Tasa base de absorción de nutrientes por tick.
pub const NUTRIENT_UPTAKE_RATE: f32 = 0.005;

/// Presupuesto de entidades con uptake por frame.
pub const MAX_NUTRIENT_UPTAKE_PER_FRAME: u32 = 64;

// ── Flora: Growth Budget Orchestration ──
/// Umbral de biomasa para activar branching (primera rama).
pub const GROWTH_BRANCH_BIOMASS_THRESHOLD: f32 = 0.3;

/// Tasa de conversión de photosynthetic_yield → biomass.
pub const PHOTO_TO_BIOMASS_RATE: f32 = 0.8;

/// Eficiencia mínima para que el crecimiento proceda.
pub const GROWTH_MIN_EFFICIENCY: f32 = 0.01;
```

---

## 6. Sistemas de Orquestación: Diseño Detallado

### 6.1 `irradiance_propagation_system` (FL1)

**Fase:** `Phase::PrePhysics` (antes de growth, después de containment)
**Query:** `(&Transform, &BaseEnergy, &OscillatorySignature)` — fuentes Lux
**Muta:** `Query<(&Transform, &mut IrradianceReceiver)>` — receptores
**Recurso:** `Res<AlchemicalAlmanac>` — para identificar banda Lux

```
Para cada fuente con freq ∈ [LUX_BAND_MIN_HZ, LUX_BAND_MAX_HZ]:
  Para cada receptor en radio FLORA_IRRADIANCE_MAX_RADIUS:
    photon_density += irradiance_at_distance_sq(source.qe, dist_sq, IRRADIANCE_LUX_DECAY)
```

**Guard:** `Changed<Transform>` en fuentes (si la fuente no se mueve, skip).
**Budget:** MAX_IRRADIANCE_SOURCES_PER_FRAME fuentes por frame.

### 6.2 `nutrient_uptake_system` (FL2)

**Fase:** `Phase::Physics`
**Query:** `(&Transform, &mut NutrientProfile)` — entidades con metabolismo
**Recurso:** `ResMut<NutrientFieldGrid>`

```
Para cada entidad con NutrientProfile:
  cell = grid.cell_at(transform.translation.xz())
  delta_C = (cell.carbon - profile.carbon) * NUTRIENT_UPTAKE_RATE * dt
  delta_N = (cell.nitrogen - profile.nitrogen) * NUTRIENT_UPTAKE_RATE * dt
  ...
  Guard: if delta.abs() > NUTRIENT_WRITE_EPS → apply
  Deplete cell proportionally
```

**Budget:** MAX_NUTRIENT_UPTAKE_PER_FRAME entidades por frame.
**LOD:** Solo Near+Mid (Far entities dormant).

### 6.3 `growth_budget_system` (FL3)

**Fase:** `Phase::PostPhysics` (después de nutrient uptake y irradiance)
**Query:** `(&NutrientProfile, &IrradianceReceiver, &mut GrowthBudget, Option<&MatterCoherence>)`
**Filter:** `Changed<NutrientProfile>.or(Changed<IrradianceReceiver>)`

```
efficiency = genetic_efficiency_for_element(bond_energy, electronegativity)
(budget, limiter) = liebig_growth_budget(C, N, P, water, efficiency)
photo_bonus = photosynthetic_growth_bonus(photon_density, absorbed_fraction)
total = budget + photo_bonus * PHOTO_TO_BIOMASS_RATE
Guard: if growth.biomass_available != total → set
```

### 6.4 `allometric_growth_system` (FL4)

**Fase:** `Phase::PostPhysics` (después de growth_budget_system)
**Query:** `(&GrowthBudget, &mut SpatialVolume, &AllometricRadiusAnchor)`
**Filter:** `Changed<GrowthBudget>`

```
max_radius = anchor.base_radius * ALLOMETRIC_MAX_RADIUS_FACTOR
delta_r = growth_size_feedback(growth.biomass_available, volume.radius, max_radius)
Guard: if delta_r.abs() > VOLUME_MIN_RADIUS → volume.set_radius(current + delta_r)
// Changed<SpatialVolume> dispara:
//   → visual_derivation recalcula scale
//   → density cambia → temperature cambia → posible transición de fase
//   → shape_inference regenera mesh (growth_morphology_system detecta Changed<GrowthBudget>)
```

---

## 7. Integración con Optimizaciones Existentes

### 7.1 BridgeCache

| Bridge | Ecuación Cacheada | Normalización |
|--------|-------------------|---------------|
| `BridgeIrradiance` (nuevo) | `irradiance_at_distance_sq` | photon_density / PHOTO_MAX_PHOTON_DENSITY → [0,1] |
| `BridgeGrowth` (nuevo) | `liebig_growth_budget` | budget / max_budget → [0,1] |

### 7.2 LOD Bands

| Banda | Irradiance | Nutrient Uptake | Growth | Morphology |
|-------|-----------|----------------|--------|------------|
| Near (≤30m) | Cada tick | Cada tick | Cada tick | Cada frame |
| Mid (30-80m) | Cada 4 ticks | Cada 4 ticks | Cada 4 ticks | Cada 4 frames |
| Far (>80m) | Skip | Skip | Skip | Dormant (mesh congelado) |

### 7.3 Changed<T> Chain

```
Changed<IrradianceReceiver> ──┐
                               ├──► growth_budget_system ──► Changed<GrowthBudget>
Changed<NutrientProfile> ─────┘                                    │
                                                                   ▼
                                               allometric_growth_system ──► Changed<SpatialVolume>
                                                                                    │
                                                                                    ▼
                                                              growth_morphology_system (mesh rebuild)
```

Si nada cambia → **cero CPU**. La cadena entera se salta por changed detection.

### 7.4 Budget-per-frame

Cada sistema respeta un presupuesto:
- Irradiance: 16 fuentes/frame
- Nutrient uptake: 64 entidades/frame
- Growth budget: ilimitado (Changed<T> ya filtra)
- Morphology rebuild: 16 mesh/frame (existente en shape_inference)

### 7.5 Quantized Color (Sprint 14)

La planta usa el mismo pipeline visual:
- `PaletteRegistry` asigna color por WorldArchetype
- `QuantizedPrecision(ρ)` degrada color en Far band
- Vertex colors del GF1 mesh usan `palette_tint_rgb()` existente

---

## 8. Demo de Validación: La Rosa

### 8.1 Escena

```
Map: 32×32 grid (pequeño, para depuración)
Nuclei:
  - terra_soil: centro, freq=85Hz, qe=500, radio=12 (sustrato)
  - lux_sun:    offset (8,0), freq=1000Hz, qe=300, radio=20 (sol)
  - aqua_rain:  offset (-5,0), freq=250Hz, qe=200, radio=10 (humedad)

Entidad Rosa:
  - Spawn en celda (16, 16) — centro del suelo Terra
  - Firma Flora: freq=85Hz, bond_energy=800, electronegativity=2.8
  - NutrientProfile, GrowthBudget, IrradianceReceiver
  - SpatialVolume(0.08) — semilla pequeña

Expectativa:
  - Tick 0-30: Rosa absorbe nutrientes del suelo, acumula fotones del Lux
  - Tick 30-100: biomass_available crece, radius crece, spine aparece
  - Tick 100-300: spine se curva hacia lux_sun (fototropismo emergente)
  - Tick 300+: branching activado si biomass > GROWTH_BRANCH_BIOMASS_THRESHOLD
  - La rosa en zona Aqua crece más rápido (water no limita)
  - La rosa lejos del Lux crece menos / recta (sin gradiente fuerte)
```

### 8.2 Criterios de Éxito Visual

1. La geometría GF1 apunta visiblemente hacia la fuente Lux.
2. Entidades con más nutrientes son más grandes (más segmentos, más ramas).
3. Entidades sin Lux no crecen (mesh mínimo o ausente).
4. El color refleja el estado: verde saludable → amarillo sin N → marrón muerto.
5. `cargo test` pasa sin regresión.

---

## 9. NO Hace

- No crea un tipo "Planta" o "Rosa" — la flora es composición de capas.
- No agrega sistemas a `Update` — todo en `FixedUpdate` con Phase.
- No modifica GF1 ni branching engine — los consume tal cual.
- No agrega crates externos.
- No implementa reproducción / dispersión de semillas (futuro).
- No implementa estaciones / ciclo día-noche (usa Climate existente si disponible).

---

## 10. Referencias

- `docs/design/THERMODYNAMIC_LADDER.md` — capas C1-C5
- `docs/design/GEOMETRY_DEFORMATION_ENGINE.md` — GF2 (futuro)
- `src/blueprint/equations.rs` — todas las funciones puras
- `src/geometry_flow/mod.rs` — GF1 spine engine
- `src/geometry_flow/branching.rs` — árbol recursivo
- `src/worldgen/shape_inference.rs` — bridge worldgen → GF1
- `src/worldgen/nutrient_field.rs` — grid de nutrientes
- `src/layers/growth.rs` — GrowthBudget, AllometricRadiusAnchor
- `src/layers/nutrient.rs` — NutrientProfile
- `src/layers/irradiance.rs` — IrradianceReceiver
- `src/entities/builder.rs` — EntityBuilder API
