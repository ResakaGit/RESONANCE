# Sprint EC-2 — Pool Components: Tipos ECS para Pools Jerárquicos

**Módulo:** `src/layers/energy_pool.rs` + `src/layers/pool_link.rs`
**Tipo:** Componentes ECS. Datos de pool y relación padre-hijo.
**Onda:** A — Requiere EC-1 (constantes y validación).
**Estado:** ⏳ Pendiente

## Objetivo

Definir los componentes ECS que representan: (1) un pool de energía distribuible a hijos, (2) el vínculo padre-hijo con tipo de extracción. Estos componentes extienden — no reemplazan — `BaseEnergy`. Una entidad puede tener `BaseEnergy` (su qe propio) + `EnergyPool` (qe distribuible a hijos).

## Responsabilidades

### EC-2A: Componente `EnergyPool`

```rust
/// Pool de energía distribuible a entidades hijas.
/// Invariante: Sigma extracted(children) <= pool por tick.
/// Ortogonal a BaseEnergy: una entidad puede tener ambos.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
pub struct EnergyPool {
    /// Energía disponible para distribución este tick (qe).
    pool: f32,
    /// Capacidad máxima estructural del pool (qe). Degradable por Type IV.
    capacity: f32,
    /// Energía adquirida por tick desde ambiente o padre (qe/tick).
    intake_rate: f32,
    /// Tasa de disipación obligatoria (segunda ley). En (0, 1).
    dissipation_rate: f32,
}
```

- 4 campos — cumple regla DOD.
- `pool` clamped a `[0.0, capacity]`.
- `capacity` clamped a `[POOL_CAPACITY_MIN, f32::MAX]`.
- `dissipation_rate` clamped a `(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX)`.
- `intake_rate` clamped a `>= 0.0`.

**Métodos:**

```rust
impl EnergyPool {
    pub fn new(pool: f32, capacity: f32, intake_rate: f32, dissipation_rate: f32) -> Self;
    pub fn pool(&self) -> f32;
    pub fn capacity(&self) -> f32;
    pub fn intake_rate(&self) -> f32;
    pub fn dissipation_rate(&self) -> f32;
    pub fn pool_ratio(&self) -> f32;  // pool / max(capacity, EPSILON)

    pub fn set_pool(&mut self, val: f32);       // clamp [0, capacity]
    pub fn set_capacity(&mut self, val: f32);    // clamp [POOL_CAPACITY_MIN, MAX]
    pub fn set_intake_rate(&mut self, val: f32); // clamp >= 0
    pub fn degrade_capacity(&mut self, amount: f32);  // capacity -= amount, clamp
    pub fn replenish(&mut self, amount: f32);    // pool += amount, clamp to capacity
}
```

- `pool_ratio()` es derivado — no se almacena. Computado en punto de uso (DOD: no guardar derivados).
- `degrade_capacity()` para Type IV (aggressive extraction con pool damage).
- Registrar en `LayersPlugin` con `Reflect`.

### EC-2B: Componente `PoolParentLink`

```rust
/// Vínculo de extracción: esta entidad extrae energía del pool padre.
/// SparseSet: solo entidades en jerarquía activa.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PoolParentLink {
    /// Entidad padre cuyo EnergyPool es la fuente.
    parent: Entity,
    /// Tipo de función de extracción (índice en ExtractionType enum).
    extraction_type: ExtractionType,
    /// Parámetro primario de la función de extracción.
    /// Semántica depende del tipo: fitness (III), aggression_factor (IV), base_rate (V).
    primary_param: f32,
}
```

- 3 campos — bajo el límite DOD.
- `parent` es `Entity` (referencia runtime, no ID persistente — OK para relaciones intra-tick).
- `extraction_type` enum cerrado (5 variantes).
- `primary_param` es el parámetro dominante de la función; parámetros secundarios viven en constantes o en componentes auxiliares si se necesitan.
- `SparseSet`: no todas las entidades participan en jerarquías.

### EC-2C: Enum `ExtractionType`

```rust
/// Las 5 funciones primitivas de extracción.
/// Cerrado: no `Box<dyn Trait>`, no trait objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum ExtractionType {
    /// Type I: Fair share (pool / n_siblings).
    Proportional,
    /// Type II: Takes up to capacity limit.
    Greedy,
    /// Type III: Share proportional to relative fitness.
    Competitive,
    /// Type IV: Extracts and damages parent capacity.
    Aggressive,
    /// Type V: Self-regulates based on parent pool state.
    Regulated,
}
```

- Exhaustive matching obligatorio (no `_ =>` — compiler catches new variants).
- Reflect derivado para inspector/debug.
- NO `String`, NO trait objects. Enum cerrado.

### EC-2D: Re-exports

En `src/layers/mod.rs`:

```rust
pub mod energy_pool;
pub mod pool_link;

pub use energy_pool::EnergyPool;
pub use pool_link::{PoolParentLink, ExtractionType};
```

## Tácticas

- **Ortogonalidad con BaseEnergy.** `EnergyPool` es un pool distribuible; `BaseEnergy` es el qe propio de la entidad. Un árbol puede tener `BaseEnergy=300` (su salud) + `EnergyPool=5000` (pool que distribuye a semillas/frutos). Son independientes.
- **Entity como parent ref es OK aquí.** El link es runtime-only, verificado cada tick. Si el parent muere, el sistema detecta entidad inválida y limpia el link (observer pattern o guard en sistema).
- **primary_param semántica por tipo:**
  - Proportional: no usado (puede ser 0.0).
  - Greedy: `capacity` del hijo (cuánto puede tomar).
  - Competitive: `fitness` del hijo.
  - Aggressive: `aggression_factor`.
  - Regulated: `base_rate`.
- **Parámetros secundarios (damage_rate, thresholds) en constantes o componente auxiliar.** Si entidades necesitan parámetros individuales, EC-3 introduce `ExtractionParams` component. V1 usa constantes globales.

## NO hace

- No implementa la lógica de distribución (eso es EC-4).
- No implementa composición HoF (eso es EC-3).
- No modifica `BaseEnergy` ni `EnergyOps`.
- No crea sistemas.
- No toca el pipeline.

## Criterios de aceptación

### EC-2A (EnergyPool)
- Test: `EnergyPool::new(500.0, 1000.0, 50.0, 0.01)` — campos correctos.
- Test: `pool` clamped a capacity: `EnergyPool::new(2000.0, 1000.0, ...)` → `pool = 1000`.
- Test: `dissipation_rate` clamped: `EnergyPool::new(..., 0.0)` → `dissipation_rate = DISSIPATION_RATE_MIN`.
- Test: `pool_ratio()` = `pool / capacity` para caso normal.
- Test: `pool_ratio()` con capacity = 0 → no NaN (clamped por `POOL_CAPACITY_MIN`).
- Test: `degrade_capacity(100.0)` reduce capacity, clamps pool si excede nueva capacity.
- Test: `replenish(500.0)` no excede capacity.
- Test: `size_of::<EnergyPool>()` = `4 * 4 = 16 bytes`.

### EC-2B (PoolParentLink)
- Test: `PoolParentLink` es `Copy`.
- Test: todos los campos accesibles via getters.
- Test: `SparseSet` storage (verified by compile + runtime query).

### EC-2C (ExtractionType)
- Test: 5 variantes exhaustivas en match (compile-time via `match` sin `_`).
- Test: `ExtractionType` es `Copy`, `Eq`, `Hash`.
- Test: `size_of::<ExtractionType>()` = 1 byte (u8 discriminant).

### EC-2D (Re-exports)
- Test: `use crate::layers::EnergyPool` compila.
- Test: `use crate::layers::PoolParentLink` compila.
- Test: `use crate::layers::ExtractionType` compila.

### General
- `cargo test --lib` sin regresión.
- Componentes registrados en `LayersPlugin` con `Reflect`.

## Referencias

- Blueprint Energy Competition Layer §1 (Pool Model)
- `src/layers/energy.rs` — `BaseEnergy` como patrón (1 campo, getters, setters con guard)
- `src/layers/containment.rs` — `ContainedIn` como patrón de referencia a Entity
- `src/layers/entropy_ledger.rs` — `SparseSet` + `Reflect` como patrón
- `src/blueprint/constants/` — constantes de EC-1E
