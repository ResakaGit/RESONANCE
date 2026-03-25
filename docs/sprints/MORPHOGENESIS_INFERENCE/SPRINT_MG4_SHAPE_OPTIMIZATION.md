# Sprint MG-4 — Shape Optimization (Constructal / Myring)

**Módulo:** `src/simulation/morphogenesis.rs` + `src/blueprint/equations/` + `src/bridge/` (opcional cache)
**Tipo:** Sistema ECS morfológico + función pura de descenso acotado + componente ≤4 campos.
**Onda:** C — Paralelo con MG-5 y MG-6; requiere MG-3.
**Estado:** ⏳ Pendiente

## Objetivo

Cerrar el loop **arrastre → costo de forma → geometría**: a partir de `FlowVector`, `AmbientPressure`, `SpatialVolume` y el estado del DAG (flujos/disipación ya actualizados en MG-3), ajustar el parámetro `fineness_ratio` que alimenta GF1 para minimizar `shape_cost` en el tiempo, usando `inferred_drag_coefficient`, `shape_cost` y `vascular_transport_cost` (MG-1) como oráculo.

**Resultado emergente:** una criatura en agua densa (ρ=1000, v=4) converge a fineness ~5.0 (fusiforme, C_D bajo). La misma criatura en aire (ρ=1.2, v=2) mantiene fineness ~2.0 (más compacta, menor presión de optimización).

## Responsabilidades

### MG-4A: Componente `MorphogenesisShapeParams`

```rust
/// Parámetros de forma inferidos por el optimizer.
/// Traducidos a `GeometryInfluence` por el código de worldgen/mesh.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct MorphogenesisShapeParams {
    /// Ratio largo/diámetro. 1.0 = esfera, >3.0 = fusiforme, >6.0 = torpedo.
    pub fineness_ratio: f32,
    /// Escala longitudinal (metros). Derivada de SpatialVolume.
    pub length_scale: f32,
    /// C_shape del tick actual (diagnóstico, no input).
    pub current_shape_cost: f32,
}
```

- 3 campos — cumple regla ≤4.
- SparseSet: solo entidades con `MetabolicGraph`.
- Registrar en `LayersPlugin` con `Reflect`.
- `fineness_ratio` es el control. `length_scale` y `current_shape_cost` son derivados/diagnóstico.

**Mapeo a GF1:**
```
fineness_ratio → length_budget = length_scale * fineness_ratio
                 radius_base   = length_scale / fineness_ratio
```
Invariante testeado: `length_budget / max(radius_base, ε) ≈ fineness_ratio²`. El código que arma `GeometryInfluence` aplica este mapeo (mismo patrón que `worldgen/shape_inference.rs`).

### MG-4B: Función pura de descenso acotado

```rust
/// Descenso acotado del fineness_ratio para minimizar shape_cost.
/// Máximo `max_iter` pasos por llamada; `damping` ∈ (0, 1] controla convergencia.
/// Retorna nuevo fineness_ratio clamped a [FINENESS_MIN, FINENESS_MAX].
pub fn bounded_fineness_descent(
    current_fineness: f32,
    medium_density: f32,
    velocity: f32,
    projected_area: f32,
    vascular_cost: f32,
    damping: f32,
    max_iter: u32,
) -> f32
```

- **Algoritmo:** gradiente numérico finite-difference. En cada iteración:
  1. `C_minus = shape_cost(ρ, v, inferred_drag_coefficient(L, D_minus), A, C_vasc)` con `fineness - δ`.
  2. `C_plus  = shape_cost(ρ, v, inferred_drag_coefficient(L, D_plus),  A, C_vasc)` con `fineness + δ`.
  3. `grad = (C_plus - C_minus) / (2δ)`.
  4. `fineness -= damping * grad`.
  5. Clamp a `[FINENESS_MIN, FINENESS_MAX]`.
- `δ = 0.1` (paso finite-difference). Constante en `constants.rs`.
- No adaptativo aleatorio. Sin RNG. Determinista.

### MG-4C: `shape_optimization_system`

```rust
/// Ajusta fineness_ratio minimizando shape_cost por descenso acotado.
pub fn shape_optimization_system(
    mut query: Query<
        (&MetabolicGraph, &FlowVector, &AmbientPressure, &SpatialVolume,
         &mut MorphogenesisShapeParams),
        Without<Dead>,
    >,
) {
    for (graph, flow, pressure, volume, mut shape) in &mut query {
        let velocity = flow.velocity().length();
        let density  = pressure.terrain_viscosity();  // proxy ρ del medio
        let radius   = volume.radius();
        let proj_area = std::f32::consts::PI * radius * radius;
        let vasc_cost = graph_vascular_cost(graph);    // Σ transport_cost de aristas

        let new_fineness = equations::bounded_fineness_descent(
            shape.fineness_ratio, density, velocity, proj_area, vasc_cost,
            constants::SHAPE_OPTIMIZER_DAMPING,
            constants::SHAPE_OPTIMIZER_MAX_ITER,
        );
        let new_cost = equations::shape_cost(
            density, velocity,
            equations::inferred_drag_coefficient(new_fineness * radius * 2.0, radius * 2.0),
            proj_area, vasc_cost,
        );

        if (shape.fineness_ratio - new_fineness).abs() > SHAPE_OPTIMIZER_EPSILON {
            shape.fineness_ratio = new_fineness;
            shape.length_scale = radius * 2.0;
            shape.current_shape_cost = new_cost;
        }
    }
}
```

- **Phase:** `Phase::MorphologicalLayer`.
- **Query:** 5 tipos (justificación: los 4 inputs + el output son todos necesarios y no se pueden derivar entre sí; `MorphogenesisShapeParams` es el componente que escribe).
- **Orden:** después de `metabolic_graph_step_system` (necesita DAG actualizado); antes de `surface_rugosity_system` y `albedo_inference_system` según contrato de pipeline del track.

### MG-4D: Constantes

```rust
// --- Morfogénesis: Shape Optimizer ---
pub const FINENESS_MIN: f32 = 1.0;                  // Esfera (mínimo compacto)
pub const FINENESS_MAX: f32 = 8.0;                  // Torpedo extremo
pub const FINENESS_DEFAULT: f32 = 1.5;              // Valor inicial (ligeramente alargado)
pub const SHAPE_OPTIMIZER_EPSILON: f32 = 0.01;      // Guard change detection
pub const SHAPE_FD_DELTA: f32 = 0.1;                // Paso finite-difference
```

(Constantes de damping y max_iter ya definidas en MG-1.)

### MG-4E: Integración con ecuaciones existentes

- Reutilizar `shape_inferred_length` y `shape_inferred_direction` de `equations/morphogenesis_shape/` para la traducción final a `GeometryInfluence`. MG-4 **solo** añade `bounded_fineness_descent`; no duplica ni reemplaza las funciones de inferencia de worldgen.

### MG-4F: BridgeCache (cuando aplique)

- Si profiling muestra costo >0.1ms para ≥50 entidades, registrar `BridgeShape` con clave cuantizada `(fineness_q, density_q, velocity_q)`.
- Cuantización: multiply-round `f32 → u16` con factor de escala en `constants.rs`.
- No bloqueante para el sprint — solo si el benchmark de MG-8 lo exige.

## Tácticas

- **Histéresis natural por damping.** Con `SHAPE_OPTIMIZER_DAMPING = 0.3` y `max_iter = 3`, el fineness cambia máximo `~0.3 * 3 * grad` por tick. Evita oscilaciones sin smoothing extra.
- **LOD Far:** congelar `fineness_ratio` (no llamar al optimizer). Near: recalcular cada tick. Mid: cada N ticks.
- **Convergencia suave.** El optimizer no busca el mínimo global — busca mejorar localmente cada tick. La acumulación de ticks converge al óptimo local sin spikes visuales.
- **Sin reescritura de mallas.** Este sistema solo escribe `MorphogenesisShapeParams`. El mesh builder de GF1 lee los parámetros de forma stateless — no hay acoplamiento directo.
- **`graph_vascular_cost` es O(edges).** Suma `transport_cost` de aristas; max 16 → trivial.

## NO hace

- No implementa albedo ni rugosity (MG-5, MG-7).
- No reescribe mallas GF1 — solo parámetros de influencia; el mesh builder sigue stateless.
- No toca colores Hz/pureza salvo contrato futuro con MG-8.
- No modifica las funciones de shape inference existentes en `morphogenesis_shape/`.

## Dependencias

- MG-1 (`shape_cost`, `inferred_drag_coefficient`, `vascular_transport_cost`, constantes).
- MG-3 (DAG con flujos/disipación coherentes).
- GF1 existente (`geometry_flow/`): `GeometryInfluence` (lectura/contrato, no escritura directa).
- `src/layers/flow.rs` — `FlowVector` (2 campos: `velocity`, `dissipation_rate`).
- `src/layers/volume.rs` — `SpatialVolume` (1 campo: `radius`).

## Criterios de aceptación

### MG-4A (Componente)
- Test: `MorphogenesisShapeParams` tiene `Reflect` y `SparseSet`.
- Test: `MorphogenesisShapeParams::default()` → `fineness_ratio = FINENESS_DEFAULT` (1.5).
- Test: mapeo invariante — `fineness_ratio = 4.0, radius = 1.0` → `length_budget = 8.0`, `radius_base = 0.5` → `length_budget / radius_base = 16 ≈ fineness²`.

### MG-4B (Función pura)
- Test: `bounded_fineness_descent(1.5, 1000.0, 4.0, 3.14, 12.0, 0.3, 3)` → fineness > 1.5 (agua densa + velocidad alta empuja a fusiforme).
- Test: `bounded_fineness_descent(1.5, 1.2, 0.5, 3.14, 12.0, 0.3, 3)` → fineness ≈ 1.5 (aire + baja velocidad → poca presión, cambio mínimo).
- Test: `bounded_fineness_descent(8.0, 1000.0, 4.0, ...)` → fineness = 8.0 (ya en techo, clamped).
- Test: `bounded_fineness_descent(1.0, 1000.0, 4.0, ...)` → fineness > 1.0 (se aleja de esfera bajo presión).
- Test: determinismo — mismos inputs → misma salida, 1000 llamadas.
- Test: `velocity = 0` → fineness sin cambio significativo (sin arrastre, solo vascular_cost fija).

### MG-4C (Sistema — integración)
- Test: mismo arquetipo en dos contextos:
  - **Agua densa:** ρ=1000, v=4.0. Tras 10 ticks → `fineness_ratio > 3.0`.
  - **Aire ligero:** ρ=1.2, v=2.0. Tras 10 ticks → `fineness_ratio < 2.5`.
- Test: baseline esfera (fineness=1.0 inicial) → `Δ shape_cost < 0` tras 5 ticks (el optimizer siempre mejora o mantiene).
- Test: entidad sin `MetabolicGraph` → no recibe `MorphogenesisShapeParams` (backward compatible).
- Test: `SHAPE_OPTIMIZER_MAX_ITER` usado == valor en `constants.rs` (assert de configuración).

### MG-4D (Histéresis)
- Test: input oscilante (ρ alterna 1000↔1.2 cada tick) → `fineness_ratio` no oscila más de `±0.5` por tick (damping acota cambio).
- Test: input estable durante 50 ticks → `fineness_ratio` converge (Δ < `SHAPE_OPTIMIZER_EPSILON` en últimos 10 ticks).

### General
- `cargo test --lib` sin regresión.

## Referencias

- `docs/design/MORPHOGENESIS.md` §3.3, §5.1, §6 MG-4, §7 (riesgos optimizer)
- `src/geometry_flow/mod.rs` — `GeometryInfluence` (13 campos, DTO stateless)
- `src/blueprint/equations/morphogenesis_shape/mod.rs` — funciones existentes de inferencia de forma
- `docs/sprints/GEOMETRY_FLOW/README.md` — GF1
- Bejan (1997) — Ley Constructal: shape_cost minimization
- Myring (1976) — Body of revolution: C_D vs fineness ratio
