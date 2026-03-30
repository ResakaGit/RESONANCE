# Track: PROTO_DNA — Código genético emergente con codones

Reemplazar genes-as-floats por secuencias de codones que codifican "aminoácidos".
Mutaciones silenciosas, redundancia del código genético, y drift neutral emergen.
El mapping codón→aminoácido evoluciona por selección.

**Invariante:** El código genético NO se hardcodea. Emerge de cuál mapping produce
organismos más fit. Zero lookup tables fijas. Axiom 6 estricto.

---

## Qué ya existe (no se toca)

| Componente | Archivo | Estado |
|-----------|---------|--------|
| `VariableGenome` (4-32 genes como f32) | `blueprint/equations/variable_genome.rs` | ✅ 62 tests |
| `mutate_variable` (duplicación/deleción) | `blueprint/equations/variable_genome.rs` | ✅ Funcional |
| `protein_fold` (HP lattice fold) | `blueprint/equations/protein_fold.rs` | ✅ 27 tests |
| `MetabolicGraph` (DAG 12 nodos) | `layers/metabolic_graph.rs` | ✅ Funcional |
| `metabolic_genome` (gene→node→topology) | `blueprint/equations/metabolic_genome.rs` | ✅ 80 tests |
| `EpigeneticState` (expression mask) | `layers/epigenetics.rs` | ✅ Funcional |
| Batch pipeline con genomes side-table | `batch/arena.rs`, `batch/pipeline.rs` | ✅ Wired |
| Observabilidad (gene_count, graph_rate) | `batch/harness.rs` | ✅ Funcional |

## Sprints (5)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [PD-1](SPRINT_PD1_CODON_GENOME.md) | Codon Genome | Medio | VG ✅ | `CodonGenome` struct con secuencia de tripletes |
| [PD-2](SPRINT_PD2_GENETIC_CODE.md) | Genetic Code Table | Bajo | PD-1 | `CodonTable` — mapping codón→aminoácido evolucionable |
| [PD-3](SPRINT_PD3_TRANSLATION.md) | Translation Pipeline | Medio | PD-1, PD-2 | `translate()` — codones→aminoácidos→Monomer chain |
| [PD-4](SPRINT_PD4_SILENT_MUTATIONS.md) | Silent Mutations | Bajo | PD-3 | Mutación de codón que NO cambia aminoácido (neutral drift) |
| [PD-5](SPRINT_PD5_BATCH_WIRING.md) | Batch Integration | Medio | PD-3, PD-4 | CodonGenome en side-table, reproducción, observabilidad |

---

## Dependency chain

```
VariableGenome (VG) ✅
    │
    ▼
PD-1: CodonGenome struct
    │
    ├──▶ PD-2: CodonTable (genetic code)
    │       │
    ▼       ▼
PD-3: Translation (codons → amino acids → monomers)
    │
    ▼
PD-4: Silent mutations (redundancy → neutral drift)
    │
    ▼
PD-5: Batch wiring (side-table, reproduction, observability)
```

## Arquitectura de archivos

```
src/
├── blueprint/
│   ├── equations/
│   │   ├── codon_genome.rs      ← PD-1: CodonGenome struct + mutation + crossover
│   │   ├── genetic_code.rs      ← PD-2: CodonTable + translation mapping
│   │   ├── codon_translation.rs ← PD-3: codons → amino acids → Monomer chain
│   │   ├── protein_fold.rs      ← YA EXISTE: consume Monomer chain (PF-2→5)
│   │   └── variable_genome.rs   ← YA EXISTE: backward compatible bridge
│   └── constants/
│       └── codon.rs             ← PD-1: MAX_CODONS, CODON_MUTATION_RATE, etc.
├── batch/
│   ├── arena.rs                 ← PD-5: CodonGenome side-table (igual que genomes[])
│   └── systems/
│       └── protein.rs           ← PD-5: usa translate() en vez de genome_to_polymer()
└── layers/                      ← SIN CAMBIOS
```

## Patrones por rol

| Rol | Patrón | Ejemplo |
|-----|--------|---------|
| **Dato** | Fixed-size array, Copy, repr(C), no heap | `CodonGenome { codons: [u8; MAX_CODONS], len: u16 }` |
| **Ecuación** | `fn(input) → output`, stateless, en `equations/` | `translate(codons, table) → [Monomer; N]` |
| **Constante** | Derivada de las 4 fundamentales, en `constants/` | `CODON_MUTATION_RATE = DISSIPATION_SOLID × 30` |
| **Table** | Fixed-size, evolucionable, Copy | `CodonTable { mapping: [u8; 64] }` (64 codones → 20 amino) |
| **Cache** | Struct pre-computado, una llamada por cambio | `CodonPhenotype { chain, fold, function }` |
| **System** | Lee side-table, escribe EntitySlot, stateless | `protein_from_codons(world)` |
| **Test** | Contrato + lógica + errores + determinismo | `silent_mutation_preserves_amino_acid()` |

## Axiomas en cada sprint

| Sprint | Ax1 | Ax2 | Ax3 | Ax4 | Ax5 | Ax6 | Ax7 | Ax8 |
|--------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| PD-1 | codón=energy encoding | — | — | mutation has cost | conservation | sequence emerges | — | codon freq |
| PD-2 | — | — | codes compete | degeneracy=dissipation buffer | — | code table emerges | — | — |
| PD-3 | amino=energy unit | pool: Σ amino ≤ genome | — | translation cost | — | function from sequence | — | freq alignment |
| PD-4 | — | — | — | silent mut = no cost | conserved | neutral drift emerges | — | — |
| PD-5 | — | pool in batch | — | maintenance cost | total qe audit | — | — | — |

## Constantes derivadas (todas de las 4 fundamentales)

| Constante | Valor | Derivación |
|-----------|-------|-----------|
| `MAX_CODONS` | 96 | `MAX_GENES × 3` (3 nucleótidos por codón) |
| `AMINO_ACID_TYPES` | 8 | `2^3` (3 bits por aminoácido, simplificado) |
| `CODON_MUTATION_RATE` | 0.15 | `DISSIPATION_SOLID × 30` (por nucleótido) |
| `TRANSLATION_COST` | 0.01 | `DISSIPATION_SOLID × 2` (costo por codón traducido) |
| `SILENT_MUTATION_FRACTION` | ~0.25 | Emergente de redundancia (64 codones → 8 aminoácidos) |

---

## Resumen de cambios

| Archivo | Tipo | Cambio |
|---------|------|--------|
| `blueprint/equations/codon_genome.rs` | Nuevo | CodonGenome, CodonTable, translate_genome, mutate_codon, crossover_codon, classify_mutation, silent_mutation_fraction. 28 tests. |
| `blueprint/constants/codon.rs` | Nuevo | 8 constantes derivadas de DISSIPATION_SOLID. |
| `batch/arena.rs` | Mod | +codon_genomes + codon_tables side-tables en SimWorldFlat. |
| `batch/systems/morphological.rs` | Mod | Reproduction propaga codones + code table. Abiogenesis inicializa codones. |
| `batch/systems/protein.rs` | Mod | Usa translate_genome() cuando codones disponibles. |
| `batch/harness.rs` | Mod | +codon_count_mean en GenerationStats. |
