# Sprint PD-1 — CodonGenome: Secuencia de tripletes

**Módulo:** `src/blueprint/equations/codon_genome.rs` (nuevo)
**Constantes:** `src/blueprint/constants/codon.rs` (nuevo)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente

---

## Objetivo

Struct que almacena genoma como secuencia de codones (tripletes de 2 bits = u8).
Cada codón codifica 1 de 8 "aminoácidos". Mutación a nivel de nucleótido individual.

## Diseño

### `CodonGenome`

```rust
/// Fixed-size codon genome. No heap. Copy.
/// Each codon = u8 ∈ [0, 63] (6 bits = 3 nucleótidos × 2 bits each).
/// Sequence length evolves (duplication/deletion at codon level).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct CodonGenome {
    pub codons: [u8; MAX_CODONS],  // 96 codones max
    pub len: u16,                   // active codons
    pub sigma: f32,                 // self-adaptive mutation rate
}
```

### Nucleotide encoding

```
2 bits per nucleotide:  00=A, 01=C, 10=G, 11=U
3 nucleotides per codon: 6 bits → 64 possible codons
64 codons → 8 amino acids (8:1 redundancy → silent mutations emerge)
```

### Mutation operations (stateless, deterministic)

| Función | Firma | Axioma |
|---------|-------|--------|
| `mutate_codon_genome(&CG, seed) → CG` | Point mutation en nucleótidos individuales | Ax4: mutation has cost |
| `duplicate_codon(&CG, seed) → CG` | Copiar codón al final de la secuencia | Ax6: length emerges |
| `delete_codon(&CG, seed) → CG` | Eliminar codón (nunca bajo MIN_CODONS) | Ax4: simplification |
| `crossover_codon(&CG, &CG, seed) → CG` | Recombinación en punto de corte | Ax6: diversity |

### Bridge: CodonGenome ↔ VariableGenome

```rust
/// Convert CodonGenome → VariableGenome (backward compatible).
/// Each codon group of 3 → 1 gene value via translate + normalize.
pub fn codon_to_variable(cg: &CodonGenome, table: &CodonTable) -> VariableGenome
```

## Tests (TDD)

### Contrato
- `default_has_min_codons` — len = MIN_CODONS (12 = 4 genes × 3)
- `codons_in_valid_range` — all codons ∈ [0, 63]
- `len_never_exceeds_max` — len ≤ MAX_CODONS after any operation
- `len_never_below_min` — len ≥ MIN_CODONS after deletion

### Lógica
- `point_mutation_changes_one_nucleotide` — exactly 1 of 6 bits flips
- `duplication_grows_by_one` — len increases by 1
- `deletion_shrinks_by_one` — len decreases by 1
- `crossover_deterministic` — same parents + seed → same child

### Errores
- `mutate_nan_sigma_safe` — sigma=NaN → clamped
- `empty_genome_no_panic` — len=0 handled gracefully

## Criterios de aceptación

- `CodonGenome` es Copy, repr(C), no heap.
- Todas las mutaciones son `fn(&CG, u64) → CG` — stateless.
- Constantes en `constants/codon.rs`, derivadas de las 4 fundamentales.
- 12+ tests.

## Referencias

- `blueprint/equations/variable_genome.rs` — VariableGenome (backward compatible)
- `blueprint/equations/protein_fold.rs` — Monomer chain (consumer)
- Crick (1968): "The Origin of the Genetic Code"
