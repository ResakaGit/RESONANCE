# Sprint EC-6 — Conservation Ledger: Contabilidad Energética de Pools

**Módulo:** `src/layers/pool_ledger.rs` + `src/simulation/metabolic/pool_conservation.rs`
**Tipo:** Componente SparseSet + sistema ECS de verificación.
**Onda:** C — Requiere EC-4. Paralelo con EC-5.
**Estado:** ⏳ Pendiente

## Objetivo

Implementar el libro contable de conservación energética para pools: tracking de intake, extracción total, disipación, y delta por tick. Verificar post-tick que el pool invariant se cumple. Análogo al `EntropyLedger` (MG-6) pero para el dominio de pools jerárquicos.

## Principio

> El ledger no es estado — es un espejo derivado del último tick de distribución. Recomputado, nunca persistido como verdad. Su función es doble: observabilidad (debug, UI) y enforcement (debug asserts de conservación).

## Responsabilidades

### EC-6A: Componente `PoolConservationLedger`

```rust
/// Libro contable de conservación de pool. Recomputado cada tick.
/// Solo entidades con EnergyPool activo.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PoolConservationLedger {
    /// Energía total extraída por hijos este tick (qe).
    pub total_extracted: f32,
    /// Energía disipada este tick (segunda ley).
    pub total_dissipated: f32,
    /// Delta neto del pool este tick: intake - extracted - dissipated.
    pub net_delta: f32,
    /// Número de hijos activos que extrajeron.
    pub active_children: u16,
}
```

- 4 campos (3 × f32 + 1 × u16 + padding) — cumple DOD max 4.
- `SparseSet`: solo pools con actividad.
- `active_children` es `u16`: suficiente para 65535 hijos (MAX_CHILDREN_PER_POOL = 64).
- Derivado cada tick. No es fuente de verdad.

### EC-6B: Sistema `pool_conservation_system`

```rust
/// Verifica conservación y escribe PoolConservationLedger.
/// Ejecuta DESPUÉS de pool_distribution_system.
pub fn pool_conservation_system(
    pools: Query<(Entity, &EnergyPool)>,
    children: Query<&PoolParentLink, Without<Dead>>,
    mut ledgers: Query<&mut PoolConservationLedger>,
    mut commands: Commands,
)
```

**Algoritmo por pool:**

1. Contar hijos activos y sumar extracciones del tick (leyendo `BaseEnergy` delta o recalculando desde `evaluate_extraction`).
2. Calcular disipación: `dissipation_loss(pool_prev, dissipation_rate)`.
3. `net_delta = intake_rate - total_extracted - total_dissipated`.
4. Construir `PoolConservationLedger`.
5. Guard: `if old != new { *old = new; }`.
6. **Debug assert:** `total_extracted + total_dissipated <= pool_prev + intake_rate + POOL_CONSERVATION_EPSILON`.

- **Phase:** `Phase::MetabolicLayer`, `.after(pool_distribution_system)`.
- Violación del assert en debug = bug en EC-4 (distribution no respetó el invariant).

### EC-6C: Snapshot de Pool (para verificación)

Para verificar conservación sin re-evaluar extracciones, el sistema de distribución (EC-4) debe comunicar los valores al ledger. Dos opciones:

**Opción A (recomendada):** EC-4 escribe `PoolConservationLedger` directamente después de distribuir. EC-6 solo verifica y adiciona debug asserts. Esto evita re-cómputo.

**Opción B:** EC-6 re-evalúa desde el estado del pool (pool_prev, pool_now, intake). `total_extracted = pool_prev + intake - pool_now - dissipation`. Derivación algebraica, no re-evaluación de funciones.

Elegir en el PR. Ambas son correctas; Opción A es más eficiente.

### EC-6D: Función pura de verificación

```rust
/// Verifica la conservación de un pool tick.
/// Retorna error relativo: (intake - extracted - dissipated - delta) / max(pool, EPSILON).
pub fn conservation_error(
    pool_before: f32,
    pool_after: f32,
    intake: f32,
    total_extracted: f32,
    total_dissipated: f32,
) -> f32
```

- Error = `(pool_before + intake - total_extracted - total_dissipated - pool_after).abs() / max(pool_before, EPSILON)`.
- Esperado: `< POOL_CONSERVATION_EPSILON` siempre.
- Si error > epsilon → bug en distribución.

```rust
/// Verifica el pool invariant global: Sigma qe(children) <= pool(parent).
pub fn verify_pool_invariant(
    pool: f32,
    children_qe: &[f32],
) -> bool
```

- `sum(children_qe) <= pool + POOL_CONSERVATION_EPSILON`.
- Nota: este invariante se verifica sobre las extracciones del tick, no sobre el qe total de los hijos (que puede incluir energía propia).

### EC-6E: Constantes

```rust
pub const POOL_CONSERVATION_EPSILON: f32 = 1e-3;   // Re-export de EC-1E si aplica
pub const CONSERVATION_WARNING_THRESHOLD: f32 = 1e-2;  // Warn en logs si error > este
```

## Tácticas

- **Análogo a EntropyLedger (MG-6C).** Mismo patrón: derivado, SparseSet, recomputado, guard change detection. La diferencia es dominio: MG-6 = DAG metabólico; EC-6 = pool jerárquico.
- **Debug asserts, no panics en release.** Violación de conservación en debug = breakpoint inmediato. En release = clamp silencioso + metric. El juego no crashea por floating point drift.
- **El ledger es readonly para gameplay.** Ningún sistema de gameplay debe leer `PoolConservationLedger` para tomar decisiones. Es para debug/observabilidad. `PoolDiagnostic` (EC-5) es el componente de diagnóstico para gameplay.

## NO hace

- No modifica la distribución de energía (eso es EC-4).
- No modifica pools ni links.
- No emite eventos de colapso (eso sería un sistema de gameplay).
- No implementa conservation-by-construction (eso ya lo hace `scale_extractions_to_available` en EC-1/EC-4).

## Criterios de aceptación

### EC-6A (Ledger)
- Test: `PoolConservationLedger` es `Copy`.
- Test: 4 campos, `SparseSet`, `Reflect`.
- Test: `size_of::<PoolConservationLedger>()` <= 16 bytes.

### EC-6B (Sistema)
- Test: app mínima, 1 pool + 2 hijos → ledger insertado con `total_extracted` = sum(claimed).
- Test: `net_delta = intake - extracted - dissipated` exacto.
- Test: idempotente — si nada cambia, ledger no muta.

### EC-6D (Verificación)
- Test: `conservation_error(1000, 840, 50, 200, 10) = 0.0` (perfecto).
- Test: `conservation_error(1000, 850, 50, 200, 10)` → error > 0 (10 qe de drift).
- Test: `verify_pool_invariant(1000, &[300, 300, 300])` → true.
- Test: `verify_pool_invariant(1000, &[500, 400, 200])` → false.

### Conservación end-to-end
- Test: escenario 1 pool + 4 hijos, 100 ticks → `conservation_error < EPSILON` en cada tick.
- Test: escenario con Type IV (aggressive) → conservation OK porque daño es a capacity, no a conservation.
- Test: stress: 10 pools, 50 hijos, 1000 ticks → zero conservation violations.

### General
- `cargo test --lib` sin regresión.
- >=15 tests unitarios.

## Referencias

- Blueprint Energy Competition Layer §7 (Invariants the Engine Must Enforce)
- `src/layers/entropy_ledger.rs` — Precedente directo (MG-6C)
- `src/simulation/metabolic/morphogenesis.rs` — `entropy_ledger_system` como patrón
- EC-4 (sistema que produce los datos que el ledger contabiliza)
- EC-1 (`dissipation_loss`, `POOL_CONSERVATION_EPSILON`)
