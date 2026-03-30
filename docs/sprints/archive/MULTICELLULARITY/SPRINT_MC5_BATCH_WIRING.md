# Sprint MC-5 — Batch Integration: multicelularidad en evolutionary loop

**Módulos:** `src/batch/systems/multicellular.rs` (nuevo) + `src/batch/harness.rs`
**Tipo:** System wiring, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MC-4

---

## Objetivo

Integrar adhesión + colonia + expresión diferencial en el batch simulator.
Organismos multicelulares emergen durante la simulación y su fitness es mayor.

## Diseño

### Nuevo batch system: `multicellular_step(world)`

```rust
/// Phase::MorphologicalLayer — after metabolic_graph_infer, before growth_inference.
pub fn multicellular_step(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    // 1. Build adjacency matrix from collision data + existing StructuralLinks
    //    (scratch.collision_pairs already computed by collision system)
    let adjacency = build_adjacency(world, scratch);

    // 2. Check adhesion for all colliding pairs
    //    If affinity > threshold → create bond (mark in adjacency)
    apply_adhesion(world, &mut adjacency);

    // 3. Detect colonies
    let colonies = detect_colonies(&adjacency, world.alive_mask);

    // 4. Compute positional gradient per entity
    let gradient = positional_gradient(
        &extract_positions(world), &adjacency, &colonies.colony_id, world.alive_mask
    );

    // 5. Modulate expression masks based on position
    for each alive entity with colony_id > 0:
        world.entities[i].expression_mask = modulate_expression(
            gradient[i], &world.entities[i].expression_mask, EXPRESSION_MODULATION_RATE
        );

    // 6. Pay adhesion maintenance cost
    for each bond: drain adhesion_cost from both entities
}
```

### Pipeline insertion

```
// pipeline.rs
systems::metabolic_graph_infer(self);
systems::protein_fold_infer(self);
systems::multicellular_step(self, scratch);  // ← NEW (after metabolism, before morphology)
systems::senescence(self);
```

### Observabilidad en GenerationStats

```rust
pub struct GenerationStats {
    ...
    pub colony_count_mean: f32,         // mean colonies per world
    pub multicellular_rate: f32,        // fraction of entities in colonies ≥ 3
    pub max_colony_size_mean: f32,      // mean size of largest colony per world
    pub specialization_index: f32,      // how different are expression masks within colonies
}
```

## Tests

### Adhesion
- `collision_creates_bond` — two entities colliding with similar freq → bond forms
- `distant_entities_no_bond` — far apart → no adhesion
- `different_freq_no_bond` — same position but different freq → no adhesion

### Colony
- `three_bonded_cells_one_colony` — detect colony correctly
- `colony_enables_specialization` — after multicellular_step, mask differs between border/interior

### Conservation
- `adhesion_cost_conserves_energy` — qe decreases by bond cost
- `multicellular_step_deterministic` — same world state → same result

### Observability
- `stats_colony_count_reflects_population` — colony_count_mean matches
- `stats_multicellular_rate_increases` — over generations, rate should increase

## Criterios de aceptación

- `multicellular_step` is a batch system: `(&mut SimWorldFlat, &mut ScratchPad)`.
- Uses adjacency from ScratchPad (already computed by collision system).
- Modulates expression_mask in-place (integrates with existing EpigeneticState).
- 4 new observability fields in GenerationStats.
- 8+ tests.
- Zero new components (uses existing StructuralLink, EpigeneticState fields in EntitySlot).
