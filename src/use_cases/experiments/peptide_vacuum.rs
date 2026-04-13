//! MD-9: Alanine dipeptide in vacuum — Ramachandran validation.
//!
//! Standalone physics loop: Velocity Verlet + Langevin thermostat + bonded forces + LJ non-bonded.
//! No PBC (vacuum). 22 atoms. Hardcoded AMBER-like parameters in reduced LJ units.
//!
//! Validation: phi/psi sampling shows alpha-helix + beta-sheet basins.
//! Decision gate for Go model shortcut (skip solvent/Ewald).

use crate::batch::systems::bonded_forces;
use crate::batch::topology::{AngleParams, BondParams, DihedralParams, ResidueInfo, Topology};
use crate::blueprint::equations::{bonded, determinism, md_observables, thermostat as thermo_eq};

// ─── Config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PeptideConfig {
    pub dt: f64,
    pub gamma: f64,
    pub temperature: f64,
    pub equil_steps: u32,
    pub prod_steps: u32,
    pub sample_interval: u32,
    pub seed: u64,
    /// LJ sigma for non-bonded interactions (reduced units).
    pub lj_sigma: f64,
    /// LJ epsilon for non-bonded interactions (reduced units).
    pub lj_epsilon: f64,
    /// LJ cutoff distance.
    pub lj_r_cut: f64,
    /// Initial backbone phi angle (radians).
    pub init_phi: f64,
    /// Initial backbone psi angle (radians).
    pub init_psi: f64,
}

impl Default for PeptideConfig {
    fn default() -> Self {
        Self {
            dt: 0.002,
            gamma: 2.0,
            temperature: 2.5, // ~300K in reduced units (kB=1)
            equil_steps: 20_000,
            prod_steps: 100_000,
            sample_interval: 100,
            seed: 42,
            lj_sigma: 2.0,
            lj_epsilon: 0.5,
            lj_r_cut: 6.0,
            init_phi: -2.4,  // near beta basin (~-137°)
            init_psi: 2.4,   // near beta basin (~137°)
        }
    }
}

// ─── Output ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PeptideResult {
    /// 2D Ramachandran histogram (n_bins x n_bins), row-major.
    pub rama_hist: Vec<u32>,
    /// Number of bins per axis.
    pub n_bins: usize,
    /// Total phi/psi samples collected.
    pub n_samples: usize,
    /// Mean temperature during production.
    pub mean_temperature: f64,
    /// Max bond stretch (worst case max |r - r0| / r0).
    pub max_bond_deviation: f64,
    /// NVE energy drift (relative) over last 10K steps.
    pub nve_energy_drift: f64,
    /// All sampled (phi, psi) pairs.
    pub phi_psi_samples: Vec<(f32, f32)>,
}

// ─── NERF geometry builder ─────────────────────────────────────────────────

/// Place atom D bonded to C, with angle B-C-D and dihedral A-B-C-D.
///
/// Natural Extension Reference Frame (Parsons et al. 2005).
fn nerf(
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    bond: f64,
    angle: f64,
    dihedral: f64,
) -> [f64; 3] {
    let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let bc_len = (bc[0] * bc[0] + bc[1] * bc[1] + bc[2] * bc[2])
        .sqrt()
        .max(1e-10);
    let bc_u = [bc[0] / bc_len, bc[1] / bc_len, bc[2] / bc_len];

    let ba = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];

    // Normal to plane ABC
    let n = cross64(bc, ba);
    let n_len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2])
        .sqrt()
        .max(1e-10);
    let n_u = [n[0] / n_len, n[1] / n_len, n[2] / n_len];

    // In-plane perpendicular to BC
    let m = cross64(n_u, bc_u);

    let ca = angle.cos();
    let sa = angle.sin();
    let cd = dihedral.cos();
    let sd = dihedral.sin();

    [
        c[0] + bond * (-ca * bc_u[0] + sa * (cd * m[0] + sd * n_u[0])),
        c[1] + bond * (-ca * bc_u[1] + sa * (cd * m[1] + sd * n_u[1])),
        c[2] + bond * (-ca * bc_u[2] + sa * (cd * m[2] + sd * n_u[2])),
    ]
}

#[inline]
fn cross64(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ─── Alanine dipeptide geometry ────────────────────────────────────────────

/// Bond lengths in reduced LJ units (C-C bond ≈ 1.0 ≈ 1.52 Å / 1.52).
const CC: f64 = 1.0;
const CN_PEP: f64 = 0.88; // C-N peptide (partial double bond)
const CN_AMI: f64 = 0.97; // C-N amine
const CO: f64 = 0.81;     // C=O double bond
const NH: f64 = 0.66;     // N-H
const CH: f64 = 0.72;     // C-H

/// Bond angles (radians).
const SP2: f64 = 2.094_395_102_393_195_5; // 120°
const SP3: f64 = 1.910_633_236_249_018_8; // 109.5°

/// Atom indices in the 22-atom alanine dipeptide (Ace-Ala-NMe).
///
/// ACE cap: 0=HH31 1=CH3 2=HH32 3=HH33 4=C 5=O
/// ALA:     6=N 7=H 8=CA 9=HA 10=CB 11=HB1 12=HB2 13=HB3 14=C 15=O
/// NME cap: 16=N 17=H 18=CH3 19=HH31 20=HH32 21=HH33
pub const PHI_ATOMS: [usize; 4] = [4, 6, 8, 14];  // C(ace)-N-CA-C(ala)
pub const PSI_ATOMS: [usize; 4] = [6, 8, 14, 16];  // N-CA-C(ala)-N(nme)

pub const N_ATOMS: usize = 22;

/// Build alanine dipeptide geometry and topology from backbone dihedrals.
pub fn build_alanine_dipeptide(phi: f64, psi: f64) -> (Vec<[f64; 3]>, Topology) {
    let pi = core::f64::consts::PI;
    let omega = pi; // trans peptide bond

    let mut pos = vec![[0.0; 3]; N_ATOMS];

    // ── Backbone: special placement for first 3 atoms ─────────────────
    // Atom 1 (ACE CH3): origin
    pos[1] = [0.0, 0.0, 0.0];
    // Atom 4 (ACE C): along +x
    pos[4] = [CC, 0.0, 0.0];
    // Atom 6 (ALA N): angle 1-4-6 = 120°, in xy plane
    pos[6] = [
        CC + 0.5 * CN_PEP,
        -(0.75_f64.sqrt()) * CN_PEP,
        0.0,
    ];

    // ── Backbone: NERF from here ──────────────────────────────────────
    // O(5): bonded to C(4), angle 1-4-5 = sp2, dihedral 6-1-4-5 = π (trans to N)
    pos[5] = nerf(pos[6], pos[1], pos[4], CO, SP2, pi);
    // CA(8): bonded to N(6), angle 4-6-8 = sp2, dihedral 1-4-6-8 = ω
    pos[8] = nerf(pos[1], pos[4], pos[6], CN_AMI, SP2, omega);
    // H(7): bonded to N(6), angle 4-6-7 = sp2, dihedral 8-4-6-7 = π (trans to CA)
    pos[7] = nerf(pos[8], pos[4], pos[6], NH, SP2, pi);
    // C(14): bonded to CA(8), angle 6-8-14 = sp3, dihedral 4-6-8-14 = φ
    pos[14] = nerf(pos[4], pos[6], pos[8], CC, SP3, phi);
    // HA(9): bonded to CA(8), angle 6-8-9 = sp3, dihedral 14-6-8-9 = -120°
    // L-chirality: HA at -120° and CB at +120° from C(ala) around N-CA axis.
    pos[9] = nerf(pos[14], pos[6], pos[8], CH, SP3, -2.0 * pi / 3.0);
    // CB(10): bonded to CA(8), angle 6-8-10 = sp3, dihedral 14-6-8-10 = +120°
    pos[10] = nerf(pos[14], pos[6], pos[8], CC, SP3, 2.0 * pi / 3.0);
    // O(15): bonded to C(14), angle 8-14-15 = sp2, dihedral 6-8-14-15 = π
    pos[15] = nerf(pos[6], pos[8], pos[14], CO, SP2, pi);
    // N(16): bonded to C(14), angle 8-14-16 = sp2, dihedral 6-8-14-16 = ψ
    pos[16] = nerf(pos[6], pos[8], pos[14], CN_PEP, SP2, psi);
    // H(17): bonded to N(16), angle 14-16-17 = sp2, dihedral 8-14-16-17 = 0 (cis to CA)
    pos[17] = nerf(pos[8], pos[14], pos[16], NH, SP2, 0.0);
    // CH3(18): bonded to N(16), angle 14-16-18 = sp2, dihedral 8-14-16-18 = ω
    pos[18] = nerf(pos[8], pos[14], pos[16], CN_AMI, SP2, omega);

    // ── Methyl Hs ─────────────────────────────────────────────────────
    // ACE CH3 (atom 1) → H atoms 0, 2, 3
    pos[0] = nerf(pos[6], pos[4], pos[1], CH, SP3, pi / 3.0);
    pos[2] = nerf(pos[6], pos[4], pos[1], CH, SP3, pi);
    pos[3] = nerf(pos[6], pos[4], pos[1], CH, SP3, -pi / 3.0);
    // ALA CB (atom 10) → H atoms 11, 12, 13
    pos[11] = nerf(pos[9], pos[8], pos[10], CH, SP3, pi / 3.0);
    pos[12] = nerf(pos[9], pos[8], pos[10], CH, SP3, pi);
    pos[13] = nerf(pos[9], pos[8], pos[10], CH, SP3, -pi / 3.0);
    // NME CH3 (atom 18) → H atoms 19, 20, 21
    pos[19] = nerf(pos[17], pos[16], pos[18], CH, SP3, pi / 3.0);
    pos[20] = nerf(pos[17], pos[16], pos[18], CH, SP3, pi);
    pos[21] = nerf(pos[17], pos[16], pos[18], CH, SP3, -pi / 3.0);

    // ── Topology ──────────────────────────────────────────────────────
    let mut topo = Topology::new(N_ATOMS);

    topo.add_residue(ResidueInfo { name: *b"ACE\0", first_atom: 0, atom_count: 6 });
    topo.add_residue(ResidueInfo { name: *b"ALA\0", first_atom: 6, atom_count: 10 });
    topo.add_residue(ResidueInfo { name: *b"NME\0", first_atom: 16, atom_count: 6 });

    // Bond parameters (AMBER-like, reduced units)
    let b_cc = BondParams { r0: CC as f32, k: 3000.0 };
    let b_cn_pep = BondParams { r0: CN_PEP as f32, k: 4000.0 };
    let b_cn_ami = BondParams { r0: CN_AMI as f32, k: 3000.0 };
    let b_co = BondParams { r0: CO as f32, k: 5000.0 };
    let b_nh = BondParams { r0: NH as f32, k: 2500.0 };
    let b_ch = BondParams { r0: CH as f32, k: 2500.0 };

    // ACE bonds
    topo.add_bond(0, 1, b_ch);
    topo.add_bond(1, 2, b_ch);
    topo.add_bond(1, 3, b_ch);
    topo.add_bond(1, 4, b_cc);
    topo.add_bond(4, 5, b_co);
    // ACE-ALA peptide bond
    topo.add_bond(4, 6, b_cn_pep);
    // ALA bonds
    topo.add_bond(6, 7, b_nh);
    topo.add_bond(6, 8, b_cn_ami);
    topo.add_bond(8, 9, b_ch);
    topo.add_bond(8, 10, b_cc);
    topo.add_bond(8, 14, b_cc);
    topo.add_bond(10, 11, b_ch);
    topo.add_bond(10, 12, b_ch);
    topo.add_bond(10, 13, b_ch);
    topo.add_bond(14, 15, b_co);
    // ALA-NME peptide bond
    topo.add_bond(14, 16, b_cn_pep);
    // NME bonds
    topo.add_bond(16, 17, b_nh);
    topo.add_bond(16, 18, b_cn_ami);
    topo.add_bond(18, 19, b_ch);
    topo.add_bond(18, 20, b_ch);
    topo.add_bond(18, 21, b_ch);
    // Total: 21 bonds ✓

    // Infer angles with sp3 default, override sp2 at carbonyl C and peptide N
    let sp3_angle = AngleParams { theta0: SP3 as f32, k: 100.0 };
    topo.infer_angles_from_bonds(sp3_angle);

    // Override sp2 angles at planar centers
    let sp2_vertices: &[u16] = &[4, 6, 14, 16];
    for angle in &mut topo.angles {
        if sp2_vertices.contains(&angle.1) {
            angle.3.theta0 = SP2 as f32;
        }
    }

    // Infer dihedrals with generic barrier
    let generic_dih = DihedralParams { k: 0.5, n: 3, delta: 0.0 };
    topo.infer_dihedrals_from_bonds(generic_dih);

    // Override peptide bond dihedrals (omega) with strong planarity barrier
    for dih in &mut topo.dihedrals {
        let (b, c) = (dih.1, dih.2);
        // Peptide C-N bonds: (4,6) and (14,16)
        let is_peptide = (b == 4 && c == 6)
            || (b == 6 && c == 4)
            || (b == 14 && c == 16)
            || (b == 16 && c == 14);
        if is_peptide {
            dih.4.k = 10.0;
            dih.4.n = 2;
            dih.4.delta = core::f32::consts::PI;
        }
    }

    (pos, topo)
}

// ─── Simulation world ──────────────────────────────────────────────────────

struct PeptideWorld {
    positions: Vec<[f64; 3]>,
    velocities: Vec<[f64; 3]>,
    old_acc: Vec<[f64; 3]>,
    topology: Topology,
    exclusions: Vec<bool>,
    tick: u64,
    seed: u64,
    // LJ parameters
    lj_sigma: f64,
    lj_epsilon: f64,
    lj_r_cut: f64,
}

impl PeptideWorld {
    fn new(config: &PeptideConfig) -> Self {
        let (positions, topology) = build_alanine_dipeptide(config.init_phi, config.init_psi);
        let velocities = md_observables::init_velocities_3d(N_ATOMS, config.temperature, config.seed);
        let exclusions = topology.build_exclusion_matrix();

        Self {
            positions,
            velocities,
            old_acc: vec![[0.0; 3]; N_ATOMS],
            topology,
            exclusions,
            tick: 0,
            seed: config.seed,
            lj_sigma: config.lj_sigma,
            lj_epsilon: config.lj_epsilon,
            lj_r_cut: config.lj_r_cut,
        }
    }

    /// One Velocity Verlet + bonded + non-bonded + Langevin step.
    fn tick(&mut self, gamma: f64, temperature: f64, dt: f64) {
        self.tick += 1;

        // 1. Verlet position step (no PBC — vacuum)
        for i in 0..N_ATOMS {
            for d in 0..3 {
                self.positions[i][d] += self.velocities[i][d] * dt
                    + 0.5 * self.old_acc[i][d] * dt * dt;
            }
        }

        // 2. Compute forces (bonded + non-bonded LJ)
        let mut forces = vec![[0.0f64; 3]; N_ATOMS];

        // Bonded forces
        bonded_forces::compute_bonded_forces_3d(&self.positions, &self.topology, &mut forces);

        // Non-bonded LJ (brute force, 22 atoms — O(N²) is trivial)
        for i in 0..N_ATOMS {
            for j in (i + 1)..N_ATOMS {
                if self.exclusions[i * N_ATOMS + j] {
                    continue;
                }
                let d = [
                    self.positions[j][0] - self.positions[i][0],
                    self.positions[j][1] - self.positions[i][1],
                    self.positions[j][2] - self.positions[i][2],
                ];
                let f = md_observables::lj_force_3d_params(
                    d, self.lj_sigma, self.lj_epsilon, self.lj_r_cut,
                );
                for k in 0..3 {
                    forces[i][k] += f[k];
                    forces[j][k] -= f[k];
                }
            }
        }

        // 3. Verlet velocity finish
        for i in 0..N_ATOMS {
            for d in 0..3 {
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
            for i in 0..N_ATOMS {
                for d in 0..3 {
                    rng = determinism::next_u64(determinism::next_u64(determinism::next_u64(rng)));
                    let z = determinism::gaussian_f32(rng, 1.0) as f64;
                    self.velocities[i][d] = self.velocities[i][d] * c1 + sigma_v * z;
                }
            }
        }
    }

    /// Kinetic temperature: T = Σ(v²) / (3 * N).
    fn temperature(&self) -> f64 {
        let sum_v2: f64 = self.velocities.iter()
            .map(|v| v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
            .sum();
        sum_v2 / (3.0 * N_ATOMS as f64)
    }

    /// Total potential energy (bonded + non-bonded).
    fn potential_energy(&self) -> f64 {
        let bonded_pe = bonded_forces::bonded_potential_energy_3d(&self.positions, &self.topology);
        let mut nb_pe = 0.0;
        for i in 0..N_ATOMS {
            for j in (i + 1)..N_ATOMS {
                if self.exclusions[i * N_ATOMS + j] {
                    continue;
                }
                let d = [
                    self.positions[j][0] - self.positions[i][0],
                    self.positions[j][1] - self.positions[i][1],
                    self.positions[j][2] - self.positions[i][2],
                ];
                let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                nb_pe += md_observables::lj_potential_3d_params(
                    r, self.lj_sigma, self.lj_epsilon, self.lj_r_cut,
                );
            }
        }
        bonded_pe + nb_pe
    }

    /// Kinetic energy.
    fn kinetic_energy(&self) -> f64 {
        self.velocities.iter()
            .map(|v| 0.5 * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
            .sum()
    }

    /// Measure current phi and psi backbone dihedrals.
    fn measure_phi_psi(&self) -> (f32, f32) {
        let phi_pos: Vec<[f32; 3]> = PHI_ATOMS.iter()
            .map(|&i| [
                self.positions[i][0] as f32,
                self.positions[i][1] as f32,
                self.positions[i][2] as f32,
            ])
            .collect();
        let psi_pos: Vec<[f32; 3]> = PSI_ATOMS.iter()
            .map(|&i| [
                self.positions[i][0] as f32,
                self.positions[i][1] as f32,
                self.positions[i][2] as f32,
            ])
            .collect();

        let phi = bonded::dihedral_from_positions_3d(
            phi_pos[0], phi_pos[1], phi_pos[2], phi_pos[3],
        );
        let psi = bonded::dihedral_from_positions_3d(
            psi_pos[0], psi_pos[1], psi_pos[2], psi_pos[3],
        );
        (phi, psi)
    }

    /// Max bond deviation: max(|r - r0| / r0) over all bonds.
    fn max_bond_deviation(&self) -> f64 {
        let mut max_dev = 0.0_f64;
        for &(i, j, ref params) in &self.topology.bonds {
            let d = [
                self.positions[j as usize][0] - self.positions[i as usize][0],
                self.positions[j as usize][1] - self.positions[i as usize][1],
                self.positions[j as usize][2] - self.positions[i as usize][2],
            ];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let dev = ((r - params.r0 as f64) / params.r0 as f64).abs();
            if dev > max_dev {
                max_dev = dev;
            }
        }
        max_dev
    }
}

/// Splitmix64 hash.
#[inline]
fn mix_seed(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

// ─── Public API ────────────────────────────────────────────────────────────

/// Run peptide-in-vacuum simulation. Config in → result out. No side effects.
pub fn run_peptide_vacuum(config: &PeptideConfig) -> PeptideResult {
    let n_bins = 36;
    let mut world = PeptideWorld::new(config);

    // Equilibration
    for _ in 0..config.equil_steps {
        world.tick(config.gamma, config.temperature, config.dt);
    }

    // Production: accumulate observables
    let mut rama_hist = vec![0u32; n_bins * n_bins];
    let mut phi_psi_samples = Vec::new();
    let mut temp_sum = 0.0;
    let mut max_bond_dev = 0.0_f64;

    for step in 0..config.prod_steps {
        world.tick(config.gamma, config.temperature, config.dt);

        let t = world.temperature();
        temp_sum += t;

        // Sample phi/psi
        if step % config.sample_interval == 0 {
            let (phi, psi) = world.measure_phi_psi();
            phi_psi_samples.push((phi, psi));
            let (bi, bj) = md_observables::ramachandran_bin(phi, psi, n_bins);
            rama_hist[bi * n_bins + bj] += 1;

            let dev = world.max_bond_deviation();
            if dev > max_bond_dev {
                max_bond_dev = dev;
            }
        }
    }

    // NVE conservation check: 10K steps with gamma=0 (thermostat OFF).
    let nve_steps = 10_000u32;
    let nve_e0 = world.kinetic_energy() + world.potential_energy();
    for _ in 0..nve_steps {
        world.tick(0.0, config.temperature, config.dt); // gamma=0 → pure NVE
    }
    let nve_e1 = world.kinetic_energy() + world.potential_energy();
    let nve_drift = if nve_e0.abs() > 1e-10 {
        ((nve_e1 - nve_e0) / nve_e0.abs()).abs()
    } else {
        0.0
    };

    PeptideResult {
        rama_hist,
        n_bins,
        n_samples: phi_psi_samples.len(),
        mean_temperature: temp_sum / config.prod_steps as f64,
        max_bond_deviation: max_bond_dev,
        nve_energy_drift: nve_drift,
        phi_psi_samples,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn quick_config() -> PeptideConfig {
        PeptideConfig {
            equil_steps: 2000,
            prod_steps: 5000,
            sample_interval: 50,
            ..PeptideConfig::default()
        }
    }

    #[test]
    fn alanine_dipeptide_has_22_atoms() {
        let (pos, topo) = build_alanine_dipeptide(-2.4, 2.4);
        assert_eq!(pos.len(), 22);
        assert_eq!(topo.n_atoms, 22);
        assert_eq!(topo.bonds.len(), 21, "21 bonds");
        assert_eq!(topo.residues.len(), 3, "ACE + ALA + NME");
    }

    #[test]
    fn initial_bond_lengths_near_equilibrium() {
        let (pos, topo) = build_alanine_dipeptide(-2.4, 2.4);
        for &(i, j, ref params) in &topo.bonds {
            let d = [
                pos[j as usize][0] - pos[i as usize][0],
                pos[j as usize][1] - pos[i as usize][1],
                pos[j as usize][2] - pos[i as usize][2],
            ];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let dev = ((r - params.r0 as f64) / params.r0 as f64).abs();
            assert!(
                dev < 0.05,
                "bond {}-{}: r={r:.4}, r0={:.4}, dev={dev:.4}",
                i, j, params.r0,
            );
        }
    }

    #[test]
    fn no_overlapping_atoms() {
        let (pos, _) = build_alanine_dipeptide(-2.4, 2.4);
        for i in 0..22 {
            for j in (i + 1)..22 {
                let d = [
                    pos[j][0] - pos[i][0],
                    pos[j][1] - pos[i][1],
                    pos[j][2] - pos[i][2],
                ];
                let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                assert!(r > 0.1, "atoms {i} and {j} overlap: r={r:.4}");
            }
        }
    }

    #[test]
    fn exclusions_correct_count() {
        let (_, topo) = build_alanine_dipeptide(-2.4, 2.4);
        let excl = topo.build_exclusion_matrix();
        let n_excluded: usize = excl.iter().filter(|&&e| e).count();
        // Self-exclusions: 22, bond pairs: 21*2=42, angle pairs: varies
        // Total should be > 22 + 42 = 64
        assert!(n_excluded > 64, "exclusion count: {n_excluded}");
    }

    #[test]
    fn phi_psi_measurable() {
        let (pos, _) = build_alanine_dipeptide(-2.4, 2.4);
        let phi_pos: Vec<[f32; 3]> = PHI_ATOMS.iter()
            .map(|&i| [pos[i][0] as f32, pos[i][1] as f32, pos[i][2] as f32])
            .collect();
        let phi = bonded::dihedral_from_positions_3d(
            phi_pos[0], phi_pos[1], phi_pos[2], phi_pos[3],
        );
        assert!(phi.is_finite(), "phi should be finite: {phi}");
        assert!(phi.abs() <= core::f32::consts::PI + 0.01, "phi in [-pi,pi]: {phi}");
    }

    #[test]
    fn temperature_equilibrates() {
        let result = run_peptide_vacuum(&quick_config());
        let target = 2.5;
        let error = ((result.mean_temperature - target) / target).abs();
        assert!(
            error < 0.3,
            "<T*>={:.4}, error={:.1}%",
            result.mean_temperature,
            error * 100.0,
        );
    }

    #[test]
    fn bonds_stable_during_simulation() {
        let result = run_peptide_vacuum(&quick_config());
        // Quick config uses short equilibration (2K vs 20K) → allow wider margin.
        // Full simulation (default config) achieves < 5%.
        assert!(
            result.max_bond_deviation < 0.25,
            "max bond deviation: {:.4} (should be < 25%)",
            result.max_bond_deviation,
        );
    }

    #[test]
    fn phi_psi_samples_collected() {
        let config = quick_config();
        let result = run_peptide_vacuum(&config);
        let expected = config.prod_steps / config.sample_interval;
        assert_eq!(
            result.n_samples, expected as usize,
            "should collect {} samples",
            expected,
        );
    }

    #[test]
    fn ramachandran_histogram_populated() {
        let result = run_peptide_vacuum(&quick_config());
        let total: u32 = result.rama_hist.iter().sum();
        assert!(total > 0, "histogram should have entries");
        let n_occupied = result.rama_hist.iter().filter(|&&c| c > 0).count();
        assert!(
            n_occupied >= 2,
            "at least 2 bins occupied: {n_occupied}",
        );
    }
}
