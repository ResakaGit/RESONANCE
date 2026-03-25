# Blueprint: Motor de Inferencia Morfológica — De la Energía a la Forma

Referencia de contrato para el blueprint teórico [`docs/design/MORPHOGENESIS.md`](../design/MORPHOGENESIS.md).
Conecta con: [`blueprint_living_organ_inference.md`](blueprint_living_organ_inference.md), [`blueprint_thermodynamic_ladder.md`](blueprint_thermodynamic_ladder.md), [`blueprint_geometry_flow.md`](blueprint_geometry_flow.md).
Template base: [`00_contratos_glosario.md`](00_contratos_glosario.md).

## 1) Propósito y frontera

- **Qué resuelve:** Unifica los subsistemas de inferencia (organ inference, GF1, V7 shape inference, visual derivation, thermodynamic ladder) bajo una teoría formal donde la morfología, el color y la estructura interna de entidades vivas se derivan como soluciones óptimas de un problema de minimización de producción de entropía. Introduce el `MetabolicGraph` (DAG de exergía) como representación del "alma lógica" de una entidad compleja, y los solvers que incrustan ese grafo en el espacio 3D.

- **Qué NO resuelve:**
  - No reemplaza las 14 capas ECS existentes — las consume y extiende.
  - No toca gameplay MOBA (facciones, habilidades, fog of war) — esos operan sobre el fenotipo resultante.
  - No simula fluidos (CFD) ni elementos finitos (FEM) — usa aproximaciones analíticas (Myring body, Hagen-Poiseuille simplificado).
  - No aplica a entidades simples (proyectiles, cristales, celdas de bioma) — solo a entidades con `MetabolicGraph`.

- **Naturaleza:** ~12 funciones puras nuevas en `equations.rs` + 2 tipos compuestos (`MetabolicGraph`, `EntropyLedger`) + 6 sistemas ECS + 2 component markers (`InferredAlbedo`, `EntropyLedger`) + extensión del pipeline visual + patrón de composición funcional Matrioska (Writer Monad termodinámico).

## 2) Superficie pública (contrato)

### Tipos nuevos

| Tipo | Ubicación | Naturaleza | Campos clave |
|------|-----------|------------|--------------|
| `ExergyNode` | `layers/metabolic_graph.rs` | struct, Copy | role: OrganRole, efficiency: f32, activation_energy: f32, thermal_output: f32, entropy_rate: f32 |
| `ExergyEdge` | `layers/metabolic_graph.rs` | struct, Copy | flow_rate: f32, max_capacity: f32, transport_cost: f32 |
| `MetabolicGraph` | `layers/metabolic_graph.rs` | Component, SparseSet | nodes: ArrayVec<12>, edges: ArrayVec<16>, adjacency: ArrayVec<16>, total_entropy_rate: f32 |
| `InferredAlbedo` | `layers/metabolic_graph.rs` | Component, SparseSet | albedo: f32 |
| `OrganOutput` | `blueprint/equations.rs` | struct, Copy | mass_out, exergy_out, waste_mass, heat_dissipated |
| `ChainOutput` | `blueprint/equations.rs` | struct, Copy | final_exergy, total_heat, total_waste, per_node_heat: [f32; 12] |
| `EntropyLedger` | `layers/metabolic_graph.rs` | Component, SparseSet | total_heat_generated, total_waste_generated, entropy_rate, exergy_efficiency |

### Funciones puras nuevas (en `equations.rs`)

| Función | Firma (simplificada) | Propósito |
|---------|----------------------|-----------|
| `carnot_efficiency` | (t_core, t_env) → f32 | Límite termodinámico: η_max = 1 - T_cold/T_hot |
| `entropy_production` | (q_diss, t_core) → f32 | S_gen = Q/T por nodo |
| `exergy_balance` | (j_in, η, e_a) → f32 | Trabajo útil disponible |
| `heat_capacity` | (qe, c_v) → f32 | Masa térmica efectiva |
| `vascular_transport_cost` | (μ, L, r) → f32 | Hagen-Poiseuille: μL³/r⁴ |
| `shape_cost` | (ρ, v, C_D, A, C_vasc) → f32 | Arrastre + estructura interna |
| `inferred_drag_coefficient` | (L, D_max) → f32 | Myring body: C_D(fineness) |
| `inferred_albedo` | (Q_met, I, A_proj, ε, T_core, T_env, A_surf, h) → f32 | Despeja α del balance radiativo |
| `organ_transform` | (mass_in, exergy_in, η, E_a) → OrganOutput | Writer monad: output útil + (W, Q) |
| `evaluate_metabolic_chain` | (graph, M_init, E_init) → ChainOutput | HOF: compone cadena, acumula Σ Q y Σ W |
| `inferred_surface_rugosity` | (Q, V, T_core, T_env, h) → f32 | Ley cuadrático-cúbica: Q/V alto → más superficie |
| `metabolic_graph_from_manifest` | (manifest, t_core, t_env) → MetabolicGraph | Construye DAG desde OrganManifest |

### Sistemas ECS nuevos

| Sistema | Phase | Reads | Writes |
|---------|-------|-------|--------|
| `metabolic_graph_step_system` | MetabolicLayer | MetabolicGraph, BaseEnergy, AmbientPressure | Mut MetabolicGraph (flujos, Q_diss, S_gen) |
| `entropy_constraint_system` | MetabolicLayer | MetabolicGraph, AmbientPressure | Mut MetabolicGraph (clamp η) |
| `shape_optimization_system` | MorphologicalLayer | MetabolicGraph, FlowVector, AmbientPressure, SpatialVolume | Mut GeometryInfluence |
| `albedo_inference_system` | MorphologicalLayer | MetabolicGraph, IrradianceReceiver, AmbientPressure, SpatialVolume | Insert/Mut InferredAlbedo |
| `entropy_ledger_system` | MetabolicLayer | MetabolicGraph, BaseEnergy | Insert/Mut EntropyLedger (recompute cada tick) |
| `surface_rugosity_system` | MorphologicalLayer | EntropyLedger, SpatialVolume, AmbientPressure | Mut GeometryInfluence (rugosity param) |

### Eventos y resources

- **Lee:** `Res<Time<Fixed>>`, `Res<EnergyFieldGrid>` (para gradientes locales).
- **No introduce eventos nuevos.** Los sistemas son pull-based (query cada tick).
- **No modifica recursos existentes.** Solo lee para contexto ambiental.

## 3) Invariantes y precondiciones

1. **Conservación por nodo:** `Σ J_in = Σ J_out + P_work + Q_diss` — validado en `metabolic_graph_step_system`. Violación → warn + redistribución proporcional.
2. **Carnot siempre:** `node.efficiency ≤ carnot_efficiency(T_core, T_env)` — clamped automáticamente. Si T_env → T_core, η → 0 (motor se apaga).
3. **DAG acíclico:** Validado en construcción. No hay ciclos. Si un edge apunta a ancestro, panic en debug, skip en release.
4. **Mínimo funcional:** Todo MetabolicGraph tiene al menos 1 nodo Captador (Root o Leaf o Sensory) y 1 nodo Disipador (implícito en superficie).
5. **Sin NaN:** Todas las ecuaciones con divisiones guardan `max(denom, EPSILON)`.
6. **Albedo acotado:** `α ∈ [0.05, 0.95]` — clamp duro para evitar negro total o blanco total.
7. **Shape optimizer convergente:** Max 3 iteraciones por frame. Damping factor 0.3. Converge en ~10 frames desde estado inicial.
8. **Backward compatible:** Entidades sin `MetabolicGraph` no se ven afectadas. Los 6 sistemas nuevos filtran por `With<MetabolicGraph>`.
9. **Writer Monad conservación:** `organ_transform()` garantiza `M_in = M_out + W` y `E_in = E_out + Q + E_a`. Violación → panic en debug.
10. **EntropyLedger es derivado:** Se recomputa cada tick desde `evaluate_metabolic_chain()`. Nunca se lee de frame anterior. No es estado persistente.
11. **Rugosity acotada:** `rugosity ∈ [1.0, 4.0]` — 1.0 = esfera lisa, 4.0 = superficie con aletas/pliegues máximos.
12. **Isomorfismo de escala:** `organ_transform()` es la misma función a toda escala del DAG. Los parámetros (η, E_a) cambian, la ecuación no.

## 4) Comportamiento runtime

### Posición en el pipeline

```
FixedUpdate
│
├─ SimulationClockSet
├─ Phase::Input
├─ Phase::ThermodynamicLayer        ← AmbientPressure, thermal_transfer (existente)
├─ Phase::AtomicLayer
├─ Phase::ChemicalLayer
├─ Phase::MetabolicLayer            ← metabolic_graph_step_system
│                                    ← entropy_constraint_system (.after(step))
│                                    ← entropy_ledger_system (.after(constraint))
├─ Phase::MorphologicalLayer        ← shape_optimization_system
                                     ← surface_rugosity_system (.after(shape))
                                     ← albedo_inference_system (.after(rugosity))
                                     ← shape_color_inference_system (existente, lee InferredAlbedo)
```

### Orden relativo

- `metabolic_graph_step_system` corre **después** de `nutrient_uptake_system` y `photosynthesis_system` (necesita J_in actualizado).
- `entropy_constraint_system` corre **después** de `metabolic_graph_step_system` (necesita Q_diss actualizado para recalcular η_max).
- `shape_optimization_system` corre **después** de step (necesita MetabolicGraph actualizado).
- `albedo_inference_system` corre **después** de shape (necesita área de superficie actualizada).
- Pipeline visual existente (`shape_color_inference_system`) corre **después** de albedo (lee `InferredAlbedo` si presente).

### Determinismo

Sin RNG en ninguna ecuación nueva. Mismos inputs → mismo fenotipo. El shape optimizer usa gradient descent con step size fijo (no adaptativo).

### Side-effects

- Modifica `MetabolicGraph` in-place (flujos, derivados).
- Inserta/modifica `EntropyLedger` (recomputado cada tick, no persistente).
- Inserta/modifica `InferredAlbedo` (marker component).
- Modifica `GeometryInfluence` (fineness ratio + rugosity).
- **No** spawna ni despawna entidades.
- **No** emite eventos.

## 5) Implementación y trade-offs

### Estrategia técnica

- **Aproximaciones analíticas, no numéricas.** El shape optimizer no ejecuta CFD; usa la fórmula de Myring body para C_D como función del fineness ratio. Esto es O(1) por entidad, no O(N³).
- **DAG estático, flujos dinámicos.** La topología del grafo se fija en spawn (o en cambio de lifecycle stage); solo los flujos se actualizan por tick. Rebuild del DAG solo cuando `LifecycleStageCache.stage` cambia.
- **Bridge cache para albedo y shape.** `BridgeAlbedo` y `BridgeShape` evitan recómputo cuando inputs no cambiaron significativamente. Cuantización [0,1] con 64 bins.
- **Writer Monad sin allocation.** `OrganOutput` es Copy (16 bytes, stack). `evaluate_metabolic_chain()` itera el DAG en orden topológico, acumula Q y W en registros locales, retorna `ChainOutput` por valor. Sin heap, sin Vec, sin Box.
- **Memoización Matrioska multi-escala.** A nivel Near: evaluación completa por tick. A nivel Mid: `BridgeMetabolicStep` cachea resultado de `evaluate_metabolic_chain()`. A nivel Far: se usa `EntropyLedger` del último tick activo (congelado). Mismo patrón que LOD existente para propagation.

### Costo vs valor

| Métrica | Estimación |
|---------|------------|
| Memoria por MetabolicGraph | ~400 bytes (12 nodos × 20B + 16 edges × 12B + overhead) |
| Memoria por EntropyLedger | 16 bytes (4 × f32) |
| CPU por tick (step + constraint + ledger) | ~3μs por entidad (step O(N_edges) + chain eval O(N_nodes)) |
| CPU shape optimizer (3 iter) | ~5μs por entidad (solo en Near LOD) |
| CPU surface rugosity | ~0.5μs por entidad (aritmética trivial) |
| CPU albedo inference | ~1μs por entidad |
| Entidades esperadas con DAG | 50-200 (criaturas vivas; no proyectiles/terreno) |
| Budget total | ~0.7ms para 200 entidades en Near → dentro del frame budget |

### Límites conocidos

- **No simula turbulencia.** El C_D es para flujo laminar. A velocidades muy altas el modelo subestima arrastre.
- **No modela crecimiento óseo.** La forma cambia por fineness ratio pero no por stress mecánico (no hay FEM).
- **Albedo es escalar, no espectral.** Un albedo por entidad, no por longitud de onda. Suficiente para gameplay pero no físicamente exacto.
- **Max 12 nodos.** DAGs complejos (vertebrado con 50 órganos) necesitarían agrupación.

## 6) Fallas y observabilidad

### Modos de falla esperados

| Falla | Síntoma | Detección | Respuesta |
|-------|---------|-----------|-----------|
| T_core ≈ T_env | η → 0, motor se apaga | `total_entropy_rate < ε` | Entidad entra en "dormancia térmica" (LifecycleStage::Dormant) |
| J_in = 0 (sin alimento) | Nodos downstream se quedan sin flujo | `BaseEnergy.qe` decrece | Mecanismo de inanición existente (`metabolic_stress`) |
| Shape optimizer no converge | Fineness ratio oscila | `|Δ_fineness| > threshold` después de 3 iter | Mantener valor anterior (histéresis) |
| Albedo = NaN | Color roto | Guard en ecuación (`isnan` check) | Fallback α = 0.5 |
| Chain conservación violated | M_in ≠ M_out + W | `debug_assert!` en `organ_transform()` | Panic en debug; silently redistribute en release |
| EntropyLedger stale (no MetabolicGraph) | Ledger sin DAG upstream | System guards `With<MetabolicGraph>` | Entidades sin DAG nunca obtienen ledger |
| Rugosity genera tris excesivos | Frame drop por mesh complejo | Rugosity clamped [1.0, 4.0]; aletas → LOD primitivas | Far = sin aletas, Mid = sprite, Near = mesh |

### Señales / telemetría

- `MetabolicGraph.total_entropy_rate` → metric en DebugPlugin (promedio/max por frame).
- `InferredAlbedo.albedo` → visualizable como heatmap en debug overlay.
- Shape convergence counter → logged a `info!` si > 3 iter.
- `EntropyLedger.exergy_efficiency` → metric en DebugPlugin (eficiencia metabólica global).
- `EntropyLedger.total_waste_generated` → alarm si acumula sin excreción (sistema excretor no drena W).
- Surface rugosity → debug gizmo (wireframe de superficie efectiva vs esfera ideal).

## 7) Checklist de atomicidad

- **¿Una responsabilidad principal?** Sí — inferir fenotipo (forma + color) desde restricciones termodinámicas.
- **¿Acopla más de un dominio?** Lee datos de múltiples capas (L0, L1, L3, L5, L6) pero solo **escribe** en su propio dominio (MetabolicGraph, InferredAlbedo, GeometryInfluence). Coupling de lectura es aceptable; coupling de escritura está contenido.
- **¿Debería dividirse?** Tres dominios conceptuales:
  1. **Evaluación del DAG** (step, constraint, ledger) → `simulation/morphogenesis_metabolic.rs`
  2. **Inferencia de forma** (shape optimizer, rugosity) → `simulation/morphogenesis_shape.rs`
  3. **Inferencia de color** (albedo) → `simulation/morphogenesis_color.rs`

  Se implementan como sistemas separados. Si el módulo unificado `morphogenesis.rs` supera ~500 LOC, dividir en los 3 archivos. El Writer Monad (organ_transform, evaluate_metabolic_chain) vive en `equations.rs` como función pura — no en el módulo de sistemas.

## 8) Referencias cruzadas

- `docs/design/MORPHOGENESIS.md` — Especificación completa con auditoría, plan de sprints, análisis de riesgos
- `docs/design/BLUEPRINT.md` §3-§12 — Capas 0-9 originales
- `docs/design/THERMODYNAMIC_LADDER.md` — Escalera TL1-TL6
- `docs/arquitectura/blueprint_living_organ_inference.md` — OrganManifest, 12 roles, LifecycleStageCache
- `docs/arquitectura/blueprint_geometry_flow.md` — GF1, GeometryInfluence, spine/mesh
- `docs/arquitectura/blueprint_energy_field_inference.md` — Campo → muestra visual
- `docs/arquitectura/blueprint_thermodynamic_ladder.md` — BridgeCache, LOD
- `docs/arquitectura/blueprint_ecosystem_autopoiesis.md` — Ciclo de vida autopoiético
- `src/blueprint/equations.rs` — Todas las ecuaciones puras (destino de las nuevas)
- `src/layers/organ.rs` — OrganRole, OrganManifest, LifecycleStage
- `src/geometry_flow/` — GF1 motor stateless
- `src/bridge/cache.rs` — BridgeCache<B>: patrón de memoización que valida Matrioska lazy evaluation
- `src/worldgen/lod.rs` — LOD Near/Mid/Far: coarse-graining existente (Renormalization Group)

### Fundamentos teóricos

- **Landauer (1961):** Límite inferior de disipación por transformación: Q ≥ kT ln 2. Justifica que organ_transform() siempre produzca Q > 0.
- **Renormalization Group (Wilson/Kadanoff):** Invarianza de ecuaciones al cambiar escala. Justifica que organ_transform() sea isomorfa a toda escala del DAG.
- **Ley Constructal (Bejan):** La forma emerge para facilitar corrientes de flujo. La composición HOF dicta flujos internos → la forma es la solución.
- **Prigogine (1977):** Estructuras Disipativas. El organismo minimiza S_gen → el fenotipo es la configuración de mínima entropía.
- **WBE (1997):** Redes fractales optimizan transporte. vascular_transport_cost() implementa Hagen-Poiseuille → branching óptimo.
