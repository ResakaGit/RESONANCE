# Sprint EC-5 — Competition Dynamics: Matriz, Equilibrio, Dominancia, Colapso

**Módulo:** `src/blueprint/equations/energy_competition/dynamics.rs` + `src/simulation/metabolic/competition_dynamics.rs`
**Tipo:** Funciones puras (análisis) + sistema ECS (detección runtime).
**Onda:** C — Requiere EC-4 (distribución funcional). Paralelo con EC-6.
**Estado:** ⏳ Pendiente

## Objetivo

Implementar las ecuaciones de dinámica competitiva: matriz de competencia N×N, detección de equilibrio, condición de dominancia estable (ESS), condición de colapso de host, y estimación de ticks hasta colapso. Provee diagnóstico analítico del estado competitivo de cada pool — no modifica la distribución (eso es EC-4).

## Responsabilidades

### EC-5A: Matriz de Competencia (funciones puras)

```rust
/// Calcula la matriz de competencia C[i][j] para N hijos de un pool.
/// C[i][j] = efecto de la extracción de hijo i sobre la energía disponible de hijo j.
/// Diagonal C[i][i] = energía neta retenida por hijo i.
pub fn competition_matrix(
    extractions: &[f32],
    available: f32,
) -> [[f32; MAX_COMPETITION_MATRIX]; MAX_COMPETITION_MATRIX]
```

- `MAX_COMPETITION_MATRIX = 16` (constante). Pools con >16 hijos: partición o aproximación.
- `C[i][j] = -extractions[i] / max(available, EPSILON)` para `i != j`.
- `C[i][i] = extractions[i] - extractions[i] * sum(extractions[k!=i]) / max(available, EPSILON)`.
- Stack-allocated `[[f32; 16]; 16]`. Sin heap.
- Simétrica para Type I (Proportional). Asimétrica para Type III (Competitive).

```rust
/// Índice de competencia: qué tan desigual es la distribución.
/// 0.0 = perfectamente equitativo, 1.0 = un solo hijo toma todo.
/// Gini coefficient sobre las extracciones.
pub fn competition_intensity(extractions: &[f32]) -> f32
```

- Gini = `sum(|x_i - x_j|) / (2 * n * sum(x_i))`.
- Guard: todos 0 → retorna 0.0.
- Rango: `[0.0, 1.0]`.

### EC-5B: Detección de Equilibrio y Dominancia

```rust
/// ¿El pool está en equilibrio estable?
/// Requiere: intake = Sigma extract + loss (dentro de epsilon).
pub fn detect_equilibrium(
    intake: f32,
    total_extracted: f32,
    dissipation_loss: f32,
    epsilon: f32,
) -> bool
```

- Wrapper de `is_pool_equilibrium` (EC-1) con contexto adicional.

```rust
/// ¿Hijo i tiene dominancia estable (ESS)?
/// Condición: extract(i) > extract(j) para todo j != i, Y pool viable.
pub fn detect_dominance(
    extractions: &[f32],
    dominant_index: usize,
    pool_viable: bool,
) -> bool
```

- `pool_viable = pool > POOL_VIABILITY_THRESHOLD`.
- Un ESS requiere que ningún competidor pueda invadir y extraer más.

```rust
/// ¿El pool está colapsando?
/// Detecta si el net drain excede intake + reserva.
pub fn detect_collapse(
    pool: f32,
    intake: f32,
    total_extracted: f32,
    loss: f32,
) -> PoolHealthStatus
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolHealthStatus {
    Healthy,
    Stressed,      // pool_ratio < 0.3
    Collapsing,    // net drain > 0, ticks_to_collapse < COLLAPSE_WARNING_TICKS
    Collapsed,     // pool = 0
}
```

### EC-5C: Predicción de Trayectoria

```rust
/// Estima la trayectoria del pool: si sigue drenando al rate actual, cuándo colapsa.
pub fn predict_pool_trajectory(
    pool: f32,
    net_drain_per_tick: f32,
    capacity: f32,
) -> PoolTrajectory
```

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoolTrajectory {
    /// Ticks estimados hasta colapso (u32::MAX si estable/creciente).
    pub ticks_to_collapse: u32,
    /// Ticks estimados hasta capacidad máxima (u32::MAX si drenando).
    pub ticks_to_full: u32,
    /// Rate de cambio por tick.
    pub net_change_per_tick: f32,
}
```

- Predicción lineal (v1). Modelos más sofisticados post-v1.

### EC-5D: Sistema `competition_dynamics_system`

```rust
/// Analiza la dinámica competitiva de cada pool y escribe diagnósticos.
/// Read-only sobre pools y links. Escribe PoolDiagnostic component.
pub fn competition_dynamics_system(
    pools: Query<(Entity, &EnergyPool)>,
    children: Query<(&PoolParentLink, &BaseEnergy), Without<Dead>>,
    mut diagnostics: Query<&mut PoolDiagnostic>,
    mut commands: Commands,
)
```

- **Phase:** `Phase::MetabolicLayer`, `.after(pool_distribution_system)`.
- Por cada pool: recolectar extracciones de hijos, computar `competition_intensity`, `detect_collapse`, `predict_pool_trajectory`.
- Escribir resultado en `PoolDiagnostic` component (SparseSet, en la entidad pool).
- Guard change detection: `if old != new`.

### EC-5E: Componente `PoolDiagnostic`

```rust
/// Diagnóstico competitivo del pool. Recomputado cada tick.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PoolDiagnostic {
    /// Intensidad de competencia (Gini). [0, 1].
    pub competition_intensity: f32,
    /// Estado de salud del pool.
    pub health_status: PoolHealthStatus,
    /// Ticks estimados hasta colapso.
    pub ticks_to_collapse: u32,
}
```

- 3 campos (DOD: bajo límite).
- `SparseSet`: solo pools con hijos activos.
- Derivado — no es estado persistente. Recomputado cada tick.

### EC-5F: Constantes

```rust
pub const MAX_COMPETITION_MATRIX: usize = 16;
pub const POOL_VIABILITY_THRESHOLD: f32 = 10.0;       // qe mínimo para ser viable
pub const COLLAPSE_WARNING_TICKS: u32 = 100;           // umbral para Collapsing status
pub const POOL_STRESSED_RATIO: f32 = 0.3;              // pool_ratio bajo = stressed
```

## Tácticas

- **La matriz es diagnóstica, no causal.** EC-5 analiza; EC-4 distribuye. La matriz no modifica la distribución. Es para observabilidad, debugging, y sistemas downstream que reaccionen a competencia.
- **Stack-allocated matrix.** `[f32; 16][16] = 1024 bytes`. Cabe en stack. Sin heap para N <= 16.
- **Gini es O(N²).** Con N <= 16, son 256 operaciones. Aceptable cada tick.
- **PoolHealthStatus como enum, no String.** 4 variantes → exhaustive match, sin allocations.

## NO hace

- No modifica distribución de energía (eso es EC-4).
- No modifica pools ni links.
- No implementa respuesta a colapso (eso sería un sistema de gameplay futuro).
- No implementa migración (re-parenting post-colapso). Extensión post-v1.

## Criterios de aceptación

### EC-5A (Matriz)
- Test: 3 hijos con extracción uniforme [100, 100, 100] → `competition_intensity = 0.0`.
- Test: 3 hijos con extracción [300, 0, 0] → `competition_intensity = ~0.67`.
- Test: `competition_matrix` simétrica para extracciones iguales.
- Test: diagonal positiva (retención neta).

### EC-5B (Equilibrio/Dominancia)
- Test: `detect_equilibrium(100, 90, 10, 1e-3) = true`.
- Test: `detect_dominance(&[500, 300, 200], 0, true) = true`.
- Test: `detect_dominance(&[500, 300, 200], 0, false) = false` (pool no viable).
- Test: `detect_collapse(0, ...)` → `Collapsed`.
- Test: `detect_collapse(1000, 50, 200, 10)` → `Healthy` (intake covers).
- Test: `detect_collapse(100, 10, 200, 10)` → `Collapsing`.

### EC-5C (Predicción)
- Test: `predict_pool_trajectory(1000, 100, 2000)` → `ticks_to_collapse = 10`.
- Test: `predict_pool_trajectory(1000, -50, 2000)` → `ticks_to_full = 20`.
- Test: `net_drain = 0` → `ticks_to_collapse = u32::MAX`.

### EC-5D (Sistema)
- Test: app mínima, 1 pool + 3 hijos → `PoolDiagnostic` insertado con valores correctos.
- Test: idempotente — si nada cambia, `PoolDiagnostic` no muta.

### General
- `cargo test --lib` sin regresión.
- >=20 tests unitarios.

## Referencias

- Blueprint Energy Competition Layer §3 (Competition Dynamics), §3.1–§3.5
- `src/blueprint/equations/ecology/` — Precedente de análisis competitivo
- EC-4 (sistema de distribución que produce las extracciones analizadas)
- EC-1 (`is_pool_equilibrium`, `is_host_collapsing`, `ticks_to_collapse`)
