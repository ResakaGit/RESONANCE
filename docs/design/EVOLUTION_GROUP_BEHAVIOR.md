# Blueprint — Evolution & Group Behavior

**Versión:** 1.0
**Depende de:** AXIOMATIC_CLOSURE, EMERGENCE_TIERS, MORPHOGENESIS
**Estado:** Sprint creado — `docs/sprints/EMERGENCE_EVOLUTION/README.md`

---

## 1. Problema

La simulación produce vida (abiogenesis axiomática) y forma (constructal morphogenesis), pero:
- Fauna no se reproduce → no hay evolución animal
- `mobility_bias` no muta → la forma corporal no evoluciona
- 6 sistemas de emergence tienen ecuaciones + componentes pero ningún system los conecta

Sin evolución no hay adaptación. Sin grupo no hay cultura. Sin cultura no hay civilización.

## 2. Derivación axiomática

Todo lo que sigue se deriva de los 8 axiomas sin excepciones arbitrarias.

### Reproducción (Axiomas 1, 4, 5)

```
Parent tiene qe. Offspring recibe fracción.
Axioma 4: transferencia no es 100% eficiente → loss.
Axioma 5: qe(parent_after) + qe(offspring) ≤ qe(parent_before).
No se crea energía en reproducción.
```

Condición de reproducción: `qe > threshold AND satiation > min AND age > min_age`. Todos derivados de balance energético — un organismo que no puede alimentarse no puede reproducirse.

### Herencia + Mutación (Axioma 8)

```
Offspring hereda OscillatorySignature (frecuencia = identidad).
InferenceProfile se copia con perturbación (mutate_bias).
Perturbación es determinista (hash del entity ID, no RNG).
```

La frecuencia del offspring ≈ frecuencia del parent. Si muta demasiado → interferencia destructiva con la especie → selección natural actúa (Axioma 3).

### Selección natural (Axiomas 3, 4)

```
Competencia por energía (Axioma 3) → los menos eficientes pierden.
Disipación (Axioma 4) → todos pierden energía continuamente.
Los que no obtienen suficiente → mueren (qe < QE_MIN_EXISTENCE).
Los que sobreviven → se reproducen → pasan traits.
```

No hay fitness function. No hay evaluación top-down. La selección ES la consecuencia de los axiomas operando sobre entidades con varianza en traits.

### Group behavior (Axioma 6)

```
Emergencia a escala: comportamiento de N = interacciones de N-1.
No hay "grupos" como concepto. Hay entidades que:
  - interfieren constructivamente (Axioma 8) → se sincronizan (entrainment)
  - compiten y cooperan (Axioma 3) → forman alianzas si beneficio > solo
  - modelan a otros (Theory of Mind) → predicen comportamiento vecino
  - comparten información (Cultural transmission) → memes se propagan
```

Un "grupo" emerge cuando varias entidades sincronizan frecuencia (entrainment), forman alianza estable (cooperation), y comparten memes (culture). No se programa.

## 3. Cadena de emergencia

```
Axiomas 1-8
  ├─ Abiogenesis axiomática        ← implementado
  ├─ Morphogenesis constructal     ← implementado
  ├─ Trophic succession            ← implementado
  ├─ Reproducción + herencia       ← EV-1, EV-2
  ├─ Selección natural             ← consecuencia automática (no es un system)
  ├─ Theory of Mind (ET-2)         ← EV-3
  ├─ Symbiosis (ET-5)              ← EV-4
  ├─ Epigenetics (ET-6)            ← EV-5
  ├─ Niche displacement (ET-9)     ← EV-6
  ├─ Coalitions (ET-8)             ← parcial, ya funcional
  ├─ Entrainment (AC-2)            ← funcional
  ├─ Cooperation (AC-5)            ← funcional
  └─ Culture (ET-3)                ← funcional
```

## 4. Diseño de cada system

### Principios compartidos

- **Stateless:** system lee components, computa vía ecuación pura, escribe resultado. No `static`, no cache manual (BridgeCache si necesario).
- **DoD:** query ≤ 5 component types. Math en `blueprint/equations/`. Constantes en `blueprint/constants/`.
- **HOF:** `query.iter().filter(...).map(...)` sobre `for entity in query { if ... }`.
- **Change detection:** `if old != new { set(new) }`.
- **Phase assignment:** Cada system tiene Phase explícita.
- **No duplicación:** Reusar `SpatialIndex::query_radius`, `mutate_bias`, `EntityBuilder`.

### EV-1: Fauna reproduction

```
Phase: MorphologicalLayer (after abiogenesis)
Reads: BaseEnergy, TrophicState, SenescenceProfile, InferenceProfile, CapabilitySet, Transform
Writes: Commands (spawn offspring), BaseEnergy (drain parent)
Guard: caps.has(MOVE) && caps.has(REPRODUCE)
Equation: can_reproduce_fauna(qe, satiation, age) → bool
Spawn: EntityBuilder with inherited+mutated InferenceProfile + MOVE + behavior stack
```

### EV-2: mobility_bias mutation

```
Change: 1 línea en reproduction_spawn_system
No new system. No new equations.
mutate_bias(profile.mobility_bias, d_mobility, MUTATION_MAX_DRIFT)
```

### EV-3: Theory of Mind (ET-2)

```
Phase: Input (before behavior decisions)
Reads: OtherModelSet, Transform, BaseEnergy (self), Transform+OscillatorySignature+BaseEnergy (targets)
Writes: OtherModelSet (update predictions), BaseEnergy (maintenance cost)
SpatialIndex: query_radius(position, model_range)
Max neighbors: OtherModelSet.MODEL_SLOTS (4)
Eviction: is_model_worth_maintaining(benefit, cost) == false → slot freed
```

### EV-4: Symbiosis effects (ET-5)

```
Phase: MetabolicLayer
Reads: SymbiosisLink, BaseEnergy (self + partner)
Writes: BaseEnergy (drain/benefit both), Commands (remove unstable links)
Guard: partner entity exists (let-else)
Equation: mutualism_benefit, parasitism_drain, is_symbiosis_stable
No SpatialIndex needed — link is direct entity reference.
```

### EV-5: Epigenetic adaptation (ET-6)

```
Phase: MorphologicalLayer (before constructal)
Reads: EpigeneticState, InferenceProfile, AmbientPressure
Writes: EpigeneticState (expression_mask), BaseEnergy (silencing_cost)
Equation: should_express_gene(env_signal, threshold, mask), silencing_cost
No SpatialIndex.
```

### EV-6: Niche displacement (ET-9)

```
Phase: MorphologicalLayer (after constructal)
Reads: NicheProfile, Transform, OscillatorySignature
Writes: NicheProfile (center, width)
SpatialIndex: query_radius for competitors
Equation: niche_overlap, competitive_pressure, character_displacement
Threshold: only displace if overlap > NICHE_OVERLAP_DISPLACEMENT_THRESHOLD
```

## 5. Invariantes

1. `Σ qe` no aumenta en ninguna operación de este sprint (Axiomas 4, 5).
2. Offspring qe + parent qe_after ≤ parent qe_before (reproducción conservativa).
3. Mutación es determinista dado entity ID (no RNG, no `rand` crate).
4. Selección natural es consecuencia, no causa — ningún system evalúa fitness.
5. Group formation es emergente — ningún system crea "grupos".
6. Theory of Mind costs qe — modelar al otro NO es gratis (Axioma 4).
7. Epigenetic changes are reversible — no heritable (herencia solo vía InferenceProfile).
8. Niche displacement conserves total niche breadth (character displacement, not expansion).

## 6. Tests (TDD)

Cada system: ≥ 3 unit tests + 1 integration test.

### Unit (pure equations, no Bevy)
- `can_reproduce_fauna_*` (threshold checks)
- `mutate_bias_*` (already exists, verify mobility)
- `update_prediction_*` (already exists, regression)
- `mutualism_benefit_*` (already exists, regression)
- `should_express_gene_*` (already exists, regression)
- `niche_overlap_*` (already exists, regression)

### Integration (MinimalPlugins app)
- `fauna_reproduces_when_conditions_met` (spawn + verify offspring)
- `offspring_profile_differs_from_parent` (mutation)
- `theory_of_mind_model_converges` (multi-tick accuracy increase)
- `symbiosis_mutual_benefit_positive_sum` (energy conservation)
- `epigenetic_mask_responds_to_environment` (pressure change → mask change)
- `overlapping_niches_diverge_over_time` (multi-tick displacement)

### Conservation (property-based)
- `proptest: reproduction_conserves_energy` (parent + offspring ≤ parent_before)
- `proptest: symbiosis_conserves_energy` (sum_after ≤ sum_before)

## 7. Esfuerzo estimado

| Tarea | Líneas nuevas | Ecuaciones nuevas | Tests nuevos |
|-------|--------------|-------------------|-------------|
| EV-1 (fauna repro) | ~80 | 1 (`can_reproduce_fauna`) | 4 |
| EV-2 (mobility mutation) | ~5 | 0 | 2 |
| EV-3 (theory of mind) | ~60 | 0 (ya existen) | 4 |
| EV-4 (symbiosis effects) | ~40 | 0 (ya existen) | 3 |
| EV-5 (epigenetics) | ~50 | 0 (ya existen) | 3 |
| EV-6 (niche adaptation) | ~50 | 0 (ya existen) | 3 |
| **Total** | **~285** | **1** | **19** |

## 8. Qué emerge al completar

Con las 6 tareas implementadas:

- **Evolución real:** generaciones de fauna con varianza en traits → selección natural → adaptación al ambiente
- **Forma evoluciona:** mobility_bias muta → constructal produce proporciones distintas por linaje
- **Theory of Mind:** entidades predicen comportamiento de vecinos → estrategia emerge
- **Symbiosis:** pares obligados intercambian energía → co-evolución
- **Plasticidad:** misma genética (InferenceProfile) → diferente fenotipo según ambiente (EpigeneticState)
- **Diversificación:** niches se separan bajo presión competitiva → especiación ecológica

Combinado con lo que ya funciona (entrainment, cooperation, culture, coalitions):
- Manadas sincronizan frecuencia (entrainment)
- Cooperan cuando beneficio > solo (cooperation)
- Comparten memes (culture)
- Forman alianzas estables (coalitions)
- **→ Grupos estables con identidad cultural, coordinación temporal, y evolución adaptativa**

## 9. Referencias

- `docs/design/EMERGENCE_TIERS.md` — Diseño original T1-T4
- `docs/design/AXIOMATIC_CLOSURE.md` — 8 axiomas + 5 composiciones
- `docs/arquitectura/blueprint_emergence_tiers.md` — Contratos de implementación
- `src/simulation/reproduction/mod.rs` — Sistema de reproducción existente (flora)
- `src/blueprint/equations/emergence/` — 18 archivos de ecuaciones puras
- `src/layers/{epigenetics,other_model,symbiosis,niche,senescence,self_model,language}.rs` — 7 componentes
