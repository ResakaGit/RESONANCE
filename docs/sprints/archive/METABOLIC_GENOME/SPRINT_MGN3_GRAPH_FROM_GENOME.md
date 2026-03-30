# Sprint MGN-3 — MetabolicGraph from VariableGenome (Compositor)

**Módulo:** `src/blueprint/equations/metabolic_genome.rs` (extensión)
**Tipo:** Pure math compositor, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MGN-1, MGN-2

---

## Objetivo

Función que compone MGN-1 (gene → node) + MGN-2 (topology) en un MetabolicGraph completo.
Una sola llamada: `VariableGenome → MetabolicGraph`.

---

## Diseño

### `metabolic_graph_from_variable_genome(genome) → Result<MetabolicGraph, MetabolicGraphError>`

```rust
pub fn metabolic_graph_from_variable_genome(
    genome: &VariableGenome,
    expression_mask: &[f32; 4],
) -> Result<MetabolicGraph, MetabolicGraphError> {
    // 1. Skip genes 0-3 (core biases, not nodes)
    // 2. For each gene 4..len: gene_to_exergy_node() IF gated expression > threshold
    //    Silenced genes (mask ≈ 0) don't generate nodes (Axiom 6: conditional expression)
    // 3. Infer topology from active nodes
    // 4. Build via MetabolicGraphBuilder (validates DAG, indices, captor existence)
    // 5. Return Result (builder can fail if no captor node)
}
```

**Axiom 5:** Epigenetic gating controls which genes express → which nodes exist.
**Axiom 6:** Graph structure emerges from genome + environment, not templates.

### Expression threshold

```rust
const NODE_EXPRESSION_THRESHOLD: f32 = 0.2;  // gene × mask must exceed this to create node
```

Un gen silenciado (mask=0) no genera nodo. Un gen parcialmente expresado (mask=0.5 × gene=0.3 = 0.15) tampoco. Solo genes con suficiente expresión producen nodos funcionales.

### Cache-friendly: `GenomeMetabolicPhenotype`

```rust
pub struct GenomeMetabolicPhenotype {
    pub graph: MetabolicGraph,
    pub node_count: u8,
    pub edge_count: u8,
    pub total_capacity: f32,
    pub estimated_throughput: f32,
}
```

Computed once per entity per tick (or on genome change).

---

## Tests

### Contrato
- `four_gene_genome_empty_graph` — 4 genes → 0 nodos extra → graph vacío (solo core biases)
- `eight_gene_genome_has_nodes` — 8 genes → 4 nodos metabólicos
- `fully_silenced_genome_empty_graph` — mask=[0;4] → ningún gen expresado → graph vacío
- `result_error_on_no_captor` — si solo hay nodos actuator (no root/leaf) → MetabolicGraphError

### Lógica
- `node_count_scales_with_genome_length` — más genes → más nodos (hasta 12 max)
- `partial_expression_filters_weak_genes` — genes con mask×value < threshold → excluidos
- `graph_is_valid_dag` — MetabolicGraphBuilder::build() succeeds (no cycles)
- `graph_has_captor_node` — siempre hay al menos 1 Root/Leaf/Sensory si genes ≥ 5
- `epigenetic_mask_changes_graph_structure` — mask [1,0,1,0] silencia mobility+resilience vías

### Integración con VG
- `mutated_genome_produces_different_graph` — mutate_variable → re-infer → graphs differ
- `crossover_genomes_merge_topologies` — child graph is valid hybrid
- `cost_proportional_to_graph_complexity` — genome_maintenance_cost correlates with node_count

### Edge cases
- `max_genome_32_genes_valid_graph` — 32 genes → 12 nodes max, ≤16 edges, valid DAG
- `all_genes_same_value_symmetric_graph` — uniform genome → symmetric topology
- `deterministic` — same genome + same mask → same graph

---

## Criterios de aceptación

- `metabolic_graph_from_variable_genome()` es fn pura: `(&VG, &[f32;4]) → Result<MG, Error>`.
- Reutiliza MetabolicGraphBuilder (no duplica validación).
- 15+ tests.
- Zero Bevy imports.
- `cargo test --lib metabolic_genome` sin regresión.

---

## Referencias

- `src/layers/metabolic_graph.rs` — MetabolicGraphBuilder, MetabolicGraphError
- `src/blueprint/equations/metabolic_genome.rs` — gene_to_exergy_node (MGN-1), infer_topology (MGN-2)
- `src/blueprint/equations/variable_genome.rs` — VariableGenome, gated_effective_bias
