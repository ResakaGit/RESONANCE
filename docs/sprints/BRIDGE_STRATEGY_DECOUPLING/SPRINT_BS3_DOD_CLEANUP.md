# BS-3: Limpieza DoD — Dead Code, Campos Derivados, Metrics Split

**Objetivo:** Eliminar violaciones de Data-Oriented Design y código muerto. Reducir superficie de mantenimiento.

**Estado:** PENDIENTE
**Esfuerzo:** S (~90 LOC neto, más eliminación)
**Bloqueado por:** —
**Desbloquea:** — (hacer primero, sin dependencias)

---

## ~~Fix 1: Eliminar `PerformanceCachePolicy`~~ — CANCELADO

**Análisis corregido:** `PerformanceCachePolicy` NO es código muerto. Se usa activamente en:
- `entity_shape_inference_system` — cache hit check via `dependency_signature`
- 5 archetypes en `catalog.rs` — spawned con `enabled: true, scope: StableWindow`
- `reproduction/mod.rs` — incluido en offspring spawn
- `abiogenesis/mod.rs` — incluido en spawning

El component implementa **mesh cache invalidation por entidad** (firma compacta de inputs → skip rebuild si no cambió). Es complementario a `BridgeCache<B>` (Resource-level) — `PerformanceCachePolicy` opera a nivel entity.

**Acción:** Ninguna. Mantener tal cual.

---

## Fix 2: Eliminar `MetabolicGraph::total_entropy_rate` (campo derivado — Regla 13)

### Problema

`src/layers/metabolic_graph.rs:66`:
```rust
pub struct MetabolicGraph {
    nodes: [ExergyNode; 12],  // 1
    nodes_len: u8,             // 2
    edges: [ExergyEdge; 16],   // 3
    edges_len: u8,             // 4
    total_entropy_rate: f32,   // 5 ← DERIVADO + campo excedente
}
```

Violaciones:
- **Regla 13:** "NO storing derived values as components — compute at point of use"
- **Regla 2:** 5 campos (max 4 por component)
- `total_entropy_rate = Σ nodes[..n].entropy_rate` — O(n), n ≤ 12, **costo negligible**

### Acción

1. Eliminar campo `total_entropy_rate` de `MetabolicGraph`
2. Eliminar `set_total_entropy_rate()` y `total_entropy_rate()` accessors
3. Añadir función pura en su lugar:

```rust
// blueprint/equations/metabolic_graph.rs (NUEVO o en mod.rs)

/// Tasa de entropía total del grafo — Σ entropy_rate de nodos activos.
/// Coste: O(n), n ≤ 12. Compute on demand, no almacenar.
#[inline]
pub fn total_entropy_rate(nodes: &[ExergyNode]) -> f32 {
    nodes.iter().map(|n| n.entropy_rate).sum()
}
```

4. Migrar todos los call sites de `graph.total_entropy_rate()` a `equations::total_entropy_rate(graph.nodes())`
5. Actualizar `MetabolicGraphBuilder::build()` — ya no inicializa campo

### Tests

```
// En blueprint/equations/ (unit, TDD)
total_entropy_rate_empty_slice_returns_zero
total_entropy_rate_single_node_returns_its_rate
total_entropy_rate_multiple_nodes_returns_sum
total_entropy_rate_ignores_zero_rate_nodes

// Regresión en layers/metabolic_graph.rs
metabolic_graph_has_four_fields_after_removal  // compile-time: struct tiene 4 campos
```

---

## Fix 3: Split `BridgeMetrics<B>` (8 campos → 2 structs de 4)

### Problema

`src/bridge/metrics.rs:68-79`:
```rust
pub struct BridgeMetrics<B: BridgeKind> {
    pub layer_name: &'static str,       // 1  identidad
    pub window_hits: u64,               // 2  hit/miss
    pub window_misses: u64,             // 3  hit/miss
    pub window_evictions: u64,          // 4  eviction
    pub fill_len: usize,                // 5  capacity
    pub fill_capacity: usize,           // 6  capacity
    pub computations_saved: u64,        // 7  aggregate
    pub total_lookups: u64,             // 8  aggregate
    _marker: PhantomData<B>,
}
```

8 campos. No es Component (es Resource), pero viola el espíritu de max-4 y dificulta la lectura.

### Acción

Split en 2 Resources por bridge:

```rust
/// Estadísticas de ventana — reseteadas cada collect_interval.
pub struct BridgeWindowMetrics<B: BridgeKind> {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub fill_ratio: f32,  // fill_len / fill_capacity (derivado, pero f32 es cheap)
    _marker: PhantomData<B>,
}

/// Estadísticas acumuladas — nunca se resetean.
pub struct BridgeLifetimeMetrics<B: BridgeKind> {
    pub layer_name: &'static str,
    pub total_lookups: u64,
    pub computations_saved: u64,
    pub capacity: usize,
    _marker: PhantomData<B>,
}
```

**Alternativa simplificada:** Si el split genera demasiado boilerplate de registro (11×2=22 Resources), **aceptar DEBT inline** y documentar:

```rust
// DEBT: 8 campos en Resource (no Component). Split generaría 22 Resources de boilerplate.
// Mantener monolítico mientras no haya consumer que lea subconjuntos.
```

**Decisión: evaluar costo/beneficio en implementación.** Si el boilerplate de 22 Resources es excesivo, documentar DEBT y no splitear.

---

## Fix 4: `BridgeLayerRow::recommendations` — Vec\<String\> → enum flags

### Problema

`src/bridge/metrics.rs:50-58`:
```rust
pub struct BridgeLayerRow {
    pub recommendations: Vec<String>,  // heap alloc en struct de reporte
}
```

Heap allocation para recomendaciones que son siempre de un set cerrado (2-3 variantes).

### Acción

```rust
/// Recomendaciones de tuning — bitflags, zero alloc.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheRecommendations(u8);

impl CacheRecommendations {
    pub const NONE: Self = Self(0);
    pub const INCREASE_CAPACITY: Self = Self(1 << 0);
    pub const WIDEN_BANDS: Self       = Self(1 << 1);
    pub const REDUCE_HYSTERESIS: Self = Self(1 << 2);

    #[inline] pub fn contains(self, flag: Self) -> bool { self.0 & flag.0 != 0 }
    #[inline] pub fn insert(&mut self, flag: Self)      { self.0 |= flag.0; }
}
```

Reemplazar `Vec<String>` por `CacheRecommendations` en `BridgeLayerRow`.

Migrar `row_recommendations()` para retornar `CacheRecommendations` en vez de `Vec<String>`.

Si hay consumers que formatean las recomendaciones como texto, añadir:

```rust
impl CacheRecommendations {
    pub fn as_strs(&self) -> impl Iterator<Item = &'static str> + '_ {
        [
            (Self::INCREASE_CAPACITY, "increase cache_capacity"),
            (Self::WIDEN_BANDS, "widen bands or add cache"),
            (Self::REDUCE_HYSTERESIS, "reduce hysteresis margin"),
        ].iter().filter(|(f, _)| self.contains(*f)).map(|(_, s)| *s)
    }
}
```

### Tests

```
cache_recommendations_none_is_empty
cache_recommendations_insert_and_contains
cache_recommendations_multiple_flags_orthogonal
cache_recommendations_as_strs_matches_flags
row_recommendations_high_evictions_sets_increase_capacity
row_recommendations_low_hit_rate_sets_widen_bands
```

---

## Fix 5: Extraer `shape_cache_signature` a `blueprint/equations/`

### Problema

`src/simulation/lifecycle/entity_shape_inference.rs:82-92` — bitwise ops inline:

```rust
let rug_bucket = surface_opt.map(|s| ((s - 1.0) / 3.0 * 7.0) as u16 & 0x7).unwrap_or(0);
let alb_bucket = albedo_opt.map(|a| (a * 3.0) as u16 & 0x3).unwrap_or(0);
let new_sig = base_sig.wrapping_add(rug_bucket.wrapping_shl(1) | alb_bucket.wrapping_shl(5));
```

Math inline en sistema. Violación Regla 8 (math en `blueprint/equations/`).

### Acción

Crear función pura:

```rust
// blueprint/equations/shape_cache.rs (NUEVO)

/// Firma compacta para invalidación de mesh cache.
/// Layout: bits [0] = base, [1:3] = rugosity bucket (3 bits), [5:6] = albedo bucket (2 bits).
#[inline]
pub fn shape_cache_signature_extended(
    base_sig: u16,
    rugosity: Option<f32>,
    albedo: Option<f32>,
) -> u16 {
    let rug = rugosity
        .map(|s| ((s - 1.0) / 3.0 * 7.0) as u16 & 0x7)
        .unwrap_or(0);
    let alb = albedo
        .map(|a| (a * 3.0) as u16 & 0x3)
        .unwrap_or(0);
    base_sig.wrapping_add(rug.wrapping_shl(1) | alb.wrapping_shl(5))
}
```

Reemplazar inline en `entity_shape_inference.rs` con llamada a `equations::shape_cache_signature_extended(...)`.

### Tests

```
shape_cache_sig_no_modulation_equals_base
shape_cache_sig_rugosity_only_shifts_bits_1_to_3
shape_cache_sig_albedo_only_shifts_bits_5_to_6
shape_cache_sig_both_modulations_compose
shape_cache_sig_none_options_return_base
shape_cache_sig_max_rugosity_clamps_to_3_bits
shape_cache_sig_max_albedo_clamps_to_2_bits
```

---

## Archivos tocados

| Archivo | Cambio |
|---------|--------|
| `src/layers/performance_cache.rs` | **ELIMINAR** |
| `src/layers/mod.rs` | - re-export PerformanceCachePolicy |
| `src/layers/metabolic_graph.rs` | - campo total_entropy_rate, - accessors |
| `src/blueprint/equations/metabolic_graph.rs` | **NUEVO** o añadir a mod.rs |
| `src/blueprint/equations/shape_cache.rs` | **NUEVO** |
| `src/blueprint/equations/mod.rs` | + re-exports |
| `src/bridge/metrics.rs` | CacheRecommendations enum, split o DEBT |
| `src/simulation/lifecycle/entity_shape_inference.rs` | - inline bitwise, + equations call |
| Plugins que registraban PerformanceCachePolicy | - register_type |

---

## Checklist pre-merge

- [ ] `grep -r "PerformanceCachePolicy" src/` devuelve 0 resultados
- [ ] `grep -r "total_entropy_rate" src/layers/` devuelve 0 (solo en equations/)
- [ ] `MetabolicGraph` tiene exactamente 4 campos
- [ ] `BridgeLayerRow` no contiene `Vec<String>`
- [ ] Shape cache signature test cubre bit layout completo
- [ ] `cargo test --lib` verde
- [ ] Zero regresiones en batch tests
