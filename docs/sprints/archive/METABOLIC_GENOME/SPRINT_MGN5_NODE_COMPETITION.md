# Sprint MGN-5 — Node Competition: Nodos compiten por flujo exergético

**Módulo:** `src/blueprint/equations/metabolic_genome.rs` (extensión)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MGN-3

---

## Contexto

Hoy el `step_dag` distribuye flujo proporcionalmente a `max_capacity` de cada edge.
Todos los nodos reciben su "porción justa". En biología real, órganos compiten por
irrigación sanguínea — los más eficientes capturan más recursos.

---

## Objetivo

Ecuación pura que redistribuye flujo entrante entre edges salientes de un nodo
usando la eficiencia de los nodos destino como peso competitivo.

**Axioma 3** (Competition as Primitive): `share ∝ efficiency × capacity` — no igualitario.
**Axioma 4** (Dissipation): competencia tiene costo — overhead de distribución.
**Axioma 2** (Pool Invariant): `Σ J_out ≤ J_in` estricto.

---

## Diseño

### `competitive_flow_distribution(j_in, edges, target_efficiencies) → [f32; MAX_EDGES]`

```rust
/// Distribute available exergy among outgoing edges competitively.
///
/// Each edge's share = (target_η × edge_capacity) / Σ(target_η × edge_capacity).
/// Axiom 3: magnitude = base × interference_factor (efficiency IS the interference).
/// Axiom 2: Σ shares ≤ j_in (guaranteed by normalization).
/// Axiom 4: competition overhead = COMPETITION_OVERHEAD_RATE × n_competitors.
pub fn competitive_flow_distribution(
    j_in: f32,
    outgoing: &[(u8, f32, f32)],  // (edge_idx, capacity, target_efficiency)
    overhead_rate: f32,
) -> ([f32; METABOLIC_GRAPH_MAX_EDGES], f32) {
    // 1. Compute competition overhead (Axiom 4)
    let overhead = overhead_rate * outgoing.len() as f32;
    let available = (j_in - overhead).max(0.0);

    // 2. Weighted score per edge: η_target × capacity
    // 3. Normalize: share_i = available × score_i / Σ scores
    // 4. Clamp each share ≤ capacity (bottleneck)
    // 5. Return (shares[], total_overhead)
}
```

### Constantes (derivadas de axiomas)

```rust
/// Competition overhead per competing edge. Axiom 4: distributing has a cost.
/// Derived: DISSIPATION_SOLID × 2 = 0.01 qe per competitor per tick.
pub const COMPETITION_OVERHEAD_RATE: f32 = DISSIPATION_SOLID * 2.0;
```

### Integración con `step_dag`

No modifica `step_dag` directamente. Provee la ecuación como alternativa a la
distribución proporcional actual. El sistema existente puede llamar esta función
en lugar de su distribución interna — cambio de 1 línea en `propagate_flows`.

---

## Tests

### Contrato — Pool Invariant (Axiom 2)
- `distribution_sum_le_input` — `Σ shares ≤ j_in` para cualquier input
- `distribution_sum_le_input_fuzz` — 100 random inputs, invariant holds
- `zero_input_zero_output` — j_in=0 → all shares=0
- `negative_input_clamped` — j_in=-5 → treated as 0

### Lógica — Competition (Axiom 3)
- `higher_efficiency_gets_more_flow` — η=0.9 edge gets more than η=0.3 edge
- `equal_efficiency_equal_capacity_equal_share` — same η + same cap → equal
- `zero_efficiency_gets_zero` — η=0 node gets nothing
- `single_outgoing_gets_all` — 1 edge → gets all available (minus overhead)
- `capacity_bottleneck_respected` — share ≤ capacity even if η is high

### Lógica — Overhead (Axiom 4)
- `overhead_increases_with_competitors` — 3 edges costs more than 1 edge
- `overhead_reduces_available` — available = j_in - overhead < j_in
- `all_zero_efficiency_only_overhead_lost` — no distribution, only overhead consumed

### Determinismo
- `deterministic` — same inputs → same outputs (no RNG)

### Edge cases
- `empty_outgoing_returns_zero` — no edges → no distribution
- `nan_efficiency_safe` — NaN η → treated as 0
- `very_large_input_no_overflow` — j_in=1e10 → clamped shares, no NaN

---

## Criterios de aceptación

- `competitive_flow_distribution()` es fn pura: `(f32, &[...], f32) → ([f32; N], f32)`.
- Pool Invariant: `Σ output ≤ input` verificado en TODOS los tests.
- Overhead > 0 siempre que hay 2+ edges (Axiom 4).
- 14+ tests.
- Zero side effects, zero Bevy, zero heap.

---

## Referencias

- `src/simulation/metabolic/morphogenesis.rs:propagate_flows` — distribución actual (proporcional)
- `src/blueprint/equations/derived_thresholds.rs` — DISSIPATION_SOLID
- `src/layers/metabolic_graph.rs` — METABOLIC_GRAPH_MAX_EDGES
