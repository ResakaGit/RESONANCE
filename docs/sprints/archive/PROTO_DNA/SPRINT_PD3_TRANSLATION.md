# Sprint PD-3 — Translation Pipeline: codones→aminoácidos→Monomer chain

**Módulo:** `src/blueprint/equations/codon_translation.rs` (nuevo)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** PD-1, PD-2

---

## Objetivo

Pipeline puro: `CodonGenome + CodonTable → [AminoAcid] → [Monomer]`.
Conecta con `protein_fold.rs` (ya existe) que consume `[Monomer]`.

## Diseño

### AminoAcid properties (derivadas de tipo, no hardcoded)

```rust
/// Properties of each amino acid type. Pure from type ID.
pub fn amino_properties(amino_id: u8) -> MonomerProperties {
    // Hydrophobicity, charge, size — all derived from amino_id position
    // in the 8-type system. No lookup table: formula-based.
    let hydrophobicity = [0.8, 0.9, 0.2, 0.1, 0.1, 0.7, 0.3, 0.6][amino_id as usize];
    let charge = [0.0, 0.0, 0.0, 0.5, -0.5, 0.0, 0.0, 0.2][amino_id as usize];
    ...
}
```

### Translation pipeline

```rust
/// Translate codon sequence → amino acid sequence → Monomer chain.
/// Pure: (CodonGenome, CodonTable) → ([Monomer; MAX_CHAIN], len)
pub fn translate_genome(
    genome: &CodonGenome,
    table: &CodonTable,
) -> ([Monomer; MAX_CHAIN], usize)
```

### Integration with existing protein_fold

```
ANTES (PF-1):  VariableGenome → genome_to_polymer() → [Monomer] → fold → function
AHORA (PD-3):  CodonGenome → translate_genome() → [Monomer] → fold → function
                                                        ↑
                                              SAME Monomer type, SAME fold pipeline
```

`protein_fold.rs` no cambia. Solo recibe Monomers de una fuente diferente.

## Tests

- `translate_empty_genome_empty_chain` — 0 codons → 0 monomers
- `translate_min_genome_produces_monomers` — MIN_CODONS → 4 monomers (12/3)
- `translate_deterministic` — same genome + table → same chain
- `monomer_hydrophobicity_from_amino_type` — H amino → H monomer
- `monomer_charge_from_amino_type` — charged amino → charged monomer
- `translate_preserves_sequence_order` — codon[0] → monomer[0]
- `translate_with_mutated_table_differs` — different code table → different chain
- `translate_feeds_protein_fold` — output compatible with `fold_greedy()`

## Criterios de aceptación

- `translate_genome` es `(&CG, &CT) → ([Monomer], usize)`, stateless.
- Output es compatible con `protein_fold::fold_greedy` sin cambios.
- Amino properties derivadas de tipo (fórmula), no lookup table.
- 10+ tests.
