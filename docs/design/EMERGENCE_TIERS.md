# Emergence Tiers — Design Document

## Axioma Central

```
dS/dt = R(S) - D(S) - C(S)
```

Toda entidad en Resonance, a toda escala, evoluciona según esta ecuación:
- **R(S):** Extracción de recursos del entorno (intake de qe)
- **D(S):** Disipación intrínseca (metabolismo basal + senescencia + entropia)
- **C(S):** Costo de coordinación (cooperación, comunicación, cognición)

La emergencia surge cuando los sistemas de nivel superior **reducen C** mientras **aumentan R** — la coordinación tiene rendimientos crecientes.

---

## Arquitectura de 4 Tiers

```
T4: Meta-Emergencia ─────────────── Instituciones + Lenguaje + Conciencia
    ↑ requiere                      (coordinación simbólica a distancia)
T3: Escala Espacio-Temporal ─────── Timescales + Multiscale + Tectónica + LOD
    ↑ requiere                      (la historia importa en múltiples escalas)
T2: Organización Colectiva ─────── Simbiosis + Epigenética + Senescencia + Coaliciones + Nicho
    ↑ requiere                      (entidades forman sistemas acoplados)
T1: Adaptación Individual ──────── Memoria + TOM + Cultura + Infraestructura
    ↑ se construye sobre            (cada individuo aprende y modifica el mundo)
L0-L13: 14 Capas ECS ────────────── BaseEnergy → StructuralLink
```

---

## Por Qué Este Diseño

### 1. Stateless-First es no-negociable

Cada componente ET tiene **máximo 4 campos** y usa **arrays fijos** `[T; N]`:
- `AssociativeMemory`: `[MemoryEntry; 8]` — 8 entradas, sin Vec
- `CulturalMemory`: `[MemeEntry; 4]` — 4 memes, sin heap
- `LanguageCapacity`: `[u32; 8]` — vocabulario como array de hashes

**Por qué:** Vec en componentes significa allocación en el heap en cada tick. Con 10k+ entidades, el GC pressure destruye el framerate. Los arrays fijos viven en la ECS sparse table — coherentes con la caché L1 del CPU.

### 2. Ecuaciones puras en blueprint/equations

```
src/blueprint/equations/emergence/
├── associations.rs      // ET-1
├── theory_of_mind.rs    // ET-2
├── culture.rs           // ET-3
├── infrastructure.rs    // ET-4
├── symbiosis.rs         // ET-5
├── epigenetics.rs       // ET-6
├── senescence.rs        // ET-7
├── coalitions.rs        // ET-8
├── niche.rs             // ET-9
├── timescale.rs         // ET-10
├── multiscale.rs        // ET-11
├── tectonics.rs         // ET-12
├── geological_lod.rs    // ET-13
├── institutions.rs      // ET-14
├── language.rs          // ET-15
└── self_model.rs        // ET-16
```

**Por qué:** Las ecuaciones puras son trivialmente testeables sin Bevy. La física es matemáticamente verificable. El 90% del comportamiento emergente vive en estas ~500 líneas de funciones `fn(inputs) -> output`.

### 3. BridgeCache — El Impacto en Rendimiento

La arquitectura de caché tiene **dos regímenes**:

#### Régimen Normal (T1-T2): Small backend
```
entidades con misma banda → misma clave → hit de caché → 0 cálculo
```
- `EpigeneticBridge`: 16 ticks de throttle + cache → 16× speedup sobre cálculo bruto
- `SenescenceBridge`: LUT de 8 bandas → O(1) siempre
- `MemeSpreadBridge`: misma densidad → mismo spread_rate

#### Régimen Crítico (T2-4 Coaliciones): Large backend
```
Nash O(n²) × 50 coaliciones × 8 miembros = ~1600 comparaciones/tick sin caché
Con CoalitionBridge Large(512) + eval_interval=10: ~160 comparaciones / 10 ticks = 16 comparaciones/tick
```
La diferencia: **100× speedup** en el peor caso.

#### Regla de selección de backend:
```
Ecuación con complejidad > O(n) sobre grupos → Large(≥128)
Ecuación con complejidad O(1) o O(n) individual → Small(≤128)
Ecuación con LUT discreta → Small(≤32)
```

### 4. Bajo Acoplamiento: Cómo se Respeta

Los módulos ET se acoplan **sólo por componentes compartidos**, nunca por llamadas directas:

```
ET-3 escribe en CulturalMemory.memes[]
ET-10 lee CulturalMemory.memes[] para cultural_offset
→ ET-3 no sabe que existe ET-10
→ ET-10 no sabe que existe ET-3
→ El componente es la interfaz
```

Esto es **ECS arquitectura pura**: los sistemas son consumidores de componentes, no dependientes de otros sistemas.

Los únicos canales de comunicación inter-módulo explícitos son:
- `MemeAdoptedEvent` (ET-3 → replay/observabilidad)
- `CoalitionChangedEvent` (ET-8 → invalidación de caché)
- `InfrastructureInvestEvent` (ET-4 ↔ ET-14)
- `TectonicEvent` + `TerrainMutationEvent` (ET-12 → topology)

### 5. TDD: Tres Niveles de Test

```
Nivel 1 — Unit (sin Bevy, sin ECS):
  blueprint/equations/emergence/*.rs → #[cfg(test)] mod tests
  Cobertura: ecuaciones puras, casos borde, invariantes matemáticos
  Velocidad: < 1ms por test

Nivel 2 — Integration (MinimalPlugins, 1 tick):
  tests/emergence_*.rs
  Pattern:
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, my_system);
    let entity = app.world.spawn((Component1::new(), Component2::new())).id();
    app.update();
    let result = app.world.get::<ComponentN>(entity).unwrap();
    assert_eq!(result.field, expected);

Nivel 3 — Conservation (invariante energético):
  tests/emergence_conservation.rs
  assert!(total_qe_after <= total_qe_before + epsilon)  // energía no se crea
```

---

## Ondas de Implementación

### Onda 0: Fundación Individual (ET-1, ET-5, ET-7, ET-10)
Prereqs: ninguno. Las cuatro patas del aprendizaje, symbiosis, aging, timescales.

### Onda A: Adaptación Colectiva (ET-2, ET-3, ET-6, ET-9, ET-11)
Prereqs: Onda 0. Theory of mind, cultura, epigenética, nicho, señal multi-escala.

### Onda B: Modificación del Mundo (ET-4, ET-8, ET-12, ET-13)
Prereqs: Onda A. Infraestructura, coaliciones, tectónica, LOD geológico.

### Onda C: Meta-Emergencia (ET-14, ET-15, ET-16)
Prereqs: Onda B. Instituciones, lenguaje, conciencia funcional.

---

## Métricas de Emergencia Observable

Cuando el track ET está completo, estas métricas son medibles in-game:

| Métrica                        | Módulo fuente         | Threshold observable                    |
|-------------------------------|----------------------|------------------------------------------|
| Divergencia cultural regional  | ET-3                 | `cultural_distance([A], [B]) > 1.5`     |
| Nichos diferenciados           | ET-9                 | `niche_overlap < 0.1` entre compidores  |
| Coaliciones Nash estables      | ET-8                 | `coalition_stability > 0` por 100+ ticks |
| Lenguaje compartido regional   | ET-15                | `shared_vocabulary_ratio > 0.6` en grupo |
| Instituciones trans-generacionales | ET-14          | Institución viva después de muertos fundadores |
| Entidades conscientes          | ET-16                | `FunctionallyConscious` marker presente |
| Baldwin fixation               | ET-10                | `genetic_baseline` aumenta a lo largo de τ_g |

---

## Integración con Simulación Existente

### Fases del Pipeline (SystemSet ordering)

```
SimulationClockSet                  ← tick_id increment
Phase::ThermodynamicLayer           ← ET-11 aggregation, ET-13 LOD controller
Phase::AtomicLayer                  ← (no ET here)
Phase::ChemicalLayer                ← ET-5 symbiosis effects
Phase::MetabolicLayer               ← ET-4 infra, ET-7 senescence, ET-8 coalitions,
                                       ET-12 tectonic mutation, ET-13 aggregation/physics
Phase::Input                        ← ET-1 memory, ET-2 TOM, ET-3 culture,
                                       ET-10 cultural sync, ET-11 consumer, ET-15 language
Phase::MorphologicalLayer           ← ET-6 epigenetics, ET-9 niche, ET-10 Baldwin,
                                       ET-12 tectonic stress, ET-14 institutions,
                                       ET-16 self_model (LAST)
```

### Plugins de Registro

```rust
pub struct EmergenceTier1Plugin;  // ET-1..ET-4
pub struct EmergenceTier2Plugin;  // ET-5..ET-9
pub struct EmergenceTier3Plugin;  // ET-10..ET-13
pub struct EmergenceTier4Plugin;  // ET-14..ET-16

// Cada plugin registra:
// - Componentes + reflect
// - Resources (incluyendo BridgeCache<XBridge>)
// - Eventos
// - Sistemas con .in_set(Phase::X).after(y)
```

---

## Anti-Patrones Evitados

| Anti-patrón                              | Solución ET                                          |
|------------------------------------------|------------------------------------------------------|
| Vec en componentes (heap por entidad)    | Arrays fijos `[T; N]`                                |
| HashMap en hot path                      | Sorted Vec + binary search, o BridgeCache            |
| String en componentes                    | `u32` hash (`behavior_hash`, `symbol`, `rule_hash`)  |
| God-system (>5 component types)          | Un sistema por transformación                        |
| Lógica en métodos con side effects       | `&self` puro, sistemas hacen la escritura            |
| Box<dyn Trait> para estrategias          | `strategy: u8` + match en ecuación                  |
| Entity como ID persistente               | `WorldEntityId.0: u32` o hash determinista           |
| O(n²) sin caché                          | CoalitionBridge Large(512) + eval_interval           |
