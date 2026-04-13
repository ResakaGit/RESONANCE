//! MD-4: LJ fluid validation — thermodynamic properties vs 2D literature.
//!
//! Standalone physics loop in reduced LJ units (sigma=1, epsilon=1, m=1, k_B=1).
//! Uses Velocity Verlet (MD-0) + Langevin thermostat (MD-1) + PBC (MD-2) + cell list (MD-3).
//!
//! Validation target: 2D LJ fluid at T*=1.0, rho*=0.7.
//! Literature: Toxvaerd 1977, Smit & Frenkel 1991.

use crate::blueprint::equations::{
    determinism, md_observables, pbc, thermostat as thermo_eq, verlet,
};

// ─── Config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LjFluidConfig {
    pub n_particles: usize,
    pub density: f64,
    pub temperature: f64,
    pub dt: f32,
    pub r_cut: f32,
    pub gamma: f64,
    pub equil_steps: u32,
    pub prod_steps: u32,
    pub seed: u64,
    /// Spatial dimensions: 2 or 3. Default 3 (MD-7).
    pub dimensions: usize,
}

impl Default for LjFluidConfig {
    fn default() -> Self {
        Self {
            n_particles: 100,
            density: 0.7,
            temperature: 1.0,
            dt: 0.005,
            r_cut: 2.5,
            gamma: 1.0,
            equil_steps: 5000,
            prod_steps: 10000,
            seed: 42,
            dimensions: 3,
        }
    }
}

// ─── Output ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LjFluidResult {
    pub mean_temperature: f64,
    pub mean_pressure: f64,
    pub mean_pe_per_particle: f64,
    pub rdf: Vec<(f64, f64)>,
    pub rdf_peak_r: f64,
    pub rdf_peak_height: f64,
}

// ─── Standalone LJ world — 3D f64 (MD-7) ──────────────────────────────────

struct LjWorld {
    positions: Vec<[f64; 3]>,
    velocities: Vec<[f64; 3]>,
    old_acc: Vec<[f64; 3]>,
    n: usize,
    box_length: f64,
    dt: f64,
    r_cut: f64,
    dim: usize, // 2 or 3
    tick: u64,
    seed: u64,
}

impl LjWorld {
    fn new(config: &LjFluidConfig) -> Self {
        let dim = config.dimensions;
        let box_length = if dim == 3 {
            (config.n_particles as f64 / config.density).cbrt()
        } else {
            (config.n_particles as f64 / config.density).sqrt()
        };

        let (positions, velocities) = if dim == 3 {
            let p = md_observables::cubic_lattice_3d(config.n_particles, box_length);
            let v = md_observables::init_velocities_3d(config.n_particles, config.temperature, config.seed);
            (p, v)
        } else {
            // 2D: z=0 for all particles
            let p2 = md_observables::square_lattice_2d(config.n_particles, box_length);
            let v2 = md_observables::init_velocities_2d(config.n_particles, config.temperature, config.seed);
            let p = p2.iter().map(|p| [p[0] as f64, p[1] as f64, 0.0]).collect();
            let v = v2.iter().map(|v| [v[0] as f64, v[1] as f64, 0.0]).collect();
            (p, v)
        };

        Self {
            positions,
            velocities,
            old_acc: vec![[0.0; 3]; config.n_particles],
            n: config.n_particles,
            box_length,
            dt: config.dt as f64,
            r_cut: config.r_cut as f64,
            dim,
            tick: 0,
            seed: config.seed,
        }
    }

    fn bl3(&self) -> [f64; 3] {
        [self.box_length, self.box_length, self.box_length]
    }

    /// One Verlet + LJ forces + Langevin step.
    fn tick(&mut self, gamma: f64, temperature: f64) {
        self.tick += 1;
        let dt = self.dt;
        let bl = self.bl3();

        // 1. Verlet position step + PBC wrap
        for i in 0..self.n {
            for d in 0..self.dim {
                self.positions[i][d] += self.velocities[i][d] * dt
                    + 0.5 * self.old_acc[i][d] * dt * dt;
                self.positions[i][d] = pbc::wrap_f64(self.positions[i][d], self.box_length);
            }
        }

        // 2. Compute LJ forces
        let mut forces = vec![[0.0f64; 3]; self.n];
        if self.dim == 3 {
            let cl = crate::batch::neighbor_list::CellList3D::build(
                &self.positions, self.n, bl, self.r_cut,
            );
            if let Some(cl) = &cl {
                cl.for_each_pair(&self.positions, |i, j, dx, dy, dz| {
                    let f = md_observables::lj_force_reduced_3d([dx, dy, dz], self.r_cut);
                    for d in 0..3 {
                        forces[i][d] += f[d];
                        forces[j][d] -= f[d];
                    }
                });
            } else {
                self.brute_force_3d(&mut forces);
            }
        } else {
            self.brute_force_2d_in_3d(&mut forces);
        }

        // 3. Verlet velocity finish
        for i in 0..self.n {
            for d in 0..self.dim {
                self.velocities[i][d] += 0.5 * (self.old_acc[i][d] + forces[i][d]) * dt;
            }
            self.old_acc[i] = forces[i];
        }

        // 4. Langevin thermostat
        if gamma > 0.0 {
            let c1 = 1.0 - gamma * dt;
            let sigma_v = thermo_eq::langevin_velocity_sigma(gamma, temperature, dt, 1.0);
            let tick_seed = mix_seed(self.seed ^ self.tick);
            let mut rng = tick_seed;
            for i in 0..self.n {
                for d in 0..self.dim {
                    rng = determinism::next_u64(determinism::next_u64(determinism::next_u64(rng)));
                    let z = determinism::gaussian_f32(rng, 1.0) as f64;
                    self.velocities[i][d] = self.velocities[i][d] * c1 + sigma_v * z;
                }
            }
        }
    }

    fn brute_force_3d(&self, forces: &mut [[f64; 3]]) {
        let bl = self.bl3();
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let d = pbc::minimum_image_3d(self.positions[i], self.positions[j], bl);
                let f = md_observables::lj_force_reduced_3d(d, self.r_cut);
                for k in 0..3 {
                    forces[i][k] += f[k];
                    forces[j][k] -= f[k];
                }
            }
        }
    }

    fn brute_force_2d_in_3d(&self, forces: &mut [[f64; 3]]) {
        let bl = self.bl3();
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let d = pbc::minimum_image_3d(self.positions[i], self.positions[j], bl);
                // Only x,y for 2D
                let f = md_observables::lj_force_reduced_3d([d[0], d[1], 0.0], self.r_cut);
                forces[i][0] += f[0];
                forces[i][1] += f[1];
                forces[j][0] -= f[0];
                forces[j][1] -= f[1];
            }
        }
    }

    /// Kinetic temperature: T = Σ(v²) / (D * N).
    fn temperature(&self) -> f64 {
        let sum_v2: f64 = self.velocities[..self.n]
            .iter()
            .map(|v| {
                let mut s = 0.0;
                for d in 0..self.dim { s += v[d] * v[d]; }
                s
            })
            .sum();
        sum_v2 / (self.dim as f64 * self.n as f64)
    }

    /// Potential energy per particle.
    fn potential_energy_per_particle(&self) -> f64 {
        let bl = self.bl3();
        let mut pe = 0.0f64;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let d = pbc::minimum_image_3d(self.positions[i], self.positions[j], bl);
                let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                pe += md_observables::lj_potential_reduced_f64(r, self.r_cut);
            }
        }
        pe / self.n as f64
    }

    /// Virial pressure.
    fn pressure(&self, temperature: f64) -> f64 {
        let bl = self.bl3();
        let volume = self.box_length.powi(self.dim as i32);
        let mut virial_sum = 0.0f64;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let d = pbc::minimum_image_3d(self.positions[i], self.positions[j], bl);
                let f = md_observables::lj_force_reduced_3d(d, self.r_cut);
                virial_sum += d[0] * f[0] + d[1] * f[1] + d[2] * f[2];
            }
        }
        md_observables::virial_pressure(virial_sum, self.n, volume, temperature, self.dim)
    }
}

/// Splitmix64 hash (same as in thermostat system).
#[inline]
fn mix_seed(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

// ─── Public API ────────────────────────────────────────────────────────────

/// Run LJ fluid validation. Config in → result out. No side effects.
pub fn run_lj_fluid(config: &LjFluidConfig) -> LjFluidResult {
    let mut world = LjWorld::new(config);
    let bl = world.bl3();
    let volume = world.box_length.powi(config.dimensions as i32);

    // Equilibration
    for _ in 0..config.equil_steps {
        world.tick(config.gamma, config.temperature);
    }

    // Production: accumulate observables
    let mut temp_sum = 0.0;
    let mut pressure_sum = 0.0;
    let mut pe_sum = 0.0;
    let rdf_r_max = world.box_length as f64 / 2.0;
    let mut rdf = md_observables::RdfAccumulator::new(rdf_r_max, 100, config.n_particles, volume);

    for step in 0..config.prod_steps {
        world.tick(config.gamma, config.temperature);
        let t = world.temperature();
        temp_sum += t;
        pressure_sum += world.pressure(t);
        pe_sum += world.potential_energy_per_particle();

        // RDF every 10 steps
        if step % 10 == 0 {
            for i in 0..world.n {
                for j in (i + 1)..world.n {
                    let d = pbc::minimum_image_3d(world.positions[i], world.positions[j], bl);
                    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    rdf.add_pair(r);
                }
            }
            rdf.end_frame();
        }
    }

    let n_prod = config.prod_steps as f64;
    let rdf_data = rdf.normalize();
    let (peak_r, peak_g) = rdf_data
        .iter()
        .filter(|(r, _)| *r > 0.5)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
        .copied()
        .unwrap_or((0.0, 0.0));

    LjFluidResult {
        mean_temperature: temp_sum / n_prod,
        mean_pressure: pressure_sum / n_prod,
        mean_pe_per_particle: pe_sum / n_prod,
        rdf: rdf_data,
        rdf_peak_r: peak_r,
        rdf_peak_height: peak_g,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config_2d() -> LjFluidConfig {
        LjFluidConfig {
            n_particles: 64,
            density: 0.7,
            temperature: 1.0,
            dt: 0.004,
            r_cut: 2.5,
            gamma: 2.0,
            equil_steps: 3000,
            prod_steps: 5000,
            seed: 42,
            dimensions: 2,
        }
    }

    #[test]
    fn temperature_equilibrates() {
        let result = run_lj_fluid(&small_config_2d());
        let error = ((result.mean_temperature - 1.0) / 1.0).abs();
        assert!(
            error < 0.10,
            "<T*>={:.4}, error={:.1}%",
            result.mean_temperature,
            error * 100.0,
        );
    }

    #[test]
    fn energy_conserved_nve() {
        // No thermostat → NVE → energy should be conserved
        let mut world = LjWorld::new(&LjFluidConfig {
            n_particles: 50,
            density: 0.5,
            temperature: 1.0,
            gamma: 0.0, // NVE
            dt: 0.002,
            r_cut: 2.5,
            equil_steps: 0,
            prod_steps: 0,
            seed: 42,
            dimensions: 2,
        });

        // Warm up with thermostat to get physical state
        for _ in 0..1000 {
            world.tick(1.0, 1.0);
        }

        // Switch to NVE: measure energy conservation
        let ke0: f64 = world.velocities[..world.n]
            .iter()
            .map(|v| 0.5 * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
            .sum();
        let pe0 = world.potential_energy_per_particle() * world.n as f64;
        let e0 = ke0 + pe0;

        for _ in 0..1000 {
            world.tick(0.0, 1.0); // gamma=0 → NVE
        }

        let ke1: f64 = world.velocities[..world.n]
            .iter()
            .map(|v| 0.5 * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
            .sum();
        let pe1 = world.potential_energy_per_particle() * world.n as f64;
        let e1 = ke1 + pe1;

        let drift = ((e1 - e0) / e0.abs()).abs();
        assert!(
            drift < 0.01,
            "NVE energy drift: {drift:.6} (E0={e0:.2}, E1={e1:.2})",
        );
    }

    #[test]
    fn rdf_peak_near_sigma() {
        let result = run_lj_fluid(&small_config_2d());
        assert!(
            result.rdf_peak_r > 0.9 && result.rdf_peak_r < 1.2,
            "RDF peak at r*={:.3}, expected ~1.0",
            result.rdf_peak_r,
        );
    }

    #[test]
    fn rdf_peak_has_structure() {
        let result = run_lj_fluid(&small_config_2d());
        assert!(
            result.rdf_peak_height > 1.5,
            "RDF peak height {:.2} should show liquid structure (>1.5)",
            result.rdf_peak_height,
        );
    }

    #[test]
    fn pressure_finite_and_positive_at_gas_density() {
        let result = run_lj_fluid(&LjFluidConfig {
            n_particles: 50,
            density: 0.3, // gas-like
            temperature: 2.0, // high T
            equil_steps: 2000,
            prod_steps: 3000,
            ..small_config_2d()
        });
        assert!(
            result.mean_pressure.is_finite(),
            "pressure must be finite",
        );
        assert!(
            result.mean_pressure > 0.0,
            "gas at high T should have P>0: P={:.4}",
            result.mean_pressure,
        );
    }

    // ── 3D tests (MD-7) ────────────────────────────────────────────────────

    #[test]
    fn temperature_equilibrates_3d() {
        let result = run_lj_fluid(&LjFluidConfig {
            n_particles: 64,
            density: 0.5,
            temperature: 1.0,
            dt: 0.004,
            r_cut: 2.5,
            gamma: 2.0,
            equil_steps: 3000,
            prod_steps: 5000,
            seed: 42,
            dimensions: 3,
        });
        let error = ((result.mean_temperature - 1.0) / 1.0).abs();
        assert!(
            error < 0.15,
            "3D <T*>={:.4}, error={:.1}%",
            result.mean_temperature,
            error * 100.0,
        );
    }

    #[test]
    fn energy_conserved_nve_3d() {
        let mut world = LjWorld::new(&LjFluidConfig {
            n_particles: 32,
            density: 0.4,
            temperature: 1.0,
            gamma: 0.0,
            dt: 0.002,
            r_cut: 2.5,
            equil_steps: 0,
            prod_steps: 0,
            seed: 42,
            dimensions: 3,
        });

        for _ in 0..1000 {
            world.tick(1.0, 1.0);
        }

        let ke0: f64 = world.velocities[..world.n]
            .iter()
            .map(|v| 0.5 * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
            .sum();
        let pe0 = world.potential_energy_per_particle() * world.n as f64;
        let e0 = ke0 + pe0;

        for _ in 0..1000 {
            world.tick(0.0, 1.0);
        }

        let ke1: f64 = world.velocities[..world.n]
            .iter()
            .map(|v| 0.5 * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
            .sum();
        let pe1 = world.potential_energy_per_particle() * world.n as f64;
        let e1 = ke1 + pe1;

        let drift = ((e1 - e0) / e0.abs()).abs();
        assert!(
            drift < 0.01,
            "3D NVE drift: {drift:.6} (E0={e0:.2}, E1={e1:.2})",
        );
    }
}
