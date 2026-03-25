# BLUEPRINT — Bridge Optimizer: Cuantización Estable + Cache por Capa

---

## 1. Objetivo

Reducir el costo computacional de las transiciones entre capas sin modificar sus ecuaciones ni su semántica.

```text
Un material mantiene su composición atómica porque le es menos costoso que perderla.
Los resultados matemáticos entre capas TIENDEN a estados estables.
Computar lo estable es redundante. Computar lo raro es necesario.
```

El Bridge Optimizer es un decorador stateless que envuelve cada ecuación de transición con:
- **Normalización por bandas**: inputs continuos (f32) se cuantizan a valores canónicos discretos.
- **Cache de resultados**: inputs cuantizados → espacio finito → memoizable al 100%.
- **Sesgo de estabilidad (histéresis)**: cerca de bordes de banda, el valor se sesga hacia el estado actual.

Resultado esperado: misma física, mismos resultados (±epsilon configurable), fracción del costo computacional.

---

## 2. Herencia obligatoria

El Bridge Optimizer hereda sin excepciones:

- Pipeline: `Input → PrePhysics → Physics → Reactions → PostPhysics`
- Ecuaciones puras en `src/blueprint/equations.rs` — NO se modifican.
- Arquitectura por capas ortogonales (Capas 0-13).
- Determinismo operativo: con optimizer desactivado, resultados bit-a-bit idénticos.
- Contratos explícitos por módulo.

El optimizer NO es una capa. No responde una pregunta sobre la energía. Es una estrategia de cómputo que acelera las preguntas existentes.

---

## 3. Principios de diseño

1. **Transparencia total**: `enabled: false` → bypass completo, cero efecto en resultados. La diferencia entre ON y OFF es solo performance.
2. **Stateless en diseño**: el `BridgeConfig<B>` es inmutable en runtime. La cache es mutable pero descartable (borrarla no cambia resultados). El decorador no tiene estado propio.
3. **Decorador genérico**: una sola implementación de `bridge_compute` cubre todas las ecuaciones. Sin repetición de código por capa.
4. **Zero-alloc en hot path**: inputs y outputs son `Copy`. Sin `Box`, sin `dyn`, sin allocación.
5. **Configurable por capa**: cada ecuación tiene su propia rigidez, bandas, histéresis y capacidad de cache.
6. **Cache descentralizada e inyectable**: cada bridge posee su cache como `BridgeCache<B>` (generic Resource). Aislamiento por tipo en compile time — una cache nunca busca entre datos de otra ecuación. Zero overhead de routing (no hay HashMap global). Compartir cache entre bridges es opt-in explícito (mismo marker type), no default.
7. **Data-driven**: bandas y configuraciones se definen en RON, no hardcodeadas. Tuneable sin recompilar.
8. **Context-fill**: la cache se llena con valores reales de la partida durante warmup, no con valores teóricos.

---

## 4. Tabla de módulos

| # | Módulo | Tipo | Responsabilidad | Entradas | Salidas |
|---|--------|------|-----------------|----------|---------|
| 01 | `bridge/config` | Tipos puros | `BridgeConfig<B>`, BandDef, Rigidity, CachePolicy | — | tipos compartidos |
| 02 | `bridge/normalize` | Stateless | Normalización escalar, Vec2, multi-dim + histéresis | f32/Vec2, BandDef[], hint | valor canónico |
| 03 | `bridge/cache` | Resource genérico | `BridgeCache<B>` tipada por bridge, evicción LRU, sin registry global | key u64, CachedValue | lookup/insert |
| 04 | `bridge/decorator` | Stateless | Trait Bridgeable + fn bridge_compute genérico | input, config, cache | output (hit o compute) |
| 05 | `bridge/bridged_ops` | SystemParam | BridgedPhysicsOps, BridgedInterferenceOps wrappers | entity queries | resultados bridged |
| 06 | `bridge/context_fill` | Sistema | Pipeline warmup → filling → active | tick count, fill level | fase actual |
| 07 | `bridge/metrics` | Observabilidad | Contadores hit/miss, dashboard, recomendaciones | BridgeCache stats | BridgeMetrics |
| 08 | `bridge/presets` | Data-driven | RON config loader, presets de rigidez, hot reload | bridge_config.ron | BridgeConfigs resource |

---

## 5. Tipos nuevos

### 5.1 BridgeConfig

Configuración de normalización por ecuación. Contiene bandas, histéresis, capacidad de cache, política de evicción, flag enabled.

### 5.2 BandDef

Rango de normalización: (min, max, canonical, stable). Los valores dentro del rango se tratan como equivalentes y se sustituyen por el canonical.

### 5.3 BridgeCache\<B: BridgeKind\> (generic Resource)

Cache tipada por bridge. Cada bridge inserta su propia `BridgeCache<B>` como Resource independiente en Bevy.
Bevy trata `BridgeCache<DensityBridge>` y `BridgeCache<TemperatureBridge>` como Resources completamente
separados — aislamiento por tipo en compile time, zero cost de routing en runtime.

Storage interno: `CacheBackend` (Vec lineal para capacity ≤ 256, FxHashMap para mayor). Evicción LRU con tick counter.

**Aislamiento por defecto**: una cache nunca busca entre datos de otra ecuación. Si dos bridges necesitan
compartir cache (caso excepcional), se configura explícitamente apuntando ambos al mismo marker type
(`type Cache = SharedMarker`). El compilador verifica compatibilidad de tipos.

### 5.4 CachedValue (enum)

Union de outputs posibles: `Scalar(f32)`, `State(MatterState)`, `Vector(Vec2)`. 12 bytes max.

### 5.5 BridgePhase (enum)

Fases del pipeline de context-fill: `Warmup` (bypass + grabar), `Filling` (activo + sin evicción), `Active` (operación normal).

### 5.6 BridgeMetrics\<B: BridgeKind\>

Contadores tipados por bridge: hits, misses, evictions, fill_level. Cada bridge tiene su propio Resource de métricas. Ventana deslizante para métricas actuales.

---

## 6. Pipeline del decorador

```text
Input f32/Vec2
    │
    ▼
┌──────────────────────┐
│  1. Normalizar input │  BandDef[] + hysteresis + band_hint
│     → canonical      │
├──────────────────────┤
│  2. Hash → key u64   │  fxhash de inputs canónicos
├──────────────────────┤
│  3. Cache lookup     │
│     HIT → return     │  ← fast path (~2ns)
│     MISS ↓           │
├──────────────────────┤
│  4. Compute original │  equations::fn(normalized_inputs)
├──────────────────────┤
│  5. Cache insert     │  LRU eviction si llena
├──────────────────────┤
│  6. Return result    │
└──────────────────────┘
```

### 6.1 Context-fill pipeline

```text
Fase Warmup (ticks 0-100):
  bridges bypass, pero cada resultado se graba en cache

Fase Filling (ticks 100-150):
  bridges activos, evicción deshabilitada (solo insertar)

Fase Active (tick 150+):
  operación normal, LRU activo, métricas cuentan
```

---

## 7. Aplicación por capa — Rigidez configurable

| Capa / Ecuación | Rigidez | Bandas | Histéresis | Cache | Hit rate esperado |
|-----------------|---------|--------|------------|-------|-------------------|
| Capa 0×1 → density | Moderada | 20 qe × 5 radius | 2.0 | 100 | ~90% |
| density → temperature | Moderada | 20 | 1.0 | 20 | ~90% |
| temp → MatterState | Rígida | 7 × 10 | 5.0 | 70 | ~99% |
| Capa 2 interference | Flexible | 6 × 6 × time_quant | 0.05 | 500 | ~60% |
| Capa 3 dissipation | Moderada | 10 × 8 dirs | 0.5 | 80 | ~85% |
| Capa 3 drag | Moderada | 10 × 10 × 8 | 0.3 | 800 | ~85% |
| Capa 5 engine | Moderada | 10 × 10 × 10 | 1.0 | 200 | ~85% |
| Capa 7 will force | Rígida | 8 × 10 | 0.0 | 80 | ~99% |
| Capa 8 catalysis | Flexible | 10 × 20 × 5 | 0.01 | 1000 | ~95% |
| Capa 8 collision | Flexible | multi-dim | 0.5 | 1000 | ~80% |

La rigidez puede ajustarse dinámicamente: en calma → más rígida; en combate → más flexible.

---

## 8. Sesgo de estabilidad (histéresis)

Principio físico: un sistema en equilibrio permanece en equilibrio a menos que una fuerza externa lo perturbe.

```text
Sin sesgo (oscilación):
  tick 1: temp 29.8 → Solid
  tick 2: temp 30.1 → Liquid  ← cambió
  tick 3: temp 29.9 → Solid   ← cambió otra vez

Con sesgo (estable):
  tick 1: temp 29.8, current=Solid → bias → Solid
  tick 2: temp 30.1, current=Solid → bias → Solid  ← se queda
  tick 3: temp 31.5, current=Solid → bias → Liquid  ← ahora sí (superó margen)
```

El `band_hint` (estado actual de la entidad) se pasa como parámetro extra. El sesgo solo se rompe cuando el valor supera el umbral por el margen de histéresis configurado. Físicamente correcto Y computacionalmente barato.

---

## 9. Cuantización de vectores (Vec2)

Dirección cuantizada a sectores angulares:
- 8 sectores: N/NE/E/SE/S/SW/W/NW — suficiente para drag, will force.
- 16/32 sectores: subdivisiones para interferencia espacial / catálisis.

Magnitud cuantizada por bandas escalares (reutiliza `normalize_scalar`).

Casos especiales:
- `Vec2::ZERO` → sector estático (255), magnitud 0. Caso más común para entidades estáticas.
- Will force WASD → exactamente 8 direcciones + zero. La cuantización coincide con el input real → hit rate 100%.

Cache key para Vec2: `(direction_sector: u8, magnitude_band: u16)` empaquetado en u64.

---

## 10. Integración con el pipeline existente

### Sin modificar `equations.rs`

Las funciones puras permanecen intactas. El optimizer las envuelve, no las reemplaza.

```text
ANTES: dissipation_system → equations::effective_dissipation(rate, vel, friction)
DESPUÉS: dissipation_system → bridge.compute(rate, vel, friction)
                                └─ normalize → lookup → (miss?) equations::effective_dissipation
```

### SystemParam wrappers — cache inyectada por tipo

Los sistemas existentes cambian UN import: `PhysicsOps` → `BridgedPhysicsOps`. La API es idéntica. Cero cambios en lógica de sistema.

Cada bridge inyecta su propia cache tipada. No hay registro global:

```text
BridgedPhysicsOps (SystemParam):
  inner: PhysicsOps                                     ← el original
  density_config: Res<BridgeConfig<DensityBridge>>      ← config inyectada
  density_cache: ResMut<BridgeCache<DensityBridge>>     ← cache aislada
  temp_config: Res<BridgeConfig<TemperatureBridge>>     ← config inyectada
  temp_cache: ResMut<BridgeCache<TemperatureBridge>>    ← cache aislada
  phase_config: Res<BridgeConfig<PhaseBridge>>          ← config inyectada
  phase_cache: ResMut<BridgeCache<PhaseBridge>>         ← cache aislada
```

Bevy resuelve cada Resource por tipo en compile time. Zero HashMap lookup.
Cada cache es independiente — nunca hay contaminación cruzada entre ecuaciones.

### Respeta el pipeline

Se integra como SystemParam wrapper. No cambia el orden de fases, no agrega sistemas nuevos al schedule, no modifica el grafo de dependencias.

---

## 11. Posición arquitectónica

- **No es una capa**: no responde una pregunta sobre la energía. Es una estrategia de cómputo.
- **No modifica ecuaciones**: `equations.rs` permanece intacto.
- **Es transparente**: `enabled: false` en todas las configs → sistema produce exactamente los mismos resultados.
- **Es stateless en diseño**: config inmutable, cache descartable, decorador sin estado.
- **Respeta el pipeline**: SystemParam wrapper, no modifica schedule ni grafo de dependencias.
- **Cache descentralizada**: cada bridge posee su cache como generic Resource (`BridgeCache<B>`). Aislamiento por tipo en compile time, zero routing overhead. Compartir es opt-in explícito, no default.

---

## 12. Organización de código

```text
src/
  bridge/                          ← NUEVO módulo
    mod.rs                         ← re-exports
    config.rs                      ← BridgeConfig, BandDef, Rigidity, EvictionPolicy
    normalize.rs                   ← normalización escalar + Vec2 + multi-dim
    cache.rs                       ← BridgeCache<B>, CacheBackend, CachedValue
    decorator.rs                   ← trait Bridgeable + fn bridge_compute genérico
    metrics.rs                     ← hit/miss counters, dashboard data
    bridged_ops.rs                 ← BridgedPhysicsOps, BridgedInterferenceOps
    context_fill.rs                ← BridgePhase, warmup pipeline
    presets.rs                     ← configs predeterminadas + RON loader

  assets/
    bridge_config.ron              ← configuración data-driven
```

---

## 13. Estimación de impacto

### Escenario: 200 entidades, 60 ticks/s

| Métrica | Sin optimizer | Con optimizer (85% hit avg) |
|---------|---------------|----------------------------|
| Cómputos/s | 120,000 | 18,000 reales + 102,000 lookups |
| Costo lookup | — | ~2ns cada uno |
| Reducción | — | ~80% de cómputo de ecuaciones |

### Escenario: 1000 entidades (V7 worldgen)

| Métrica | Sin optimizer | Con optimizer (85% hit avg) |
|---------|---------------|----------------------------|
| Cómputos/s | 600,000 | 90,000 reales + 510,000 lookups |
| Impacto | Frame drops probables | Pipeline fluido |

---

## 14. Trade-offs

| Decisión | Valor | Costo |
|----------|-------|-------|
| Normalización por bandas | Espacio infinito → finito, cacheable | Epsilon de cuantización (configurable) |
| Cache descentralizada por tipo (`BridgeCache<B>`) | Aislamiento compile-time, zero routing, cada bridge gestiona su propia cache | Más Resources en Bevy (uno por bridge), monomorphization |
| Histéresis de estabilidad | Elimina oscilaciones, reduce cómputos | Latencia en transiciones legítimas (configurable) |
| Context-fill warmup | Cache llena de valores reales | ~1.7s de warmup al inicio de partida |
| Decorador genérico monomorphizado | Zero overhead en hot path, sin dyn | Código compilado más grande (monomorphization) |
| Config data-driven RON | Tuneable sin recompilar | Dependencia de archivo externo |
| Dual backend cache (Vec/FxHashMap) | Óptimo para cada tamaño | Complejidad de implementación |

---

## 15. Riesgos y mitigación

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| Epsilon de normalización visible en gameplay | Medio | Bandas alineadas a umbrales existentes (phase transitions = epsilon cero) |
| Cache memory footprint excesivo | Bajo | Capacities configurables, LRU eviction, métricas de fill |
| Warmup insuficiente para contextos grandes | Medio | Duración configurable + context-fill on-demand post-warmup |
| Overhead del decorador supera el ahorro en worst-case | Bajo | Benchmark B10 valida; fast-path < 10ns |
| Config RON inválida (gaps/overlaps en bandas) | Medio | Validador al cargar + fallback a preset default |
| Hit rate bajo en ecuaciones con variación alta (interference) | Medio | Cuantización temporal agresiva + cache más grande |
| Hot reload cambia bandas → cache inconsistente | Bajo | Clear cache en hot reload (solo afecta desarrollo) |

---

## 16. Sprints

Ver `docs/sprints/BRIDGE_OPTIMIZER/README.md` para el plan completo de implementación.

| Sprint | Entregable | Onda |
|--------|-----------|------|
| B1 | Normalización escalar + histéresis | 0 |
| B2 | `BridgeCache<B>` genérico descentralizado + evicción LRU | A |
| B3 | Trait Bridgeable + bridge_compute genérico | B |
| B4 | BridgedPhysicsOps (density, temperature, phase) | C |
| B5 | BridgedInterferenceOps (interference, catalysis, collision) | C |
| B6 | Normalización Vec2 (dirección + magnitud) | A |
| B7 | Context-fill pipeline (warmup → fill → enable) | D |
| B8 | Config data-driven RON + presets | A |
| B9 | Métricas + panel debug | E |
| B10 | Benchmark comparativo | F |

---

## 17. Referencias

- `docs/design/BLUEPRINT.md` (modelo de capas)
- `DESIGNING.md` (filosofía de capas y tests)
- `docs/arquitectura/blueprint_layer_bridge_optimizer.md` (blueprint de arquitectura detallado)
- `docs/design/V7.md` (V7, worldgen que se beneficia del optimizer)
- `src/blueprint/equations.rs` (funciones que se envuelven)
