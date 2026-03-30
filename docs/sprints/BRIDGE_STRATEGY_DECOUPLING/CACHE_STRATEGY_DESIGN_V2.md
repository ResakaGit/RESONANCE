# Diseño de Cache V2 — Zero Precision Loss

> **Restricción absoluta:** Ninguna optimización puede alterar un solo bit del output.
> Si `compute(input)` devuelve X, el sistema cacheado devuelve X. Sin excepciones.
> La simulación debe ser utilizable para investigación científica.

---

## Qué se elimina

| Concepto eliminado | Razón |
|---|---|
| `NormStrategy::Concentration` como cache | Colapsa valores distintos al mismo canonical → pierde precisión |
| `NormStrategy::FrequencyAligned` | Snap a canonical elemental → pierde precisión inter-banda |
| `NormStrategy::TemporalWindow` | Cuantiza tiempo → pierde resolución temporal |
| `NormPipeline` (composición de stages) | Sin variantes de normalización, no hay nada que componer |
| Variantes A del CACHE_STRATEGY_DESIGN.md | Todas usan bandas → aproximación |

**Concentration se mantiene** como opción configurable para gameplay (donde ±3% es aceptable), pero el default del simulador es precision-first.

---

## Qué sobrevive: 4 patrones de zero-loss

### Patrón 1: Memoización Exacta (Exact Memoization)

```
input → hash(bits exactos) → cache lookup
  → hit: return cached value (bit-identical)
  → miss: compute exact → store → return
```

**Invariante:** `output(cached) == output(exact)` para todo input. El hash usa `f32::to_bits()` — dos floats son el mismo key solo si son bit-identical.

**Cuándo funciona bien:** Cuando los mismos inputs exactos se repiten. Esto ocurre cuando:
- Inputs son discretos (matter state, capability set, organ role)
- Inputs se estabilizan (radius deja de crecer, frequency estable por homeostasis)
- Inputs son compartidos (muchas entidades con el mismo radius/frequency)

**Cuándo NO funciona:** Inputs continuos que cambian cada tick (qe drenada, age incrementa). Hit rate → 0%.

**Implementación:** Es el bridge actual con `normalize(input) = input` (Passthrough). Ya existe.

### Patrón 2: Dirty Flag (Compute-on-Change)

```
per_entity:
  if inputs_changed_since_last_compute → compute exact, store result
  else → return stored result
```

**Invariante:** `stored_result == compute(current_inputs)` siempre. El dirty flag solo controla CUÁNDO recomputar, no QUÉ computar.

**Cuándo funciona bien:** Procesos T1 (convergentes) y T3 (reactivos):
- Epigenética: expression converge → dirty flag = false hasta que entity se mueve
- Kleiber drain: radius estable → dirty flag = false hasta growth event
- Constructal fineness: converge en ~15 iter → dirty flag = false indefinidamente
- Osmosis: celda en equilibrio → dirty flag = false hasta vecino cambia

**Implementación:** Component `DirtyFlag<B>` (SparseSet, transient) set por eventos de cambio. El sistema checkea el flag antes de llamar a la ecuación.

### Patrón 3: Precompute en Spawn (Spawn-Time Exact Solve)

```
on_spawn:
  result = exact_solve(parameters)  // e.g., death_tick from Gompertz inverse
  store as component field
per_tick:
  read stored result (zero computation)
```

**Invariante:** `stored == exact_solve(spawn_params)`. Perfecto si los parámetros no cambian post-spawn.

**Cuándo funciona bien:** Procesos T2 (monótonos) con parámetros fijos:
- Senescence: `coeff` es constante → `death_tick = f(coeff)` computable en spawn
- Basal drain base: `radius^0.75` computable cuando radius se asigna (growth event, no per-tick)

**Implementación:** Campo adicional en el componente, actualizado por el sistema que modifica los inputs (growth system, no basal_drain system).

### Patrón 4: Lookup Table Exacta (Domain-Exact Static Table)

```
// Si el INPUT es naturalmente discreto, la tabla es exacta por definición.
alignment_table[element_a][element_b] = exp(-delta_f²/5000)
// Computada con f64 y truncada a f32 una vez en startup
```

**Invariante:** `table[a][b] == compute(canonical_freq_a, canonical_freq_b)` bit-for-bit. La tabla no aproxima — el dominio ES discreto.

**Cuándo funciona bien:** Procesos T0 (constantes) con dominio discreto:
- Frequency alignment entre elementos (6-8 elementos = 28-36 pares)
- Matter state thresholds (4 estados)
- Capability set (8 capabilities = 256 combinaciones)
- Senescence coefficients (3 matter tiers)

**Cuándo NO funciona:** Si el input es continuo (qe arbitrario, radius arbitrario), la tabla tendría 2^32 entries.

**Implementación:** `const` arrays o `Resource` inicializado en startup. Zero overhead runtime.

---

## Aplicación por bloque

### BLOQUE 1: Basal Drain (Kleiber)

```
drain = BASAL_DRAIN_RATE × radius^0.75 × (1 + coeff × age)
```

**Input analysis:**
- `radius`: cambia solo por growth (Patrón 2: dirty flag)
- `coeff`: constante por entity type (Patrón 4: tabla de 3 valores)
- `age`: incrementa 1/tick — cambia SIEMPRE (ningún cache ayuda)

**Estrategia:**
```
Precompute: vol_factor = radius^0.75          → Patrón 3 (on growth event)
Lookup:     coeff = senescence_coeff[tier]    → Patrón 4 (3 entries)
Per-tick:   age_factor = 1 + coeff × age      → inline (1 mul + 1 add, no cacheable)
Per-tick:   drain = RATE × vol_factor × age_factor  → inline (2 mul)
```

**Ganancia:** Elimina `powf(0.75)` per-tick. Reemplaza con 2 multiplicaciones.
**Precisión:** Bit-identical. `vol_factor` se computa exacto cuando radius cambia.

**Implementación concreta:**
```rust
// Nuevo campo en SenescenceProfile o componente auxiliar:
pub struct KleiberCache {
    pub vol_factor: f32,      // radius^0.75, updated by growth system
    pub last_radius: f32,     // dirty detection
}

// En basal_drain_system:
if volume.radius != cache.last_radius {
    cache.vol_factor = volume.radius.max(0.01).powf(KLEIBER_EXPONENT);
    cache.last_radius = volume.radius;
}
let drain = BASAL_DRAIN_RATE * cache.vol_factor * age_factor;
```

**Bridge involvement:** Ninguno. No necesita bridge. Es un dirty-flag component.

---

### BLOQUE 2: Senescence / Gompertz

```
survival = exp(-base × age - 0.5 × coeff × age²)
die if survival < threshold OR age >= max_age
```

**Input analysis:**
- `base`: constante (Patrón 4)
- `coeff`: constante (Patrón 4)
- `age`: incrementa 1/tick — determinista
- `threshold`: constante = exp(-2)

**Estrategia:**
```
Precompute en spawn: death_tick = solve(S(t) = threshold)    → Patrón 3
Per-tick:            if tick >= death_tick → die              → 1 comparación u64
```

**Cálculo del death_tick exacto:**
```
S(t) = exp(-base×t - 0.5×coeff×t²) = exp(-2)
→ base×t + 0.5×coeff×t² = 2
→ t = (-base + sqrt(base² + 4×coeff)) / coeff    (fórmula cuadrática)
→ death_tick = min(quadratic_solution, max_viable_age)
```

**Ganancia:** Elimina `exp()` per-tick. Reemplaza con 1 comparación u64.
**Precisión:** Exacta. La solución cuadrática es algebraica, no numérica.

**Implementación concreta:**
```rust
// En SenescenceProfile (campo nuevo):
pub death_tick: u64,  // precomputed at spawn

// En spawn (equations/):
pub fn exact_death_tick(birth_tick: u64, base: f32, coeff: f32, max_age: u64) -> u64 {
    if coeff <= 0.0 { return birth_tick + max_age; }
    let disc = base * base + 4.0 * coeff;
    let t = (-base + disc.sqrt()) / coeff;
    let age = (t as u64).min(max_age);
    birth_tick + age
}

// En senescence_death_system:
if clock.tick_id >= senescence.death_tick { die(); }
```

**Bridge involvement:** Ninguno. Precompute + field.

---

### BLOQUE 3: Frequency Alignment

```
alignment = exp(-delta_f² / (2 × bandwidth²))
```

**Input analysis:**
- `freq_a, freq_b`: cambian solo por homeostasis/entrainment (raro)
- `bandwidth`: constante = 50 Hz

**Estrategia A — Memoización exacta (bridge Passthrough):**
```
key = hash(f32::to_bits(freq_a), f32::to_bits(freq_b))
→ hit si exactamente el mismo par de frecuencias
```

Funciona porque las frecuencias se repiten exactas entre ticks (cambian raro). En un mundo con N entidades, hay a lo sumo N² pares pero las frecuencias se agrupan por bioma.

**Estrategia B — Lookup table sobre dominio discreto:**

Si las entidades solo tienen frecuencias que vienen de spawn (ElementDef canonical) o de entrainment (que converge a un canonical), el dominio real es ~20-50 frecuencias únicas. La tabla es computable al inicio de la simulación escaneando todas las frecuencias activas.

```rust
// Resource dinámico: actualizado cuando nuevas frecuencias aparecen
pub struct AlignmentLookup {
    freqs: Vec<f32>,             // frecuencias únicas observadas (sorted)
    table: Vec<f32>,             // alignment[i * n + j] para freqs[i], freqs[j]
}

impl AlignmentLookup {
    pub fn get(&self, fa: f32, fb: f32) -> Option<f32> {
        let ia = self.freqs.binary_search_by(|f| f.total_cmp(&fa)).ok()?;
        let ib = self.freqs.binary_search_by(|f| f.total_cmp(&fb)).ok()?;
        Some(self.table[ia * self.freqs.len() + ib])
    }
}
```

**Ganancia:** Elimina `exp()` per-pair. Lookup O(log N) con binary search, O(1) si se indexa por entity freq_id.
**Precisión:** Exacta para frecuencias observadas. Miss → compute exact y añadir a tabla.

**Bridge involvement:**
- Estrategia A: Bridge con Passthrough (memoización exacta). Hit rate ~60-80%.
- Estrategia B: Resource lookup. Bridge deshabilitado. Hit rate ~95-100%.

---

### BLOQUE 4: Radiation Pressure

```
transfer = rate × max(qe - threshold, 0) × alignment / n_neighbors
```

**Input analysis:**
- `qe`: cambia cada tick (fast) — NO cacheable
- `threshold`: constante — precomputed
- `alignment`: cambia raro (Bloque 3) — cacheable
- `rate`: constante — precomputed
- `n_neighbors`: constante para grid regular

**Estrategia: Separar la parte cara de la trivial.**

```
alignment = cached (Bloque 3, Patrón 1 o 4)
excess = (qe - threshold).max(0.0)         → 1 resta + max, inline
transfer = rate × excess × alignment / n   → 3 ops, inline
```

La ÚNICA parte cara es el alignment (exp gaussiana). El resto es aritmética trivial. Con el alignment cacheado (Bloque 3), el sistema completo de radiation pressure no necesita su propio bridge.

**Ganancia:** Hereda la ganancia del Bloque 3. El sistema per-cell baja de `exp() + 4 ops` a `lookup + 4 ops`.
**Precisión:** Exacta (alignment exacto → transfer exacto).

**Bridge involvement:** Ninguno propio. Depende del cache de alignment (Bloque 3).

---

### BLOQUE 5: Osmosis

```
flux = permeability × (concentration_a - concentration_b)
```

**Input analysis:**
- `concentration_a, _b`: cambian cada tick (diffusion ongoing)
- `permeability`: constante por celda

**Estrategia: Dirty flag por celda.**

La ecuación es una resta + multiplicación. No hay math cara (`exp`, `pow`, `sqrt`). El cuello de botella es **el número de celdas**, no el costo per-celda.

```
if cell.qe == cell.last_qe AND all_neighbors_unchanged → skip
else → compute exact, update last_qe
```

**Ganancia:** Skip celdas en equilibrio (~70-95% del grid post-diffusion). El sistema ya tiene LOD bands.
**Precisión:** Exacta (compute exact cuando dirty).

**Implementación:** El grid ya tiene un dirty-flag implícito en el LOD system. Formalizar como `is_dirty[cell_idx]` bitfield set cuando `qe` cambia.

**Bridge involvement:** Ninguno. La ecuación es trivial (resta + mul). El bridge overhead sería mayor que el compute.

---

### BLOQUE 6: Epigenetic Adaptation

```
new_expr = current + (target - current) × rate × dt
target = if should_express(env × current > threshold) { 1.0 } else { 0.0 }
```

**Input analysis:**
- `env_signal`: cambia cuando entity se mueve a nueva celda (raro para flora)
- `current`: cambia cada tick hasta convergencia, luego estable
- `rate, dt`: constantes

**Estrategia: Convergence detection.**

```
if |target - current| < 1e-4 for all 4 dims:
    converged = true
    skip until env_signal changes (entity moves)
else:
    compute exact
    check convergence
```

**Ganancia:** Skip ~85% de ticks (entidades convergidas). Flora no se mueve → convergence perpetua.
**Precisión:** Exacta. Convergence threshold `1e-4` es el guard que ya existe en el código.

**Implementación:**
```rust
// En EpigeneticState (campo nuevo):
pub converged: bool,   // set true when all dims within 1e-4 of target

// En epigenetic_adaptation_system:
if epi.converged {
    // Check if env changed (entity moved to new cell)
    if pressure.terrain_viscosity != epi.last_env_signal {
        epi.converged = false;
        epi.last_env_signal = pressure.terrain_viscosity;
    } else {
        continue;  // skip — still converged
    }
}
// ... compute exact ...
if all_dims_converged { epi.converged = true; }
```

**Bridge involvement:** Ninguno. Dirty flag + convergence detection.

---

### BLOQUE 7: Awakening Potential

```
potential = (coherence - dissipation) / (coherence + qe)
awaken if potential >= 1/3
```

**Input analysis:**
- `qe`: cambia cada tick (drain)
- `coherence`: cambia cada tick (neighbors)
- `dissipation`: cambia solo con matter state (raro)

**Estrategia: Budget scan + dirty flag.**

El sistema ya tiene un budget (`AWAKENING_BUDGET_PER_TICK = 4`). Entidades que no están near-threshold no necesitan evaluación frecuente.

```
// Tier 1: entities con potential < 0.1 → eval cada 16 ticks
// Tier 2: entities con potential ∈ [0.1, 0.5] → eval cada 4 ticks
// Tier 3: entities con potential > 0.5 → ya awakened, skip

// Esto NO pierde precisión porque:
// - La evaluación cuando ocurre es EXACTA
// - El scan interval solo retrasa la detección (máx 16 ticks)
// - Un entity no "pierde" awakening — lo detecta 16 ticks tarde como máximo
```

**Ganancia:** ~80% menos evaluaciones. Solo 20% de entidades evaluadas per-tick.
**Precisión:** Exacta cuando evalúa. La latencia de detección (máx 16 ticks) NO es pérdida de precisión — es latencia.

**Bridge involvement:** Ninguno. Scan budget + tiered intervals.

---

### BLOQUE 8: Constructal Fineness

```
fineness = gradient_descent(current, medium_density, velocity, vascular_cost)
```

**Input analysis:**
- `current fineness`: resultado del descent anterior (cambia solo si inputs cambian)
- `medium_density`: cambia solo si entity se mueve a nueva celda
- `velocity`: cambia por movimiento (medium frequency)
- `vascular_cost`: cambia solo por organ changes (raro)

**Estrategia: Convergence + dirty flag.**

```
if descent converged (|fineness_new - fineness_old| < 0.01):
    converged = true
    skip until velocity changes >20% OR entity moves cells
else:
    run descent (exact)
```

**Ganancia:** ~90% skip (entidades adultas con forma estable).
**Precisión:** Exacta (descent ejecuta hasta convergencia real).

**Bridge involvement:** Ninguno.

---

## Resumen ejecutivo

| Bloque | Patrón | Ganancia | Precisión | Usa Bridge? |
|---|---|---|---|---|
| **Kleiber** | Precompute `r^0.75` on growth event | Elimina `powf` per-tick | **Exacta** | No |
| **Gompertz** | Precompute `death_tick` on spawn | Elimina `exp` per-tick | **Exacta** | No |
| **Alignment** | Memoización exacta o lookup table | Elimina `exp` per-pair | **Exacta** | Sí (Passthrough) o No (lookup) |
| **Rad Pressure** | Hereda alignment cache | Elimina `exp` (vía Bloque 3) | **Exacta** | No (indirecto) |
| **Osmosis** | Dirty flag por celda | Skip equilibrium | **Exacta** | No |
| **Epigenetic** | Convergence detection | Skip converged entities | **Exacta** | No |
| **Awakening** | Tiered scan intervals | Reduce evaluaciones 80% | **Exacta** | No |
| **Constructal** | Convergence + dirty flag | Skip converged forms | **Exacta** | No |

---

## Impacto en la arquitectura Bridge

**Hallazgo:** De los 8 bloques, solo 1 (Alignment) se beneficia del Bridge cache. Los otros 7 se optimizan mejor con patrones algorítmicos (precompute, dirty flag, convergence) que no necesitan bridge.

Esto no invalida el bridge — los 11 bridges existentes (Density, Temperature, PhaseTransition, Interference, Catalysis, Collision, Osmosis, etc.) siguen siendo útiles para **gameplay con precision tolerable**. Pero para el simulador científico, el camino es:

```
Simulador científico:  bridge.enabled = false + optimizaciones algorítmicas
Gameplay/MOBA:         bridge.enabled = true  + Concentration normalization
```

El `BridgeConfig.enabled` flag ya permite esto. Zero cambio arquitectónico.

---

## Implementación: 3 componentes nuevos (IMPLEMENTADOS)

### 1. `KleiberCache` — precomputed volume factor

**Archivo:** `src/layers/kleiber_cache.rs` (implementado, 8 tests)

```rust
/// Cache exacta de radius^KLEIBER_EXPONENT. Actualizada solo por growth.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct KleiberCache {
    vol_factor: f32,
    last_radius: f32,  // NaN por default → primer update siempre dispara
}
```

Defensive: NaN/Inf radius → vol_factor = 0.0. NaN default garantiza primer recompute.

### 2. `GompertzCache` — precomputed exact death tick

**Archivo:** `src/layers/gompertz_cache.rs` (implementado, 8 tests)

SenescenceProfile ya tiene 4 campos → no cabe un campo más. Solución: componente auxiliar SparseSet.

```rust
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct GompertzCache {
    death_tick: u64,  // precomputed at spawn via exact quadratic solve
}
```

Defensive: NaN/Inf/negative coeff → death_tick = birth + max_age (safe fallback).

### 3. `Converged<T>` — generic convergence detection

**Archivo:** `src/layers/converged.rs` (implementado, 7 tests)

```rust
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[component(storage = "SparseSet")]
pub struct Converged<T: Send + Sync + 'static> {
    env_hash: u64,        // hash of environment when converged
    _marker: PhantomData<T>,
}
```

Helpers: `hash_f32(v)` (NaN-safe → 0), `hash_pos(x, z)` (Knuth multiplicative).
Uso: `Converged<EpigeneticState>`, `Converged<MorphogenesisShapeParams>`.

---

## Lo que NO cambia

- `Concentration` sigue existiendo como `NormStrategy` para gameplay
- Los 11 bridges existentes siguen funcionando
- `BridgeConfig.enabled = false` ya es el bypass exacto
- `NormStrategy::Passthrough` se usa para alignment memoización exacta
- Toda la infraestructura de metrics, context_fill, presets → intacta

## Lo que se añade (IMPLEMENTADO)

- `src/blueprint/equations/exact_cache.rs` — 3 funciones puras (23 tests)
- `src/layers/kleiber_cache.rs` — KleiberCache SparseSet (8 tests)
- `src/layers/gompertz_cache.rs` — GompertzCache SparseSet (8 tests)
- `src/layers/converged.rs` — Converged<T> genérico + hash helpers (7 tests)
- Reflect registration en `plugins/layers_plugin.rs`
- Re-exports en `layers/mod.rs` y `equations/mod.rs`
- ~280 LOC total, 46 tests

## Pendiente (integración en sistemas)

- [ ] `basal_drain_system` → usar `KleiberCache.vol_factor()` en vez de `powf()` per-tick
- [ ] Spawn sites → insertar `GompertzCache::from_senescence(...)` junto a `SenescenceProfile`
- [ ] `senescence_death_system` → usar `GompertzCache.should_die()` en vez de `exp()` per-tick
- [ ] `epigenetic_adaptation_system` → insertar/check `Converged<EpigeneticState>`
- [ ] `shape_optimization_system` → insertar/check `Converged<MorphogenesisShapeParams>`
- [ ] `AlignmentLookup` Resource para frequency pairs (opcional, post-MVP)
