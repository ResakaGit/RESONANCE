# MD-12: Ewald Summation

**Effort:** 3 weeks | **Blocked by:** MD-7, MD-8 | **Blocks:** MD-14

**ADR:** [ADR-025 Ewald vs Cutoff Decision](../../arquitectura/ADR/ADR-025-ewald-vs-cutoff-decision.md)

## Problem

Coulomb force is long-range (1/r). In a periodic box, the sum over all
periodic images converges conditionally. Direct summation is O(N^2) and
slow to converge. Ewald splits the sum into fast-converging parts.

## Decision Gate

**Before starting MD-12:** evaluate if Phase 3 (Go model folding) needs Ewald.
Go model native contacts are short-range (< 8 A). If Go model works with
cutoff Coulomb + reaction field, skip Ewald entirely (save 3 weeks).

See Risk R6 in track README.

## Theory

Ewald splits Coulomb into 3 parts:
1. **Real space:** short-range, erfc-screened, O(N) with cell list
2. **Reciprocal space:** long-range, Fourier series, O(N^{3/2}) bare
3. **Self correction:** subtract self-interaction

```
E_Coulomb = E_real + E_recip - E_self
```

Parameter alpha controls the split: `alpha = 5.0 / L_box` (standard).

## Implementation

### `blueprint/equations/ewald.rs`

```rust
pub fn ewald_real_pair(q_i: f64, q_j: f64, r: f64, alpha: f64) -> f64
pub fn ewald_self_correction(charges: &[f64], alpha: f64) -> f64
pub fn ewald_reciprocal(
    positions: &[[f64; 3]], charges: &[f64],
    box_lengths: [f64; 3], alpha: f64, k_max: u32,
) -> f64
```

### Alternative: reaction field

If Ewald is not needed for Go model, implement simpler reaction field
correction instead:

```rust
pub fn reaction_field_correction(q_i: f64, q_j: f64, r: f64, r_cut: f64, epsilon_rf: f64) -> f64
```

## Tests

| Test | Criterion |
|------|-----------|
| `ewald_nacl_crystal_energy` | Madelung constant within 1% |
| `ewald_real_decays_with_alpha` | Higher alpha → faster real decay |
| `ewald_self_correction_positive` | E_self > 0 for charged system |
| `reaction_field_matches_at_r_cut` | Smooth transition at cutoff |

## Acceptance Criteria

- [x] Decision: Ewald or reaction field (based on Gate 1→2)
- [x] Chosen method implemented with >= 4 tests
- [x] Energy conservation with long-range electrostatics
