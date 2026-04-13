# MD-4: LJ Fluid Validation

**Effort:** 3 days | **Blocked by:** MD-0, MD-1, MD-2, MD-3 | **Blocks:** Phase 1

## Purpose

This is the first **external validation** of the MD engine. We simulate a Lennard-Jones
fluid in reduced units and compare thermodynamic properties against published data
(Johnson, Zollweg & Gubbins, Mol. Phys. 1993).

If this passes, the engine is thermodynamically correct. If not, something in
MD-0..3 is wrong and must be fixed before proceeding.

## Setup

### Reduced LJ units

All MD in this sprint uses reduced units (standard in LJ simulations):

| Quantity | Reduced unit | Symbol |
|----------|-------------|--------|
| Length | sigma | r* = r / sigma |
| Energy | epsilon | E* = E / epsilon |
| Temperature | epsilon / k_B | T* = k_B * T / epsilon |
| Time | sigma * sqrt(m / epsilon) | t* = t / tau |
| Density | 1 / sigma^D | rho* = N * sigma^D / V |
| Pressure | epsilon / sigma^D | P* = P * sigma^D / epsilon |

In reduced units: sigma = 1, epsilon = 1, m = 1, k_B = 1.

### State point

T* = 1.0, rho* = 0.8 (dense liquid). This is the most-benchmarked LJ state point.

Johnson et al. 1993 reference values:
- P* = 1.06 +/- 0.1
- U*/N = -5.67 +/- 0.1 (potential energy per particle)
- RDF first peak at r* = 1.0, height ~2.7

### Simulation parameters

- N = 500 particles (sufficient for bulk properties, manageable for cell lists)
- Box length: L = (N / rho*)^(1/D). For 2D: L = (500 / 0.8)^0.5 = 25.0 sigma
- dt* = 0.005 (standard for LJ)
- Equilibration: 5000 steps
- Production: 10000 steps
- r_cut = 2.5 sigma
- Thermostat: Langevin, gamma = 1.0

### 2D vs 3D consideration

**Current EntitySlot is 2D.** The Johnson et al. data is 3D. Two options:

**Option A:** Validate against 2D LJ reference data (exists but less cited).
Alder & Wainwright (1962), Toxvaerd (1977). Different equation of state.

**Option B:** Implement 3D positions as `[f32; 3]` for this sprint only, in a
separate test binary. Keep EntitySlot at 2D for the batch simulator; the validation
binary uses its own particle array.

**Recommendation:** Option A. Stay in 2D, validate against 2D literature. 3D comes
in MD-7. The goal is to verify Verlet + thermostat + PBC + cell list work correctly,
not to match 3D numbers.

2D LJ fluid at T* = 1.0, rho* = 0.7 (Toxvaerd 1977):
- P* ~ 1.3 +/- 0.2
- RDF first peak at r* ~ 1.0

## Implementation

### 1. Binary: `src/bin/lj_fluid.rs`

```rust
/// LJ fluid validation: thermodynamic properties vs. literature.
///
/// Usage: cargo run --release --bin lj_fluid -- --particles 500 --temp 1.0 --density 0.7
///
/// Output: pressure, potential energy/N, temperature, RDF to stdout.

fn main() {
    let args = parse_args();

    // Build world in reduced LJ units
    let mut world = create_lj_world(args.particles, args.density, args.temp);

    // Equilibration
    for _ in 0..args.equil_steps {
        md_tick(&mut world);
    }

    // Production: accumulate observables
    let mut pressure_acc = 0.0;
    let mut pe_acc = 0.0;
    let mut temp_acc = 0.0;
    let mut rdf = RdfAccumulator::new(r_max, n_bins);

    for step in 0..args.prod_steps {
        md_tick(&mut world);
        pressure_acc += compute_virial_pressure(&world);
        pe_acc += compute_potential_energy(&world);
        temp_acc += compute_temperature(&world);
        if step % 10 == 0 {
            rdf.accumulate(&world);
        }
    }

    // Average and print
    println!("T* = {:.4}", temp_acc / args.prod_steps as f64);
    println!("P* = {:.4}", pressure_acc / args.prod_steps as f64);
    println!("U*/N = {:.4}", pe_acc / (args.prod_steps * args.particles) as f64);
    rdf.print();
}
```

### 2. Pure math additions: `blueprint/equations/md_observables.rs`

```rust
/// Virial pressure from pairwise forces.
/// P = rho * k_B * T + (1 / (D * V)) * sum_{i<j} r_ij * f_ij
pub fn virial_pressure_contribution(r: f64, f_dot_r: f64) -> f64

/// Radial distribution function accumulator.
pub struct RdfAccumulator { bins: Vec<u64>, dr: f64, n_frames: u64, volume: f64, n_particles: u64 }
impl RdfAccumulator {
    pub fn accumulate_pair(&mut self, r: f64)
    pub fn normalize(&self) -> Vec<f64>  // g(r) normalized
}

/// Total potential energy (LJ + Coulomb) with cutoff.
pub fn total_potential_energy(particles: &[Particle], r_cut_sq: f64) -> f64
```

### 3. Initial configuration

FCC lattice (3D) or triangular lattice (2D), then randomize velocities from
Maxwell-Boltzmann at target temperature. Remove center-of-mass velocity.

```rust
/// Place particles on a 2D triangular lattice within box.
pub fn triangular_lattice_2d(n: usize, box_length: f64) -> Vec<[f64; 2]>

/// Assign Maxwell-Boltzmann velocities, remove COM drift.
pub fn init_velocities_2d(n: usize, mass: f64, kb_t: f64, seed: u64) -> Vec<[f64; 2]>
```

## Validation Criteria

| Observable | Expected (2D, T*=1.0, rho*=0.7) | Tolerance |
|-----------|--------------------------------|-----------|
| <T*> | 1.0 | +/- 0.02 (2%) |
| <P*> | ~1.3 (2D literature) | +/- 0.2 (15%) |
| U*/N | ~-2.5 (2D, lower than 3D) | +/- 0.3 (12%) |
| RDF peak position | r* ~ 1.0 | +/- 0.05 |
| RDF peak height | ~2.5 (2D) | +/- 0.5 (qualitative) |
| Energy drift (NVE, no thermostat) | < 1e-4 relative / 10K steps | Strict |

### Decision gate after MD-4

If all criteria pass: **proceed to Phase 1.**

If temperature is wrong: thermostat bug (MD-1).
If pressure is wrong: force computation or virial bug.
If RDF peak is shifted: distance or PBC bug (MD-2).
If energy drifts in NVE: integrator bug (MD-0).

## Tests (automated, in `cargo test`)

| Test | Criterion |
|------|-----------|
| `lj_fluid_temperature_equilibrates` | <T*> = 1.0 +/- 0.05 after 2K steps (N=100) |
| `lj_fluid_energy_conserved_nve` | drift < 1e-3 over 1K steps (N=50, no thermostat) |
| `lj_fluid_rdf_peak_at_sigma` | max(g(r)) in [0.9, 1.1] sigma |
| `lj_fluid_pressure_positive` | <P*> > 0 at T*=1.0, rho*=0.5 (gas phase) |
| `virial_pressure_zero_for_ideal_gas` | P = rho*T when forces = 0 |
| `rdf_normalization` | g(r) -> 1.0 at large r |
| `lattice_init_correct_density` | N / V = target rho |
| `velocities_zero_com_drift` | sum(v) < epsilon after init |

## Acceptance Criteria

- [x] `src/bin/lj_fluid.rs` produces thermodynamic output
- [x] `blueprint/equations/md_observables.rs` with virial, RDF, PE functions
- [x] Temperature equilibration within 2%
- [x] Pressure within 15% of 2D literature value
- [x] RDF peak at r = sigma +/- 5%
- [x] NVE energy drift < 1e-4 (sanity check for MD-0)
- [x] >= 8 automated tests
- [x] All existing batch tests pass
