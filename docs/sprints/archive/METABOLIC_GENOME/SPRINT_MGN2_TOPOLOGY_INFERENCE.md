# Sprint MGN-2 — Topology Inference: Gene Positions → DAG Edges

**Módulo:** `src/blueprint/equations/metabolic_genome.rs` (extensión)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MGN-1

---

## Objetivo

Inferir la topología (edges) del DAG metabólico desde las posiciones de los genes.
Sin templates. Sin hardcoded connections. Los edges emergen de proximidad génica + tier ordering.

---

## Diseño

### Regla de conexión: tier-ordered adjacency

```
Edge(gene_i, gene_j) existe SI:
  1. tier(i) < tier(j)           — flujo va de captor → process → actuator (DAG)
  2. dimension(i) == dimension(j) — misma "vía metabólica" (growth→growth, mobility→mobility)
     OR dimension(i) == 0         — Root/Core/Fruit (growth) conecta a todo (hub metabólico)
  3. No crea ciclo                — validado por MetabolicGraphBuilder
```

### `infer_topology(nodes, gene_indices) → Vec<(from, to, capacity)>`

```rust
pub fn infer_topology(
    gene_count: usize,
    gene_values: &[f32],
) -> [(u8, u8, f32); METABOLIC_GRAPH_MAX_EDGES] {
    // Para cada par de nodos (i, j) donde tier(i) < tier(j):
    //   - Si dimension match OR hub: edge(i→j)
    //   - Capacity = METABOLIC_EDGE_CAPACITY_BASE × min(gene[i], gene[j])
    //   - Transport cost = DISSIPATION_SOLID × |i - j| (distancia génica = Axiom 7)
}
```

**Axiom 7:** `transport_cost ∝ |gene_position_i - gene_position_j|`. Genes lejanos → conexión cara.
**Axiom 4:** Cada edge pierde energía (`transport_cost > 0`).
**Axiom 2:** DAG estricto — `tier(from) < tier(to)` garantiza Pool Invariant en flujo.

### Output

Fixed-size array `[(from, to, capacity); MAX_EDGES]` con `edge_count: u8`.
No heap. Deterministic.

---

## Tests

### Contrato
- `empty_genome_no_edges` — 4 genes (core only) → 0 edges
- `five_genes_one_edge` — gene[4] (captor) existe solo → 0 edges (necesita target)
- `two_extra_genes_same_dimension_one_edge` — gene[4] + gene[8] (ambos dim 0) → 1 edge

### Lógica
- `tier_ordering_respected` — todas las edges van tier bajo → tier alto
- `transport_cost_proportional_to_distance` — genes lejanos tienen cost mayor
- `capacity_scales_with_gene_value` — gene=1.0 → capacity alta; gene=0.1 → capacity baja
- `hub_dimension_connects_cross` — gene dim=0 (growth) puede conectar a dim=1,2,3
- `non_hub_only_same_dimension` — gene dim=1 solo conecta a dim=1

### DAG validity
- `no_cycles_produced` — topology nunca crea ciclos
- `max_edges_respected` — nunca excede METABOLIC_GRAPH_MAX_EDGES
- `deterministic` — mismo genome → misma topology

### Edge cases
- `max_genome_32_genes_topology_valid` — 32 genes → graph con ≤16 edges, all DAG-valid
- `all_same_value_genes_uniform_capacity` — todos genes=0.5 → capacities uniformes

---

## Criterios de aceptación

- `infer_topology()` es fn pura: `(usize, &[f32]) → edges array`.
- Nunca produce ciclos.
- Transport cost derivado de Axiom 7 (distancia génica).
- 12+ tests.
- Zero Bevy imports.

---

## Referencias

- `src/blueprint/constants/metabolic_graph_mg2.rs` — METABOLIC_EDGE_CAPACITY_BASE
- `src/blueprint/equations/derived_thresholds.rs` — DISSIPATION_SOLID
- `src/layers/metabolic_graph.rs` — METABOLIC_GRAPH_MAX_EDGES, ExergyEdge
