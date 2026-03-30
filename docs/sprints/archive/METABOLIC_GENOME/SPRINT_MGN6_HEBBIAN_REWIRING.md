# Sprint MGN-6 — Hebbian Rewiring: Edges se fortalecen/debilitan por uso

**Módulo:** `src/blueprint/equations/metabolic_genome.rs` (extensión)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MGN-3

---

## Contexto

Hoy los edges del MetabolicGraph tienen `max_capacity` fija (set al construir).
En biología, vasos sanguíneos que transportan más se ensanchan (angiogénesis),
los que no se usan se atrofian. Es la regla de Hebb aplicada a redes metabólicas:
"lo que fluye junto, crece junto".

---

## Objetivo

Ecuación pura que ajusta `max_capacity` de cada edge basándose en su uso relativo
durante el tick actual.

**Axioma 4** (Dissipation): fortalecer un edge tiene costo energético.
**Axioma 7** (Distance Attenuation): edges largas (high transport_cost) son más caras de mantener.
**Axioma 6** (Emergence): la red se re-cablea sola, no se programa.

---

## Diseño

### `hebbian_capacity_update(edges, current_flows) → [f32; MAX_EDGES]`

```rust
/// Adjust edge capacities based on usage (Hebbian rule for metabolism).
///
/// `new_cap = cap + learning_rate × (flow/cap - baseline) × cap`
///
/// - flow/cap > baseline → edge strengthens (used more than average)
/// - flow/cap < baseline → edge weakens (underused)
/// - Minimum capacity = EDGE_MIN_CAPACITY (never fully atrophied)
/// - Maximum capacity = EDGE_MAX_CAPACITY (physical limit)
/// - Strengthening cost = DISSIPATION_SOLID × Δcapacity (Axiom 4)
///
/// Returns: (new_capacities[], total_rewiring_cost)
pub fn hebbian_capacity_update(
    capacities: &[f32],
    flows: &[f32],
    transport_costs: &[f32],
    edge_count: usize,
    learning_rate: f32,
) -> ([f32; METABOLIC_GRAPH_MAX_EDGES], f32) {
    // 1. Compute utilization ratio per edge: flow / capacity
    // 2. Compute baseline utilization: mean(utilization)
    // 3. Delta = learning_rate × (utilization - baseline) × capacity
    // 4. new_cap = (cap + delta).clamp(MIN, MAX)
    // 5. Rewiring cost = DISSIPATION_SOLID × Σ|delta| × transport_cost (far edges cost more)
}
```

### Constantes

```rust
/// Minimum edge capacity — atrophied but not dead. Axiom 4 prevents zero.
pub const EDGE_MIN_CAPACITY: f32 = 1.0;
/// Maximum edge capacity — physical limit of the "vessel".
pub const EDGE_MAX_CAPACITY: f32 = 200.0;
/// Learning rate for Hebbian update. Slow adaptation (0.01 = 1% per tick).
/// Derived: DISSIPATION_SOLID × 2 = 0.01
pub const HEBBIAN_LEARNING_RATE: f32 = DISSIPATION_SOLID * 2.0;
/// Baseline utilization (50% = neutral, no change).
pub const HEBBIAN_BASELINE: f32 = 0.5;
```

### Propiedad clave: estabilidad

La red converge a un equilibrio donde cada edge tiene capacity proporcional a su uso.
No oscila porque `learning_rate` es pequeño (0.01) y baseline es 0.5.
Edges que fluyen >50% de capacity crecen. Edges que fluyen <50% encogen.

---

## Tests

### Contrato — Conservation (Axiom 4)
- `rewiring_has_positive_cost` — fortalecimiento cuesta energía
- `weakening_has_zero_cost` — atrofia no cuesta (solo fortalecimiento)
- `total_cost_proportional_to_change` — más cambio → más costo

### Lógica — Hebb's Rule
- `high_flow_strengthens` — flow=90% cap → capacity crece
- `low_flow_weakens` — flow=10% cap → capacity decrece
- `baseline_flow_no_change` — flow=50% cap → capacity estable (delta ≈ 0)
- `strengthening_bounded_by_max` — nunca excede EDGE_MAX_CAPACITY
- `weakening_bounded_by_min` — nunca cae bajo EDGE_MIN_CAPACITY
- `zero_flow_decays_to_min` — flow=0 sostenido → capacity → MIN over time

### Lógica — Distance cost (Axiom 7)
- `far_edge_costs_more_to_strengthen` — high transport_cost → higher rewiring cost
- `close_edge_cheaper` — low transport_cost → lower rewiring cost

### Estabilidad
- `converges_after_repeated_application` — 100 ticks de mismo flow → capacity estable
- `oscillation_dampened` — alternating high/low flow → capacity stays near average

### Determinismo
- `deterministic` — same inputs → same outputs (no RNG)

### Edge cases
- `empty_edges_no_panic` — 0 edges → returns zeros
- `nan_flow_safe` — NaN flow → treated as 0
- `single_edge_follows_hebb` — 1 edge behaves correctly alone

---

## Criterios de aceptación

- `hebbian_capacity_update()` es fn pura: `(&[f32], &[f32], &[f32], usize, f32) → ([f32; N], f32)`.
- Capacity siempre en [EDGE_MIN_CAPACITY, EDGE_MAX_CAPACITY].
- Cost > 0 solo cuando capacity increases (Axiom 4).
- 15+ tests.
- Zero side effects.

---

## Referencias

- `src/simulation/metabolic/morphogenesis.rs:write_results` — donde se escriben flows al graph
- `src/blueprint/equations/derived_thresholds.rs` — DISSIPATION_SOLID
- Hebb (1949): "When an axon of cell A is near enough to excite cell B and repeatedly takes part in firing it, some growth process or metabolic change takes place"
