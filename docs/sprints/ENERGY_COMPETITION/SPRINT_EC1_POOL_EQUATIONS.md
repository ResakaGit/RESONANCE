# Sprint EC-1 — Pool Equations: Matemática Pura de Pools y Extracción

**Módulo:** `src/blueprint/equations/energy_competition/` + `src/blueprint/constants/`
**Tipo:** Funciones puras sin ECS. Fundamento matemático de todo el track.
**Onda:** 0 — Bloqueante para todos los demás sprints.
**Estado:** ⏳ Pendiente

## Objetivo

Implementar las ecuaciones puras que formalizan: conservación de pool por tick, piso de disipación (segunda ley), las 5 funciones de extracción (Proporcional, Greedy, Competitiva, Agresiva, Regulada), fitness relativo, y condiciones de equilibrio/colapso. Todas en `blueprint/equations/energy_competition/` como funciones `pub fn` sin dependencias ECS.

## Responsabilidades

### EC-1A: Ecuaciones de Conservación de Pool

- `pool_next_tick(pool: f32, intake: f32, total_extracted: f32, dissipation_rate: f32) -> f32`
  - `pool(t+1) = (pool + intake - total_extracted - pool * dissipation_rate).max(0.0)`
  - Guard: `dissipation_rate` clamped a `(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX)`.
  - Guard: `total_extracted` clamped a `pool + intake` (no se extrae más de lo disponible).
  - Invariante: retorno >= 0.

- `dissipation_loss(pool: f32, dissipation_rate: f32) -> f32`
  - `loss = pool * dissipation_rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX)`
  - La segunda ley: `loss > 0` siempre que `pool > 0`.
  - Guard: `pool < 0` → 0.

- `available_for_extraction(pool: f32, intake: f32, dissipation_rate: f32) -> f32`
  - `available = (pool + intake - dissipation_loss(pool, dissipation_rate)).max(0.0)`
  - Lo que queda después de la disipación obligatoria. Los hijos compiten por esto.

### EC-1B: Funciones de Extracción (5 Tipos)

- `extract_proportional(available: f32, n_siblings: u32) -> f32`
  - **Type I — Fair Share:** `available / max(n_siblings, 1)`
  - Estable por construcción. Sin ventaja posicional.
  - Guard: `n_siblings = 0` → retorna `available` (hijo único).

- `extract_greedy(available: f32, capacity: f32) -> f32`
  - **Type II — Capacity-Bounded:** `min(available, capacity.max(0.0))`
  - Toma todo lo que puede hasta su límite estructural.
  - Order-dependent: primero en extraer tiene ventaja. El sistema resuelve esto.

- `extract_competitive(available: f32, fitness: f32, total_fitness: f32) -> f32`
  - **Type III — Relative Fitness:** `available * fitness / max(total_fitness, EPSILON)`
  - Zero-sum: ganancia de uno = pérdida de otro.
  - Guard: `fitness < 0` → clamp 0. `total_fitness <= 0` → retorna 0.
  - Invariante: `extract(i) / extract(j) = fitness(i) / fitness(j)`.

- `extract_aggressive(available: f32, aggression_factor: f32, damage_rate: f32) -> (f32, f32)`
  - **Type IV — Pool-Damaging:** retorna `(taken, pool_damage)`.
  - `taken = available * aggression_factor.clamp(0.0, 1.0)`
  - `pool_damage = taken * damage_rate.clamp(0.0, 1.0)`
  - `pool_damage` reduce la **capacidad** del padre (degradación estructural).
  - Auto-terminante si no se controla: destruye la fuente.

- `extract_regulated(available: f32, pool_ratio: f32, base_rate: f32, threshold_low: f32, threshold_high: f32) -> f32`
  - **Type V — Homeostatic:**
    ```
    if pool_ratio > threshold_high → base_rate * REGULATED_AGGRESSIVE_MULT
    if pool_ratio in [threshold_low, threshold_high] → base_rate
    if pool_ratio < threshold_low → base_rate * REGULATED_THROTTLE_MULT
    ```
  - `pool_ratio = pool / max(capacity, EPSILON)`.
  - Produce homeostasis emergente. Requiere feedback del estado del padre.
  - Guard: `base_rate < 0` → clamp 0. Thresholds clamped a `[0, 1]`.

### EC-1C: Fitness y Scaling

- `relative_fitness(fitness: f32, sibling_fitnesses: &[f32]) -> f32`
  - `ratio = fitness / max(sum(sibling_fitnesses), EPSILON)`
  - Rango: `[0.0, 1.0]`.
  - Guard: todos los fitness = 0 → retorna `1.0 / max(n, 1)` (fallback proporcional).

- `scale_extractions_to_available(extractions: &mut [f32], available: f32)`
  - Si `sum(extractions) > available`: escalar proporcionalmente para que `sum = available`.
  - `factor = available / max(sum, EPSILON)`.
  - `extractions[i] *= factor` para todo i.
  - Invariante post: `sum(extractions) <= available + EPSILON`.
  - In-place mutation en slice (no allocations).

### EC-1D: Condiciones de Estado

- `is_pool_equilibrium(intake: f32, total_extracted: f32, loss: f32, epsilon: f32) -> bool`
  - `(intake - total_extracted - loss).abs() < epsilon`
  - El pool no cambia. Estado estable.

- `is_host_collapsing(pool: f32, intake: f32, total_extracted: f32, loss: f32) -> bool`
  - `total_extracted + loss > intake + pool`
  - El pool se vaciará este tick. Condición de colapso.

- `ticks_to_collapse(pool: f32, net_drain_per_tick: f32) -> u32`
  - `if net_drain_per_tick <= 0.0 { return u32::MAX; }` (no colapsa)
  - `(pool / net_drain_per_tick).ceil() as u32`
  - Estimación determinista de ticks restantes.

### EC-1E: Constantes

```rust
// --- Energy Competition: Pool ---
pub const DISSIPATION_RATE_MIN: f32 = 0.001;      // Piso: 0.1% por tick mínimo
pub const DISSIPATION_RATE_MAX: f32 = 0.5;         // Techo: 50% por tick máximo
pub const DISSIPATION_RATE_DEFAULT: f32 = 0.01;    // Default: 1% por tick
pub const POOL_CAPACITY_MIN: f32 = 1.0;            // Capacidad mínima viable

// --- Energy Competition: Extraction ---
pub const EXTRACTION_EPSILON: f32 = 1e-6;          // Tolerancia para divisiones
pub const REGULATED_AGGRESSIVE_MULT: f32 = 1.5;    // Multiplicador agresivo (Type V)
pub const REGULATED_THROTTLE_MULT: f32 = 0.3;      // Multiplicador throttle (Type V)
pub const REGULATED_THRESHOLD_LOW_DEFAULT: f32 = 0.3;
pub const REGULATED_THRESHOLD_HIGH_DEFAULT: f32 = 0.7;
pub const AGGRESSION_FACTOR_DEFAULT: f32 = 0.5;
pub const DAMAGE_RATE_DEFAULT: f32 = 0.1;

// --- Energy Competition: Conservation ---
pub const POOL_CONSERVATION_EPSILON: f32 = 1e-3;   // Tolerancia de conservación en debug assert
```

## Tácticas

- **Reusar pattern de MG-1.** Funciones O(1) aritmética, sin BridgeCache (se añade en EC-4 si necesario).
- **`scale_extractions_to_available` es la clave.** Garantiza el pool invariant sin importar qué funciones de extracción usen los hijos. Es el enforcement point.
- **No modelar orden de extracción aquí.** Eso es decisión del sistema (EC-4). Las ecuaciones son order-independent: cada una recibe `available` y retorna `claimed`.
- **Test-driven.** Tests ANTES de implementación. Los invariantes físicos son el contrato.
- **Stack-only.** `&[f32]` slices, no `Vec`. El caller (sistema EC-4) provee el buffer.

## NO hace

- No crea componentes ECS (eso es EC-2).
- No crea sistemas (eso es EC-4+).
- No modifica ecuaciones existentes (`trophic`, `ecology`, `competitive_exclusion` permanecen intactas).
- No toca el pipeline de simulación.
- No introduce dependencias de crates nuevos.
- No define la composición HoF de funciones de extracción (eso es EC-3).

## Criterios de aceptación

### EC-1A (Conservación)
- Test: `pool_next_tick(1000.0, 50.0, 200.0, 0.01) = 1000 + 50 - 200 - 10 = 840.0`.
- Test: `pool_next_tick(100.0, 0.0, 200.0, 0.01)` → clamp, no extrae más de disponible.
- Test: `dissipation_loss(1000.0, 0.01) = 10.0`.
- Test: `dissipation_loss(0.0, 0.01) = 0.0`.
- Test: `dissipation_loss(1000.0, 0.0)` → clamped a `DISSIPATION_RATE_MIN`.
- Test: `available_for_extraction(1000.0, 50.0, 0.01) = 1040.0` (1000+50-10).

### EC-1B (Extracción)
- Test: `extract_proportional(1000.0, 4) = 250.0`.
- Test: `extract_proportional(1000.0, 0) = 1000.0`.
- Test: `extract_greedy(1000.0, 500.0) = 500.0`.
- Test: `extract_greedy(1000.0, 2000.0) = 1000.0` (clamped).
- Test: `extract_competitive(1000.0, 0.6, 1.0) = 600.0`.
- Test: `extract_competitive(1000.0, 0.0, 1.0) = 0.0`.
- Test: `extract_competitive(1000.0, 0.5, 0.0) = 0.0` (no fitness total).
- Test: `extract_aggressive(1000.0, 0.5, 0.1)` → `(500.0, 50.0)`.
- Test: `extract_aggressive(1000.0, 0.0, 0.1)` → `(0.0, 0.0)`.
- Test: `extract_regulated(1000.0, 0.8, 100.0, 0.3, 0.7) = 150.0` (aggressive zone).
- Test: `extract_regulated(1000.0, 0.5, 100.0, 0.3, 0.7) = 100.0` (normal zone).
- Test: `extract_regulated(1000.0, 0.1, 100.0, 0.3, 0.7) = 30.0` (throttle zone).

### EC-1C (Fitness/Scaling)
- Test: `relative_fitness(0.6, &[0.6, 0.3, 0.1]) = 0.6`.
- Test: `relative_fitness(0.0, &[0.0, 0.0]) = 0.5` (fallback).
- Test: `scale_extractions_to_available(&mut [600.0, 300.0, 100.0], 500.0)` → `[300.0, 150.0, 50.0]`.
- Test: `scale_extractions_to_available(&mut [100.0, 100.0], 500.0)` → sin cambio (suma < available).
- Test: post-scaling `sum <= available + EPSILON` para 50 combinaciones aleatorias.

### EC-1D (Condiciones)
- Test: `is_pool_equilibrium(100.0, 90.0, 10.0, 1e-3) = true`.
- Test: `is_pool_equilibrium(100.0, 50.0, 10.0, 1e-3) = false`.
- Test: `is_host_collapsing(100.0, 50.0, 200.0, 10.0) = true`.
- Test: `is_host_collapsing(1000.0, 50.0, 200.0, 10.0) = false`.
- Test: `ticks_to_collapse(1000.0, 100.0) = 10`.
- Test: `ticks_to_collapse(1000.0, 0.0) = u32::MAX`.

### General
- `cargo test --lib` pasa sin regresión.
- Todas las funciones tienen `///` doc-comments con la fórmula.
- Ninguna función accede a ECS, Bevy, o estado mutable global.
- >=30 tests unitarios.

## Referencias

- Blueprint Energy Competition Layer §1–§2 (Pool Model, Extraction Functions)
- `src/blueprint/equations/core_physics/` — `effective_dissipation()` como patrón
- `src/blueprint/equations/trophic.rs` — `prey_qe_transfer()` como precedente de extracción
- `src/blueprint/equations/ecology/` — `evolution_survival_score()` como precedente de fitness
- `src/blueprint/constants/` — estructura existente por dominio
