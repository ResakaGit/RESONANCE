# MD-16: Replica Exchange MD (REMD)

**Effort:** 2 weeks | **Blocked by:** MD-15 | **Blocks:** MD-17

**ADR:** [ADR-024 REMD Swap Criterion](../../arquitectura/ADR/ADR-024-remd-swap-criterion.md)

## Problem

Go model proteins have rugged energy landscapes with many local minima.
Standard MD at low temperature gets trapped. At high temperature, the native
state is unstable. No single temperature samples both.

## Solution

REMD (Parallel Tempering): run N replicas at different temperatures
simultaneously. Periodically swap configurations between adjacent temperatures
via Metropolis criterion. High-T replicas escape traps; low-T replicas
benefit from the exploration.

## Theory

Swap criterion (detailed balance):

```
P_swap = min(1, exp(Delta))
Delta = (beta_i - beta_j) * (E_i - E_j)
```

where beta = 1/(k_B T). Satisfies detailed balance → correct canonical
ensemble at each temperature.

Temperature ladder: geometric spacing `T_i = T_min * (T_max/T_min)^(i/(N-1))`.
Target acceptance ratio: 20-50% (controls temperature spacing).

## Implementation

### `batch/systems/remd.rs`

```rust
pub struct ReplicaState {
    pub temperature: f64,
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub potential_energy: f64,
}

/// Attempt Metropolis swap between replicas i and j.
/// Returns true if swap accepted.
pub fn attempt_swap(
    replica_i: &ReplicaState,
    replica_j: &ReplicaState,
    rng_state: u64,
) -> bool

/// Build geometric temperature ladder.
pub fn temperature_ladder(t_min: f64, t_max: f64, n_replicas: usize) -> Vec<f64>

/// Run REMD: N replicas, M steps, swap every swap_interval.
pub fn run_remd(
    config: &RemdConfig,
    topology: &Topology,
    initial_positions: &[[f64; 3]],
) -> RemdResult
```

### Swap implementation

Swap = exchange temperatures (not coordinates). Each replica continues from
its current state but with the new temperature. Velocities rescaled to match
new temperature: `v_new = v * sqrt(T_new / T_old)`.

### Axiom mapping

Axiom 4: multiple dissipation rates sample the free energy landscape at
different scales. High T = fast exploration. Low T = precise folding.

## Tests

| Test | Criterion |
|------|-----------|
| `swap_always_accepted_for_equal_temps` | P_swap = 1 when T_i = T_j |
| `swap_detailed_balance` | P(i→j) * P_eq(i) = P(j→i) * P_eq(j) |
| `temperature_ladder_geometric` | T_{i+1}/T_i = const |
| `acceptance_ratio_in_range` | 20-50% for reasonable T spacing |
| `velocity_rescaling_preserves_ke_ratio` | KE_new/KE_old = T_new/T_old |
| `remd_samples_lower_energy_at_low_t` | <E> at low T < <E> at high T |

## Acceptance Criteria

- [x] Metropolis swap with detailed balance
- [x] Geometric temperature ladder
- [x] Velocity rescaling on swap
- [x] Acceptance ratio monitoring
- [x] >= 6 tests
