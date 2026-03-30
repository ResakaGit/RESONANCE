
---

## Resumen de cambios

| Archivo | Tipo | Cambio |
|---------|------|--------|
| `blueprint/equations/variable_genome.rs` | Nuevo | VariableGenome (4-32 genes), genome_maintenance_cost (Kleiber), effective_bias, mutate_variable (duplication/deletion), crossover_variable, gated_effective_biases, capabilities_from_genome, serialize/deserialize, GenomePhenotype cache. 62 tests. |
| `batch/arena.rs` | Mod | +genomes side-table en SimWorldFlat. |
| `batch/systems/morphological.rs` | Mod | Reproduction usa VariableGenome con mutación variable. |
| `batch/systems/metabolic_graph.rs` | Nuevo | metabolic_graph_infer lee genomes side-table. |
| `batch/harness.rs` | Mod | Repopulate propaga VariableGenome. +gene_count_mean observabilidad. |
