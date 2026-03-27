# Blueprint: Emergence Tiers — Contratos de Módulo

> Axioma universal: `dS/dt = R(S) - D(S) - C(S)`
> Aplica en toda escala: célula → individuo → grupo → civilización.
> R = extracción de recursos, D = disipación, C = costo de coordinación.

---

## Índice de Módulos

| ID  | Módulo                    | Tier | Capa principal           | BridgeKind           | Onda |
|-----|---------------------------|------|--------------------------|----------------------|------|
| ET-1  | Associative Memory      | T1-1 | `layers/memory.rs`       | AssociativeDecayBridge | 0  |
| ET-2  | Theory of Mind          | T1-2 | `layers/theory_of_mind.rs` | OtherModelBridge   | A  |
| ET-3  | Cultural Transmission   | T1-3 | `sim/emergence/culture.rs` | MemeSpreadBridge   | A  |
| ET-4  | Infrastructure          | T1-4 | `sim/emergence/infrastructure.rs` | FieldModBridge | B |
| ET-5  | Obligate Symbiosis      | T2-1 | `layers/symbiosis.rs`    | SymbiosisBridge      | 0  |
| ET-6  | Epigenetic Expression   | T2-2 | `layers/epigenetics.rs`  | EpigeneticBridge     | A  |
| ET-7  | Programmed Senescence   | T2-3 | `layers/senescence.rs`   | SenescenceBridge     | 0  |
| ET-8  | Dynamic Coalitions      | T2-4 | `sim/emergence/coalitions.rs` | CoalitionBridge ⚠ | B |
| ET-9  | Multidimensional Niche  | T2-5 | `layers/niche.rs`        | NicheOverlapBridge   | A  |
| ET-10 | Multiple Timescales    | T3-1 | `layers/timescale.rs`    | TimescaleBridge      | 0  |
| ET-11 | Multi-Scale Info       | T3-2 | `sim/emergence/multiscale.rs` | AggSignalBridge | A  |
| ET-12 | Continental Drift      | T3-3 | `sim/emergence/tectonics.rs` | TectonicBridge   | B  |
| ET-13 | Geological Time LOD    | T3-4 | `sim/emergence/geological_lod.rs` | LODPhysicsBridge | B |
| ET-14 | Institutions           | T4-1 | `sim/emergence/institutions.rs` | InstitutionBridge | C |
| ET-15 | Language               | T4-2 | `layers/language.rs`     | SymbolBridge         | C  |
| ET-16 | Functional Consciousness | T4-3 | `layers/self_model.rs` | SelfModelBridge      | C  |

⚠ = cache CRÍTICO (Large backend)

---

## Análisis de Rendimiento de Caché

### Estrategia por BridgeKind

| BridgeKind           | Backend      | Clave                               | Hit Rate | Justificación                              |
|----------------------|-------------|-------------------------------------|----------|--------------------------------------------|
| AssociativeDecayBridge | Small(32) | `(stimulus_hash, tick/1000)`       | ~70%     | Mismos estímulos se repiten; ventanas de tick |
| OtherModelBridge     | Small(32)   | `(modeler_id, target_id, tick/5)`  | ~80%     | Throttle 5-tick explícito                  |
| MemeSpreadBridge     | Small(64)   | `(behavior_hash, density_band)`    | ~75%     | Misma zona → mismo comportamiento          |
| FieldModBridge       | Small(32)   | `(cell_idx, delta_band)`           | ~85%     | decay es lento, misma celda por muchos ticks |
| SymbiosisBridge      | Small(128)  | `(entity_a_band, entity_b_band)`   | ~65%     | Pares de entidades en misma zona           |
| EpigeneticBridge     | Small(64)   | `(env_band, gene_mask_hash)`       | ~75%     | env_sample_rate=16 ticks de throttle       |
| SenescenceBridge     | Small(32)   | LUT `band_index_of(age, BANDS)`    | ~90%     | LUT discreta + bandas fijas                |
| **CoalitionBridge**  | **Large(512)** | `coalition_id`              | ~85%     | ⚠ O(n²) Nash — REQUIERE Large             |
| NicheOverlapBridge   | Small(64)   | `hash(niche_a_band, niche_b_band)` | ~70%     | Nichos cambian lentamente                  |
| TimescaleBridge      | Small(32)   | `(lod_band, timescale_tier)`       | ~75%     | Pocos LOD levels posibles                  |
| AggSignalBridge      | Small(128)  | `(cell_idx, region_idx)`           | ~90%     | Update 8-tick → misma key por 8 ticks      |
| TectonicBridge       | Small(32)   | `(plate_id, stress_band)`          | ~80%     | Tectónica es lenta (eval cada 500 ticks)   |
| LODPhysicsBridge     | Small(16)   | `(lod_level, tick_compression)`    | ~85%     | Sólo 4 LOD levels                         |
| InstitutionBridge    | Small(64)   | `(rule_hash, compliance_band)`     | ~75%     | Compliance cambia lentamente               |
| SymbolBridge         | Small(64)   | `(vocab_band, semantic_hash)`      | ~70%     | vocab crece lentamente                     |
| SelfModelBridge      | Small(32)   | `(accuracy_band, horizon_band)`    | ~70%     | Accuracy EMA suave                         |

### Principio de diseño caché

Todos los BridgeKind del track ET siguen el mismo contrato:
```rust
impl BridgeKind for XBridge {}
// Registrado en EmergenceTier{N}Plugin:
app.insert_resource(BridgeCache::<XBridge>::new(CacheBackend::Small(N)));
```

El sistema genérico `BridgeCache<B>` no requiere modificación — el track ET son 16 marcadores nuevos sin tocar infraestructura existente.

---

## Contratos de Módulo (8 secciones)

### ET-1: Associative Memory

**Propósito:** Persistencia de asociaciones estímulo→outcome con decay temporal.

**Contrato:**
- Input: `OscillatorySignature` (frecuencia), posición relativa, `BaseEnergy` outcome.
- Output: `AssociativeMemory.entries[]` actualizado; `expected_stimulus_value()` para decisiones.
- Invariant: `entry_count ≤ 8`. LRU por `strength` cuando lleno. Sin Vec/heap.

**Fases:** `Phase::Input` (after SensoryLayer), `Phase::MetabolicLayer` (costo de maintenance).

**Runtime:** O(8) por entidad por tick. AssociativeDecayBridge cachea `association_strength` para ventanas de tick.

**Trade-offs:** Array fijo de 8 entradas limita la memoria pero garantiza coherencia de caché L1. Suficiente para comportamiento adaptativo.

**Fallos:** Si `decay_rate=0`, las entradas no expiran → memoria saturada → LRU activa.

**Atomicidad:** `associative_memory_update_system` lee energía, escribe memoria. No modifica energía directamente — el costo se aplica vía `entry_cost_system`.

**Tests:** `association_strength`, `expected_stimulus_value`, LRU replacement, decay a cero.

---

### ET-2: Theory of Mind

**Propósito:** Modelado predictivo de entidades externas para anticipar comportamiento.

**Contrato:**
- Input: `OscillatorySignature` del target, posición relativa, `BaseEnergy` del target.
- Output: `OtherModelSet.models[]` con `predicted_freq` y `accuracy`.
- Invariant: `model_count ≤ 4`. `update_interval=5` ticks throttle. Sin String ni Entity como ID.

**Fases:** `Phase::Input` (after AssociativeMemory, before BehaviorSet::Decide).

**Runtime:** O(4×nearby) por entidad. OtherModelBridge: key `(modeler, target, tick/5)` — mismo resultado en ventana de 5 ticks → 80% hit rate.

**Trade-offs:** 4 modelos activos limita el "circle of attention". Suficiente para MOBA (enemigos directos).

**Fallos:** Target muerto → `model_count` decrece por `is_model_worth_maintaining`.

**Atomicidad:** Lee otros agentes (`Res<Query>` o snapshot), escribe sólo propio `OtherModelSet`.

---

### ET-3: Cultural Transmission

**Propósito:** Imitación como adaptación rápida — comportamientos se propagan sin genética.

**Contrato:**
- Input: `BaseEnergy` de modelos vecinos, `CulturalMemory` de modelos.
- Output: `CulturalMemory.memes[]` con nuevo `MemeEntry`. `MemeAdoptedEvent` emitido.
- Invariant: `meme_count ≤ MAX_MEMES=4`. Sin duplicados por `behavior_hash`.

**Fases:** `Phase::Input` (after TheoryOfMind).

**Runtime:** O(nearby×memes) por entidad. MemeSpreadBridge: key `(behavior_hash, density_band)`.

**Trade-offs:** Sin transmisión genética vertical — sólo horizontal. Divergencia regional emerge de aislamiento geográfico.

**Fallos:** Si `adoption_cost > expected_gain`, el meme no se adopta → convergencia natural.

**Atomicidad:** Lee snapshot de vecinos, escribe sólo propio `CulturalMemory`. `MemeAdoptedEvent` para SF-7 trazabilidad.

---

### ET-4: Infrastructure

**Propósito:** Modificación persistente del campo energético por actividad acumulada.

**Contrato:**
- Input: `InfrastructureInvestEvent { investor, cell_idx, qe_invested }`.
- Output: `InfrastructureGrid.modifications[cell_idx]` aumentado. Bonus de intake a entidades en celda.
- Invariant: `modifications` mismo tamaño que `EnergyFieldGrid` (1024). No modifica EnergyFieldGrid directamente.

**Fases:** `Phase::MetabolicLayer` (update + decay), `Phase::MetabolicLayer` (intake bonus, after update).

**Runtime:** O(1024) para decay global cada tick. O(nearby_cells) para intake bonus. FieldModBridge: key `(cell_idx, delta_band)` → ~85% hit rate.

**Trade-offs:** Decay garantiza entropía — sin mantenimiento, colapsa. El "olvido" es físico, no una regla.

**Fallos:** `MAX_INFRASTRUCTURE_DELTA=100.0` previene overflow. `MIN_ACTIVE_DELTA=0.1` skip de celdas vacías.

---

### ET-5: Obligate Symbiosis

**Propósito:** Dependencias energéticas mutuas entre pares de entidades.

**Contrato:**
- Input: `SymbiosisLink { partner_id, relationship, bonus_factor, drain_rate }`.
- Output: `AlchemicalEngine.intake` boosted (mutualismo) o `BaseEnergy.qe` drenado (parasitismo).
- Invariant: SparseSet — mayoría de entidades sin el componente.

**Fases:** `Phase::ChemicalLayer` (after catalysis, before metabolic).

**Runtime:** O(entities_with_link × partners) por tick. SymbiosisBridge: key `(entity_a_band, entity_b_band)`.

**Trade-offs:** Obligate dependency emerge de `is_obligate_dependency` — sin código extra. Las relaciones son asimétricas por diseño (parásito ≠ host).

---

### ET-6: Epigenetic Expression

**Propósito:** Modulación fenotípica reversible por condiciones energéticas del entorno.

**Contrato:**
- Input: `EnergyFieldGrid.cell_qe_at_world(x, z)`. `InferenceProfile` como genotipo base.
- Output: `EpigeneticState.expression_mask[4]` ajustado. `InferenceProfile` modificado via `apply_expression`.
- Invariant: `expression_mask[i] ∈ [0,1]`. Cambios lentos por `adaptation_speed`.

**Fases:** `Phase::MorphologicalLayer` (before visual_contract_sync).

**Runtime:** O(entities × 4_genes) cada `env_sample_rate=16` ticks → 16× throttle efectivo.

**Trade-offs:** Plasticidad reversible — rápida vs. cambio genético. No heritable por sí sola (ET-10 Baldwin lo hace heritable).

---

### ET-7: Programmed Senescence

**Propósito:** Mortalidad intrínseca dependiente de la edad — recambio generacional.

**Contrato:**
- Input: `SenescenceProfile { tick_birth, senescence_coeff, max_viable_age, strategy: u8 }`.
- Output: `BaseEnergy.qe` reducido por `age_drain` cada tick.
- Invariant: `strategy: u8` (no enum en componente). LUT de bandas de edad.

**Fases:** `Phase::MetabolicLayer` (integrado en metabolic_stress_system).

**Runtime:** O(entities) por tick. SenescenceBridge usa LUT — misma banda → mismo resultado → ~90% hit rate.

**Trade-offs:** `strategy: u8` cumple Hard Block 7. `ReproductionStrategy` enum vive en blueprint/equations únicamente.

---

### ET-8: Dynamic Coalitions ⚠

**Propósito:** Alianzas N>2 con estabilidad Nash colectiva.

**Contrato:**
- Input: `CoalitionMember { coalition_id, role, join_tick, coordination_cost }`.
- Output: Intake bonus para miembros. `CoalitionChangedEvent` en disolución.
- Invariant: `MAX_COALITION_MEMBERS=8`. Evaluación cada `EVAL_INTERVAL=10` ticks. CoalitionBridge **Large(512)** mandatorio.

**Fases:** `Phase::MetabolicLayer` (after symbiosis).

**Runtime:** O(n²) Nash por coalición → **CRÍTICO**. Sin caché: inaceptable. Con Large(512) + eval_interval=10 → ~85% hit rate → aceptable.

**Trade-offs:** Large(512) usa FxHashMap → colisiones posibles → LRU limpia entradas viejas. La exactitud matemática no se degrada (peor caso: recalcula).

**Fallos:** Invalidación en `CoalitionChangedEvent` — garantiza coherencia de caché.

---

### ET-9: Multidimensional Niche

**Propósito:** Diferenciación ecológica en espacio de 4 dimensiones (Hutchinson).

**Contrato:**
- Input: `NicheProfile { center[4], width[4], displacement_rate, specialization }`.
- Output: `NicheProfile.center[d]` ajustado por character displacement cuando overlap > threshold.
- Invariant: Snapshot anti-aliasing. Eval cada `NICHE_EVAL_INTERVAL=20` ticks.

**Fases:** `Phase::MorphologicalLayer`.

**Runtime:** O(nearby × 4_dims) cada 20 ticks. NicheOverlapBridge: key `hash(niche_a_band, niche_b_band)` → ~70% hit rate.

**Trade-offs:** Exclusión competitiva emerge sin programarla — acumulación de displacement a lo largo de ticks.

---

### ET-10: Multiple Timescales

**Propósito:** Efecto Baldwin — comportamientos aprendidos se vuelven instintivos gradualmente.

**Contrato:**
- Input: `TimescaleAdapter { genetic_baseline, epigenetic_offset, cultural_offset, learned_offset }`.
- Output: `genetic_baseline` aumenta gradualmente cuando `learned_offset > 0` y `selection_pressure` alta.
- Invariant: 4 campos exactos. Cada campo es escrito por un sistema distinto (bajo acoplamiento).

**Fases:** `Phase::MorphologicalLayer` (Baldwin, last in tier T3); `Phase::Input` (cultural sync).

**Runtime:** Baldwin eval cada `GENETIC_EVAL_INTERVAL=1000` ticks → CPU ≈ 0.

**Trade-offs:** Sin cromosomas, sin crossover. El "gen" es un único `f32`. Suficiente para gameplay emergente.

---

### ET-11: Multi-Scale Information

**Propósito:** Señales agregadas en 3 escalas espaciales — evita recomputo distribuido.

**Contrato:**
- Input: `EnergyFieldGrid` (32×32).
- Output: `MultiscaleSignalGrid { local[1024], regional[64], global: f32 }` actualizado.
- Invariant: `regional[i]` = media de `local[4×4 block]`. `global` = media de `local`.

**Fases:** `Phase::ThermodynamicLayer` (aggregation, primera fase); `Phase::Input` (consumer, after aggregation).

**Runtime:** O(1024) para aggregation cada 8 ticks. AggSignalBridge: key `(cell_idx, region_idx)` → ~90% hit rate para grupos colocalizados.

**Trade-offs:** Un Resource global vs. señales por entidad. Tradeoff: resolución vs. memoria. 32×32 = 1024 locales, 8×8 = 64 regionales — manejable.

---

### ET-12: Continental Drift

**Propósito:** Modificación geológica del campo energético vía actividad tectónica.

**Contrato:**
- Input: `TectonicState { plates[4], global_tick }`.
- Output: `EnergyFieldGrid` modificado por `TectonicEvent`. `TerrainMutationEvent` emitido.
- Invariant: MAX_PLATES=4 array fijo. Epicentro deterministico por `(tick_id ^ plate_id)`. Eval cada 500 ticks.

**Fases:** `Phase::MorphologicalLayer` (stress + mutation, last in tier T3).

**Runtime:** O(4 plates × 1024 cells) cada 500 ticks → CPU ≈ 0 en régimen normal.

**Trade-offs:** Determinismo sobre realismo — la "aleatoriedad" geológica es pseudo-aleatoria reproducible.

---

### ET-13: Geological Time LOD

**Propósito:** Compresión temporal dinámica para simulación de escalas geológicas.

**Contrato:**
- Input: `entity_count`, `GeologicalLODState`, `MultiscaleSignalGrid`.
- Output: LOD level ajustado. Entidades con `LODCompressed` marker. `PopulationGroup` en `aggregate_groups`.
- Invariant: Desagregación determinista por `entity.index()` como seed.

**Fases:** `Phase::ThermodynamicLayer` (LOD controller); `Phase::MetabolicLayer` (aggregation + compressed physics).

**Runtime:** LOD controller cada 100 ticks. Aggregation O(entities) cuando LOD > 0. LODPhysicsBridge: Small(16) → prácticamente 100% hit rate (4 levels × 4 compressions).

**Trade-offs:** LOD global (no per-entidad) simplifica la arquitectura a costo de granularidad.

---

### ET-14: Institutions

**Propósito:** Reglas de coordinación con enforcement que trascienden individuos.

**Contrato:**
- Input: `InstitutionMember { institution_id, contribution, compliance: u8, join_tick }`. `InstitutionRegistry`.
- Output: Surplus distribuido a miembros compliant. `InstitutionEvent` en cambios.
- Invariant: Instituciones son Resources, no entidades. Persisten sin fundadores.

**Fases:** `Phase::MorphologicalLayer` (stability + distribution, after coalitions ET-8).

**Runtime:** O(institutions × members) cada `eval_interval=20` ticks. InstitutionBridge: key `(rule_hash, compliance_band)`.

**Trade-offs:** Sin democracia formal. Liderazgo = mayor contribución. Simple y suficiente para gameplay emergente.

---

### ET-15: Language

**Propósito:** Vocabulario simbólico compartido para comunicación a distancia.

**Contrato:**
- Input: `LanguageCapacity { vocabulary[8], vocab_count, signal_range, encoding_cost }`.
- Output: Nuevos símbolos aprendidos si `symbol_fitness > 0`. `vocab_count` aumenta.
- Invariant: `vocabulary: [u32; 8]` sin Vec. Símbolos son hashes `u32` — sin String.

**Fases:** `Phase::Input` (after cultural_transmission_system).

**Runtime:** O(nearby × vocab_size) cada `transmission_interval=5` ticks. SymbolBridge: key `(vocab_hash_a, vocab_hash_b)`.

**Trade-offs:** MAX_VOCAB_SIZE=8. Suficiente para proto-lenguaje funcional. Sin sintaxis formal.

---

### ET-16: Functional Consciousness

**Propósito:** Automodelo + planificación a largo plazo = conciencia funcional.

**Contrato:**
- Input: `SelfModel { predicted_qe, planning_horizon, self_accuracy, metacog_cost }`.
- Output: `SelfModel` actualizado. `FunctionallyConscious` marker insertado cuando umbral alcanzado.
- Invariant: `FunctionallyConscious` como SparseSet. `self_accuracy` actualizado por EMA. Planificación sólo para entidades conscientes.

**Fases:** `Phase::MorphologicalLayer` (update, last in entire ET track); `Phase::Input` (planning, before behavior).

**Runtime:** Update cada 5 ticks. Planning sólo para `With<FunctionallyConscious>` — subset pequeño. SelfModelBridge: ~70% hit rate.

**Trade-offs:** Conciencia funcional emerge gradualmente — no es binaria. Umbral `accuracy>0.7 AND horizon>100` es calibrable.

---

## Acoplamiento entre Módulos

```
ET-1 AssociativeMemory ──→ ET-10 TimescaleAdapter.learned_offset
ET-3 CulturalMemory ─────→ ET-10 TimescaleAdapter.cultural_offset
ET-6 EpigeneticState ────→ ET-10 TimescaleAdapter.epigenetic_offset
ET-3 MemeEntry.behavior_hash ─→ ET-15 LanguageCapacity.vocabulary
ET-8 CoalitionRegistry ──→ ET-14 InstitutionRegistry (fundación)
ET-11 MultiscaleSignal ──→ ET-13 GeologicalLOD (señal regional)
ET-12 TectonicState ─────→ ET-13 GeologicalLOD (compresión tectónica)
ET-15 LanguageCapacity ──→ ET-16 SelfModel (comunicación de planes)
```

**Regla de acoplamiento:** Todos los cruces se hacen via componentes compartidos o eventos. Nunca llamadas directas de sistema a sistema. El bajo acoplamiento es estructural, no una disciplina.

---

## TDD: Patrón de Tests por Tier

### Tier T1 (Individual Adaptation)
```rust
// Unit test (sin Bevy):
#[test] fn association_strength_decay_halves_at_tau() { ... }
#[test] fn expected_value_returns_zero_for_empty_memory() { ... }

// Integration test (MinimalPlugins):
#[test] fn memory_update_system_records_outcome() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // spawn entity with AssociativeMemory
    // run 1 tick
    // assert entry_count == 1
}
```

### Tier T2 (Collective Organization)
```rust
// Conservation test (invariante energético):
#[test] fn symbiosis_mutualism_does_not_create_energy() {
    // Suma de qe antes y después es igual (mutualism sólo redistribuye intake)
}

// Nash stability test:
#[test] fn coalition_with_negative_stability_dissolves() { ... }
```

### Tier T3 (Spatial/Temporal Scale)
```rust
// Determinism test (INV-4):
#[test] fn tectonic_epicenter_is_deterministic() {
    let seed_1 = compute_epicenter(tick=1000, plate_id=0);
    let seed_2 = compute_epicenter(tick=1000, plate_id=0);
    assert_eq!(seed_1, seed_2);
}

// LOD test:
#[test] fn lod_compressed_entities_dont_run_individual_physics() { ... }
```

### Tier T4 (Meta-Emergence)
```rust
// Emergent threshold test:
#[test] fn consciousness_requires_both_accuracy_and_horizon() {
    assert!(!consciousness_threshold(0.8, 50));
    assert!(!consciousness_threshold(0.5, 200));
    assert!(consciousness_threshold(0.8, 200));
}

// Transcendence test (instituciones sobreviven a fundadores):
#[test] fn institution_persists_after_founder_despawn() { ... }
```

---

## Invariantes del Track

1. **INV-ET1:** Ningún componente ET usa `Vec`, `Box`, `String`, `HashMap`.
2. **INV-ET2:** Todas las ecuaciones ET son pure `fn` en `blueprint/equations/emergence/`.
3. **INV-ET3:** Cada BridgeKind ET es un struct vacío en `bridge/config.rs` o `bridge/impls/`.
4. **INV-ET4:** `CoalitionBridge` usa `CacheBackend::Large(512)` — no Small.
5. **INV-ET5:** `strategy: u8` en componentes — enum sólo en blueprint/equations.
6. **INV-ET6:** Sistemas ET asignados a `Phase::` explícito — nunca `Update`.
7. **INV-ET7:** Tests deterministas en T3/T4 — misma seed → mismos resultados.
8. **INV-ET8:** `FunctionallyConscious` sólo para entidades con `self_accuracy > 0.7 AND planning_horizon > 100`.
