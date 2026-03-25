# Sprint Primer — Guía Mínima para Implementar en Resonance

> Leé esto antes de tocar código. Si algo no está claro, preguntá antes de escribir.

---

## La Arquitectura en una frase

**DOD puro (Data-Oriented Design) + Higher-Order Functions stateless + cache por composición.**

No es hexagonal. No es clean architecture. No hay ports/adapters. En ECS:
- Los **componentes** son el dominio.
- Los **sistemas** son los use cases.
- **Bevy** es la infraestructura.

---

## Las 7 Reglas que Importan

### 1. Datos separados de lógica — siempre

```
Componente = datos puros (max 4 campos)
Sistema     = una transformación (lee datos → escribe datos)
Ecuación    = math pura en blueprint/equations/ (sin ECS, sin estado)
```

Si tu componente tiene un método que muta otra cosa → está mal. Si tu sistema hace dos cosas distintas → partilo.

### 2. Stateless-first: las funciones puras son la base

```rust
// BIEN: función pura, testeable en aislamiento
pub fn carnot_efficiency(t_core: f32, t_env: f32) -> f32 {
    if t_core <= t_env { return 0.0; }
    1.0 - t_env / t_core.max(f32::EPSILON)
}

// MAL: math inline en un sistema
fn my_system(query: Query<&mut Energy>) {
    for mut e in &mut query {
        e.efficiency = 1.0 - env_temp / e.temp.max(0.001); // NO
    }
}
```

**¿Por qué?** Porque lo stateless:
- Se testea sin Bevy (unit test puro).
- Se cachea sin riesgo (misma entrada → misma salida, siempre).
- Se compone (HOF: funciones que reciben/retornan funciones).
- Se paraleliza gratis (sin locks, sin data races).

### 3. Cuando la complejidad crece, usá patrones — no más código

| Señal de complejidad | Patrón que aplica |
|---|---|
| Sistema con >5 tipos en la query | Partí en dos sistemas con un evento entre ellos |
| Componente con >4 campos | Partí en dos componentes (capas ortogonales) |
| Misma lógica con N variantes | Enum cerrado + match exhaustivo (no trait objects) |
| Composición de comportamientos | HOF: funciones que envuelven funciones (fold sobre stack) |
| Valor caro de computar | BridgeCache: cache determinista por composición de inputs |
| Estado derivado que se repite | No lo guardes — computá en punto de uso |
| N entidades comparten un recurso | Pool pattern: distribución proporcional + invariante de conservación |

**El objetivo es reducir complejidad, no moverla.** Si tu abstracción tiene más LOC que el código que reemplaza → no la necesitás.

### 4. Enums cerrados, no trait objects

```rust
// BIEN: el compilador verifica exhaustividad
match extraction_type {
    ExtractionType::Proportional => extract_proportional(available, n),
    ExtractionType::Greedy       => extract_greedy(available, capacity),
    ExtractionType::Competitive  => extract_competitive(available, fitness, total),
    ExtractionType::Aggressive   => extract_aggressive(available, aggression, damage).0,
    ExtractionType::Regulated    => extract_regulated(available, ratio, rate, lo, hi),
}

// MAL: indirección dinámica, no exhaustivo, no Copy
Box<dyn ExtractionFn>
```

Agregás una variante → el compilador te dice todos los match que faltan. Con trait objects, te enterás en runtime.

### 5. Composición funcional > herencia > configuración

Resonance usa **HOFs (Higher-Order Functions)** para componer comportamientos:

```
Base function   + Modifier stack    = Phenotype
─────────────────────────────────────────────────
proportional    + stress_response   = opportunistic generalist
greedy          + threshold_gated   = conservative specialist
aggressive      + cap_per_tick      = controlled parasite
regulated       + stress_response   = resilient homeostatic
```

El fenotipo **no se almacena** — es la evaluación de la función compuesta. Stack-allocated, `Copy`, determinista.

### 6. Cache por composición (BridgeCache)

Si una función pura es cara y se llama muchas veces con los mismos inputs:

```
inputs → hash → cache lookup → hit? return cached : compute → store → return
```

Funciona **solo** si la función es stateless. Por eso stateless-first es requisito, no preferencia. El cache es consecuencia del diseño, no un add-on.

### 7. Invariantes se verifican, no se asumen

```rust
// Post-condición: sum(extracted) <= available
debug_assert!(
    total_extracted <= available + EPSILON,
    "Pool invariant violated: {total_extracted} > {available}"
);
```

Debug asserts en funciones puras. El sistema no debería poder producir un estado inválido, pero si lo produce, te enterás inmediato — no 1000 ticks después.

---

## Pipeline: dónde va cada cosa

```
FixedUpdate (determinista, timestep fijo):
  Phase::Input                  ← lee input del jugador
  Phase::ThermodynamicLayer     ← física térmica, contención, motores
  Phase::AtomicLayer            ← disipación, colisión, movimiento
  Phase::ChemicalLayer          ← reacciones, catálisis, fotosíntesis
  Phase::MetabolicLayer         ← grafos metabólicos, trófico, pools
  Phase::MorphologicalLayer     ← forma, color, crecimiento, reproducción

Update (visual, no determinista):
  Derivación visual, color, UI
```

**Regla:** gameplay → `FixedUpdate` + `Phase`. Visual → `Update`.

---

## Estructura de archivos: dónde va cada cosa

```
Ecuación nueva       → src/blueprint/equations/{dominio}/
Constante nueva      → src/blueprint/constants/{dominio}.rs
Componente nuevo     → src/layers/{nombre}.rs  (+ re-export en mod.rs)
Sistema nuevo        → src/simulation/{subdir}/{nombre}.rs
Evento nuevo         → src/events.rs  (+ registro en bootstrap.rs)
Arquetipo de spawn   → src/entities/archetypes.rs
Builder extension    → src/entities/builder.rs
```

---

## Checklist rápido antes de mergear

- [ ] ¿Componentes con <=4 campos?
- [ ] ¿Math en `blueprint/equations/`, no inline en sistemas?
- [ ] ¿Constantes en `blueprint/constants/`, no magic numbers?
- [ ] ¿Sistema registrado con `.in_set(Phase::X)`?
- [ ] ¿Guard de change detection (`if old != new`)?
- [ ] ¿Sin `unwrap()`/`expect()` en sistemas?
- [ ] ¿Sin `unsafe`, `async`, `Arc<Mutex>`, `HashMap` en hot path?
- [ ] ¿Sin `String` ni `Box<dyn Trait>` en componentes?
- [ ] ¿Tests unitarios para funciones puras?
- [ ] ¿`cargo test --lib` verde?

---

## Resumen mental

```
Simple > Easy.
Stateless > Stateful.
Compose > Configure.
Derive > Store.
Assert > Assume.
4 fields > 8 fields.
1 system = 1 job.
Enum > Trait object.
Stack > Heap.
f32 > f64.
```

Si dudás entre dos enfoques, elegí el que sea más fácil de testear sin Bevy.
