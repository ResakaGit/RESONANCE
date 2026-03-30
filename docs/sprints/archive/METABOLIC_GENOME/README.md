# Track: METABOLIC_GENOME — Genoma Variable → Red Metabólica Evolucionable

Conectar `VariableGenome` (VG-1–6) con `MetabolicGraph` (MG-2–7).
Los genes extras generan nodos metabólicos. La topología se infiere de posición génica.
La red evoluciona por duplicación/deleción/mutación del genoma.

**Invariante:** Ningún nodo se hardcodea. Todo emerge de genes × physics.
Zero templates. Zero OrganRole assignment manual. Axiom 6 estricto.

---

## Sprints (7)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable | Completitud |
|--------|--------|----------|---------------|------------|:-----------:|
| [MGN-1](SPRINT_MGN1_GENE_TO_NODE.md) | Gene → Node mapping | Medio | VG-1–6 ✅ | `gene_to_exergy_node()` | 70% base |
| [MGN-2](SPRINT_MGN2_TOPOLOGY_INFERENCE.md) | Topology inference | Medio | MGN-1 | `infer_topology()` | 70% base |
| [MGN-3](SPRINT_MGN3_GRAPH_FROM_GENOME.md) | Graph from genome | Bajo | MGN-2 | `metabolic_graph_from_variable_genome()` | 70% base |
| [MGN-4](SPRINT_MGN4_EVOLUTION_INTEGRATION.md) | Evolution integration | Medio | MGN-3 | Mutation → graph re-inference | 70% ✅ |
| [MGN-5](SPRINT_MGN5_NODE_COMPETITION.md) | Node competition | Bajo | MGN-3 | `competitive_flow_distribution()` | → 80% |
| [MGN-6](SPRINT_MGN6_HEBBIAN_REWIRING.md) | Hebbian rewiring | Bajo | MGN-3 | `hebbian_capacity_update()` | → 90% |
| [MGN-7](SPRINT_MGN7_INTERNAL_CATALYSIS.md) | Internal catalysis | Medio | MGN-3 | `catalytic_activation_reduction()` | → 100% |

---

## Dependency chain

```
VG-1–6 (genoma variable) ✅
    → MGN-1 (gene → node)
        → MGN-2 (topology)
            → MGN-3 (graph compositor) ─────────────────────────────────────
                │                        │              │                   │
                → MGN-4 (evolution)      → MGN-5        → MGN-6            → MGN-7
                                          (competition)  (rewiring)         (catalysis)
```

MGN-5, MGN-6, MGN-7 son **paralelos** — todos dependen solo de MGN-3.

## Axiomas respetados

| Axioma | Cómo se aplica |
|--------|---------------|
| 1. Everything is Energy | Nodos = concentraciones de qe. Edges = flujos de qe. |
| 2. Pool Invariant | `Σ J_out(node) ≤ J_in(node)`. Validado en DAG step. |
| 4. Dissipation | Cada nodo tiene `efficiency < 1.0`. Transport_cost > 0. |
| 5. Conservation | `evaluate_metabolic_chain` verifica `Σ output ≤ Σ input`. |
| 6. Emergence | Topología inferida de genes, no programada. Role inferido de bias. |
| 7. Distance | `transport_cost ∝ |gene_i - gene_j|` (distancia génica). |
| 8. Oscillatory | `activation_energy` modulada por frequency alignment. |

## Constantes usadas (existentes, no nuevas)

| Constante | Fuente | Uso |
|-----------|--------|-----|
| `KLEIBER_EXPONENT` (0.75) | derived_thresholds | Scaling de efficiency por nodo |
| `DISSIPATION_SOLID` (0.005) | derived_thresholds | Base transport_cost |
| `METABOLIC_EDGE_CAPACITY_BASE` (50.0) | metabolic_graph_mg2 | Capacidad base de edge |
| `ROLE_EFFICIENCY_FACTOR[12]` | metabolic_graph_mg2 | η por OrganRole |
| `ROLE_ACTIVATION_ENERGY[12]` | metabolic_graph_mg2 | E_a por OrganRole |
| `CAPABILITY_BIAS_THRESHOLD` (0.3) | variable_genome | Gate de capabilities |

## Qué NO se modifica

- `MetabolicGraph` struct (ya funcional, 12 nodes × 16 edges)
- `MetabolicGraphBuilder` (se reutiliza)
- 6 sistemas MG existentes (step, constraint, ledger, shape, albedo, rugosity)
- Pipeline phases
- `ExergyNode`, `ExergyEdge` structs

## Qué se crea (nuevo, encapsulado en equations/)

| Función | Archivo | Toca existente? |
|---------|---------|----------------|
| `gene_to_exergy_node()` | `blueprint/equations/metabolic_genome.rs` | NO |
| `infer_role_from_gene()` | `blueprint/equations/metabolic_genome.rs` | NO |
| `infer_topology()` | `blueprint/equations/metabolic_genome.rs` | NO |
| `metabolic_graph_from_variable_genome()` | `blueprint/equations/metabolic_genome.rs` | NO |
| `rebuild_metabolic_graph()` batch system | `batch/systems/morphological.rs` | SÍ: añade 1 system call |

---

## Resumen de cambios

| Archivo | Tipo | Cambio |
|---------|------|--------|
| `blueprint/equations/metabolic_genome.rs` | Nuevo | gene_to_exergy_node, infer_topology, metabolic_graph_from_variable_genome, competitive_flow_distribution, hebbian_capacity_update, catalytic_activation_reduction. 68 tests. |
| `blueprint/equations/protein_fold.rs` | Nuevo | genome_to_polymer, fold_greedy (lattice 2D), contact_map, infer_protein_function, compute_protein_phenotype. 27 tests. |
| `batch/systems/metabolic_graph.rs` | Nuevo | metabolic_graph_infer: maintenance cost + efficiency bonus. 7 tests. |
| `batch/systems/protein.rs` | Nuevo | protein_fold_infer: fold + catalytic bonus. 5 tests. |
| `simulation/metabolic/metabolic_graph_genome.rs` | Nuevo | Bevy systems: genome_to_metabolic_graph_system + drain. 5 tests. |
| `batch/pipeline.rs` | Mod | +metabolic_graph_infer + protein_fold_infer en MetabolicLayer. |
| `plugins/metabolic_plugin.rs` | Mod | +2 Bevy systems registrados. |
