# MD-17: Folding Validation

**Effort:** 2 weeks | **Blocked by:** MD-15, MD-16 | **Blocks:** MD-18

**ADRs:** [ADR-023 Go Model](../../arquitectura/ADR/ADR-023-frequency-modulated-go-model.md), [ADR-024 REMD](../../arquitectura/ADR/ADR-024-remd-swap-criterion.md)

## Purpose

The culmination of the MD track. Fold a small protein (villin headpiece, 35
residues) using the Resonance Go model with Axiom 8 frequency modulation.
Compare RMSD to native PDB structure.

## Target Protein

**Villin headpiece (HP35):** PDB 1VII. 35 residues, 3 helices. One of the
fastest-folding proteins (~10 us). Standard benchmark for Go model validation.

## Protocol

1. Load PDB 1VII → extract C-alpha positions (35 atoms)
2. Build native contact map (distance < 8 A, |i-j| >= 3)
3. Assign frequencies (Strategy A: amino acid type)
4. Generate unfolded initial structure (extended chain)
5. Run REMD: 8 replicas, T = [0.5, 0.6, ..., 1.5] (reduced units)
6. Production: 10M steps per replica
7. Extract lowest-energy structure from lowest-T replica
8. Compute RMSD to native

## Implementation

### Binary: `src/bin/fold_go.rs`

```rust
fn main() {
    let native = load_pdb_calpha("assets/pdb/1VII.pdb");
    let contacts = native_contact_map(&native, 8.0, 3);
    let frequencies = assign_amino_acid_frequencies(&sequence);
    let topology = build_go_topology(&native, &contacts, &frequencies);
    let initial = extended_chain(35, 3.8);  // 3.8 A C-alpha spacing

    let result = run_remd(&RemdConfig { ... }, &topology, &initial);

    let best_rmsd = result.min_rmsd;
    println!("Best RMSD: {:.2} A", best_rmsd);
    println!("Native Q:  {:.2}", result.best_q);
    // ...
}
```

### Milestone criterion

RMSD < 5 A from native structure. This is relaxed because:
- Go model is coarse-grained (C-alpha only)
- No explicit solvent
- 2-body contacts only (no multi-body terms)

Standard Go models achieve 2-4 A on villin. Our frequency-modulated version
should be comparable if frequencies are well-assigned.

## Validation

| Observable | Expected | Tolerance |
|-----------|----------|-----------|
| Best RMSD | < 5.0 A | — |
| Native Q at folded | > 0.8 | — |
| Folding T | Between 0.8-1.2 (reduced) | — |
| Coherence at folded | > 0.7 | — |
| Coherence at unfolded | < 0.3 | — |

## The Publishable Result

If folding succeeds AND frequency-modulated Go model shows:
1. Folding rate depends on frequency coherence (not just contacts)
2. Coherence spectrum distinguishes folded from misfolded
3. Mutation = frequency shift → quantifiable ΔΔG

Then this is a publishable result: first Go model with oscillatory modulation.

## Acceptance Criteria

- [x] `src/bin/fold_go.rs` folds villin headpiece
- [x] RMSD < 5 A from native
- [x] Q > 0.8 in folded state
- [x] Frequency coherence correlates with Q
- [x] Comparison: classical Go vs Axiom 8 Go
