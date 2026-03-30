# Sprint PD-2 — Genetic Code Table: mapping codón→aminoácido evolucionable

**Módulo:** `src/blueprint/equations/genetic_code.rs` (nuevo)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** PD-1

---

## Objetivo

Tabla de código genético que mapea 64 codones → 8 aminoácidos.
La tabla misma puede mutar (el código genético evoluciona).
Redundancia emerge naturalmente: 64/8 = 8 codones por aminoácido.

## Diseño

### `CodonTable`

```rust
/// Genetic code: maps 64 codons → 8 amino acid types.
/// The table itself can mutate (code evolution).
/// Default: systematic mapping (codon / 8 → amino).
/// Evolved: random reassignments that survived selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodonTable {
    pub mapping: [u8; 64],  // codon_id → amino_acid_id (0..7)
}
```

### Amino acid types (8, no 20 — simplificado)

```
0 = Hydrophobic-small  (Ala, Val analog)
1 = Hydrophobic-large  (Leu, Ile analog)
2 = Polar-neutral      (Ser, Thr analog)
3 = Polar-charged+     (Lys, Arg analog)
4 = Polar-charged-     (Asp, Glu analog)
5 = Aromatic           (Phe, Trp analog)
6 = Flexible           (Gly, Pro analog)
7 = Structural         (Cys analog — disulfide bonds)
```

### Funciones

| Función | Firma | Axioma |
|---------|-------|--------|
| `default_table() → CodonTable` | Systematic: `codon / 8` | Ax6: starting point, not final |
| `translate_codon(table, codon) → u8` | Pure lookup | — |
| `mutate_table(table, seed) → CodonTable` | Reassign 1 random codon | Ax6: code evolves |
| `redundancy(table, amino) → u8` | Count codons mapping to this amino | — |
| `is_silent(table, old_codon, new_codon) → bool` | Both map to same amino | Ax4: no phenotypic cost |

## Tests

- `default_table_covers_all_aminos` — each amino has ≥1 codon
- `default_table_uniform_redundancy` — each amino has 8 codons
- `translate_in_range` — output ∈ [0, 7]
- `mutate_changes_one_entry` — exactly 1 mapping changes
- `mutate_deterministic` — same seed → same result
- `redundancy_sum_is_64` — Σ redundancy(amino_i) = 64
- `is_silent_same_amino_true` — two codons mapping to same amino
- `is_silent_different_amino_false`

## Criterios de aceptación

- `CodonTable` es Copy, [u8; 64], no heap.
- `translate_codon` es `(table, u8) → u8`, zero allocation.
- Default table tiene redundancia uniforme (8:1).
- 8+ tests.
