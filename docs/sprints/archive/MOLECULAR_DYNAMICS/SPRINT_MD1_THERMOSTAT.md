# MD-1: Langevin Thermostat

**Effort:** 1 week | **Blocked by:** MD-0 | **Blocks:** MD-4

## Problem

Without temperature control, the system is NVE (constant energy). We need NVT
(constant temperature) to:
- Sample the Boltzmann distribution (prerequisite for free energy)
- Control kinetic energy (prevent runaway heating from force errors)
- Model coupling to a heat bath (biological systems are thermostatted)

## Why Langevin (not Berendsen)

| Criterion | Berendsen | Langevin |
|-----------|-----------|----------|
| Correct ensemble | No (suppresses fluctuations) | Yes (canonical NVT) |
| Implementation | Trivial (velocity rescaling) | Moderate (friction + noise) |
| Axiom 4 compatibility | Rescaling is not dissipation | Friction IS dissipation |
| Stochastic component | None | Random kicks (thermal noise) |

Langevin maps directly to Axiom 4: friction = dissipation. The random kicks represent
the heat bath. Berendsen produces wrong fluctuations — it's a thermostat that doesn't
satisfy the fluctuation-dissipation theorem.

## Theory

Langevin equation of motion:

```
m * a = F_conservative - gamma * m * v + sqrt(2 * gamma * m * k_B * T / dt) * R(t)
```

Where:
- `gamma` = friction coefficient (1/time). From Axiom 4: `DISSIPATION_LIQUID * 10`.
- `R(t)` = random unit Gaussian per atom per dimension per step.
- `k_B * T` = thermal energy scale.
- The friction-noise balance satisfies fluctuation-dissipation theorem:
  `<v^2> = k_B * T / m` at equilibrium (Maxwell-Boltzmann).

## Implementation

### 1. Pure math: `blueprint/equations/thermostat.rs`

```rust
/// Langevin friction force per component.
/// F_friction = -gamma * mass * velocity.
pub fn langevin_friction(gamma: f64, mass: f64, velocity: f64) -> f64

/// Langevin random kick magnitude.
/// sigma = sqrt(2 * gamma * mass * kB * T / dt).
pub fn langevin_noise_sigma(gamma: f64, mass: f64, kb_t: f64, dt: f64) -> f64

/// Instantaneous kinetic temperature from velocities.
/// T = (2 / (N_dof * k_B)) * sum(0.5 * m * v^2).
pub fn kinetic_temperature(masses: &[f64], velocities: &[[f64; D]], k_b: f64) -> f64

/// Generate Maxwell-Boltzmann velocity for a given mass and temperature.
/// v_component = sqrt(k_B * T / m) * gaussian_random.
pub fn maxwell_boltzmann_velocity(mass: f64, kb_t: f64, gaussian: f64) -> f64

/// Chi-squared statistic for velocity distribution vs. Maxwell-Boltzmann.
/// Used for validation tests.
pub fn velocity_distribution_chi2(velocities: &[f64], mass: f64, kb_t: f64, n_bins: usize) -> f64
```

### 2. Deterministic noise

Resonance uses deterministic RNG (`blueprint/equations/determinism.rs`). The Langevin
random kick MUST use the same deterministic hash-based RNG:

```rust
let gaussian = determinism::gaussian_f32(entity_index as u64, tick as u64, dimension as u64);
```

This preserves bit-exact reproducibility (critical for batch harness).

### 3. System: `batch/systems/thermostat.rs`

```rust
pub fn langevin_thermostat(world: &mut SimWorldFlat, config: &ThermostatConfig) {
    let dt = world.dt as f64;
    let gamma = config.gamma;
    let kb_t = config.kb * config.target_temperature;
    let tick = world.tick;

    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let mass = e.particle_mass.max(0.01) as f64;

        for dim in 0..2 {
            // Friction (Axiom 4: dissipation)
            let v = e.velocity[dim] as f64;
            let friction = langevin_friction(gamma, mass, v);

            // Random kick (heat bath coupling)
            let sigma = langevin_noise_sigma(gamma, mass, kb_t, dt);
            let r = determinism::gaussian_f32(i as u64, tick, dim as u64) as f64;
            let noise = sigma * r;

            // Apply to velocity: dv = (friction + noise) / mass * dt
            e.velocity[dim] += ((friction + noise) / mass * dt) as f32;
        }
    }
}
```

### 4. Pipeline placement

Thermostat applies AFTER Verlet velocity step (modifies velocities):

```
verlet_position_step → zero_forces → particle_forces → verlet_velocity_step → langevin_thermostat
```

### 5. Configuration

```rust
pub struct ThermostatConfig {
    pub enabled: bool,
    pub target_temperature: f64,   // in energy units (k_B * T)
    pub gamma: f64,                // friction coefficient (1/time)
    pub kb: f64,                   // Boltzmann constant (in sim units)
}
```

`gamma` derived from Axiom 4: `DISSIPATION_LIQUID * 10.0 = 0.2`.
`kb` depends on unit system — in reduced LJ units, k_B = 1.0.

## Risks and Mitigations

### Thermostat vs. Axiom 5 (conservation)

**Problem:** Langevin injects energy. Axiom 5 says total qe decreases.

**Resolution:** Thermostat = open subsystem. Track energy bookkeeping:
```
E_total + E_dissipated_by_friction - E_injected_by_noise = E_initial  (within tolerance)
```

Add `thermostat_energy_injected: f64` and `thermostat_energy_dissipated: f64` to
`SimWorldFlat`. Conservation test checks the full balance, not just E_total.

### Temperature coupling too strong/weak

**Problem:** Wrong gamma → system thermalizes too fast (over-damped, kills dynamics)
or too slow (under-damped, bad sampling).

**Mitigation:** Test with gamma sweep: [0.01, 0.1, 1.0, 10.0]. Measure relaxation
time to target T. Choose gamma where T converges in ~100 steps but velocity
autocorrelation doesn't decay in < 10 steps.

### Deterministic noise quality

**Problem:** Hash-based Gaussian may have correlations that bias the distribution.

**Mitigation:** Run chi-squared test on 10K samples from `determinism::gaussian_f32`.
Must pass at p > 0.01. If not, use Box-Muller transform on two hash outputs.

## Tests

| Test | Pass criterion |
|------|---------------|
| `langevin_friction_zero_velocity_zero_force` | F = 0 when v = 0 |
| `langevin_friction_proportional_to_velocity` | F = -gamma * m * v |
| `langevin_noise_sigma_scales_with_temperature` | sigma(2T) = sqrt(2) * sigma(T) |
| `thermostat_cools_to_zero_without_noise` | gamma > 0, T=0 → KE → 0 monotonically |
| `thermostat_equilibrates_to_target` | <T> = T_target +/- 2% after 5K steps |
| `velocity_distribution_is_maxwell_boltzmann` | chi2 test p > 0.01 after 10K steps |
| `thermostat_energy_bookkeeping_balances` | E + E_diss - E_inj = E_0 +/- 1e-3 |
| `thermostat_deterministic_across_runs` | Same seed → same trajectory |

## Acceptance Criteria

- [x] `blueprint/equations/thermostat.rs` with >= 5 pure functions, >= 8 tests
- [x] `batch/systems/thermostat.rs` with Langevin system
- [x] Pipeline: Verlet position → forces → Verlet velocity → Langevin
- [x] Temperature equilibration test passes (2% tolerance)
- [x] Maxwell-Boltzmann validation passes (chi2)
- [x] Energy bookkeeping tracks injected/dissipated
- [x] All existing batch tests pass
