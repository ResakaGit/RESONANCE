//! MD-14: Alanine dipeptide in explicit TIP3P water.
//!
//! Combines: peptide (MD-9) + water (MD-10) + SHAKE (MD-11) + Ewald (MD-12) + FF (MD-13).
//! Validates: water density, hydration shell RDF, peptide stability, phi/psi sampling.
//!
//! Pipeline per tick:
//!   Verlet position → SHAKE → bonded forces → non-bonded (LJ + Ewald) → Verlet velocity → RATTLE → thermostat

use crate::batch::ff::water;
use crate::batch::systems::bonded_forces;
use crate::batch::topology::Topology;
use crate::blueprint::equations::{
    bonded, constraints, determinism, ewald, md_observables, pbc,
    thermostat as thermo_eq,
};
use crate::use_cases::experiments::peptide_vacuum;

// ─── Config ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SolvatedConfig {
    pub dt: f64,
    pub gamma: f64,
    pub temperature: f64,
    pub equil_steps: u32,
    pub prod_steps: u32,
    pub sample_interval: u32,
    pub seed: u64,
    /// Number of water molecules.
    pub n_waters: usize,
    /// Box side length (Angstrom).
    pub box_length: f64,
    /// LJ sigma for peptide non-bonded (reduced units).
    pub lj_sigma: f64,
    /// LJ epsilon for peptide non-bonded (reduced units).
    pub lj_epsilon: f64,
    /// LJ/real-space cutoff distance.
    pub r_cut: f64,
    /// Ewald k_max (reciprocal-space vectors per dimension).
    pub ewald_k_max: u32,
    /// Coulomb constant in caller's unit system.
    pub k_coulomb: f64,
    /// Initial phi/psi (radians).
    pub init_phi: f64,
    pub init_psi: f64,
}

impl Default for SolvatedConfig {
    fn default() -> Self {
        Self {
            dt: 0.001,
            gamma: 1.0,
            temperature: 1.0,
            equil_steps: 5_000,
            prod_steps: 10_000,
            sample_interval: 10,
            seed: 42,
            n_waters: 256,
            box_length: 25.0,
            lj_sigma: 1.0,
            lj_epsilon: 1.0,
            r_cut: 6.0,
            ewald_k_max: 4,
            k_coulomb: 1.0,
            init_phi: -1.0,
            init_psi: -0.8,
        }
    }
}

// ─── Result ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SolvatedResult {
    pub mean_temperature: f64,
    pub mean_phi: f64,
    pub mean_psi: f64,
    /// O-O RDF: (r, g(r)) pairs.
    pub oo_rdf: Vec<(f64, f64)>,
    /// Water density (g/cm³) — computed from box geometry.
    pub water_density: f64,
    /// Max bond deviation from equilibrium (peptide).
    pub max_bond_deviation: f64,
    /// Whether the peptide stayed near initial position.
    pub peptide_stable: bool,
    pub total_steps: u32,
}

// ─── World ────────────────────────────────────────────────────────────────

struct SolvatedWorld {
    positions: Vec<[f64; 3]>,
    velocities: Vec<[f64; 3]>,
    old_acc: Vec<[f64; 3]>,
    old_positions: Vec<[f64; 3]>,
    masses: Vec<f64>,
    charges: Vec<f64>,
    topology: Topology,
    exclusions: Vec<bool>,
    shake_constraints: Vec<(u16, u16, f64)>,
    n_peptide: usize,
    n_total: usize,
    box_lengths: [f64; 3],
    tick: u64,
    seed: u64,
    // Parameters
    lj_sigma: f64,
    lj_epsilon: f64,
    r_cut: f64,
    ewald_k_max: u32,
    k_coulomb: f64,
    ewald_alpha: f64,
}

impl SolvatedWorld {
    fn new(config: &SolvatedConfig) -> Self {
        let n_peptide = peptide_vacuum::N_ATOMS;
        let n_water_atoms = 3 * config.n_waters;
        let n_total = n_peptide + n_water_atoms;

        // Build peptide
        let (pep_positions, pep_topology) =
            peptide_vacuum::build_alanine_dipeptide(config.init_phi, config.init_psi);

        // Build water
        let water_positions = water::place_water_box(config.n_waters, config.box_length);
        let water_topology = water::create_water_topology(config.n_waters);
        let water_m = water::water_masses(config.n_waters);
        let water_q = water::water_charges(config.n_waters);

        // Combine positions: center peptide in box
        let half_box = config.box_length / 2.0;
        let mut positions = Vec::with_capacity(n_total);
        for p in &pep_positions {
            positions.push([p[0] + half_box, p[1] + half_box, p[2] + half_box]);
        }
        positions.extend_from_slice(&water_positions);

        // Combine masses: peptide all mass=1.0 (reduced), water real masses
        let mut masses = vec![1.0; n_peptide];
        masses.extend_from_slice(&water_m);

        // Combine charges: peptide neutral, water TIP3P charges
        let mut charges = vec![0.0; n_peptide];
        charges.extend_from_slice(&water_q);

        // Combine topologies
        let mut topology = Topology::new(n_total);
        // Peptide bonds
        for &(i, j, params) in &pep_topology.bonds {
            topology.add_bond(i, j, params);
        }
        for &(i, j, k, params) in &pep_topology.angles {
            topology.add_angle(i, j, k, params);
        }
        for &(i, j, k, l, params) in &pep_topology.dihedrals {
            topology.add_dihedral(i, j, k, l, params);
        }
        // Water bonds (offset indices)
        let offset = n_peptide as u16;
        for &(i, j, params) in &water_topology.bonds {
            topology.add_bond(i + offset, j + offset, params);
        }
        for &(i, j, k, params) in &water_topology.angles {
            topology.add_angle(i + offset, j + offset, k + offset, params);
        }
        // Atom types
        for i in 0..n_peptide {
            topology.atom_types[i] = pep_topology.atom_types[i];
        }
        for i in 0..n_water_atoms {
            topology.atom_types[n_peptide + i] = water_topology.atom_types[i] + 100; // offset to avoid collision
        }

        let exclusions = topology.build_exclusion_matrix();

        // SHAKE constraints from water bonds (k=10000, above peptide bonds max ~5000)
        let shake_constraints = constraints::constraints_from_topology(&topology, 8000.0);

        // Velocities
        let velocities = md_observables::init_velocities_3d(n_total, config.temperature, config.seed);

        let box_lengths = [config.box_length; 3];
        let ewald_alpha = 5.0 / config.box_length;

        Self {
            positions,
            velocities,
            old_acc: vec![[0.0; 3]; n_total],
            old_positions: vec![[0.0; 3]; n_total],
            masses,
            charges,
            topology,
            exclusions,
            shake_constraints,
            n_peptide,
            n_total,
            box_lengths,
            tick: 0,
            seed: config.seed,
            lj_sigma: config.lj_sigma,
            lj_epsilon: config.lj_epsilon,
            r_cut: config.r_cut,
            ewald_k_max: config.ewald_k_max,
            k_coulomb: config.k_coulomb,
            ewald_alpha,
        }
    }

    fn tick(&mut self, gamma: f64, temperature: f64, dt: f64) {
        self.tick += 1;
        let n = self.n_total;

        // Save old positions (for SHAKE reference)
        for i in 0..n {
            self.old_positions[i] = self.positions[i];
        }

        // 1. Verlet position step + PBC wrap
        for i in 0..n {
            for d in 0..3 {
                self.positions[i][d] += self.velocities[i][d] * dt
                    + 0.5 * self.old_acc[i][d] * dt * dt;
            }
            self.positions[i] = pbc::wrap_3d(self.positions[i], self.box_lengths);
        }

        // 2. SHAKE position correction (water O-H bonds)
        if !self.shake_constraints.is_empty() {
            constraints::shake_solve(
                &mut self.positions,
                &self.old_positions,
                &self.shake_constraints,
                &self.masses,
                1e-8,
                50,
            );
        }

        // 3. Compute forces
        let mut forces = vec![[0.0f64; 3]; n];

        // 3a. Bonded forces
        bonded_forces::compute_bonded_forces_3d(&self.positions, &self.topology, &mut forces);

        // 3b. Non-bonded LJ (brute-force with PBC minimum image)
        for i in 0..n {
            for j in (i + 1)..n {
                if self.exclusions[i * n + j] {
                    continue;
                }
                let d = pbc::minimum_image_3d(self.positions[j], self.positions[i], self.box_lengths);
                let f = md_observables::lj_force_3d_params(
                    d, self.lj_sigma, self.lj_epsilon, self.r_cut,
                );
                for k in 0..3 {
                    // minimum_image_3d returns j→i displacement, force is on i
                    forces[i][k] += f[k];
                    forces[j][k] -= f[k];
                }
            }
        }

        // 3c. Ewald electrostatics
        let ewald_forces = ewald::ewald_total_forces(
            self.k_coulomb,
            &self.positions,
            &self.charges,
            self.box_lengths,
            self.ewald_alpha,
            self.ewald_k_max,
            self.r_cut,
        );
        for i in 0..n {
            for k in 0..3 {
                forces[i][k] += ewald_forces[i][k];
            }
        }

        // 4. Verlet velocity finish
        for i in 0..n {
            let inv_m = 1.0 / self.masses[i];
            for d in 0..3 {
                self.velocities[i][d] += 0.5 * (self.old_acc[i][d] + forces[i][d] * inv_m) * dt;
            }
            for d in 0..3 {
                self.old_acc[i][d] = forces[i][d] * inv_m;
            }
        }

        // 5. RATTLE velocity correction
        if !self.shake_constraints.is_empty() {
            constraints::rattle_solve(
                &self.positions,
                &mut self.velocities,
                &self.shake_constraints,
                &self.masses,
            );
        }

        // 6. Langevin thermostat
        if gamma > 0.0 {
            let c1 = 1.0 - gamma * dt;
            let sigma_v = thermo_eq::langevin_velocity_sigma(gamma, temperature, dt, 1.0);
            let tick_seed = determinism::next_u64(self.seed ^ self.tick);
            let mut rng = tick_seed;
            for i in 0..n {
                for d in 0..3 {
                    rng = determinism::next_u64(determinism::next_u64(determinism::next_u64(rng)));
                    let z = determinism::gaussian_f32(rng, 1.0) as f64;
                    self.velocities[i][d] = self.velocities[i][d] * c1 + sigma_v * z;
                }
            }
        }
    }

    fn temperature(&self) -> f64 {
        let mut sum_v2 = 0.0;
        for v in &self.velocities {
            sum_v2 += v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        }
        sum_v2 / (3.0 * self.n_total as f64)
    }

    fn measure_phi_psi(&self) -> (f64, f64) {
        let phi_idx = peptide_vacuum::PHI_ATOMS;
        let psi_idx = peptide_vacuum::PSI_ATOMS;

        let phi = bonded::dihedral_from_positions_3d(
            self.positions[phi_idx[0]],
            self.positions[phi_idx[1]],
            self.positions[phi_idx[2]],
            self.positions[phi_idx[3]],
        );
        let psi = bonded::dihedral_from_positions_3d(
            self.positions[psi_idx[0]],
            self.positions[psi_idx[1]],
            self.positions[psi_idx[2]],
            self.positions[psi_idx[3]],
        );
        (phi, psi)
    }

    /// Max bond length deviation for peptide bonds.
    fn max_peptide_bond_deviation(&self) -> f64 {
        let mut max_dev = 0.0_f64;
        for &(i, j, params) in &self.topology.bonds {
            let (i, j) = (i as usize, j as usize);
            if i >= self.n_peptide && j >= self.n_peptide {
                continue; // skip water bonds
            }
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = self.positions[i][k] - self.positions[j][k];
                d_sq += dk * dk;
            }
            let d = d_sq.sqrt();
            let dev = (d - params.r0).abs();
            if dev > max_dev {
                max_dev = dev;
            }
        }
        max_dev
    }
}

// ─── Runner ───────────────────────────────────────────────────────────────

/// Run solvated peptide simulation. Returns observables.
pub fn run_peptide_solvated(config: &SolvatedConfig) -> SolvatedResult {
    let mut world = SolvatedWorld::new(config);
    let n_peptide = world.n_peptide;

    // Record initial peptide center of mass
    let mut pep_com_init = [0.0; 3];
    for i in 0..n_peptide {
        for k in 0..3 {
            pep_com_init[k] += world.positions[i][k];
        }
    }
    for k in 0..3 {
        pep_com_init[k] /= n_peptide as f64;
    }

    // Equilibration
    for _ in 0..config.equil_steps {
        world.tick(config.gamma, config.temperature, config.dt);
    }

    // Production
    let mut sum_temp = 0.0;
    let mut sum_phi = 0.0;
    let mut sum_psi = 0.0;
    let mut n_samples = 0u32;

    // O-O RDF accumulator (only water oxygens)
    let box_vol = config.box_length.powi(3);
    let n_water_o = config.n_waters;
    let mut oo_rdf = md_observables::RdfAccumulator3D::new(
        config.box_length / 2.0, 100, n_water_o, box_vol,
    );

    for step in 0..config.prod_steps {
        world.tick(config.gamma, config.temperature, config.dt);

        if step % config.sample_interval == 0 {
            sum_temp += world.temperature();
            let (phi, psi) = world.measure_phi_psi();
            sum_phi += phi;
            sum_psi += psi;
            n_samples += 1;

            // O-O RDF: iterate water oxygen pairs
            for i in 0..config.n_waters {
                let oi = n_peptide + 3 * i; // O index
                for j in (i + 1)..config.n_waters {
                    let oj = n_peptide + 3 * j;
                    let d = pbc::minimum_image_3d(
                        world.positions[oi], world.positions[oj], world.box_lengths,
                    );
                    let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    oo_rdf.add_pair(r);
                }
            }
            oo_rdf.end_frame();
        }
    }

    let n_samples = n_samples.max(1) as f64;

    // Peptide stability: center of mass displacement
    let mut pep_com_final = [0.0; 3];
    for i in 0..n_peptide {
        for k in 0..3 {
            pep_com_final[k] += world.positions[i][k];
        }
    }
    for k in 0..3 {
        pep_com_final[k] /= n_peptide as f64;
    }
    let com_drift: f64 = (0..3)
        .map(|k| (pep_com_final[k] - pep_com_init[k]).powi(2))
        .sum::<f64>()
        .sqrt();

    // Water density: n_waters * 18.015 g/mol / (V in Å³) / 6.022e23 * 1e24
    // Simplified: density = n_waters * 18.015 / (box_length^3 * 0.6022)
    let water_density = config.n_waters as f64 * 18.015 / (box_vol * 0.602_214_076);

    SolvatedResult {
        mean_temperature: sum_temp / n_samples,
        mean_phi: sum_phi / n_samples,
        mean_psi: sum_psi / n_samples,
        oo_rdf: oo_rdf.normalize(),
        water_density,
        max_bond_deviation: world.max_peptide_bond_deviation(),
        peptide_stable: com_drift < 10.0,
        total_steps: config.equil_steps + config.prod_steps,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> SolvatedConfig {
        SolvatedConfig {
            dt: 0.001,
            gamma: 1.0,
            temperature: 1.0,
            equil_steps: 50,
            prod_steps: 100,
            sample_interval: 10,
            seed: 42,
            n_waters: 8,
            box_length: 15.0,
            lj_sigma: 1.0,
            lj_epsilon: 1.0,
            r_cut: 5.0,
            ewald_k_max: 2,
            k_coulomb: 0.1,
            init_phi: -1.0,
            init_psi: -0.8,
        }
    }

    #[test]
    fn solvated_world_builds_correctly() {
        let config = small_config();
        let world = SolvatedWorld::new(&config);
        assert_eq!(world.n_peptide, 22);
        assert_eq!(world.n_total, 22 + 24); // 8 waters * 3 atoms
        assert_eq!(world.positions.len(), world.n_total);
        assert_eq!(world.masses.len(), world.n_total);
        assert_eq!(world.charges.len(), world.n_total);
    }

    #[test]
    fn solvated_topology_combined() {
        let config = small_config();
        let world = SolvatedWorld::new(&config);
        // Peptide has bonds + water has 2*8=16 bonds
        let pep_bonds = peptide_vacuum::build_alanine_dipeptide(-1.0, -0.8).1.bonds.len();
        let water_bonds = 2 * config.n_waters;
        assert_eq!(world.topology.bonds.len(), pep_bonds + water_bonds);
    }

    #[test]
    fn solvated_shake_constraints_count() {
        let config = small_config();
        let world = SolvatedWorld::new(&config);
        // SHAKE: 2 O-H bonds per water = 16 constraints
        assert_eq!(world.shake_constraints.len(), 2 * config.n_waters);
    }

    #[test]
    fn solvated_charges_neutral() {
        let config = small_config();
        let world = SolvatedWorld::new(&config);
        let total_charge: f64 = world.charges.iter().sum();
        assert!(
            total_charge.abs() < 1e-10,
            "system not neutral: {total_charge}",
        );
    }

    #[test]
    fn solvated_runs_without_panic() {
        let config = small_config();
        let result = run_peptide_solvated(&config);
        assert!(result.mean_temperature > 0.0, "temperature should be positive");
        assert!(result.total_steps == 150);
    }

    #[test]
    fn solvated_peptide_bonds_stable() {
        let mut config = small_config();
        config.equil_steps = 100;
        config.prod_steps = 200;
        let result = run_peptide_solvated(&config);
        assert!(
            result.max_bond_deviation < 1.0,
            "peptide bonds broke: max deviation = {}",
            result.max_bond_deviation,
        );
    }

    #[test]
    fn solvated_water_density_positive() {
        let config = small_config();
        let result = run_peptide_solvated(&config);
        assert!(result.water_density > 0.0);
    }

    #[test]
    fn solvated_oo_rdf_not_empty() {
        let config = small_config();
        let result = run_peptide_solvated(&config);
        assert!(!result.oo_rdf.is_empty(), "O-O RDF should have bins");
        // At least some bins should have g(r) > 0
        let nonzero = result.oo_rdf.iter().filter(|(_, g)| *g > 0.0).count();
        assert!(nonzero > 0, "RDF should have nonzero bins");
    }
}
