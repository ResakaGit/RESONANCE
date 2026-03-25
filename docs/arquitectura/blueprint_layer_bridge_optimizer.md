# Blueprint: Layer Bridge Optimizer — Cuantización Estable + Cache por Capa

> **Principio**: Un material mantiene su composición atómica porque le es menos costoso que perderla.
> Los resultados de las matemáticas entre capas TIENDEN a estar en estados estables.
> Computar lo estable es redundante. Computar lo raro es necesario.

---

## 1. El Problema

Cada tick, el pipeline ejecuta ~10 ecuaciones por cada entidad, muchas de las cuales producen
resultados idénticos o casi idénticos al tick anterior:

| Ecuación | Frecuencia de cambio real | Ticks redundantes estimados |
|----------|--------------------------|----------------------------|
| `density = qe / volume` | Solo cambia si qe o radius cambian | ~90% redundante |
| `temperature = density / k` | Derivada de density | ~90% redundante |
| `state_from_temperature` | Solo cambia en transiciones de fase (raras) | ~99% redundante |
| `interference(f1, f2, t)` | Cambia cada tick por `t` | ~0% redundante (pero cuantizable) |
| `drag_force` | Solo si la entidad se mueve | ~70% redundante |
| `collision_transfer` | Solo durante contacto | ~80% redundante |
| `catalysis_result` | Solo durante impacto de hechizo | ~95% redundante |

**Observación**: la mayoría de entidades están en equilibrio la mayor parte del tiempo.
Un árbol es sólido. Un río es líquido. La lava es plasma. Los estados cambian raramente.

**Oportunidad**: si podemos detectar "nada cambió significativamente" → devolver el resultado anterior sin computar.

---

## 2. Dos Pilares: Normalización + Cache

### Pilar 1: Normalización por Bandas (Cuantización)

Los inputs continuos (f32) se **cuantizan** a valores canónicos dentro de bandas configurables.

**Antes** (continuo):
```text
density = 47.3  → temperature = 47.3  → state = Liquid
density = 47.8  → temperature = 47.8  → state = Liquid
density = 48.1  → temperature = 48.1  → state = Liquid
```
Tres cómputos distintos. Mismo resultado.

**Después** (cuantizado):
```text
density = 47.3  → normalize → 50.0  → cache HIT → Liquid
density = 47.8  → normalize → 50.0  → cache HIT → Liquid
density = 48.1  → normalize → 50.0  → cache HIT → Liquid
```
Un cómputo. Tres hits.

### Pilar 2: Cache de Resultados (Memoización Cuantizada)

Con inputs cuantizados, el espacio de entradas posibles se reduce de infinito (f32) a finito (N bandas).
Un cache de N entradas cubre el 100% de los casos posibles.

**Tamaños de cache configurables por capa**:

| Nivel | Entradas | Uso |
|-------|----------|-----|
| Micro | 10-50 | Capas con pocos estados distintos (MatterState: 4 valores) |
| Standard | 100-500 | Capas con variación moderada (density bands) |
| Large | 1,000-10,000 | Capas con variación alta (interference con muchas frecuencias) |
| Context-fill | dinámico | La cache se llena con valores del contexto actual de la partida |

**Context-fill** es la táctica más poderosa: al inicio de la partida (warmup), el pipeline
computa normalmente. Cada resultado se almacena en cache. Después de N ticks, la cache
ya tiene todos los valores que esta partida particular va a necesitar. A partir de ahí,
casi todo es cache hit.

---

## 3. Sesgo de Estabilidad (Stability Bias)

### El principio físico

> Un sistema en equilibrio permanece en equilibrio a menos que una fuerza externa lo perturbe.
> Cambiar de estado cuesta energía. No cambiar es gratis.

Esto se traduce en **histéresis** en la normalización: cerca de un borde de banda,
el valor se sesga hacia el estado actual, no hacia el nuevo.

**Sin sesgo** (oscilación):
```text
tick 1: temp = 29.8  → state_from_temp → Solid   (umbral Liquid = 30.0)
tick 2: temp = 30.1  → state_from_temp → Liquid   ← cambió
tick 3: temp = 29.9  → state_from_temp → Solid    ← cambió otra vez
tick 4: temp = 30.2  → state_from_temp → Liquid   ← oscilación constante
```

**Con sesgo** (estable):
```text
tick 1: temp = 29.8, current = Solid  → normalize con bias → 28.0 → Solid
tick 2: temp = 30.1, current = Solid  → normalize con bias → 28.0 → Solid  ← se queda
tick 3: temp = 29.9, current = Solid  → normalize con bias → 28.0 → Solid  ← se queda
tick 4: temp = 31.5, current = Solid  → normalize con bias → 32.0 → Liquid ← ahora sí
```

El sesgo solo se rompe cuando el valor supera el umbral por un **margen de histéresis** configurable.
Esto es físicamente correcto (la materia resiste el cambio) Y computacionalmente barato.

---

## 4. Diseño del Decorador: BridgeConfig

### Concepto

El optimizer es un **decorador stateless** que envuelve cualquier función de transición entre capas.
No tiene estado propio — la configuración y la cache son recursos externos.

```text
                    ┌──────────────────────────────┐
                    │       BridgeConfig           │
                    │  (normalización + bias)       │
   input f32 ──────▶│                              │
                    │  1. Normalizar input          │
                    │  2. Buscar en cache           │
                    │     HIT → return cached       │
                    │     MISS ↓                    │
                    │  3. Computar con fn original  │──▶ output
                    │  4. Guardar en cache          │
                    │  5. Return                    │
                    └──────────────────────────────┘
```

### Estructura del config

Cada capa tiene su propio `BridgeConfig` con:

- **`bands`**: Lista de bandas de normalización. Cada banda tiene (min, max, canonical_value).
- **`hysteresis_margin`**: Margen extra para cambiar de banda (sesgo de estabilidad).
- **`cache_capacity`**: Cuántas entradas cachear.
- **`cache_policy`**: LRU, LFU, o Context-fill.
- **`precision`**: Cuántos decimales de precisión para la cuantización (afecta el tamaño del espacio).
- **`enabled`**: On/off por capa (para A/B testing de performance).

### Bandas de normalización

Una banda define un rango de valores que se tratan como equivalentes:

```text
BandDef:
  min: f32        — límite inferior del rango
  max: f32        — límite superior del rango
  canonical: f32  — valor representativo (el que se usa para computar/cachear)
  stable: bool    — ¿es una banda de equilibrio? (afecta el sesgo)
```

**Ejemplo para temperatura → MatterState**:

```text
Banda 0:  [0, 25)      → canonical 15.0   → Solid    (stable: true)
Banda 1:  [25, 30)     → canonical 27.0   → Solid    (stable: true, zona de transición)
Banda 2:  [30, 90)     → canonical 60.0   → Liquid   (stable: true)
Banda 3:  [90, 100)    → canonical 95.0   → Liquid   (stable: true, zona de transición)
Banda 4:  [100, 280)   → canonical 180.0  → Gas      (stable: true)
Banda 5:  [280, 300)   → canonical 290.0  → Gas      (stable: true, zona de transición)
Banda 6:  [300, ∞)     → canonical 400.0  → Plasma   (stable: true)
```

Con histéresis de 5.0:
- Una entidad en Solid (banda 0-1) no cambia a Liquid hasta que temp > 35.0 (30 + 5).
- Una entidad en Liquid no cambia a Solid hasta que temp < 25.0 (30 - 5).

Esto elimina la oscilación Y reduce los cómputos a 7 valores posibles en vez de infinitos.

---

## 5. Patrón Decorador — Aplicación por Capa

### La abstracción

El decorador no modifica la función original. La envuelve. La función original sigue
existiendo y funciona igual sin el decorador. Esto respeta la separación de responsabilidades
de la arquitectura.

```text
Función original (equations.rs):
  state_from_temperature(temp, bond_energy) → MatterState

Con decorador:
  bridged_state_from_temperature(temp, bond_energy, config, cache) → MatterState
    1. temp_normalized = normalize(temp, config.bands, config.hysteresis)
    2. bond_normalized = normalize(bond_energy, config.bands_bond, config.hysteresis)
    3. key = (temp_normalized, bond_normalized)
    4. if cache.contains(key) → return cache[key]
    5. result = state_from_temperature(temp_normalized, bond_normalized)
    6. cache.insert(key, result)
    7. return result
```

### Trait de decoración

Un trait genérico que cualquier función pura puede implementar. El struct marker que implementa
el trait también sirve como parámetro genérico de `BridgeCache<B>` y `BridgeConfig<B>`,
dándole a cada bridge su propia cache y config como Resources aislados en Bevy:

```text
trait Bridgeable:
  type Input: Copy                                        ← tipo de entrada
  type Output: Copy                                       ← tipo de salida
  fn compute(input: Self::Input) → Self::Output           ← la función original
  fn normalize(input: Self::Input, config: &BridgeConfig<Self>) → Self::Input  ← cuantizar
  fn cache_key(normalized: Self::Input) → u64             ← hash del input normalizado

// La cache se inyecta como BridgeCache<Self> — no hay config_id() ni BridgeId.
// El type system es el routing.
```

### Aplicación sin repetición de código

El decorador se aplica como wrapper genérico. No se reescribe por cada capa.
Una sola implementación genérica cubre. Cada fila es un Resource `BridgeCache<B>` aislado:

| Bridge (marker struct) | Input | Output | Cache propia | Compartida? |
|------------------------|-------|--------|--------------|-------------|
| `DensityBridge` | `(f32, f32)` qe, radius | `f32` | `BridgeCache<DensityBridge>` cap=100 | No |
| `TemperatureBridge` | `f32` density | `f32` | `BridgeCache<TemperatureBridge>` cap=20 | No |
| `PhaseBridge` | `(f32, f32)` temp, bond | `MatterState` | `BridgeCache<PhaseBridge>` cap=70 | No |
| `InterferenceBridge` | `(f32, f32, f32, f32, f32)` | `f32` | `BridgeCache<InterferenceBridge>` cap=500 | No |
| `DragBridge` | `(f32, f32, Vec2)` | `Vec2` | `BridgeCache<DragBridge>` cap=800 | No |
| `CollisionBridge` | `(f32, f32, f32, f32, f32)` | `f32` | `BridgeCache<CollisionBridge>` cap=1000 | No |
| `CatalysisBridge` | `(f32, f32, f32)` | `f32` | `BridgeCache<CatalysisBridge>` cap=1000 | No |

Ninguna ecuación actual se beneficia de compartir cache — los input spaces, normalizaciones
y output types son fundamentalmente distintos. La capacidad de compartir existe por corrección
de diseño (`type Cache = SharedMarker`), no por necesidad actual.

---

## 6. Configuración por Capa — Capas Estrictas vs Relajadas

### El concepto de "rigidez"

Cada capa puede ser más o menos estricta en su normalización:

| Rigidez | Bandas | Precisión | Hit rate | Uso |
|---------|--------|-----------|----------|-----|
| **Rígida** | Pocas, amplias | Baja | ~99% | Transiciones de fase, estados discretos |
| **Moderada** | Medias | Media | ~85% | Densidad, temperatura, arrastre |
| **Flexible** | Muchas, estrechas | Alta | ~60% | Interferencia, catálisis (importa precisión) |
| **Transparente** | Sin normalización | Exacta | 0% (bypass) | Debug, validación, forzar cómputo real |

**Configuración por capa**:

```text
Capa 4 (MatterState):     Rígida    — solo 4 estados posibles, histéresis alta
Capa 3 (drag/dissipation): Moderada  — varía con velocidad pero predeciblemente
Capa 0×1 (density):        Moderada  — cambia gradualmente
Capa 5 (engine intake):    Moderada  — buffer es discreto en la práctica
Capa 2 (interference):     Flexible  — depende del tiempo, más variación
Capa 8 (catalysis):        Flexible  — eventos raros pero importantes
Capa 7 (will force):       Rígida    — input WASD son 8 direcciones discretas
```

### Cambiar rigidez en runtime

La rigidez puede ajustarse dinámicamente:
- **En calma**: subir rigidez (más cache hits, menos cómputos).
- **En combate**: bajar rigidez (más precisión donde importa).
- **Por entidad**: un héroe necesita más precisión que un árbol estático.

Implementar con run conditions o con un multiplicador de precisión en el config.

---

## 7. Context-Fill: Cache que Aprende del Contexto

### La idea

La cache no se pre-llena con valores "teóricos". Se llena sola durante los primeros ticks
de la partida con los valores **reales** que este mapa, estos héroes, estos biomas producen.

### Pipeline de llenado

```text
Fase 1: WARMUP (primeros ~100 ticks)
  - BridgeConfig.enabled = false (bypass, computar todo)
  - Cada resultado se graba en la cache
  - La cache se llena con el contexto real de la partida

Fase 2: RUNTIME (tick 100+)
  - BridgeConfig.enabled = true (normalización + cache activa)
  - La cache ya tiene los valores más comunes
  - Nuevos valores se agregan on-demand (LRU eviction si llena)
```

### Ventaja

No necesitás predecir qué valores serán comunes — el contexto te lo dice.
Un mapa dominado por Terra tendrá frecuencias 50-100 Hz en cache.
Un mapa dominado por Ignis tendrá frecuencias 400-500 Hz.
La cache se adapta automáticamente.

---

## 8. Cache — Descentralizada, Tipada, Inyectable

### Principio: cada bridge posee su cache

La cache NO es un registro global. Cada bridge tiene su propia cache como **generic Resource**
aislado por tipo. Bevy trata `BridgeCache<DensityBridge>` y `BridgeCache<TemperatureBridge>`
como Resources completamente separados — aislamiento por tipo en compile time, zero cost en runtime.

```text
BridgeCache<B: BridgeKind> (Resource) — marcador de tipo; el trait de ecuación es `Bridgeable` en `bridge/decorator.rs`:
  entries: CacheBackend             — Vec lineal o FxHashMap según capacity
  capacity: usize
  hits: u64                         — para métricas
  misses: u64
  policy: EvictionPolicy

Cada bridge B inserta su propia BridgeCache<B> como Resource independiente.
No hay HashMap de routing en runtime. El type system es el routing.
```

### Por qué descentralizada y no registro global

- **Aislamiento garantizado**: una cache nunca busca entre datos de otra ecuación.
  DensityBridge solo ve entradas de density. Nunca hay contaminación cruzada.
- **Zero overhead de routing**: no hay `HashMap<BridgeId, BridgeCache>` lookup en el hot path.
  Bevy resuelve el Resource por tipo en compile time.
- **Configurable por tipo**: cada bridge define su capacity, policy y backend de forma independiente.
- **No es estado de gameplay**: se puede borrar cualquier cache individual sin afectar la simulación.
- **Inyectable**: el SystemParam de cada bridge declara qué cache usa via el tipo genérico.

### Cache compartida (opt-in explícito)

Por defecto, cada bridge tiene su propia cache. Esto es lo correcto: cada ecuación tiene
inputs distintos, normalizaciones distintas, outputs distintos.

Si en algún caso excepcional dos bridges se benefician de compartir cache (mismos inputs,
misma normalización, mismo output type), se configura explícitamente apuntando ambos al
mismo marker type:

```text
Por defecto (aislado):
  BridgeCache<DensityBridge>          ← solo density
  BridgeCache<TemperatureBridge>      ← solo temperature

Compartido (opt-in):
  BridgeCache<SharedPhysicsCache>     ← ambos bridges apuntan al mismo tipo
```

Esto requiere que ambos bridges usen el mismo `type Cache` en su impl de `Bridgeable`.
El compilador verifica compatibilidad — no se puede compartir cache entre bridges con
output types distintos.

**En la práctica**: ninguna ecuación actual se beneficia de compartir cache. Los input spaces,
normalizaciones, output types y frecuencias de actualización son fundamentalmente distintos
entre todas las ecuaciones. La capacidad existe por corrección de diseño, no por necesidad actual.

### Invalidación

La cache NO necesita invalidación explícita. Los inputs cuantizados son deterministas:
`normalize(47.3) = 50.0` siempre. Si el input cuantizado ya está en cache, el resultado es correcto.

La única razón para limpiar cache es **cambio de configuración** (si cambias las bandas de normalización,
los resultados cacheados con bandas anteriores son inválidos). Esto pasa solo al cambiar config,
no tick a tick. Como cada cache es independiente, solo se limpia la cache del bridge afectado.

---

## 9. Integración con el Pipeline Existente

### Sin modificar `equations.rs`

Las funciones puras en `equations.rs` NO se tocan. El decorador vive en una capa de wiring
entre los sistemas y las funciones:

```text
ANTES (sistema llama a ecuación directamente):
  dissipation_system → equations::effective_dissipation(rate, velocity, friction)

DESPUÉS (sistema llama al bridge que envuelve la ecuación):
  dissipation_system → bridge.compute(rate, velocity, friction)
                         └─ normalize → cache lookup → (miss?) equations::effective_dissipation
```

### SystemParam Bridge

Un `SystemParam` genérico que los sistemas usan en lugar de llamar a la función directamente.
Cada bridge inyecta su propia cache tipada — no hay registro global:

```text
BridgedPhysicsOps (SystemParam):
  inner: PhysicsOps                          — el SystemParam original
  density_config: Res<BridgeConfig<DensityBridge>>      — config de density
  density_cache: ResMut<BridgeCache<DensityBridge>>     — cache aislada de density
  temp_config: Res<BridgeConfig<TemperatureBridge>>     — config de temperature
  temp_cache: ResMut<BridgeCache<TemperatureBridge>>    — cache aislada de temperature
  phase_config: Res<BridgeConfig<PhaseBridge>>          — config de phase
  phase_cache: ResMut<BridgeCache<PhaseBridge>>         — cache aislada de phase

  fn density(&self, entity) → f32:
    let (qe, radius) = self.inner.raw_inputs(entity)
    bridge_compute::<DensityBridge>(qe, radius, &self.density_config, &mut self.density_cache)
```

Cada campo es un Resource independiente. Bevy los resuelve en compile time.
No hay HashMap lookup. No hay contaminación cruzada.

Los sistemas existentes cambian UN import: `PhysicsOps` → `BridgedPhysicsOps`.
Cero cambios en lógica de sistema. Cero cambios en ecuaciones.

---

## 10. Cuantización de Vectores (Vec2)

Algunas ecuaciones usan `Vec2` como input (velocidad, dirección). La normalización de vectores:

### Dirección cuantizada

Las direcciones se cuantizan a N sectores angulares:
- 8 sectores: N/NE/E/SE/S/SW/W/NW (suficiente para drag, voluntad).
- 16 sectores: más precisión para interferencia espacial.
- 32 sectores: alta precisión para catálisis.

### Magnitud cuantizada

La magnitud (speed) se cuantiza igual que un escalar: bandas de velocidad.

### Key del cache para Vec2

```text
cache_key = (direction_sector: u8, magnitude_band: u16)
```

Esto convierte un espacio infinito (Vec2) en un espacio finito y cacheable.

---

## 11. Métricas y Observabilidad

### Métricas por capa

Cada `BridgeCache` expone:
- **hit_rate**: `hits / (hits + misses)`. Objetivo: > 80% en runtime.
- **fill_level**: `entries.len() / capacity`. Si es 100%, considerar ampliar.
- **eviction_count**: cuántas entradas se descartaron. Alto = cache muy chica.
- **skip_rate**: qué % de entidades se saltaron por estar en estado estable.

### Dashboard de debug

Extender `DebugPlugin` con un panel que muestre hit rate por capa:

```text
[Bridge Optimizer]
  density:        98.2% hit (cache 45/100)
  temperature:    97.8% hit (cache 18/20)
  state_of_matter: 99.9% hit (cache 4/70)    ← 4 estados posibles, solo 4 en cache
  interference:   72.3% hit (cache 412/500)   ← más variación
  drag_force:     89.1% hit (cache 234/800)
  catalysis:      95.0% hit (cache 156/1000)
  will_force:     99.5% hit (cache 8/100)     ← 8 direcciones WASD
```

---

## 12. Configuración Declarativa (Data-Driven)

Las bandas y configuraciones se definen en un archivo RON/JSON, no hardcodeadas:

```text
assets/bridge_config.ron:

// Cada sección se carga como un Resource tipado independiente:
//   "density"          → Res<BridgeConfig<DensityBridge>>
//   "phase_transition" → Res<BridgeConfig<PhaseBridge>>
// El loader parsea el RON y registra cada config como Resource por tipo.

BridgeConfigFile(
  bridges: {
    "density": (
      rigidity: Moderate,
      bands: [
        (min: 0.0,   max: 10.0,  canonical: 5.0),
        (min: 10.0,  max: 30.0,  canonical: 20.0),
        (min: 30.0,  max: 60.0,  canonical: 45.0),
        (min: 60.0,  max: 100.0, canonical: 80.0),
        (min: 100.0, max: 500.0, canonical: 200.0),
      ],
      hysteresis_margin: 3.0,
      cache_capacity: 100,
      cache_policy: ContextFill,
      enabled: true,
    ),
    "phase_transition": (
      rigidity: Rigid,
      bands: [
        (min: 0.0,   max: 30.0,  canonical: 15.0),   // Solid
        (min: 30.0,  max: 100.0, canonical: 60.0),    // Liquid
        (min: 100.0, max: 300.0, canonical: 180.0),   // Gas
        (min: 300.0, max: 99999, canonical: 400.0),    // Plasma
      ],
      hysteresis_margin: 5.0,
      cache_capacity: 70,
      cache_policy: ContextFill,
      enabled: true,
    ),
    // ... cada bridge adicional como entrada independiente
  }
)
```

Esto permite:
- Tunear bandas sin recompilar.
- Tener distintas configs para distintos mapas (mapa simple = más rígido, arena PvP = más flexible).
- A/B testing de performance: cambiar una config y medir.

---

## 13. Tabla de Transiciones Cuantizadas por Capa

| Transición | Input | Output | Bandas recomendadas | Histéresis | Cache |
|-----------|-------|--------|---------------------|------------|-------|
| Capa 0×1 → density | (qe, radius) | f32 | 20 qe × 5 radius | 2.0 | 100 |
| density → temperature | f32 | f32 | 20 bandas | 1.0 | 20 |
| temp → MatterState | (temp, bond) | enum | 7 × 10 | 5.0 | 70 |
| Capa 2 interference | (f1, f2, t) | f32 | 6 × 6 × 10 | 0.05 | 500 |
| Capa 3 dissipation | (rate, vel, fric) | f32 | 10 × 8 dirs | 0.5 | 80 |
| Capa 3 drag | (visc, dens, vel) | Vec2 | 10 × 10 × 8 | 0.3 | 800 |
| Capa 5 engine | (valve, qe, space) | f32 | 10 × 10 × 10 | 1.0 | 200 |
| Capa 7 will | (intent, buf, max) | Vec2 | 8 × 10 | 0.0 | 80 |
| Capa 8 catalysis | (qe, I, crit) | f32 | 10 × 20 × 5 | 0.01 | 1000 |
| Capa 8 col_transfer | (qe_a, qe_b, I, k, dt) | f32 | 10 × 10 × 20 × 5 | 0.5 | 1000 |

**Nota (B8 / código):** la columna “Bandas recomendadas” describe productos de entrada (p. ej. qe × radius). En `assets/bridge_config.ron` y `bridge/presets.rs`, la carga data-driven usa **bandas escalar agregadas por puente** como proxy hasta bridges multi-input (B4/B5). Las columnas **Histéresis** y **Cache** son las que el motor intenta respetar fila a fila en el preset *Moderate* base. El campo `precision` de §4 del mismo documento **no** está en `BridgeConfig` todavía (RON / tipo); se añade en un sprint posterior o se declara explícitamente fuera de alcance hasta entonces.

---

## 14. Posición Arquitectónica

### No es una capa

El Bridge Optimizer no responde una pregunta sobre la energía. Es una **estrategia de cómputo**
que acelera las preguntas existentes.

### No modifica ecuaciones

Las funciones en `equations.rs` permanecen intactas. El optimizer las envuelve, no las reemplaza.

### Es transparente

Se puede desactivar completamente (`enabled: false` en todas las configs) y el sistema produce
exactamente los mismos resultados. La diferencia es solo performance.

### Es stateless en diseño

El `BridgeConfig<B>` es immutable en runtime (solo cambia al cargar config).
La cache es mutable pero es descartable (borrarla no cambia resultados, solo performance).
El decorador en sí no tiene estado — recibe config + cache + input, retorna output.

### Cache descentralizada

Cada bridge posee su propia cache como `BridgeCache<B>` (generic Resource). No hay registro
global ni HashMap de routing. Aislamiento por tipo en compile time, zero cost en runtime.
Compartir cache entre bridges es opt-in explícito (mismo marker type), no default.

### Respeta el pipeline

Se integra como SystemParam wrapper. No cambia el orden de fases, no agrega sistemas nuevos
al schedule, no modifica el grafo de dependencias.

---

## 15. Organización de Código

```text
src/
  bridge/                         ← NUEVO módulo
    mod.rs                        ← re-exports
    config.rs                     ← BridgeConfig, BandDef, Rigidity, EvictionPolicy
    normalize.rs                  ← funciones de normalización (escalar, Vec2, multi-dim)
    cache.rs                      ← BridgeCache<B>, CacheBackend, CachedValue
    decorator.rs                  ← trait Bridgeable + fn bridge_compute genérico
    context_fill.rs               ← Warmup → Filling → Active + `bridge_warmup_record*`
    metrics.rs                    ← hit/miss counters, dashboard data
    bridged_ops.rs                ← BridgedPhysicsOps, BridgedInterferenceOps (wrappers)
    presets.rs                    ← configs predeterminadas (Rigid, Moderate, Flexible)

  assets/
    bridge_config.ron             ← configuración data-driven
```

---

## 16. Plan de Sprints

| Sprint | Entregable | Depende de | Validación |
|--------|-----------|------------|------------|
| B1 | `BridgeConfig` + `BandDef` + normalización escalar | — | Test: normalizar valores en bandas, histéresis funciona |
| B2 | `BridgeCache<B>` genérico + eviction (sin registry global) | B1 | Test: insert/lookup, LRU eviction, capacity limits |
| B3 | Trait `Bridgeable` + `bridge_compute` genérico | B1+B2 | Test: decorar una función trivial, verificar hits/misses |
| B4 | `BridgedPhysicsOps` wrapper | B3 | Test: density y temperature producen mismos resultados ±epsilon |
| B5 | `BridgedInterferenceOps` wrapper | B3 | Test: interference bridged = interference original ±epsilon |
| B6 | Normalización Vec2 (dirección + magnitud) | B1 | Test: cuantización angular correcta |
| B7 | Context-fill pipeline (warmup → fill → enable) | B2+B3 | Test: cache se llena durante warmup, hits suben en runtime |
| B8 | Config data-driven (RON) + presets | B1 | Test: cargar config, aplicar bandas |
| B9 | Métricas + panel debug | B2+B3 | Test: counters correctos, DebugPlugin muestra hit rate |
| B10 | Benchmark comparativo (con vs sin optimizer) | Todo | Medir: latencia por tick, hit rates por capa, memory |

---

## 17. Estimación de Impacto

### Escenario: 200 entidades, 60 ticks/s

**Sin optimizer**:
- ~10 ecuaciones × 200 entidades × 60 ticks = 120,000 cómputos/s.

**Con optimizer (hit rate promedio 85%)**:
- 120,000 × 0.15 (misses) = 18,000 cómputos reales/s.
- 120,000 × 0.85 (hits) = 102,000 lookups (cache = ~2ns cada uno).
- **Reducción estimada: ~80% de cómputo de ecuaciones.**

### Escenario: 1000 entidades (con V7 worldgen)

- ~10 ecuaciones × 1000 entidades × 60 ticks = 600,000 cómputos/s.
- Con optimizer: ~90,000 cómputos + 510,000 lookups.
- La diferencia entre "juego fluido" y "frame drops" para mapas grandes.

---

## 18. Resumen

```text
El Bridge Optimizer NO cambia qué se computa.
Cambia CUÁNDO se computa.

Si el input no cambió significativamente → devolver el resultado anterior.
Si el input cambió → computar, guardar, devolver.

Normalización: convierte infinito → finito.
Cache: convierte finito → O(1).
Histéresis: convierte inestable → estable.
Context-fill: convierte teórico → real.
Descentralización: cada bridge posee su cache — aislamiento por tipo, zero routing.

El resultado: misma física, mismos resultados (±epsilon configurable),
fracción del costo computacional.

Todo configurable. Todo desactivable. Todo observable. Todo aislado.
```
