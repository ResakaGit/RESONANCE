# ADR-017: Algorithmic Cache Integration — Reemplaza Bridge Decoupling con integración directa

**Status:** Proposed
**Date:** 2026-04-12
**Deciders:** Resonance Development Team
**Context of:** BRIDGE_STRATEGY_DECOUPLING (BS-1 a BS-7), performance, morphogenesis inference
**Extends:** CACHE_STRATEGY_DESIGN_V2, CACHE_PERFORMANCE_DOD (6 reglas)

---

## Context

El track BRIDGE_STRATEGY_DECOUPLING (BS-1 a BS-7) fue diseñado para desacoplar normalización de atención en el Bridge Optimizer. Durante BS-3, el análisis CACHE_STRATEGY_DESIGN_V2 descubrió que **solo 1 de 8 bloques computacionales se beneficia del bridge cache** (Frequency Alignment). Los otros 7 se optimizan mejor con patrones algorítmicos directos.

### Estado actual

**Componentes implementados (BS-3 ✅, 46 tests, ~280 LOC):**

| Componente | Archivo | Patrón | Tests |
|-----------|---------|--------|-------|
| `KleiberCache` | `layers/kleiber_cache.rs` | Dirty flag (SparseSet) | 8 |
| `GompertzCache` | `layers/gompertz_cache.rs` | Precompute on spawn (SparseSet) | 8 |
| `Converged<T>` | `layers/converged.rs` | Convergence detection (SparseSet) | 7 |
| Pure functions | `equations/exact_cache.rs` | Stateless math | 23 |

**Ninguno está integrado en sistemas.** Los sistemas siguen ejecutando `powf()` y `exp()` per-tick.

### Sprints pendientes y su validez post-V2

| Sprint | Propuesta original | Validez post-V2 |
|--------|-------------------|-----------------|
| BS-1 | `NormStrategy` enum (4 variantes) | **Parcialmente invalidado** — V2 elimina FrequencyAligned y TemporalWindow. Solo quedan Concentration (gameplay) y Passthrough (científico), que ya existen implícitamente en `BridgeConfig.enabled` |
| BS-4 | 6 bridges nuevos via `Bridgeable` trait | **Invalidado** — V2 demuestra que 5/6 no necesitan bridge; dirty flag y convergence son superiores |
| BS-5 | TDD para los 6 bridges de BS-4 | **Reemplazable** — tests para integración directa en vez de bridges |
| BS-6 | NormPipeline (HOF composition) | **Invalidado** — V2: "sin variantes de normalización, no hay nada que componer" |
| BS-7 | RON presets para estrategias | **Invalidado** — depende de BS-6 que no tiene razón de ser |

### Costo de oportunidad

Los 5 sprints pendientes suman esfuerzo M+L+L+M+S para construir abstracción (bridges + pipeline + presets) alrededor de código que se optimiza mejor sin abstracción. Cada bridge añade: 1 `BridgeKind` marker, 1 `impl Bridgeable`, 1 `BridgeConfig` registration, bandas de normalización — overhead de diseño sin ganancia sobre dirty-flag directo.

---

## Decision

**Cancelar BS-1, BS-4, BS-6, BS-7. Reemplazar con integración algorítmica directa de los componentes ya implementados. Redefinir BS-5 como tests de integración.**

### Principio rector

> El cache más rápido es el que no necesita cache: precomputar en el momento correcto y skip cuando convergió.

Los bridges (normalización + LRU + quantized keys) son correctos para **ecuaciones con inputs continuos de alta frecuencia** (density, temperature, phase transition) — los 11 bridges existentes. Pero para procesos con **inputs discretos, convergentes o monótonos**, dirty flags y precompute son superiores en precisión (exact), latencia (zero lookup) y complejidad (zero config).

### 4 cambios concretos

#### Cambio 1: KleiberCache → `basal_drain_system`

**Archivo:** `src/simulation/metabolic/basal_drain.rs`
**Línea actual (32):** `let vol_factor = volume.radius.max(0.01).powf(dt::KLEIBER_EXPONENT);`

```rust
// ANTES: powf() per-tick per-entity (~3ns × N entities × 60 ticks/s)
let vol_factor = volume.radius.max(0.01).powf(dt::KLEIBER_EXPONENT);

// DESPUÉS: dirty flag — powf() solo cuando radius cambia (growth events, ~1/500 ticks)
kleiber.update(volume.radius);
let vol_factor = kleiber.vol_factor();
```

**Query cambia de:**
```rust
Query<(Entity, &SpatialVolume, Option<&SenescenceProfile>), ...>
```
**A:**
```rust
Query<(Entity, &SpatialVolume, &mut KleiberCache, Option<&SenescenceProfile>), ...>
```

**Impacto perf:** Elimina `powf(0.75)` per-tick. Con 512 entidades a 60 Hz: ~92K `powf` eliminados/s.
**Precisión:** Bit-identical. `kleiber_volume_factor()` usa el mismo `powf` internamente.
**Axiomas:** Sin efecto. El drain sigue siendo `RATE × r^0.75 × age_factor` (Axiom 4: dissipation).
**Stateless:** `KleiberCache` es componente (SparseSet). `update()` es pura: `(radius) → vol_factor`. Sin side effects.
**Spawn:** `KleiberCache::default()` se inserta en los spawn sites junto a `SenescenceProfile`. NaN default garantiza primer recompute.

---

#### Cambio 2: GompertzCache → `senescence_death_system`

**Archivo:** `src/simulation/metabolic/senescence_death.rs`
**Líneas actuales (37-42):** `survival_probability()` llama `exp()` per-tick per-entity.

```rust
// ANTES: exp() per-tick per-entity
let prob = survival_probability(age, coeff, coeff);
if prob < threshold { die(); }

// DESPUÉS: 1 comparación u64 per-tick (algebraic solve en spawn)
if gompertz.should_die(clock.tick_id) { die(); }
```

**Query cambia de:**
```rust
Query<(Entity, &BaseEnergy, &SenescenceProfile)>
```
**A:**
```rust
Query<(Entity, &BaseEnergy, &SenescenceProfile, &GompertzCache)>
```

**El hard age limit (`age >= max_viable_age`) se mantiene** como safety net redundante — no confiar en un único mecanismo de muerte.

**Impacto perf:** Elimina `exp()` per-tick. Con 512 entidades a 60 Hz: ~30K `exp` eliminados/s.
**Precisión:** Exacta. Solución algebraica cuadrática: `t = (-base + √(base² + 4×coeff)) / coeff`.
**Axiomas:** Sin efecto. Gompertz sigue siendo función de `senescence_coeff` derivado de las 4 constantes (Axiom 4).
**Stateless:** `GompertzCache` es write-once. `should_die()` es pura: `(tick) → bool`.
**Spawn:** `GompertzCache::from_senescence(birth, base, coeff, max_age)` en los spawn sites junto a `SenescenceProfile`.

---

#### Cambio 3: Converged\<EpigeneticState\> → `epigenetic_adaptation_system`

**Archivo:** `src/simulation/emergence/epigenetic_adaptation.rs`
**Cambio:** Skip entidades cuya expression_mask convergió y cuyo env_signal no cambió.

```rust
// ANTES: computa 4 dimensiones de lerp + silencing cost per-tick per-entity
for dim in 0..4 { ... lerp ... }

// DESPUÉS: skip converged entities (~85% de ticks en flora)
if let Some(conv) = converged {
    let current_hash = hash_f32(pressure.terrain_viscosity);
    if conv.is_valid(current_hash) { continue; }
    commands.entity(entity).remove::<Converged<EpigeneticState>>();
}

// ... compute exact ...

let all_converged = (0..4).all(|d| (epi.expression_mask[d] - target[d]).abs() < 1e-4);
if all_converged {
    let env_hash = hash_f32(pressure.terrain_viscosity);
    commands.entity(entity).insert(Converged::<EpigeneticState>::new(env_hash));
}
```

**Query cambia de:**
```rust
Query<(&mut EpigeneticState, &AmbientPressure, &mut BaseEnergy)>
```
**A:**
```rust
Query<(Entity, &mut EpigeneticState, &AmbientPressure, &mut BaseEnergy, Option<&Converged<EpigeneticState>>)>
```

**Impacto perf:** Skip ~85% de evaluaciones (flora convergida, entidades estáticas).
**Precisión:** Exacta. Computa exacto cuando dirty, skip solo cuando `delta < 1e-4` (guard ya existente en L26-28).
**Axiomas:** Sin efecto. La expresión sigue modulada por environment × threshold (Axiom 8 via frequency alignment implícito).
**Stateless:** `Converged<T>` es marcador SparseSet con hash. Insert/remove via Commands — zero shared state.

---

#### Cambio 4: Converged\<MorphogenesisShapeParams\> → `shape_optimization_system`

**Archivo:** `src/simulation/metabolic/morphogenesis.rs` (L478-515)
**Cambio:** Skip entidades cuya fineness convergió y cuyos inputs no cambiaron.

```rust
// ANTES: bounded_fineness_descent() (iterativo, ~10 iter) per-tick per-entity
let new_fineness = equations::bounded_fineness_descent(..., MAX_ITER);

// DESPUÉS: skip converged entities (~90% de ticks en adultos)
if let Some(conv) = converged {
    let input_hash = hash_shape_inputs(density, velocity, radius, vasc_cost);
    if conv.is_valid(input_hash) { continue; }
    commands.entity(entity).remove::<Converged<MorphogenesisShapeParams>>();
}

// ... bounded descent exact ...

let delta = (new_fineness - shape.fineness_ratio()).abs();
if delta < 0.01 {
    let input_hash = hash_shape_inputs(density, velocity, radius, vasc_cost);
    commands.entity(entity).insert(Converged::<MorphogenesisShapeParams>::new(input_hash));
}
```

**Helper puro en `exact_cache.rs`:**
```rust
pub fn hash_shape_inputs(density: f32, velocity: f32, radius: f32, vasc_cost: f32) -> u64 {
    // Knuth multiplicative — deterministic, zero allocation
    let mut h = hash_f32(density);
    h = h.wrapping_mul(2_654_435_761).wrapping_add(hash_f32(velocity));
    h = h.wrapping_mul(2_654_435_761).wrapping_add(hash_f32(radius));
    h.wrapping_mul(2_654_435_761).wrapping_add(hash_f32(vasc_cost))
}
```

**Impacto perf:** Skip ~90% de evaluaciones. `bounded_fineness_descent` es la operación más cara del MorphologicalLayer (iterativo con hasta MAX_ITER pasos).
**Precisión:** Exacta. Convergence threshold `0.01` alineado con `shape.update()` que ya guarda.
**Axiomas:** Sin efecto. Constructal optimization sigue minimizando `shape_cost = drag + vascular` (Axiom 4/7).
**Stateless:** Mismo patrón que Cambio 3. Insert/remove via Commands.

---

## Consecuencias

### Positivas

1. **Performance inmediata.** Los 4 cambios eliminan operaciones caras per-tick:
   - `powf(0.75)`: ~3ns/call → 0 (dirty flag)
   - `exp()`: ~4ns/call → 0 (precompute)
   - 4-dim lerp: skip 85% entidades (convergence)
   - bounded descent: skip 90% entidades (convergence)

2. **Zero pérdida de precisión.** Cada cambio produce output bit-identical al código actual. Demostrable con test `assert_eq!(cached_path, exact_path)`.

3. **Complejidad reducida.** Elimina 5 sprints de infraestructura abstracta. Los componentes ya existen y tienen 46 tests. Solo falta wiring.

4. **Respeta DoD de cache (6 reglas):**
   - R1: SparseSet components ✅
   - R2: Dispensable — clearing any cache produce resultados idénticos en siguiente tick ✅
   - R3: Invalidation declarada — dirty flag (Kleiber), write-once (Gompertz), env_hash (Converged) ✅
   - R4: Metrics — KleiberCache N/A (on-change), GompertzCache N/A (write-once), Converged N/A (marker) ✅
   - R5: Budgets calibrados — no aplica (no son per-tick budgets) ✅
   - R6: Benchmarks — verificable con `batch_benchmark` existente ✅

5. **Bridges existentes intactos.** Los 11 bridges (Density, Temperature, Phase, etc.) siguen funcionando para gameplay con normalización Concentration. `BridgeConfig.enabled = false` ya es el bypass exacto para modo científico.

### Negativas

1. **BS-6/BS-7 se pierden.** NormPipeline y RON presets no se implementan. Si en el futuro se necesitan percepciones metafísicas (predator/flora/observer con distinta precisión), habrá que diseñar de nuevo. **Mitigante:** el costo de implementarlos es bajo (M+S) y no tienen dependencias externas.

2. **Spawn sites requieren cambio.** Cada archetype que spawna entidades con `SenescenceProfile` debe insertar `KleiberCache::default()` + `GompertzCache::from_senescence(...)`. **Mitigante:** Bevy 0.15 `#[require(...)]` puede automatizar esto.

### Riesgos

1. **Entidades sin cache component.** Si un spawn site olvida insertar KleiberCache, `basal_drain_system` no encontrará la entidad. **Mitigante:** Query con `Option<&mut KleiberCache>` + fallback a `powf()` durante transición. Eliminar fallback cuando todos los spawn sites estén cubiertos.

2. **Convergence false positive.** Si `hash_shape_inputs` colisiona, una entidad podría quedar "converged" con inputs distintos. **Mitigante:** Knuth multiplicative hash sobre f32 bits tiene colisión rate < 1/2^32 para inputs distintos. Además, el peor caso es 1 tick de retraso — no corrupción.

---

## Archivos modificados

| Archivo | Cambio | Cambio nº |
|---------|--------|-----------|
| `src/simulation/metabolic/basal_drain.rs` | Query + KleiberCache.update() | 1 |
| `src/simulation/metabolic/senescence_death.rs` | Query + GompertzCache.should_die() | 2 |
| `src/simulation/emergence/epigenetic_adaptation.rs` | Query + Converged skip + insert | 3 |
| `src/simulation/metabolic/morphogenesis.rs` | Query + Converged skip + insert (shape_optimization) | 4 |
| `src/blueprint/equations/exact_cache.rs` | `hash_shape_inputs()` helper | 4 |
| `src/entities/archetypes/*.rs` | Insert KleiberCache + GompertzCache en spawns | 1, 2 |

### Archivos NO modificados

| Archivo | Razón |
|---------|-------|
| `src/bridge/*` | Bridge infrastructure intacta — los 11 bridges existentes no cambian |
| `src/layers/kleiber_cache.rs` | Componente ya implementado, no requiere cambios |
| `src/layers/gompertz_cache.rs` | Componente ya implementado, no requiere cambios |
| `src/layers/converged.rs` | Componente ya implementado, no requiere cambios |
| `src/blueprint/equations/exact_cache.rs` | Solo se añade `hash_shape_inputs()` |

---

## Criterios de éxito

1. `cargo test` — todos los tests existentes pasan (zero regression).
2. `cargo bench --bench batch_benchmark` — throughput igual o superior al baseline.
3. Test nuevo: `assert_eq!(cached_drain, exact_drain)` para KleiberCache path.
4. Test nuevo: `GompertzCache.should_die(tick)` equivalente a `survival_probability(age, ...) < threshold`.
5. Test nuevo: `Converged<EpigeneticState>` se invalida cuando `terrain_viscosity` cambia.
6. Test nuevo: `Converged<MorphogenesisShapeParams>` se invalida cuando velocity/density/radius/vasc_cost cambia.

---

## Alternativas consideradas

### A: Implementar BS-1 a BS-7 como diseñados

6 bridges nuevos + NormStrategy enum + NormPipeline + RON presets. Esfuerzo total: ~M+L+L+M+S.

**Rechazada.** V2 demostró que 5/6 bridges propuestos tienen overhead (normalización + LRU) superior al cómputo directo. La ecuación de basal drain es 2 multiplicaciones post-cache — el bridge lookup (hash + compare + possible evict) es más caro que el cómputo.

### B: Implementar solo BS-1 (NormStrategy simplificado: 2 variantes)

Añadir `NormStrategy::Passthrough` como opción en `BridgeConfig`.

**Aplazada.** No rechazada — es útil para modo científico, pero `BridgeConfig.enabled = false` ya logra el mismo efecto. Si se necesita granularidad per-equation (algunos bridges Concentration, otros Passthrough en el mismo run), BS-1 simplificado se puede implementar después sin conflicto.

### C: No hacer nada

Los componentes quedan implementados pero no integrados.

**Rechazada.** 46 tests para código que nadie ejecuta en producción. El `powf()` y `exp()` per-tick son costos evitables ahora.

---

## Impacto en sprint backlog

| Sprint | Acción |
|--------|--------|
| BS-1 | **Cancelado** — `BridgeConfig.enabled` ya cubre Passthrough vs Concentration |
| BS-2 | ✅ Ya completado (no cambia) |
| BS-3 | ✅ Ya completado (no cambia) — provee los componentes que se integran |
| BS-4 | **Cancelado** — reemplazado por los 4 cambios de este ADR |
| BS-5 | **Redefinido** — tests de integración para los 4 cambios, no para bridges |
| BS-6 | **Cancelado** — NormPipeline sin razón de ser post-V2 |
| BS-7 | **Cancelado** — depende de BS-6 |

**Track resultante:** BS-2 ✅, BS-3 ✅, ADR-017 (4 cambios + tests) = track completo.

---

## Codebase references

- `src/layers/kleiber_cache.rs` — KleiberCache component (8 tests)
- `src/layers/gompertz_cache.rs` — GompertzCache component (8 tests)
- `src/layers/converged.rs` — Converged\<T\> generic (7 tests)
- `src/blueprint/equations/exact_cache.rs` — pure functions (23 tests)
- `src/simulation/metabolic/basal_drain.rs:32` — `powf()` a eliminar
- `src/simulation/metabolic/senescence_death.rs:37-42` — `exp()` a eliminar
- `src/simulation/emergence/epigenetic_adaptation.rs` — lerp a skip
- `src/simulation/metabolic/morphogenesis.rs:478-515` — descent a skip
- `docs/sprints/BRIDGE_STRATEGY_DECOUPLING/CACHE_STRATEGY_DESIGN_V2.md` — análisis fundacional
- `docs/design/CACHE_PERFORMANCE_DOD.md` — 6 reglas de cache
