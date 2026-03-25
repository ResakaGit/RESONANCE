# BLUEPRINT — Morfogénesis Inferida por Flujos de Energía y Optimización Termodinámica

---

## 0. Resumen Ejecutivo

Este blueprint establece la arquitectura para un **Motor de Inferencia Morfológica** donde la forma, composición y fenotipo de entidades biológicas complejas no se diseñan explícitamente, sino que se **infieren matemáticamente** a partir de restricciones termodinámicas. Los organismos se modelan como Grafos Acíclicos Dirigidos (DAGs) de procesamiento de exergía, incrustados en un entorno físico. La forma 3D y el color emergen como la solución óptima que minimiza la producción de entropía.

**Pilares teóricos:**
- Estructuras Disipativas (Prigogine, 1977)
- Ley Constructal (Bejan, 1997)
- Escalamiento Metabólico (West-Brown-Enquist, 1997)

**Relación con Resonance:** Este no es un sistema nuevo desde cero. Es la **culminación lógica** de los subsistemas existentes (14 capas ECS, GF1 geometry flow, organ inference, V7 worldgen, thermodynamic ladder). El blueprint define las piezas faltantes y el pegamento que las unifica bajo una sola teoría.

---

## 1. Auditoría: Qué Tenemos vs Qué Falta

### Dominio Ambiental (Capa 1 del blueprint teórico)

| Concepto | Estado | Ubicación en `src/` |
|----------|--------|---------------------|
| Campo escalar de densidad | **Implementado** | `worldgen/field_grid.rs` → `EnergyFieldGrid` |
| Temperatura equivalente | **Implementado** | `blueprint/equations.rs` → `equivalent_temperature()` |
| Gradientes de nutrientes (C,N,P,H₂O) | **Implementado** | `worldgen/nutrient_field.rs` → `NutrientFieldGrid` |
| Irradiancia (campo de radiación) | **Parcial** | `layers/irradiance.rs` → `IrradianceReceiver` (receptor, sin propagación solar completa) |
| Gradiente de energía 2D | **Implementado** | `blueprint/equations.rs` → `energy_gradient_2d()` |
| Viscosidad / conductividad del medio | **Implementado** | `layers/pressure.rs` → `AmbientPressure` |
| Campo gravitatorio explícito | **Falta** | Gravedad implícita en drenaje topológico, no como campo vectorial |
| Viento / presión atmosférica | **Falta** | Sin forcing advectivo a escala macro |

### DAG Metabólico (Capa 2)

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| Motor alquímico (buffer/válvulas) | **Implementado** | `layers/engine.rs` → `AlchemicalEngine` |
| Escalamiento alométrico (intake ∝ r²) | **Implementado** | `blueprint/equations.rs` → `allometric_intake()` |
| Fotosíntesis (producción primaria) | **Implementado** | `simulation/photosynthesis.rs` |
| Captación de nutrientes | **Implementado** | `simulation/nutrient_uptake.rs` |
| Estrés metabólico / inanición | **Implementado** | `simulation/metabolic_stress.rs` |
| Recetas de transmutación | **Parcial** | `blueprint/recipes.rs` (pocos ejemplos) |
| DAG explícito de cadenas de producción | **Falta** | No hay grafo de órganos con aristas de flujo |
| Eficiencia de Carnot por nodo | **Falta** | Sin límite termodinámico en conversión |
| Producción de entropía por nodo (S_gen) | **Falta** | Sin tracking de entropía generada |
| Exergía / trabajo disponible | **Falta** | Sin cálculo de energía útil vs disipada |

### Inferencia Morfológica (Capa 3 — "El Compilador")

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| GF1 flora-tubo (spine + mesh) | **Implementado** | `geometry_flow/mod.rs`, `branching.rs` |
| Ramificación fractal con decaimiento | **Implementado** | `geometry_flow/branching.rs` |
| Dirección inferida por gradiente | **Implementado** | `blueprint/equations.rs` → `shape_inferred_direction()` |
| Inferencia de órganos (12 roles) | **Implementado** | `layers/organ.rs`, `blueprint/equations.rs` |
| Primitivas geométricas (Tube, FlatSurface, PetalFan, Bulb) | **Implementado** | `geometry_flow/primitives.rs` |
| Optimización de arrastre (C_D) → forma fusiforme | **Falta** | Arrastre existe en ecuaciones pero no retroalimenta la forma |
| Embedding espacial por minimización de C_shape | **Falta** | Sin optimizador variacional de envoltura |
| Red vascular interna (fractales WBE) | **Parcial** | Branching fractal existe; no optimiza ángulos por distribución |

### Inferencia de Color (Albedo)

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| Hz + pureza → RGB | **Implementado** | `blueprint/equations.rs` → `field_linear_rgb_from_hz_purity()` |
| Interferencia constructiva/destructiva → blend | **Implementado** | `blueprint/equations.rs` → `compound_field_linear_rgba()` |
| Escala visual por densidad + estado | **Implementado** | Derivación stateless en `worldgen/visual_derivation.rs` |
| Emisión por temperatura + estado | **Implementado** | `derive_emission()` |
| Albedo dinámico por balance térmico | **Falta** | Color es independiente del balance de calor |
| Ecuación de equilibrio radiativo completa | **Falta** | No hay feedback color ↔ temperatura |

### Termodinámica

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| Transiciones de estado (S→L→G→P) | **Implementado** | `state_from_temperature()` |
| Transferencia térmica (conducción, convección, radiación) | **Implementado** | `thermal_transfer()` |
| Disipación con arrastre cuadrático | **Implementado** | `dissipation_effective()`, `drag_force()` |
| Capacidad calorífica (C_v) | **Falta** | dT/dE sin masa térmica |
| Producción de entropía (S_gen = Q/T) | **Falta** | |
| Eficiencia de Carnot (η ≤ 1 - T_cold/T_hot) | **Falta** | |
| Segunda Ley (restricción de dirección) | **Falta** | |

### Composición Funcional (Matrioska)

| Concepto | Estado | Ubicación |
|----------|--------|-----------|
| Funciones puras en `equations.rs` (el "órgano lógico") | **Implementado** (95%) | `blueprint/equations.rs` — 900+ LOC de funciones puras |
| Sistemas como transformaciones únicas (no god-systems) | **Implementado** | Regla: "one system, one transformation" |
| Constantes centralizadas por módulo | **Implementado** | `{module}/constants.rs` en blueprint, bridge, eco, topology, worldgen |
| BridgeCache como memoización termodinámica | **Implementado** (11 tipos) | `bridge/cache.rs` — DensityBridge, InterferenceBridge, DissipationBridge, etc. |
| LOD Near/Mid/Far como coarse-graining (Renormalización) | **Implementado** | `worldgen/lod.rs` — Far cada 16 ticks, Mid cada 4 |
| Retorno Writer-monad: `fn → (output_útil, Q_diss, W_waste)` | **Falta** | Funciones retornan escalar; sin tupla de desechos explícita |
| Composición encadenada de órganos (output_i → input_{i+1}) | **Parcial** | Sistemas se encadenan por `.after()`, pero no por valor |
| Nesting jerárquico (célula → tejido → órgano → sistema) | **Falta** | Entidades son composición plana de 14 layers |
| Libro contable de entropía (`EntropyLedger`) | **Falta** | Disipación se computa in-place, no se acumula |
| Operadores de upscaling/downscaling entre escalas | **Falta** | LOD es por frecuencia, no por agregación espacial explícita |

---

## 2. Arquitectura de 4+1 Capas (Mapping a Resonance)

Los dos blueprints teóricos definen 4 capas de responsabilidad + 1 capa de composición funcional (Matrioska). Aquí se mapean a módulos existentes y nuevos:

```
┌─────────────────────────────────────────────────────────────────┐
│ CAPA 1: DOMINIO AMBIENTAL                                       │
│ (Condiciones de Contorno)                                       │
│                                                                  │
│ EXISTENTE:                          NUEVO:                       │
│ · EnergyFieldGrid (V7)             · GravityField (Vec3 global) │
│ · NutrientFieldGrid                · WindField (advección macro) │
│ · AmbientPressure (L6)             · SolarIrradiance completa   │
│ · Terrain (topology/)              · PressureWaveField           │
│ · Climate (eco/)                                                 │
├─────────────────────────────────────────────────────────────────┤
│ CAPA 2: DOMINIO ONTOLÓGICO (DAG Metabólico + Matrioska)         │
│ (HOFs monádicas de procesamiento de exergía)                     │
│                                                                  │
│ EXISTENTE:                          NUEVO:                       │
│ · AlchemicalEngine (L5)            · MetabolicGraph (DAG)       │
│ · allometric_intake/consumption    · ExergyNode (por órgano)    │
│ · Photosynthesis                   · carnot_efficiency()         │
│ · NutrientProfile (L4)            · entropy_production()        │
│ · EffectRecipe                     · exergy_balance()            │
│ · OrganManifest (inference)        · EntropyLedger (Writer)     │
│ · BridgeCache (memoización)        · organ_transform() tuplas   │
├─────────────────────────────────────────────────────────────────┤
│ CAPA 3: MOTOR DE INFERENCIA MORFOLÓGICA                          │
│ ("El Compilador": DAG + Ambiente → Fenotipo)                     │
│                                                                  │
│ EXISTENTE:                          NUEVO:                       │
│ · GF1 (spine + mesh + branch)      · drag_shape_optimizer()     │
│ · shape_inferred_direction/length  · albedo_from_thermal_bal()  │
│ · organ_inference (12 roles)       · vascular_network_optim()   │
│ · organ_attachment_points           · spatial_embedding_solver() │
│ · LifecycleStageCache              · surface_rugosity_solver()  │
├─────────────────────────────────────────────────────────────────┤
│ CAPA 4: INTEGRADOR DINÁMICO (Runtime)                            │
│ (FixedUpdate, dt, flujos entre nodos)                            │
│                                                                  │
│ EXISTENTE (100%):                   NUEVO (Matrioska runtime):   │
│ · SimulationPlugin + Phase pipe    · Lazy eval por LOD band     │
│ · FixedUpdate con Time<Fixed>      · Coarse-grain Far→Near      │
│ · 6 Phases encadenados             · BridgeMatrioska (multi-    │
│ · BridgeCache<B> (11 tipos)          escala memoizado)          │
│ · LOD Near/Mid/Far                                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Especificación de Componentes Nuevos

### 3.1 MetabolicGraph (DAG de producción)

El corazón del sistema. Grafo acíclico dirigido donde nodos = órganos funcionales, aristas = flujos de energía/materia.

```rust
/// Nodo funcional en el DAG metabólico.
/// No es un ECS component — es un dato dentro de MetabolicGraph.
pub struct ExergyNode {
    pub role: OrganRole,           // Reutiliza los 12 roles existentes
    pub efficiency: f32,           // η ∈ (0, 1] — limitado por Carnot
    pub activation_energy: f32,    // E_a (costo mínimo para operar)
    pub thermal_output: f32,       // Q_diss generado por tick (derivado)
    pub entropy_rate: f32,         // S_gen = Q_diss / T_core (derivado)
}

/// Arista dirigida: flujo de energía entre nodos.
pub struct ExergyEdge {
    pub flow_rate: f32,            // J (qe/s) — flujo actual
    pub max_capacity: f32,         // Límite por sección del "vaso"
    pub transport_cost: f32,       // μ × L³/r⁴ (WBE: costo de distribución)
}

/// DAG metabólico completo. Component ECS, SparseSet.
/// Solo entidades "vivas complejas" lo tienen.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct MetabolicGraph {
    pub nodes: ArrayVec<ExergyNode, 12>,  // Max 12 nodos (= max organs)
    pub edges: ArrayVec<ExergyEdge, 16>,  // Max 16 conexiones
    pub adjacency: ArrayVec<(u8, u8), 16>, // (from, to) indices
    pub total_entropy_rate: f32,           // Σ S_gen (derivado)
}
```

**Invariantes:**
1. Grafo acíclico (validado en construcción).
2. Al menos 1 nodo Captador y 1 nodo Disipador.
3. `efficiency` ≤ `carnot_efficiency(T_core, T_env)` para todo nodo.
4. Conservación: `Σ J_in = Σ J_out + P_work + Q_diss` por nodo.

### 3.2 Patrón Matrioska: Composición Funcional Fractal

El MetabolicGraph implementa el patrón Matrioska: cada nodo del DAG es una **función pura** que transforma `(M_in, E_in) → (M_out, E_out, W, Q)` siguiendo el Writer Monad termodinámico.

#### 3.2.1 La Función Pura como Órgano

```rust
/// Resultado de evaluar un nodo/órgano del DAG.
/// Writer monad: output útil + log de desechos (W, Q).
pub struct OrganOutput {
    pub mass_out: f32,      // M_out: masa transformada (útil)
    pub exergy_out: f32,    // E_out: exergía útil disponible
    pub waste_mass: f32,    // W: desecho material (toxinas)
    pub heat_dissipated: f32, // Q: desecho térmico (entropía generada)
}

/// Evalúa un nodo del DAG como función pura.
/// Cumple Landauer: Q ≥ kT × ln(2) por bit de transformación.
/// Cumple Carnot: η ≤ 1 - T_env / T_core.
pub fn organ_transform(
    mass_in: f32,
    exergy_in: f32,
    efficiency: f32,        // η del nodo (pre-clamped por Carnot)
    activation_energy: f32, // E_a (costo mínimo)
) -> OrganOutput;
```

**Propiedad clave:** `mass_in = mass_out + waste_mass` y `exergy_in = exergy_out + heat_dissipated + activation_energy`. Conservación estricta.

#### 3.2.2 La HOF como Sistema (Composición Encadenada)

Un "Sistema" (ej. sistema digestivo) es una HOF que compone N funciones puras.

```rust
/// Compone una cadena de funciones-órgano siguiendo la topología del DAG.
/// Routing: output_i → input_{i+1} según adjacency.
/// Aggregation: Σ Q y Σ W se acumulan en el EntropyLedger.
pub fn evaluate_metabolic_chain(
    graph: &MetabolicGraph,
    initial_mass: f32,
    initial_exergy: f32,
) -> ChainOutput;

pub struct ChainOutput {
    pub final_exergy: f32,       // Exergía neta tras toda la cadena
    pub total_heat: f32,         // Σ Q de todos los nodos
    pub total_waste: f32,        // Σ W de todos los nodos
    pub per_node_heat: [f32; 12], // Q por nodo (para inferencia morfológica)
}
```

**Invariante de composición:** La cadena es una función pura. Mismos inputs → mismos outputs. Sin estado mutable. El "registro" de desechos se propaga hacia arriba (Matrioska: célula → tejido → órgano → sistema → organismo).

#### 3.2.3 EntropyLedger (El Libro Contable)

```rust
/// Acumula Q y W a lo largo de la evaluación del DAG.
/// Vive como Component SparseSet — solo en entidades con MetabolicGraph.
/// Se recomputa cada tick (no es estado persistente, es derivado).
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct EntropyLedger {
    pub total_heat_generated: f32,    // Σ Q (input para albedo inference)
    pub total_waste_generated: f32,   // Σ W (input para excretion system)
    pub entropy_rate: f32,            // S_gen = Σ Q / T_core (W/K)
    pub exergy_efficiency: f32,       // E_out_final / E_in_initial
}
```

**Uso downstream:** El Motor de Inferencia Morfológica lee `EntropyLedger.total_heat_generated` como `Q_metabolico_interno` en la ecuación de albedo y como driver de `surface_rugosity_solver()` (más calor → más superficie → aletas/orejas/radiadores).

#### 3.2.4 Memoización Termodinámica (Evaluación Perezosa)

La Matrioska resuelve la explosión combinatoria mediante el patrón que ya existe: **BridgeCache**.

```
Escala Matrioska          Evaluación en Resonance
─────────────────         ──────────────────────────
Célula (fn pura)     →    organ_transform() — O(1) aritmética
Tejido (componer N)  →    evaluate_metabolic_chain() — O(N_nodos)
Órgano (sub-DAG)     →    BridgeMetabolicStep — memoizado por cuantización
Sistema (HOF)        →    LOD Far: resultado cacheado, sin re-evaluación
Organismo            →    EntropyLedger — resumen O(1) de todo el DAG
Población            →    Coarse-grain: media estadística de EntropyLedger
```

**Isomorfismo de escala (Renormalization Group):** Las ecuaciones son las mismas a toda escala. `organ_transform()` funciona igual para una mitocondria que para un hígado — solo cambian los parámetros (η, E_a, capacidad). Esto valida matemáticamente que la Matrioska no es un truco de software sino una propiedad del modelo físico.

#### 3.2.5 Inferencia de Superficie desde Q (Ley Cuadrático-Cúbica)

Cuando el DAG acumula un Q_total extremo pero el volumen requerido es pequeño, la matemática fuerza a maximizar A_surf sin aumentar V. Esto produce emergencia de:

- **Aletas disipadoras** (como orejas de elefante o aletas dorsales)
- **Superficies rugosas/plegadas** (como alvéolos o vellosidades intestinales)
- **Vascularización expuesta** (como crestas de gallos o branquias externas)

```rust
/// Rugosidad de superficie inferida desde ratio Q/V.
/// Si Q_total / V_total > umbral, la superficie necesita más área.
/// rugosity ∈ [1.0, max_rugosity] donde 1.0 = esfera lisa.
pub fn inferred_surface_rugosity(
    q_total: f32,
    volume: f32,
    t_core: f32,
    t_env: f32,
    convection_coeff: f32,
) -> f32;
```

### 3.3 Ecuaciones Puras (en `equations.rs`)

```rust
/// Eficiencia de Carnot: límite termodinámico superior.
/// η_max = 1 - T_cold / T_hot
pub fn carnot_efficiency(t_core: f32, t_env: f32) -> f32;

/// Producción de entropía por nodo.
/// S_gen = Q_diss / T_core (W/K equivalente)
pub fn entropy_production(q_diss: f32, t_core: f32) -> f32;

/// Balance de exergía: energía útil disponible tras pérdidas.
/// Ex = J_in × η - E_activation
pub fn exergy_balance(j_in: f32, efficiency: f32, activation_energy: f32) -> f32;

/// Costo de forma: arrastre + estructura interna (Constructal).
/// C_shape = 0.5 × ρ × v² × C_D(Ω) × A_proj + Σ(μ × L³/r⁴)
pub fn shape_cost(
    medium_density: f32, velocity: f32, drag_coeff: f32,
    projected_area: f32, vascular_cost: f32,
) -> f32;

/// Costo de transporte vascular (WBE).
/// C_transport = μ × L³ / r⁴ (Hagen-Poiseuille simplificado)
pub fn vascular_transport_cost(viscosity: f32, length: f32, radius: f32) -> f32;

/// Albedo inferido por balance térmico de superficie.
/// Despeja α de: Q_met + (1-α)×I×A_proj = ε×σ×(T⁴_core - T⁴_env)×A_surf + h×ΔT×A_surf
pub fn inferred_albedo(
    q_metabolic: f32, solar_irradiance: f32, proj_area: f32,
    emissivity: f32, t_core: f32, t_env: f32, surf_area: f32,
    convection_coeff: f32,
) -> f32;

/// Coeficiente de arrastre inferido para sólido de revolución.
/// Aproximación de Myring body: C_D = f(fineness_ratio)
pub fn inferred_drag_coefficient(length: f32, max_diameter: f32) -> f32;

/// Capacidad calorífica efectiva.
/// C_v = qe × specific_heat_factor (proporcional a masa energética)
pub fn heat_capacity(qe: f32, specific_heat_factor: f32) -> f32;
```

### 3.3 Nuevos Sistemas ECS

| Sistema | Phase | Reads | Writes | Propósito |
|---------|-------|-------|--------|-----------|
| `metabolic_graph_step_system` | MetabolicLayer | MetabolicGraph, BaseEnergy, AmbientPressure | MetabolicGraph (flujos + derivados) | Paso temporal: computa J por arista, Q_diss por nodo |
| `entropy_constraint_system` | MetabolicLayer | MetabolicGraph | MetabolicGraph (clamp eficiencias) | Aplica límite de Carnot, ajusta η si T cambia |
| `albedo_inference_system` | MorphologicalLayer | MetabolicGraph, IrradianceReceiver, AmbientPressure, SpatialVolume | Insert/Mut `InferredAlbedo` marker | Despeja α de balance térmico → modula color |
| `shape_optimization_system` | MorphologicalLayer | MetabolicGraph, FlowVector, AmbientPressure, SpatialVolume | Mut GeometryInfluence (ajusta forma) | Minimiza C_shape → modula fineness ratio |

---

## 4. Flujo de Datos: De la Energía a la Forma

```
  ┌──────────────────┐     ┌──────────────────┐
  │ Dominio Ambiental│     │  DAG Metabólico   │
  │                  │     │                   │
  │ · T_env          │     │ · ExergyNodes[]   │
  │ · ρ_medio        │     │ · ExergyEdges[]   │
  │ · I_solar        │     │ · Q_diss total    │
  │ · ∇nutrientes    │     │ · η por nodo      │
  └────────┬─────────┘     └────────┬──────────┘
           │                        │
           ▼                        ▼
  ┌─────────────────────────────────────────────┐
  │        MOTOR DE INFERENCIA (Phase::Morphological)       │
  │                                                          │
  │  1. carnot_efficiency(T_core, T_env) → η_max            │
  │  2. entropy_production(Q_diss, T_core) → S_gen           │
  │  3. shape_cost(ρ, v, C_D, A) → C_shape                  │
  │  4. inferred_albedo(Q_met, I, ...) → α                   │
  │  5. inferred_drag_coefficient(L, D) → C_D                │
  │  6. vascular_transport_cost(μ, L, r) → C_vasc            │
  │                                                          │
  │  Optimización iterativa:                                  │
  │  · Ajustar fineness_ratio hasta min(C_shape)             │
  │  · Despejar α para equilibrio térmico                    │
  │  · Propagar conteos de órganos desde OrganManifest       │
  └──────────────────────┬──────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────┐
  │         FENOTIPO RESULTANTE                  │
  │                                              │
  │  · GeometryInfluence (forma modulada)        │
  │  · GF1 mesh (spine + branching + organs)     │
  │  · Color = Hz×pureza modulado por α          │
  │  · Scale = density + state                   │
  │  · Emission = T + state                      │
  └──────────────────────────────────────────────┘
```

---

## 5. Qué Ganamos

### 5.1 Emergencia Genuina

Actualmente la forma se infiere desde `(stage × capabilities × biases × biomass)` — un sistema ya poderoso. Con el DAG metabólico, la forma se infiere desde **la termodinámica misma**: un organismo en agua densa con alto metabolismo **necesariamente** converge a forma fusiforme porque es la única solución que minimiza C_shape. No es un parámetro de diseño; es una consecuencia matemática.

### 5.2 Color con Significado Físico

Hoy: `color = f(Hz, pureza)` — decorativo, determinista, bonito, pero desacoplado del estado térmico. Con albedo inferido: un organismo sobrecalentado en un desierto **necesita** ser claro para sobrevivir. El color deja de ser cosmético y se convierte en información de supervivencia para el jugador.

### 5.3 Depth Strategica (MOBA)

El DAG metabólico crea un **sistema de órganos atacables**. Un jugador puede:
- Destruir el "nodo Captador" → cortar la alimentación
- Dañar el "nodo Distribuidor" → reducir flujo a todos los órganos downstream
- Sobrecalentar al enemigo → forzar cambio de albedo → hacerlo visible en fog

Esto añade profundidad táctica sin agregar stats ni cooldowns arbitrarios.

### 5.4 Unificación Teórica

Los 14 layers + GF1 + V7 + organ inference + thermodynamic ladder son subsistemas excelentes pero actualmente operan con coupling implícito. El Motor de Inferencia Morfológica los unifica bajo una sola teoría (minimización de producción de entropía) con ecuaciones explícitas en `equations.rs`.

### 5.5 Invarianza de Escala (Matrioska)

El patrón Writer Monad + HOF garantiza que las ecuaciones sean isomorfas a toda escala. La misma `organ_transform()` describe una mitocondria o un hígado — solo cambian (η, E_a, capacidad). Esto tiene tres consecuencias:

1. **Zoom semántico.** Un jugador podría "inspeccionar" un organismo y ver el DAG interno con sus flujos. LOD Near muestra nodos individuales; LOD Far colapsa todo a `EntropyLedger` — mismo modelo, distinta resolución.

2. **Testabilidad total.** Cada nivel de la Matrioska es una función pura testeable independientemente. No hay integración implícita que solo se valide en runtime.

3. **Composición sin límite.** Si en el futuro se quieren modelar ecosistemas como "super-organismos" (la selva como DAG de poblaciones), la misma arquitectura escala sin refactor.

### 5.6 Emergencia Morfológica Real (Ley Cuadrático-Cúbica)

La Matrioska añade un fenómeno que el DAG solo no producía: cuando `Q_total / V_total` es alto (organismo denso y caliente), la superficie **debe** aumentar sin aumentar volumen. El solver no tiene otra opción que generar aletas, pliegues o radiadores. Un dragón con alta producción metabólica emergería con crestas dorsales no porque un artista las diseñó, sino porque la termodinámica las exige.

---

## 6. Plan de Implementación por Sprints

### Sprint MG-1: Fundamentos Termodinámicos (Ecuaciones)

**Scope:** Solo `equations.rs` + `constants.rs` + tests.

- [ ] `carnot_efficiency(t_core, t_env) → f32`
- [ ] `entropy_production(q_diss, t_core) → f32`
- [ ] `exergy_balance(j_in, η, e_a) → f32`
- [ ] `heat_capacity(qe, c_v_factor) → f32`
- [ ] `vascular_transport_cost(μ, L, r) → f32`
- [ ] `shape_cost(ρ, v, C_D, A_proj, C_vasc) → f32`
- [ ] `inferred_drag_coefficient(L, D_max) → f32`
- [ ] `inferred_albedo(...) → f32`
- [ ] Constantes: `STEFAN_BOLTZMANN`, `DEFAULT_EMISSIVITY`, `DEFAULT_CONVECTION_COEFF`, `SPECIFIC_HEAT_FACTOR`
- [ ] 30+ tests unitarios validando rangos físicos y edge cases

**Dependencias:** Ninguna. Puras funciones sin ECS.

### Sprint MG-2: MetabolicGraph (Tipo + Builder)

**Scope:** Tipo de datos + constructor + validación.

- [ ] `MetabolicGraph`, `ExergyNode`, `ExergyEdge` en `layers/metabolic_graph.rs`
- [ ] `MetabolicGraphBuilder` (fluent API, valida acyclicidad)
- [ ] Integrar con `OrganManifest`: cada `OrganRole` → `ExergyNode` con eficiencia/activación default
- [ ] `metabolic_graph_from_manifest(manifest, t_core, t_env) → MetabolicGraph` en `equations.rs`
- [ ] Tests de grafo: conservación, acyclicidad, nodos mínimos

**Dependencias:** MG-1 (ecuaciones).

### Sprint MG-3: Paso Temporal del DAG

**Scope:** Sistema ECS que avanza flujos por tick.

- [ ] `metabolic_graph_step_system` en `simulation/metabolic_graph.rs`
- [ ] `entropy_constraint_system` (clamp η por Carnot cada N ticks)
- [ ] Registrar ambos en `Phase::MetabolicLayer`
- [ ] Bridge cache: `BridgeMetabolicStep` para entidades Far
- [ ] Tests de integración: flujo estable, inanición, sobrecalentamiento

**Dependencias:** MG-2 (tipo).

### Sprint MG-4: Inferencia de Forma (Shape Optimization)

**Scope:** Retroalimentación arrastre → forma.

- [ ] `shape_optimization_system` en `simulation/morphogenesis.rs`
- [ ] Alimentar `fineness_ratio` al `GeometryInfluence` existente
- [ ] El optimizador es iterativo pero acotado (max 3 pasos/frame, converge en ~10 frames)
- [ ] Tests: forma fusiforme en agua densa, forma expandida en aire

**Dependencias:** MG-3 + GF1 existente.

### Sprint MG-5: Inferencia de Albedo

**Scope:** Balance térmico → color.

- [ ] `albedo_inference_system` en `simulation/morphogenesis.rs`
- [ ] `InferredAlbedo` component (SparseSet, f32)
- [ ] Modular pipeline de color existente: `final_color = base_color × (1 - α) + white × α` (o blend por emisividad)
- [ ] Completar `IrradianceReceiver` con flujo solar real desde clima
- [ ] Tests: organismo caliente en desierto → α alto → claro

**Dependencias:** MG-3 + irradiancia.

### Sprint MG-6: Writer Monad y EntropyLedger (Matrioska Fase 1)

**Scope:** Patrón de composición funcional con acumulación de desechos.

- [ ] `OrganOutput` struct en `blueprint/equations.rs`
- [ ] `organ_transform(mass_in, exergy_in, η, E_a) → OrganOutput` en `equations.rs`
- [ ] `evaluate_metabolic_chain(graph, M_init, E_init) → ChainOutput` en `equations.rs`
- [ ] `EntropyLedger` component (SparseSet) en `layers/metabolic_graph.rs`
- [ ] `entropy_ledger_system` en `simulation/morphogenesis.rs` — evalúa cadena y escribe ledger
- [ ] Validar conservación: `M_in = M_out + W`, `E_in = E_out + Q + E_a` en 20+ tests
- [ ] Conectar `EntropyLedger.total_heat_generated` al albedo inference system (MG-5)

**Dependencias:** MG-2 (MetabolicGraph).

### Sprint MG-7: Inferencia de Superficie (Ley Cuadrático-Cúbica)

**Scope:** Q/V ratio → rugosidad de superficie → geometría emergente.

- [ ] `inferred_surface_rugosity(Q, V, T_core, T_env, h) → f32` en `equations.rs`
- [ ] `surface_rugosity_system` en `simulation/morphogenesis.rs` — Phase::MorphologicalLayer
- [ ] Modular `GeometryInfluence` para aceptar `rugosity` parameter
- [ ] Extender GF1: rugosity > 1.0 → subdivisión de superficie / aletas procedurales
- [ ] Tests: alto Q + bajo V → rugosity alta → aletas emergentes
- [ ] Tests: bajo Q → rugosity ≈ 1.0 → esfera lisa (sin cambio visual)

**Dependencias:** MG-6 (EntropyLedger) + GF1 existente.

### Sprint MG-8: Integración Visual y Demo

**Scope:** Demo jugable que muestre la teoría en acción.

- [ ] Mapa `morphogenesis_demo.ron` con 3 biomas (agua densa, desierto, bosque)
- [ ] 3 arquetipos con DAG distinto: planta terrestre, organismo acuático, criatura aérea
- [ ] Verificar emergencia: formas y colores distintos sin diseño explícito
- [ ] Verificar emergencia Matrioska: organismo en desierto → α alto + aletas, en agua → fusiforme + oscuro
- [ ] Integrar con EntityBuilder (`.with_metabolic_graph(...)`)
- [ ] Benchmark: < 1ms para 100 entidades con DAG completo + ledger

**Dependencias:** MG-4 + MG-5 + MG-7.

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Optimización de forma demasiado costosa en runtime | Lag en escenas con muchas entidades vivas | BridgeCache + LOD (Far = sin optimización, congelado) + max 3 iter/frame |
| Convergencia lenta del shape optimizer | Formas "jittering" entre frames | Histéresis + damping (misma estrategia que LifecycleStageCache) |
| MetabolicGraph añade complejidad a entidades simples | Overhead innecesario para proyectiles/cristales | SparseSet + solo entidades "vivas complejas" lo tienen; el resto no cambia |
| Albedo produce colores feos | Jugadores no entienden por qué algo es blanco | Clamp α ∈ [0.1, 0.9]; mantener matiz del elemento, solo modular luminosidad |
| Ecuaciones nuevas rompen balance MOBA | Daño/curación impredecible | Sprint MG-1 es solo ecuaciones puras con tests; no toca gameplay hasta MG-3 |
| Writer monad añade overhead de allocation | Tuplas `OrganOutput` por nodo por tick | Stack-allocated (Copy struct, 16 bytes); sin heap. O(N_nodos) es ~12 mults |
| Rugosity solver genera geometría excesiva | Tri count explota en entidades con aletas | Rugosity clamped a [1.0, 4.0]; aletas como LOD-aware primitivas (Near=mesh, Far=sprite) |
| Matrioska nesting complica debugging | Difícil rastrear de dónde viene Q_total | EntropyLedger.per_node_heat[12] expone contribución por nodo; debug overlay en DebugPlugin |
| EntropyLedger es "derivado" pero se almacena | Riesgo de desync con MetabolicGraph | Se recomputa cada tick desde evaluate_metabolic_chain(); nunca se lee de frame anterior |

---

## 8. Referencias Cruzadas

- `docs/design/BLUEPRINT.md` — Modelo base de 10→14 capas y ecuaciones originales
- `docs/design/THERMODYNAMIC_LADDER.md` — Escalera de complejidad (TL1–TL6)
- `docs/arquitectura/blueprint_living_organ_inference.md` — Organ inference existente (12 roles)
- `docs/arquitectura/blueprint_geometry_flow.md` — GF1 motor stateless
- `docs/arquitectura/blueprint_energy_field_inference.md` — Campo → muestra visual
- `docs/arquitectura/blueprint_thermodynamic_ladder.md` — Stack de rendimiento (BridgeCache, LOD)
- `docs/arquitectura/blueprint_ecosystem_autopoiesis.md` — Ciclo de vida autopoiético

### Fundamentos Teóricos (Matrioska)

- **Landauer (1961)** — Límite inferior de disipación por transformación de información: Q ≥ kT ln 2
- **Wilson / Kadanoff — Renormalization Group** — Invarianza de ecuaciones fundamentales al cambiar de escala (coarse-graining)
- **Bejan (1997) — Ley Constructal** — La forma emerge para facilitar corrientes de flujo; HOFs dictan flujos → forma los optimiza
- **Prigogine (1977) — Estructuras Disipativas** — Sistemas lejos del equilibrio minimizan producción de entropía específica
- **West, Brown, Enquist (1997) — WBE** — Redes de distribución fractales optimizan costo de transporte
