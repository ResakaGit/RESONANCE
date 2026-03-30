# Sprint MGN-4 — Evolution Integration: Genome Mutation → Graph Re-inference

**Módulo:** `src/batch/systems/morphological.rs` (1 system) + `src/simulation/lifecycle/` (1 system)
**Tipo:** System wiring, TDD.
**Estado:** ✅ Completado (2026-03-29)
**Bloqueado por:** MGN-3

---

## Objetivo

Conectar el loop evolutivo con la re-inferencia del grafo metabólico.
Cuando el genoma muta (batch o Bevy), el MetabolicGraph se re-construye.

---

## Diseño

### A. Batch simulator: `rebuild_metabolic_graph` system

En `batch/systems/morphological.rs`, después de `reproduction`:

```rust
pub fn rebuild_metabolic_graph(world: &mut SimWorldFlat) {
    for slot in alive_entities(world) {
        let genome = VariableGenome::from_slot(slot);
        let mask = slot.expression_mask;  // [f32; 4] from EpigeneticState
        // Re-infer graph only if genome changed (dirty flag or generation mismatch)
        if let Ok(graph) = metabolic_graph_from_variable_genome(&genome, &mask) {
            slot.metabolic_node_count = graph.node_count();
            slot.metabolic_edge_count = graph.edge_count();
            // Store flattened in EntitySlot for batch stepping
        }
    }
}
```

**No modifica el pipeline.** Se agrega como system call en `tick()` morphological phase.

### B. Bevy ECS: `genome_to_metabolic_graph_system`

En `simulation/lifecycle/`, nuevo system en `Phase::MorphologicalLayer`:

```rust
pub fn genome_to_metabolic_graph_system(
    mut commands: Commands,
    query: Query<
        (Entity, &InferenceProfile, &EpigeneticState),
        (Changed<InferenceProfile>, Without<MetabolicGraph>),
    >,
) {
    for (entity, profile, epigenetics) in &query {
        let genome = variable_genome_from_profile(profile);
        let mask = epigenetics.expression_mask;
        if let Ok(graph) = metabolic_graph_from_variable_genome(&genome, &mask) {
            commands.entity(entity).insert(graph);
        }
    }
}
```

**Usa `Changed<InferenceProfile>`** — solo re-infiere cuando el perfil cambia (reproducción, mutación).
**`Without<MetabolicGraph>`** — no sobreescribe grafos existentes que están siendo stepped.

### C. Re-inference trigger

Para entidades que YA tienen MetabolicGraph y cuyo genoma muta:

```rust
pub fn metabolic_graph_rebuild_system(
    mut query: Query<
        (&InferenceProfile, &EpigeneticState, &mut MetabolicGraph),
        With<PendingMorphRebuild>,
    >,
) {
    for (profile, epigenetics, mut graph) in &mut query {
        let genome = variable_genome_from_profile(profile);
        if let Ok(new_graph) = metabolic_graph_from_variable_genome(&genome, &epigenetics.expression_mask) {
            *graph = new_graph;
        }
    }
}
```

**`PendingMorphRebuild`** ya existe (D8 marker, SparseSet). Se inserta cuando InferenceProfile cambia significativamente.

---

## Tests

### Batch
- `rebuild_after_mutation_changes_graph` — mutate genome → graph topology differs
- `rebuild_respects_expression_mask` — silenced genes → fewer nodes
- `rebuild_deterministic` — same genome + mask → same graph

### Bevy
- `new_entity_gets_metabolic_graph` — spawn with InferenceProfile → graph inferred
- `changed_profile_triggers_rebuild` — mutate profile → PendingMorphRebuild → new graph
- `static_profile_no_rebuild` — unchanged profile → no wasted re-inference

### Conservation
- `graph_energy_flow_conserved` — after rebuild, step_system still conserves energy
- `graph_carnot_respected` — after rebuild, all η ≤ η_Carnot

---

## Encapsulamiento

- **Batch:** 1 system call added to `tick()` morphological phase.
- **Bevy:** 2 systems registered in `MorphologicalPlugin`, `Phase::MorphologicalLayer`.
- **Zero changes to:** MetabolicGraph struct, 6 existing MG systems, pipeline ordering.

---

## Criterios de aceptación

### Funcional
- Genoma de 8 genes → MetabolicGraph con ~4 nodos + edges.
- Mutación del genoma → grafo cambia (nueva topología).
- Epigenetic silencing → nodos desaparecen del grafo.

### Axiomas
- Conservation: `Σ J_out ≤ Σ J_in` per nodo (validado por step_system existente).
- Dissipation: transport_cost > 0 en todos los edges.
- Emergence: zero hardcoded connections.

### Performance
- Re-inference solo cuando genoma cambia (Changed<> filter + PendingMorphRebuild).
- No regression en batch benchmark (< 5% overhead).

---

## Referencias

- `src/batch/systems/morphological.rs` — reproduction, growth_inference
- `src/simulation/lifecycle/organ_lifecycle.rs` — PendingMorphRebuild, lifecycle_stage_inference
- `src/simulation/metabolic/morphogenesis.rs` — 6 MG systems (no modificar)
- `src/blueprint/equations/metabolic_genome.rs` — metabolic_graph_from_variable_genome (MGN-3)
- `src/plugins/morphological_plugin.rs` — system registration
