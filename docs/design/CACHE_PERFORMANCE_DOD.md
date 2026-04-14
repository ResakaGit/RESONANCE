# Cache & Performance — Definition of Done

> Operational contract for caching and performance management in Resonance.

## Guiding Principle

Every cache is **dispensable**: removing any cache degrades performance, never correctness.

---

## The 6 Rules

### R1. Storage: Resource or Component(SparseSet)

Every cache MUST be a `Resource` (global) or a `Component` with `#[component(storage = "SparseSet")]`
(per-entity). Never hidden state (`static`, `thread_local`, captured closures).

**Current inventory:**

| Cache | Kind | Storage |
|-------|------|---------|
| `BridgeCache<B>` | Resource (generic, 12 instances) | `Resource` |
| `KleiberCache` | Per-entity | `Component(SparseSet)` |
| `GompertzCache` | Per-entity | `Component(SparseSet)` |
| `MaterializationCellCache` | Grid-parallel | `Resource` |
| `GeometryDeformationCache` | Grid-parallel | `Resource` |
| `PerformanceCachePolicy` | Per-entity (shape inference) | `Component` |

### R2. Dispensable — zero-loss or recomputable

Clearing a cache at any tick MUST produce identical results on the next computation.
No "cache-only state" — everything is derivable from canonical components + resources.

**Validation:** `BridgeCache` doc line 1: *"dispensable Resource (no effect on results if cleared)"*.
`KleiberCache`: NaN sentinel forces recompute after clear. `GompertzCache`: write-once at spawn.

### R3. Explicit invalidation

Every cache MUST declare its invalidation mechanism. Valid mechanisms:

| Mechanism | When to use | Example |
|-----------|-------------|---------|
| **On-change** | Input changes rarely | `KleiberCache` (radius) |
| **Write-once** | Input immutable after spawn | `GompertzCache` (birth params) |
| **Fingerprint** | Complex input, cheap hash | `MaterializationCellCache` (u64 sig) |
| **LRU** | Limited space, variable access | `BridgeCache<B>` (last_used counter) |
| **Range** | Continuous input, known tolerance | `GeometryDeformationCache` (tensor range) |
| **Signature** | Composite input, u16 compact | `PerformanceCachePolicy` (dependency_signature) |

**Forbidden:** Temporal TTL (no wall-clock in sim). Time-based expiry is meaningless
in a deterministic system with discrete ticks.

### R4. Minimum metrics: hits + misses

Every cache MUST expose at least `hits` and `misses` as fields accessible via `Res`.

| Cache | Metrics | Status |
|-------|---------|--------|
| `BridgeCache<B>` | `CacheStats { hits, misses, evictions, hit_rate }` | OK |
| `MaterializationCellCache` | `MatCacheStats { hits, misses }` | OK |
| `GeometryDeformationCache` | Per-slot `hits, misses` | OK |
| `KleiberCache` | — | No counters (update() returns bool, does not accumulate) |
| `GompertzCache` | — | N/A (write-once, hit rate = 100% by design) |

**Acceptable exception:** Write-once caches (`GompertzCache`) or on-change caches with
negligible miss frequency (`KleiberCache` — only on growth events) do not require counters.

### R5. Calibrated budgets in WorldgenPerfSettings

Per-tick/frame budgets MUST have real calibrated values, not `u32::MAX`.
Base calibration: grid ≤128x128 (~16K cells) at 60 Hz.

| Budget | Value | Rationale |
|--------|-------|-----------|
| `max_material_spawn_per_tick` | 64 | ~2% of near band per tick; amortizes spawn cost |
| `max_material_despawn_per_tick` | 64 | Symmetric with spawn |
| `max_propagation_cell_writes_per_tick` | 256 | ~10% of near band; incremental propagation |
| `max_visual_derivation_per_frame` | 128 | Mesh rebuild is expensive; 128 ~ 2 ms budget at 60 Hz |

For larger grids, insert a `WorldgenPerfSettings` with proportionally scaled values.
Tests may insert low values (e.g. `max_material_spawn_per_tick: 2`) to verify throttling.

### R6. Regression benchmarks in CI

`cargo bench` MUST run in CI. Alert threshold: >10% throughput degradation
relative to the `main` branch baseline.

**Existing benchmarks:**

| Benchmark | What it measures |
|-----------|-----------------|
| `bridge_benchmark` | Hit rates per bridge (target: Density 98%, Phase 99%, Interference 72%) |
| `worldgen_field_perf` | Materialization (100 cells), dissipation (10K), freq resolution (10K) |
| `batch_benchmark` | Batch simulator throughput (ticks/sec, worlds/sec) |

---

## New cache checklist

1. **Dispensable?** Clearing the cache does not change results. If not, it's state, not cache.
2. **Correct storage?** `Resource` (global) or `Component(SparseSet)` (per-entity).
3. **Invalidation declared?** On-change / write-once / fingerprint / LRU / range / signature.
4. **Metrics?** hits + misses accessible (except trivial write-once).
5. **Benchmark?** New bench or coverage in an existing bench.
6. **Registered?** `app.register_type::<T>()` if `Component` with `Reflect`.

---

## Policy enum

`CachePolicy` has two active variants:

```rust
pub enum CachePolicy {
    Lru,         // Eviction by last_used (default)
    ContextFill, // Eviction disabled during warmup (B7)
}
```

Do not add variants without an implementation in `BridgeCache`. If LFU is needed in the future,
implement it first, then add the variant.
