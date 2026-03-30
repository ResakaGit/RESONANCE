# Sprint PD-5 — Batch Integration: CodonGenome in evolutionary loop

**Módulos:** `src/batch/arena.rs` + `src/batch/systems/protein.rs` + `src/batch/harness.rs`
**Tipo:** System wiring, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** PD-3, PD-4

---

## Objetivo

Integrar CodonGenome en el batch simulator para que la evolución use codones
en vez de floats. Reproducción con crossover de codones. Observabilidad de
silent mutation rate y code table diversity.

## Diseño

### A. Side-table en SimWorldFlat

```rust
// arena.rs — agregar junto a genomes[]
pub struct SimWorldFlat {
    ...
    pub genomes: [VariableGenome; MAX_ENTITIES],     // YA EXISTE
    pub codon_genomes: [CodonGenome; MAX_ENTITIES],  // NUEVO
    pub codon_tables: [CodonTable; MAX_ENTITIES],    // NUEVO (code evolves per lineage)
    ...
}
```

### B. Reproduction usa CodonGenome

```rust
// morphological.rs — reproduction()
let parent_cg = &world.codon_genomes[parent_idx];
let parent_ct = &world.codon_tables[parent_idx];
let child_cg = mutate_codon_genome(parent_cg, rng);
let child_ct = mutate_table(parent_ct, rng); // code table also mutates (rare)
world.codon_genomes[child_idx] = child_cg;
world.codon_tables[child_idx] = child_ct;
```

### C. protein_from_codons system

```rust
// protein.rs — reemplaza genome_to_polymer con translate_genome
pub fn protein_fold_infer(world: &mut SimWorldFlat) {
    ...
    let (chain, len) = translate_genome(&world.codon_genomes[i], &world.codon_tables[i]);
    let fold = fold_greedy(&chain, len, &frequencies, seed);
    ...
}
```

### D. Observabilidad en GenerationStats

```rust
pub struct GenerationStats {
    ...
    pub codon_count_mean: f32,       // mean codons per entity
    pub silent_mutation_rate: f32,   // fraction of mutations that are silent
    pub code_diversity: f32,         // how different are code tables across population
}
```

## Tests

### Batch
- `reproduction_propagates_codon_genome` — child has mutated codon sequence
- `reproduction_propagates_code_table` — child has (possibly mutated) code table
- `codon_genome_grows_over_generations` — duplication increases codon count

### Observabilidad
- `stats_codon_count_matches_population` — codon_count_mean reflects actual
- `stats_silent_rate_positive` — silent mutations detected in default code table

### Conservation
- `codon_batch_conserves_energy` — same conservation audit as before
- `codon_fold_deterministic` — same seed → same fold from codons

## Criterios de aceptación

- CodonGenome en side-table (DoD: cold data separated from hot EntitySlot).
- Reproduction uses codon mutation + code table mutation.
- protein_fold_infer uses translate_genome instead of genome_to_polymer.
- 3 new observability fields in GenerationStats.
- 8+ tests.
- Zero changes to EntitySlot size.
