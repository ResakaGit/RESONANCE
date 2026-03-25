# Track — Morfogénesis Inferida y Composición Funcional Matrioska

**Blueprint:** `docs/design/MORPHOGENESIS.md`
**Arquitectura:** `docs/arquitectura/blueprint_morphogenesis_inference.md`
**Alineación:** filosofía "todo es energía" + "math in equations.rs" + "one system, one transformation" + stateless-first del proyecto.
**Metodología:** TDD, funciones puras en `blueprint/equations.rs`, sistemas de una transformación, Writer Monad termodinámico para acumulación de desechos.

---

## Objetivo del track

Implementar el Motor de Inferencia Morfológica donde la forma, color y estructura de entidades vivas se **infieren** como soluciones óptimas a restricciones termodinámicas. Introduce el `MetabolicGraph` (DAG de exergía), el patrón Writer Monad (organ_transform → OrganOutput con desechos), el `EntropyLedger` (libro contable), y los solvers que derivan albedo, forma hidrodinámica y rugosidad de superficie desde la termodinámica del organismo.

**Resultado jugable:** un organismo en agua densa que converge a forma fusiforme porque es la única solución que minimiza C_shape. Una criatura en desierto que se vuelve clara porque su balance radiativo lo exige. Un dragón con crestas dorsales porque su Q/V ratio fuerza más superficie.

---

## Principio fundamental

> La morfología no se diseña. Se resuelve. Es el volcado geométrico del perfil termodinámico del organismo interactuando con su ecosistema.

La entidad tiene sus 14 capas + auxiliares (OrganManifest, GrowthBudget, NutrientProfile, CapabilitySet, InferenceProfile). De esas capas + el DAG metabólico, funciones puras computan:

```
Entity state + 14 layers
  │
  ▼ pure fn (MG-2)
MetabolicGraph (DAG: ExergyNodes + ExergyEdges)
  │
  ▼ pure fn (MG-6)
evaluate_metabolic_chain() → ChainOutput
  │ Σ Q_diss, Σ W_waste acumulados (Writer Monad)
  ▼
EntropyLedger (total_heat, total_waste, entropy_rate, exergy_efficiency)
  │
  ├──▶ inferred_albedo()         → α (color)
  ├──▶ inferred_drag_coefficient → C_D (forma)
  └──▶ inferred_surface_rugosity → rugosity (aletas/pliegues)
        │
        ▼
  GeometryInfluence (fineness + rugosity) → GF1 mesh → Fenotipo visual
```

---

## Grafo de dependencias

```
MG-1 (Ecuaciones Termodinámicas)  ── Onda 0 — bloqueante para todos
     │
     ├──► MG-2 (MetabolicGraph Types)        ── Onda A
     │         │
     │         ▼
     │    MG-3 (Paso Temporal DAG)            ── Onda B (requiere MG-2)
     │         │
     │         ├──► MG-4 (Shape Optimization) ── Onda C
     │         │
     │         ├──► MG-6 (Writer Monad +      ── Onda C (paralelo con MG-4)
     │         │         EntropyLedger)
     │         │         │
     │         │         ▼
     │         │    MG-7 (Surface Rugosity)   ── Onda D (requiere MG-6)
     │         │
     │         └──► MG-5 (Albedo Inference)   ── Onda C (paralelo con MG-4)
     │
     └──► MG-8 (Integración + Demo)           ── Onda E (requiere todo)
```

## Ondas de ejecución

| Onda | Sprints | Qué habilita |
|------|---------|-------------|
| **0** | MG-1 | Funciones puras termodinámicas, constantes |
| **A** | MG-2 | Tipos del DAG, builder, validación |
| **B** | MG-3 | Paso temporal: flujos entre nodos cada tick |
| **C** | MG-4, MG-5, MG-6 (paralelo) | Shape optimizer, albedo solver, Writer Monad |
| **D** | MG-7 | Rugosidad de superficie desde Q/V ratio |
| **E** | MG-8 | Demo visual, EntityBuilder, benchmark |

## Índice de sprints

| Sprint | Archivo | Módulo principal | Onda | Dependencias | Estado |
|--------|---------|-----------------|------|--------------|--------|
| [MG-1](SPRINT_MG1_THERMODYNAMIC_EQUATIONS.md) | Ecuaciones Termodinámicas | `src/blueprint/equations.rs` | 0 | — | ⏳ |
| [MG-2](SPRINT_MG2_METABOLIC_GRAPH_TYPES.md) | MetabolicGraph Types | `src/layers/metabolic_graph.rs` | A | MG-1 | ⏳ |
| [MG-3](SPRINT_MG3_DAG_TEMPORAL_STEP.md) | Paso Temporal DAG | `src/simulation/morphogenesis.rs` | B | MG-2 | ⏳ |
| [MG-4](SPRINT_MG4_SHAPE_OPTIMIZATION.md) | Shape Optimization | `src/simulation/morphogenesis.rs` | C | MG-3 | ⏳ |
| [MG-5](SPRINT_MG5_ALBEDO_INFERENCE.md) | Albedo Inference | `src/simulation/morphogenesis.rs` | C | MG-3; **orden runtime:** después de MG-7 (`surface_rugosity`) — ver nota staging abajo | ⏳ |
| [MG-6](SPRINT_MG6_WRITER_MONAD_LEDGER.md) | Writer Monad + EntropyLedger | `src/blueprint/equations.rs`, `src/layers/metabolic_graph.rs` | C | MG-2 | ⏳ |
| [MG-7](SPRINT_MG7_SURFACE_RUGOSITY.md) | Surface Rugosity | `src/simulation/morphogenesis.rs`, `src/geometry_flow/` | D | MG-6 | ⏳ |
| [MG-8](SPRINT_MG8_INTEGRATION_DEMO.md) | Integración + Demo | `src/entities/`, `assets/maps/` | E | Todos | ⏳ |

---

## Paralelismo seguro

| | MG-1 | MG-2 | MG-3 | MG-4 | MG-5 | MG-6 | MG-7 | MG-8 |
|---|---|---|---|---|---|---|---|---|
| **MG-4** | | | | — | ✅ | ✅ | | |
| **MG-5** | | | | ✅ | — | ✅ | | |
| **MG-6** | | | | ✅ | ✅ | — | | |

MG-4, MG-5 y MG-6 son paralelos (Onda C): no comparten archivos de escritura.

**Staging sin MG-7:** mientras no exista `surface_rugosity_system`, registrar `albedo_inference_system` con `.after(shape_optimization_system)`; al mergear MG-7, re-enlazar a `.after(surface_rugosity_system)` como en el contrato de pipeline. *Paralelo en planificación ≠ misma fase:* el ledger (MG-6) sigue en `MetabolicLayer`; albedo/rugosity en `MorphologicalLayer`.

---

## Invariantes del track

1. **Math in equations.rs.** Toda ecuación nueva (Carnot, entropy, albedo, rugosity, organ_transform) es función pura en `equations.rs`. Sistemas solo orquestan queries y llaman puras.
2. **Max 4 campos por componente.** `EntropyLedger` tiene 4 campos. `MetabolicGraph` usa ArrayVec internos, no campos planos.
3. **SparseSet para todo nuevo.** `MetabolicGraph`, `EntropyLedger`, `InferredAlbedo` — solo entidades "vivas complejas".
4. **Guard change detection.** Todo sistema verifica `if old != new` antes de mutar.
5. **Conservación estricta.** `organ_transform()` garantiza M_in = M_out + W, E_in = E_out + Q + E_a.
6. **Sin RNG.** Mismos inputs → mismo fenotipo. Sin stochasticidad en ninguna ecuación.
7. **Backward compatible.** Entidades sin MetabolicGraph → pipeline actual exacto. Cero regresión.
8. **Phase assignment.** Todo sistema nuevo → `.in_set(Phase::MetabolicLayer)` o `.in_set(Phase::MorphologicalLayer)`.
9. **Chain events.** Ordering explícito con `.after()` entre sistemas nuevos.
10. **BridgeCache donde aplique.** shape_cost y albedo son costosos → BridgeShape, BridgeAlbedo.

## Contrato de pipeline MG

```
FixedUpdate:
  SimulationClockSet
  → Phase::Input
  → Phase::ThermodynamicLayer     ← existente (thermal, dissipation)
  → Phase::AtomicLayer
  → Phase::ChemicalLayer
  → Phase::MetabolicLayer         ← metabolic_graph_step_system
                                   ← entropy_constraint_system (.after step)
                                   ← entropy_ledger_system (.after constraint)
  → Phase::MorphologicalLayer     ← shape_optimization_system
                                   ← surface_rugosity_system (.after shape)
                                   ← albedo_inference_system (.after rugosity)
                                   ← shape_color_inference_system (existente, lee InferredAlbedo)
```

---

## Ejemplo motivador: Criatura acuática

```
Criatura entity:
  L0: BaseEnergy = 500 qe
  L1: SpatialVolume = { radius: 2.0 }
  L3: FlowVector = { velocity: Vec3(4.0, 0, 0), dissipation: 0.05 }
  L4: MatterCoherence = Solid, bond 1200
  L6: AmbientPressure = { viscosity: 1000.0 (agua), delta_qe: 0.0 }
  Aux: OrganManifest = [Stem, Core, Fin×2, Sensory×2]

  ▼ metabolic_graph_from_manifest(manifest, T_core=400, T_env=280)
  → MetabolicGraph: 6 nodos, 7 aristas
    Captador(Root) → Procesador(Core) → Distribuidor(Stem) → Actuador(Fin)
                                       └→ Sensor(Sensory)

  ▼ evaluate_metabolic_chain(graph, M=100, E=500)
  → ChainOutput: final_exergy=320, total_heat=150, total_waste=30

  ▼ EntropyLedger: Q=150, W=30, S_gen=0.375, η_total=0.64

  ▼ inferred_drag_coefficient(L=4.0, D=2.0) = 0.04 (fusiforme)
  ▼ shape_cost(ρ=1000, v=4, C_D=0.04, A=3.14, C_vasc=12)
  → C_shape = 33.1 (óptimo: alto fineness ratio)

  ▼ inferred_albedo(Q=150, I=50, ...) = 0.3 (oscuro — poco sol en agua)
  ▼ inferred_surface_rugosity(Q=150, V=33.5, ...) = 1.2 (casi lisa)

  → Fenotipo: cuerpo fusiforme, oscuro, aletas laterales, liso.
     Sin diseño manual — la termodinámica lo dicta.
```

---

## Referencias cruzadas

- `docs/design/MORPHOGENESIS.md` — Especificación completa con auditoría y teoría
- `docs/arquitectura/blueprint_morphogenesis_inference.md` — Contrato de arquitectura (8 secciones)
- `docs/sprints/THERMODYNAMIC_LADDER/README.md` — Track prerequisito (TL1–TL6)
- `docs/sprints/LIVING_ORGAN_INFERENCE/README.md` — OrganManifest, 12 roles, LifecycleStageCache
- `docs/sprints/GEOMETRY_FLOW/README.md` — GF1 motor stateless
- `docs/sprints/BRIDGE_OPTIMIZER/README.md` — Patrón BridgeCache<B>
- `docs/sprints/ENERGY_PARTS_INFERENCE/README.md` — Pipeline campo → vértice
