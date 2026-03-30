# Sprint PD-4 — Silent Mutations: neutral drift from code redundancy

**Módulo:** `src/blueprint/equations/codon_genome.rs` (extensión de PD-1)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** PD-3

---

## Objetivo

Cuando un codón muta pero el nuevo codón mapea al mismo aminoácido (por redundancia
del código genético), la mutación es **silenciosa** — no cambia el fenotipo.
Esto habilita **neutral drift** (Kimura 1968): mutaciones que se acumulan
sin presión selectiva, creando diversidad genómica oculta.

## Diseño

### `classify_mutation(table, old_codon, new_codon) → MutationType`

```rust
pub enum MutationType {
    Silent,      // Same amino acid (no phenotypic effect)
    Missense,    // Different amino acid (phenotype changes)
    Nonsense,    // Stop-like: reduces effective chain length
}

pub fn classify_mutation(table: &CodonTable, old: u8, new: u8) -> MutationType {
    if old == new { return MutationType::Silent; }
    let old_amino = table.mapping[old as usize];
    let new_amino = table.mapping[new as usize];
    if old_amino == new_amino { MutationType::Silent }
    else { MutationType::Missense }
}
```

### Observabilidad: silent mutation rate

```rust
/// Fraction of mutations that are silent given current code table.
/// Higher redundancy → more silent mutations → faster neutral drift.
pub fn silent_mutation_fraction(table: &CodonTable) -> f32 {
    // For each codon, count how many of the 18 possible single-nucleotide
    // mutations (6 bits × 3 alternatives) produce the same amino acid.
    // Return: silent_count / total_mutations
}
```

### Integration con mutate_codon_genome (PD-1)

`mutate_codon_genome` ya muta nucleótidos. PD-4 agrega la clasificación post-hoc:
el batch system puede contar silent vs missense mutations para observabilidad.

## Tests

- `classify_same_codon_silent` — identical codons = Silent
- `classify_same_amino_silent` — different codon, same amino = Silent
- `classify_different_amino_missense` — different amino = Missense
- `silent_fraction_default_table` — default (8:1 redundancy) → ~75% silent
- `silent_fraction_no_redundancy_zero` — table with 1:1 mapping → 0% silent
- `silent_fraction_full_redundancy_one` — table all map to same amino → 100% silent
- `neutral_drift_accumulates` — 100 silent mutations → same protein, different codons

## Criterios de aceptación

- `classify_mutation` es `(&CT, u8, u8) → MutationType`, zero allocation.
- `silent_mutation_fraction` es `(&CT) → f32`, pure.
- Neutral drift verificable: silent mutations don't change translate() output.
- 8+ tests.

## Referencias

- Kimura (1968): "Evolutionary Rate at the Molecular Level" — neutral theory
- King & Jukes (1969): "Non-Darwinian Evolution" — genetic drift
