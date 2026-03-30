# ~~Diseño de Cache por Capa — Estrategias Metafísicas~~ SUPERSEDED

> **SUPERSEDED by [CACHE_STRATEGY_DESIGN_V2.md](./CACHE_STRATEGY_DESIGN_V2.md)**
> V2 eliminó todas las estrategias que pierden precisión. Solo sobreviven patrones
> de zero precision loss (memoización exacta, dirty flags, precompute, lookup tables).
>
> Este documento se mantiene como referencia de la investigación de física real
> (Kleiber CV, Gompertz actuarial, astrofísica multigrupo, etc.) pero las
> recomendaciones de Concentration/TemporalWindow/FrequencyAligned ya no aplican.

---

> Cada proceso físico tiene propiedades estadísticas inherentes que determinan
> cuándo es seguro reusar un valor cacheado sin perder precisión ni violar axiomas.
> Este documento define 3 variantes de `NormPipeline` por bloque cacheable,
> fundamentadas en el comportamiento real del fenómeno.

---

## Principio rector

```
No cacheamos "para ir más rápido".
Cacheamos porque la FÍSICA del proceso nos dice que el valor NO CAMBIÓ.
```

La estrategia de cache refleja la **metafísica** del proceso:
- Procesos estables → cache largo, bandas anchas
- Procesos oscilatorios → cache corto, bandas finas
- Procesos con transiciones discretas → cache por estado, invalidar en transición

---

## Taxonomía de estabilidad temporal

| Tier | Nombre | Propiedad | TTL | Ejemplos |
|------|--------|-----------|-----|----------|
| **T0** | Constante | Output fijo para inputs fijos | ∞ | Thresholds derivados, alignment lookup |
| **T1** | Convergente | Output converge y se estabiliza | Hasta perturbación | Kleiber drain, epigenética, fineness |
| **T2** | Monótono | Output cambia en una dirección predecible | Ventana de edad/tick | Gompertz hazard, senescence |
| **T3** | Reactivo | Output cambia cuando el entorno cambia | Hasta dirty flag | Osmosis, radiation pressure |
| **T4** | Oscilatorio | Output oscila periódicamente | < 1 periodo de beat | Interferencia, coherencia |

---

## BLOQUE 1: Basal Drain (Kleiber)

### Metafísica del proceso

La ley de Kleiber (`metabolismo ∝ masa^0.75`) es una de las constantes biológicas más estables conocidas. En organismos reales, la tasa metabólica basal tiene un **coeficiente de variación intra-individuo del 1.5-3%** medido en 14 días consecutivos. Solo el 2% de la variabilidad observada es atribuible a cambios dentro del mismo sujeto.

La función `radius^0.75` es **sublineal y compresiva**: un error del 1% en radius produce solo 0.75% de error en el output. Entidades más grandes son **más cacheables** porque la derivada `0.75 × r^(-0.25)` decrece con el radio.

El `age_factor = 1 + coeff × age` es **lineal y monótono** — crece 1 unidad cada `1/coeff` ticks. Para fauna (coeff=0.02), cambia ~2% cada 100 ticks.

**Tier: T1 (Convergente)** — el radius cambia solo por growth (lento). El age_factor es predecible.

### 3 Variantes de NormPipeline

#### Variante A: `KLEIBER_STABLE` (conservadora, máxima precisión)

```rust
// Bandas de radius logarítmicas (16 bins), age quantizado a 50 ticks
NormPipeline::single(NormStrategy::Concentration)
// Bandas: [0.01, 0.1), [0.1, 0.3), [0.3, 0.5), [0.5, 0.8), [0.8, 1.2),
//         [1.2, 1.8), [1.8, 2.5), [2.5, 3.5), [3.5, 5.0), [5.0, 7.0),
//         [7.0, 10.0), [10.0, 15.0), [15.0, 25.0), [25.0, 50.0), [50.0, 100.0]
// Hysteresis: 5% del ancho de banda
// Age window: 50 ticks (error < 0.1% para fauna)
```

- **Hit rate esperado:** 92%
- **Error máximo:** 1.5% (dentro del CV biológico real)
- **Cuándo usar:** Default. Entidades en crecimiento activo.

#### Variante B: `KLEIBER_ALLOMETRIC` (optimizada, bins por escala de masa)

```rust
// Bins alineados a órdenes de magnitud (como en ecología: log-mass bins)
// Age quantizado a 100 ticks (error < 0.2%)
NormPipeline::passthrough()
    .then(NormStrategy::TemporalWindow)   // age → 100-tick windows
    .then(NormStrategy::Concentration)     // radius → log-scale bands
// Bandas: [0.01, 0.1), [0.1, 1.0), [1.0, 10.0), [10.0, 100.0]
// 4 bandas anchas — aprovecha que Kleiber comprime error
// Hysteresis: 10% (alto — radio cambia muy lento)
```

- **Hit rate esperado:** 97%
- **Error máximo:** 3.8% (aceptable: la ley de Kleiber real tiene ±5% inter-especie)
- **Cuándo usar:** Mundos grandes (>1000 entidades). Entidades adultas estables.

#### Variante C: `KLEIBER_PRECOMPUTED` (lookup table, zero runtime)

```rust
// Pre-computar radius^0.75 para 256 valores de radius en [0.01, 100.0]
// Interpolar linealmente entre bins
// Age factor computado aparte (multiplicación trivial)
NormPipeline::single(NormStrategy::Passthrough)
// NO usa bridge cache — usa lookup table estática
// Equivalente a BridgeConfig { enabled: false } + tabla estática
```

- **Hit rate:** 100% (no hay cache miss — lookup directo)
- **Error máximo:** 0.4% (interpolación lineal en tabla de 256 entries)
- **Cuándo usar:** Máximo rendimiento. No necesita bridge, solo tabla.
- **Costo:** 1 KB de memoria (256 × f32)

### Justificación axiomática

- **Axiom 4 (Dissipation):** Cache no omite drain — solo reutiliza el valor calculado.
- **Axiom 5 (Conservation):** El drain se aplica íntegro; la cuantización afecta qué valor, no si se aplica.
- **Invariante:** `Σ drain(cached) ≈ Σ drain(exact) ± ε` donde `ε < 4%` (menor que la varianza biológica real).

---

## BLOQUE 2: Senescence / Gompertz

### Metafísica del proceso

La mortalidad Gompertz es **determinista y monótona**: dado `(birth_tick, coeff)`, el tick de muerte es **calculable en spawn-time**. En actuaría real, las tablas de mortalidad usan **grupos de 5 años** con hazard piecewise-constant, y esto es suficiente para todo cálculo práctico de seguros/pensiones.

La probabilidad de supervivencia `S(t) = exp(-base×t - 0.5×coeff×t²)` tiene una derivada `dS/dt = -(base + coeff×t) × S(t)` que es el **hazard de Gompertz**. El hazard se duplica cada `ln(2)/coeff` ticks — esto define la resolución temporal mínima.

**Tier: T2 (Monótono)** — solo va en una dirección. Precomputable.

### 3 Variantes de NormPipeline

#### Variante A: `GOMPERTZ_TICK_WINDOW` (age buckets, per-tick eval)

```rust
// Age quantizado a ventanas de 50 ticks
// Coeff es constante por tipo de materia (3 valores: 0.005, 0.0125, 0.02)
NormPipeline::passthrough()
    .then(NormStrategy::TemporalWindow)  // age → 50-tick windows
// Bandas de age: [0,50), [50,100), [100,150), [150,200]
// Hysteresis: 0 (age solo crece)
```

- **Hit rate esperado:** 95% (50 ticks seguidos retornan el mismo valor)
- **Error máximo:** `|S(t) - S(t+50)| ≈ coeff × 50 × S(t)` → para fauna: ~1.0 por window
- **Cuándo usar:** Default. Balance entre precisión y rendimiento.

#### Variante B: `GOMPERTZ_PRECOMPUTED_DEATH` (calcular tick de muerte en spawn)

```rust
// En spawn: death_tick = solve(S(t) = exp(-2)) → t = √(4/coeff - 2×base/coeff²)
// Almacenar death_tick como campo de SenescenceProfile
// Per-tick: solo comparar tick_id >= death_tick (zero math)
NormPipeline::single(NormStrategy::Passthrough)
// NO usa bridge — muerte es lookup de un u64
```

- **Hit rate:** N/A (no hay cache — comparación directa)
- **Error máximo:** 0 (determinista, exacto)
- **Cuándo usar:** Máximo rendimiento. Muerte es evento binario.
- **Costo:** 8 bytes extra por entidad (u64 death_tick)

#### Variante C: `GOMPERTZ_LOOKUP_TABLE` (tabla 2D pre-computada)

```rust
// Tabla: survival_probability[age_bin][coeff_tier]
// age_bin: 0..200 (granularidad 1 tick)
// coeff_tier: 3 valores (materialized, flora, fauna)
// Total: 200 × 3 = 600 entries (2.4 KB)
NormPipeline::single(NormStrategy::Passthrough)
// Bridge deshabilitado — lookup directo en tabla estática
```

- **Hit rate:** 100% (lookup directo)
- **Error máximo:** 0 (tabla exacta para los 3 coeff tiers)
- **Cuándo usar:** Cuando necesitas survival_probability (no solo muerte sí/no)
- **Costo:** 2.4 KB

### Justificación axiomática

- **Axiom 4:** La mortalidad ES dissipation — cachearla no la elimina.
- **Axiom 5:** La muerte libera energía al grid (nutrient_return). Cache no altera esto.
- **Invariante:** `death_tick(cached) == death_tick(exact)` — muerte es evento discreto, no continuo.

---

## BLOQUE 3: Frequency Alignment (Gaussiana)

### Metafísica del proceso

El alignment `exp(-Δf²/5000)` es una **Gaussiana en frecuencia**, no en tiempo. Las frecuencias de las entidades cambian **raramente** (solo por homeostasis o entrainment, procesos lentos). En astrofísica, los métodos multigrupo de transferencia radiativa usan **4-20 bins de frecuencia** y capturan la física esencial. Más de 20 bins no mejora la precisión.

La Gaussiana es **simétrica** (`alignment(a,b) = alignment(b,a)`) y tiene **sensibilidad máxima** en `Δf = σ` (el punto de inflexión). Lejos del centro (`Δf > 2σ = 100 Hz`), el alignment cae a <2% — transferencia cruzada despreciable.

**Tier: T0/T1** — inputs (frecuencias) cambian lento. Output computable como lookup estático.

### 3 Variantes de NormPipeline

#### Variante A: `ALIGNMENT_BANDED` (bins de 10 Hz)

```rust
// Frecuencias cuantizadas a bins de 10 Hz (5 bins por bandwidth)
// 100 bins para rango [0, 1000 Hz]
// Lookup table simétrica: 100 × 100 / 2 = 5000 entries
NormPipeline::single(NormStrategy::Concentration)
// Bandas: [0,10), [10,20), ..., [990,1000]
// Hysteresis: 2 Hz (frecuencias estables)
```

- **Hit rate esperado:** 88%
- **Error máximo:** 2% (bin de 10 Hz dentro de σ=50 Hz)
- **Cuándo usar:** Default. Balance óptimo.

#### Variante B: `ALIGNMENT_ELEMENTAL` (canonical por elemento)

```rust
// Frecuencias snapeadas al canonical del ElementDef más cercano
// ~6-8 elementos × 6-8 = 36-64 pares únicos
NormPipeline::single(NormStrategy::FrequencyAligned)
// Usa AlchemicalAlmanac para resolver canonical
// Zero bandas — lookup directo en tabla de elementos
```

- **Hit rate esperado:** 99% (solo 36-64 pares posibles)
- **Error máximo:** depende de la distancia al canonical
  - Dentro de una banda elemental (±25 Hz): error < 5%
  - Entre bandas: alignment ya < 10%, error irrelevante
- **Cuándo usar:** Mundos con biomas claros (frecuencias agrupadas por elemento)

#### Variante C: `ALIGNMENT_LOOKUP_2D` (tabla estática, zero runtime)

```rust
// Pre-computar alignment para TODA combinación de bandas elementales
// Tabla: alignment[element_a][element_b] (simétrica)
// 8 × 8 / 2 = 36 entries (144 bytes)
// Alternativamente: 20 × 20 = 200 entries (800 bytes) para bins de 50 Hz
NormPipeline::single(NormStrategy::Passthrough)
// Bridge deshabilitado — lookup estático O(1) en lugar de exp()
```

- **Hit rate:** 100% (no hay cache — lookup directo)
- **Error máximo:** 0 si bins = bandas elementales; <1% si bins = 50 Hz
- **Cuándo usar:** Máximo rendimiento. Elimina toda llamada a `exp()`.
- **Costo:** 800 bytes (tabla 20×20)

### Justificación axiomática

- **Axiom 8 (Oscillatory Nature):** La cuantización por bandas elementales es **coherente** con el diseño del juego (bandas de 50 Hz).
- **Axiom 7 (Distance Attenuation):** El alignment gaussiano ES atenuación por distancia en frecuencia. Cache respeta la monotonía.
- **Invariante:** `alignment(cached) ∈ [alignment(exact) × 0.95, alignment(exact) × 1.05]` — error < 5% para bins de 10 Hz.

---

## BLOQUE 4: Radiation Pressure Transfer

### Metafísica del proceso

La presión de radiación es **localmente coherente, globalmente atenuada**. En astrofísica, los métodos multigrupo resuelven transferencia radiativa con **4-22 bins de frecuencia** — más bins no mejoran significativamente. El resultado clave: "el rate de transferencia no mejora con más de 20 bins; es más efectivo optimizar la ubicación de menos bins".

El transfer `= rate × excess × alignment / n_neighbors` tiene dos componentes:
- `excess = max(qe - threshold, 0)` — cambia cada tick (fast)
- `alignment` — cambia raramente (slow, Bloque 3)

La metafísica es: **separar lo que cambia rápido de lo que cambia lento**.

**Tier: T3 (Reactivo)** — recomputa cuando la celda cambia, pero el alignment es T0.

### 3 Variantes de NormPipeline

#### Variante A: `PRESSURE_SPLIT` (alignment cacheado + excess inline)

```rust
// Cache solo la parte cara (alignment gaussiano) con ALIGNMENT_BANDED
// Excess se computa inline (resta trivial)
// Transfer final = cached_alignment × (qe - threshold).max(0) × rate / n
NormPipeline::single(NormStrategy::Concentration)
// Bandas: frecuencia en bins de 10 Hz (como ALIGNMENT_BANDED)
// Solo cachea el alignment — la multiplicación final es O(1)
```

- **Hit rate esperado:** 88% (en alignment; excess siempre se computa)
- **Error máximo:** 2% (propagado desde alignment)
- **Cuándo usar:** Default. Separa el cuello de botella (exp()) del trivial (multiplicación).

#### Variante B: `PRESSURE_DIRTY_FLAG` (skip celdas estables)

```rust
// Si una celda no cambió qe en el último tick, skip el cálculo completo
// Dirty flag: set cuando qe cambia > 1% respecto al tick anterior
// Alignment pre-computado en tabla estática (ALIGNMENT_LOOKUP_2D)
NormPipeline::passthrough()
    .then(NormStrategy::Concentration)  // qe → bandas de energía
// Dirty flag es external al bridge — sistema lo checkea antes de llamar
```

- **Hit rate esperado:** 70-95% (depende de la actividad del grid)
  - Grid estable (post-diffusion): 95%
  - Grid activo (near nucleus): 70%
- **Error máximo:** 0% (dirty flag = recompute exact; no cached = no error)
- **Cuándo usar:** Grids grandes con zonas estables (la mayoría de celdas no cambian).

#### Variante C: `PRESSURE_MULTIGROUP` (inspired por astrofísica)

```rust
// Agrupar celdas por "grupo de opacidad" (frecuencia + densidad)
// 4-6 grupos, cada uno con alignment y rate pre-promediados
// Transfer por grupo, no por celda individual
NormPipeline::passthrough()
    .then(NormStrategy::FrequencyAligned)  // freq → elemento canonical
    .then(NormStrategy::Concentration)      // qe → bandas de densidad
// Bandas de densidad: [0, solid), [solid, liquid), [liquid, gas), [gas, ∞)
// 4 estados × 6-8 elementos = 24-32 grupos
```

- **Hit rate esperado:** 97% (solo 24-32 grupos únicos)
- **Error máximo:** 15% (promediado intra-grupo, como en astrofísica multigrupo)
- **Cuándo usar:** Simulaciones masivas donde precisión per-celda no importa.
- **Tradeoff:** Pierde detalle intra-bioma a cambio de 10× menos cómputo.

### Justificación axiomática

- **Axiom 2 (Pool Invariant):** Transfer total conservado — error en distribución, no en suma.
- **Axiom 8:** Alignment gaussiano respetado (misma Gaussiana, solo cuantizada).
- **Invariante:** `Σ transfer_out(cell) == Σ transfer_in(neighbors) ± ε_grid` — conservación por double-buffer.

---

## BLOQUE 5: Osmotic Diffusion (Fick)

### Metafísica del proceso

La difusión tiene un **timescale de relajación** `τ = L²/(π²D)`. Perturbaciones decaen como `exp(-t/τ)`. Para el grid de Resonance con `D = DISSIPATION_LIQUID = 0.02`, la relajación por celda es ~5 ticks. Esto significa que después de 5 ticks sin input externo, una región local alcanza near-equilibrium.

La ecuación de Fick es **lineal en el gradiente de concentración**: `flux = D × (c_a - c_b)`. Esto es self-correcting: si over-transferes un tick, el gradiente se ajusta y el siguiente tick compensa.

**Tier: T3 (Reactivo)** — recomputa cuando celdas cambian, pero auto-corrige errores.

### 3 Variantes de NormPipeline

#### Variante A: `FICK_GRADIENT_BANDS` (gradiente cuantizado)

```rust
// Concentraciones cuantizadas a bandas; gradiente = diferencia de canonicals
// Self-correcting: error en un tick se compensa en el siguiente
NormPipeline::single(NormStrategy::Concentration)
// Bandas: [0, 5), [5, 15), [15, 30), [30, 50), [50, 80), [80, 120), [120, ∞)
// 7 bandas — resolución suficiente para difusión (linear flux)
// Hysteresis: 2 qe (evitar jitter en bordes de banda)
```

- **Hit rate esperado:** 82%
- **Error máximo:** error se auto-corrige en τ ≈ 5 ticks
- **Cuándo usar:** Default. La auto-corrección de Fick absorbe el error de cuantización.

#### Variante B: `FICK_EQUILIBRIUM_SKIP` (skip celdas en equilibrio)

```rust
// Si |c_a - c_b| < ε para todos los vecinos, celda está en equilibrio → skip
// Recompute solo cuando algún vecino cambia significativamente
NormPipeline::single(NormStrategy::Passthrough)
// No cachea el cálculo — cachea la DECISIÓN de no calcular
// Dirty-flag per-celda: set cuando |Δqe| > 1.0 respecto al tick anterior
```

- **Hit rate:** 70-95% (equilibrium skip)
  - Grid post-diffusion: 95% de celdas en equilibrio
  - Grid near nucleus: 50-70%
- **Error máximo:** 0% (when computing, computes exact)
- **Cuándo usar:** Grids grandes. Complementa LOD existente.

#### Variante C: `FICK_MULTIGRID` (resolución adaptativa)

```rust
// Coarse grid (4×4 superceldas) para difusión de largo alcance
// Fine grid (1×1) solo near nuclei y entidades
// Inspirado en métodos multigrid de CFD
NormPipeline::passthrough()
    .then(NormStrategy::TemporalWindow)    // skip every 4 ticks for far cells
    .then(NormStrategy::Concentration)      // concentration bands
// Temporal window: 4 ticks para celdas lejanas (LOD=Far)
//                  1 tick para celdas cercanas (LOD=Near)
```

- **Hit rate esperado:** 90% (superceldas lejanas se actualizan cada 4 ticks)
- **Error máximo:** 8% (multigrid error, aceptable para background diffusion)
- **Cuándo usar:** Grids >64×64 donde la difusión lejana no necesita resolución per-tick.

### Justificación axiomática

- **Axiom 4 (Dissipation):** La difusión ES dissipation. Cache no la elimina.
- **Axiom 5 (Conservation):** `Σ flux_out = Σ flux_in` por double-buffer. Cuantización no rompe conservación.
- **Invariante:** Equilibrium convergence: `|c(t) - c_eq| → 0` con o sin cache. Cache solo afecta la velocidad de convergencia.

---

## BLOQUE 6: Epigenetic Adaptation

### Metafísica del proceso

Las modificaciones epigenéticas operan en **dos escalas temporales reales**:
- **Rápida (histonas):** 15-30 minutos. Acetilación/deacetilación reactiva.
- **Lenta (metilación DNA):** Días a meses. Una vez establecida, estable por la vida del organismo.

En el modelo de Resonance, `expression = current + (target - current) × rate × dt` es un **EMA (exponential moving average)** con τ = 1/(rate × dt). Para rate=0.5, dt=0.016: τ ≈ 125 ticks. Después de 5τ (~625 ticks), la expresión está al 99% del target.

La función `should_express_gene(env × current > threshold)` es **binaria** — solo importa si cruza el threshold, no el valor exacto.

**Tier: T1 (Convergente)** — converge y se estabiliza. Perturbación = mover a otra celda.

### 3 Variantes de NormPipeline

#### Variante A: `EPIGENETIC_CONVERGENCE_DETECT` (skip post-convergence)

```rust
// Si |target - current| < 1e-3 para todas las dimensiones → converged
// Skip until entity moves to new cell (env_signal changes)
NormPipeline::single(NormStrategy::Concentration)
// Bandas de env_signal: [0, 0.2), [0.2, 0.4), [0.4, 0.6), [0.6, 0.8), [0.8, 1.0]
// 5 bandas — matchean los 5 thresholds (0.5, 0.6, 0.7, 0.8)
// Hysteresis: 0.05 (estabilidad de expresión)
```

- **Hit rate esperado:** 85% (entidades estáticas dominan)
- **Error máximo:** 0.1% (convergencia a target ±0.001)
- **Cuándo usar:** Default. Respeta las dos escalas temporales biológicas.

#### Variante B: `EPIGENETIC_BOOLEAN` (expresión binaria, como en biología computacional)

```rust
// Discretizar expression_mask a {0.0, 1.0} (expressed/silenced)
// Solo 2^4 = 16 estados posibles para el mask completo
// Recompute solo cuando env_signal cruza un threshold
NormPipeline::passthrough()
    .then(NormStrategy::Concentration)  // env_signal → 5 bandas
// Bandas con hysteresis alto (0.1) para evitar flip-flop near threshold
// Cache key: (env_band, current_binary_mask) → 5 × 16 = 80 entries
```

- **Hit rate esperado:** 98% (solo 80 combinaciones posibles)
- **Error máximo:** conceptual — pierde gradiente continuo
- **Cuándo usar:** Rendimiento extremo. Válido porque en biología computacional, redes regulatorias se modelan como Booleanas y capturan la dinámica esencial.

#### Variante C: `EPIGENETIC_WADDINGTON` (landscape de atractores)

```rust
// Modelar expresión como paisaje de Waddington con 3-4 estados atractores
// En vez de EMA continuo, snap a attractor más cercano cuando converge
// Transición entre atractores solo cuando env_signal cruza barrera
NormPipeline::passthrough()
    .then(NormStrategy::Concentration)
// Bandas de expression: [0, 0.25) → attractor 0.0 (silenced)
//                       [0.25, 0.75) → attractor 0.5 (intermediate)
//                       [0.75, 1.0] → attractor 1.0 (expressed)
// 3 attractors × 4 dims × 5 env bands = 60 × 3 = 180 entries
// Hysteresis: 0.15 (barrera de transición entre atractores)
```

- **Hit rate esperado:** 95%
- **Error máximo:** cuantización a 3 niveles (0, 0.5, 1.0)
- **Cuándo usar:** Balance entre Booleano y continuo. Refleja el modelo real de Waddington.
- **Ventaja metafísica:** Las células reales no usan gradiente continuo — tienen estados estables discretos con barreras energéticas entre ellos.

### Justificación axiomática

- **Axiom 6 (Emergence at Scale):** Epigenética ES emergencia (env → fenotipo). Cache no hardcodea el resultado.
- **Axiom 1 (Everything is Energy):** El costo metabólico de silencing se paga independientemente del cache.
- **Invariante:** `Σ silencing_cost(cached) == Σ silencing_cost(exact)` — el costo es función del mask, no de cómo se calculó.

---

## BLOQUE 7: Awakening Potential

### Metafísica del proceso

El awakening `potential = (coherence - dissipation) / (coherence + qe)` es una **fracción de eficiencia** que mide cuánta coherencia neta hay relativa a la energía total. Conceptualmente es como el **número de Reynolds** en fluidos: un ratio adimensional que determina el régimen (laminar/turbulento).

El threshold es **1/3** — algebraic break-even exacto. Es un **evento raro**: solo ~0.5 entidades/tick despiertan (budget = 4 cada 8 ticks). La mayoría de entidades están lejos del threshold.

**Tier: T3/T1 hybrid** — coherence fluctúa (T3) pero el threshold es raro (T1 entre eventos).

### 3 Variantes de NormPipeline

#### Variante A: `AWAKENING_THRESHOLD_PROXIMITY` (priorizar entidades near-threshold)

```rust
// Bandas de potential centradas en el threshold (1/3)
// Alta resolución near threshold, baja resolución lejos
NormPipeline::single(NormStrategy::Concentration)
// Bandas: [0, 0.1), [0.1, 0.25), [0.25, 0.30), [0.30, 0.34), [0.34, 0.40),
//         [0.40, 0.60), [0.60, 1.0]
// 7 bandas — 3 de ellas concentradas en [0.25, 0.40] (zona de decisión)
// Hysteresis: 0.02 near threshold, 0.05 lejos
```

- **Hit rate esperado:** 78%
- **Error máximo:** 2% near threshold (zona de decisión tiene bandas finas)
- **Cuándo usar:** Default. Resolución adaptativa donde importa.

#### Variante B: `AWAKENING_SCAN_BUDGET` (skip entidades lejanas al threshold)

```rust
// Pre-filtro: si potential < 0.2 → skip (no recalcular hasta que qe suba >20%)
// Si potential > 0.5 → ya despertó (skip)
// Solo recalcular entidades en [0.2, 0.5]
NormPipeline::passthrough()
    .then(NormStrategy::Concentration)
// 3 macro-bandas: [0, 0.2) = DORMANT, [0.2, 0.5) = CANDIDATE, [0.5, 1.0] = AWAKE
// Solo CANDIDATE se recalcula; DORMANT y AWAKE son cache hits perpetuos
```

- **Hit rate esperado:** 92% (solo 8% de entidades son candidatas en un tick típico)
- **Error máximo:** 0% para AWAKE/DORMANT; 3% para CANDIDATE
- **Cuándo usar:** Mundos con muchas entidades inert (mayoría están lejos del threshold).

#### Variante C: `AWAKENING_EVENT_DRIVEN` (recompute solo en cambio de vecindario)

```rust
// Coherence_gain depende de vecinos. Solo recalcular cuando:
//   1. Un vecino spawna/despawna
//   2. Un vecino cambia de frecuencia (homeostasis/entrainment)
//   3. La entidad se mueve a otra celda
// Dirty flag por entidad, set por los 3 eventos
NormPipeline::single(NormStrategy::Passthrough)
// Sin normalización — compute exact solo cuando dirty
// Flag reset después de cada cómputo
```

- **Hit rate:** 85-95% (la mayoría de entidades no cambian vecindario per-tick)
- **Error máximo:** 0% (compute exact when dirty)
- **Cuándo usar:** Mundos estables post-abiogenesis. Event-driven = zero overhead para entidades estáticas.

### Justificación axiomática

- **Axiom 8 (Oscillatory Nature):** El cómputo de coherence usa alignment gaussiano — cacheable por Bloque 3.
- **Axiom 6 (Emergence):** Awakening ES emergencia. El threshold no es arbitrario (1/3 es algebraic break-even).
- **Invariante:** Ninguna entidad despierta que no debería, ni se pierde una que debería.
  El pre-filtro solo afecta CUÁNDO se evalúa, no SI se evalúa.

---

## BLOQUE 8: Bounded Fineness Descent (Constructal)

### Metafísica del proceso

La optimización constructal (Adrian Bejan, 1996) dice que los sistemas que fluyen evolucionan hacia configuraciones que **minimizan la resistencia**. El descenso de gradiente en fineness converge en ~10-20 iteraciones para la mayoría de entidades, y después **se queda** porque los inputs (densidad del medio, velocidad) cambian lento.

La ecuación `drag = 0.5 × ρ × v² × C_D × A` tiene dependencia **cuadrática** en velocidad — el cambio dominante. Pero la velocidad de una entidad cambia suavemente (inercia). El costo vascular `μL³/r⁴` es extremadamente no-lineal (cuártico en radio) pero estable (la anatomía no cambia per-tick).

**Tier: T1 (Convergente)** — converge rápido, se queda estable.

### 3 Variantes de NormPipeline

#### Variante A: `CONSTRUCTAL_VELOCITY_BANDS` (bandas de velocidad)

```rust
// Velocidad cuantizada a bandas (input dominante para drag)
// Densidad del medio cuantizada a 4 estados de materia
NormPipeline::single(NormStrategy::Concentration)
// Bandas de velocidad: [0, 0.1), [0.1, 0.5), [0.5, 1.5), [1.5, 4.0), [4.0, 10.0), [10.0, ∞)
// 6 bandas — velocidad cuadrática, así que más resolución en valores bajos
// Densidad: 4 bandas (Solid/Liquid/Gas/Plasma — thresholds derivados)
// Hysteresis: 0.1 (velocidad fluctúa; evitar jitter)
```

- **Hit rate esperado:** 85%
- **Error máximo:** 5% en drag (cuadrático en v; ±10% v → ±21% drag)
- **Cuándo usar:** Default. El descent converge rápido y el cache estabiliza.

#### Variante B: `CONSTRUCTAL_CONVERGED_SKIP` (skip post-convergencia)

```rust
// Si |fineness_new - fineness_old| < 0.01 en el último update → converged
// Skip hasta que velocity o medium_density cambie >20%
NormPipeline::passthrough()
    .then(NormStrategy::Concentration)  // velocity → 6 bandas
// Cache key incluye convergence flag: converged entities ALWAYS hit
// Re-evaluate cada 50 ticks como safety net
```

- **Hit rate esperado:** 93% (la mayoría de entidades convergen en 10-20 ticks)
- **Error máximo:** 0% (only skips when provably converged)
- **Cuándo usar:** Entidades adultas. Fineness se estabiliza después del período de crecimiento.

#### Variante C: `CONSTRUCTAL_PRECOMPUTED_OPTIMA` (tabla de óptimos por régimen)

```rust
// Pre-computar fineness_optimo para cada combinación (velocity_band, matter_state)
// 6 × 4 = 24 entries — tabla estática
// Entidades nuevas arrancan en el óptimo de su régimen, no en descent random
NormPipeline::single(NormStrategy::Passthrough)
// Bridge deshabilitado — lookup directo
// Tabla: optimal_fineness[velocity_band][matter_state] → f32
```

- **Hit rate:** 100% (lookup)
- **Error máximo:** 8% (pierde granularidad intra-banda en vascular_cost)
- **Cuándo usar:** Spawn rápido + rendimiento. Skip el descent completo.
- **Costo:** 96 bytes (24 × f32)

### Justificación axiomática

- **Axiom 4 (Dissipation):** El drag IS dissipation. La forma óptima minimiza dissipation — coherente con la 2nd Law.
- **Axiom 6 (Emergence):** La forma emerge del balance drag/vascular, no de un template.
- **Invariante:** `fineness ∈ [FINENESS_MIN, FINENESS_MAX]` siempre. Cache no puede producir valores fuera de rango.

---

## Resumen: Mapa de Estrategias

| Bloque | Variante A (Default) | Variante B (Optimizada) | Variante C (Máxima) |
|--------|---------------------|------------------------|---------------------|
| **Kleiber** | Concentration, 16 log-bins, 92% | TemporalWindow + Concentration, 97% | Lookup table 256 entries, 100% |
| **Gompertz** | TemporalWindow 50-tick, 95% | Precomputed death_tick, N/A | Lookup table 600 entries, 100% |
| **Alignment** | Concentration 10 Hz bins, 88% | FrequencyAligned (Almanac), 99% | Lookup 2D 20×20, 100% |
| **Rad Pressure** | Split (alignment cached), 88% | Dirty flag, 70-95% | Multigroup 4-6 bins, 97% |
| **Osmosis** | Concentration 7 bands, 82% | Equilibrium skip, 70-95% | Multigrid + TemporalWindow, 90% |
| **Epigenetic** | Convergence detect, 85% | Boolean mask (16 states), 98% | Waddington attractors (3 levels), 95% |
| **Awakening** | Threshold proximity bands, 78% | Scan budget (3 zones), 92% | Event-driven dirty flag, 85-95% |
| **Constructal** | Velocity bands, 85% | Converged skip, 93% | Lookup 24 optima, 100% |

### Patrón recomendado por fase del juego

| Fase | Estrategia | Razón |
|------|-----------|-------|
| **Warmup (0-100 ticks)** | Variante A para todo | Calibrar caches con datos reales |
| **Growth (100-500 ticks)** | Variante A + B para Kleiber/Epigenetic | Entidades creciendo, radius inestable |
| **Stable (500+ ticks)** | Variante B/C para todo | Mundo estabilizado, maximizar hits |
| **Crisis (nucleus depletion)** | Variante A para Osmosis/Pressure | Grid en flux, necesita recalcular |

---

## Implementación

Cada variante se materializa como:

1. Un `NormPipeline` const (stack-allocated, Copy)
2. Un set de `BandDef` calibradas al proceso
3. Un `Rigidity` preset (Rigid/Moderate/Flexible)
4. Opcionalmente, una tabla estática (`&[f32]` o `[[f32; N]; M]`)

Las tablas estáticas son **complementarias** al bridge cache:
- Bridge cache: per-tick deduplication con normalización
- Lookup table: eliminación total de `exp()`, `powf()`, `sqrt()`
- Ambos coexisten: el bridge usa la tabla como backend de `compute()`

```rust
// Ejemplo: alignment usa lookup table DENTRO del Bridgeable::compute
impl Bridgeable for RadiationPressureAlignmentBridge {
    fn compute(n: Self::Input) -> Self::Output {
        // En vez de: exp(-delta_f^2 / 5000.0)
        // Usa: ALIGNMENT_TABLE[freq_band_a][freq_band_b]
        ALIGNMENT_TABLE[n.f_center_band as usize][n.f_cell_band as usize]
    }
}
```

Esto combina **normalización del bridge** (reduce inputs a bandas) con **lookup estático** (elimina math cara) para el máximo beneficio sin perder la arquitectura.
