//! MD-6: Bonded force accumulation system.
//!
//! Iterates topology bond/angle lists, calls `equations::bonded::*`,
//! accumulates into force array. Newton 3 symmetric.
//!
//! ADR-021: bonded forces separated from non-bonded, topology-driven.

use crate::batch::topology::Topology;
use crate::blueprint::equations::bonded;

/// Accumulate bonded forces (bonds + angles) into force array.
///
/// Stateless: topology + positions → forces. Newton 3: Σ forces = 0.
/// Caller zeroes the force array before calling this (or adds to existing non-bonded forces).
pub fn compute_bonded_forces_2d(
    positions: &[[f32; 2]],
    topology: &Topology,
    forces: &mut [[f64; 2]],
) {
    // ── Bonds: harmonic ────────────────────────────────────────────────────
    for &(i, j, ref params) in &topology.bonds {
        let ii = i as usize;
        let jj = j as usize;
        if ii >= positions.len() || jj >= positions.len() {
            continue;
        }
        let dx = positions[jj][0] - positions[ii][0];
        let dy = positions[jj][1] - positions[ii][1];
        let f = bonded::harmonic_bond_force(dx, dy, params.r0, params.k);
        // Force on i
        forces[ii][0] += f[0] as f64;
        forces[ii][1] += f[1] as f64;
        // Newton 3: force on j
        forces[jj][0] -= f[0] as f64;
        forces[jj][1] -= f[1] as f64;
    }

    // ── Angles: harmonic ───────────────────────────────────────────────────
    for &(a, b, c, ref params) in &topology.angles {
        let aa = a as usize;
        let bb = b as usize;
        let cc = c as usize;
        if aa >= positions.len() || bb >= positions.len() || cc >= positions.len() {
            continue;
        }
        let f = bonded::harmonic_angle_forces_2d(
            positions[aa],
            positions[bb],
            positions[cc],
            params.theta0,
            params.k,
        );
        for dim in 0..2 {
            forces[aa][dim] += f[0][dim] as f64;
            forces[bb][dim] += f[1][dim] as f64;
            forces[cc][dim] += f[2][dim] as f64;
        }
    }
}

/// Total bonded potential energy (bonds + angles).
pub fn bonded_potential_energy_2d(positions: &[[f32; 2]], topology: &Topology) -> f64 {
    let mut energy = 0.0_f64;

    for &(i, j, ref params) in &topology.bonds {
        let dx = positions[j as usize][0] - positions[i as usize][0];
        let dy = positions[j as usize][1] - positions[i as usize][1];
        let r = (dx * dx + dy * dy).sqrt();
        energy += bonded::harmonic_bond_energy(r, params.r0, params.k) as f64;
    }

    for &(a, b, c, ref params) in &topology.angles {
        let ba = [
            positions[a as usize][0] - positions[b as usize][0],
            positions[a as usize][1] - positions[b as usize][1],
        ];
        let bc = [
            positions[c as usize][0] - positions[b as usize][0],
            positions[c as usize][1] - positions[b as usize][1],
        ];
        let theta = bonded::angle_from_vectors_2d(ba, bc);
        energy += bonded::harmonic_angle_energy(theta, params.theta0, params.k) as f64;
    }

    energy
}

/// Accumulate bonded forces (bonds + angles + dihedrals) into 3D f64 force array.
///
/// Positions are f64 (MD-7). Math functions use f32 internally.
/// Newton 3: Σ forces = 0.
pub fn compute_bonded_forces_3d(
    positions: &[[f64; 3]],
    topology: &Topology,
    forces: &mut [[f64; 3]],
) {
    // ── Bonds: harmonic 3D ────────────────────────────────────────────────
    for &(i, j, ref params) in &topology.bonds {
        let ii = i as usize;
        let jj = j as usize;
        if ii >= positions.len() || jj >= positions.len() {
            continue;
        }
        let dx = (positions[jj][0] - positions[ii][0]) as f32;
        let dy = (positions[jj][1] - positions[ii][1]) as f32;
        let dz = (positions[jj][2] - positions[ii][2]) as f32;
        let f = bonded::harmonic_bond_force_3d(dx, dy, dz, params.r0, params.k);
        for d in 0..3 {
            forces[ii][d] += f[d] as f64;
            forces[jj][d] -= f[d] as f64;
        }
    }

    // ── Angles: harmonic 3D ───────────────────────────────────────────────
    for &(a, b, c, ref params) in &topology.angles {
        let aa = a as usize;
        let bb = b as usize;
        let cc = c as usize;
        if aa >= positions.len() || bb >= positions.len() || cc >= positions.len() {
            continue;
        }
        let pa = [positions[aa][0] as f32, positions[aa][1] as f32, positions[aa][2] as f32];
        let pb = [positions[bb][0] as f32, positions[bb][1] as f32, positions[bb][2] as f32];
        let pc = [positions[cc][0] as f32, positions[cc][1] as f32, positions[cc][2] as f32];
        let f = bonded::harmonic_angle_forces_3d(pa, pb, pc, params.theta0, params.k);
        for d in 0..3 {
            forces[aa][d] += f[0][d] as f64;
            forces[bb][d] += f[1][d] as f64;
            forces[cc][d] += f[2][d] as f64;
        }
    }

    // ── Dihedrals: proper dihedral 3D ─────────────────────────────────────
    for &(a, b, c, d, ref params) in &topology.dihedrals {
        let ai = a as usize;
        let bi = b as usize;
        let ci = c as usize;
        let di = d as usize;
        if ai >= positions.len() || bi >= positions.len()
            || ci >= positions.len() || di >= positions.len()
        {
            continue;
        }
        let pa = [positions[ai][0] as f32, positions[ai][1] as f32, positions[ai][2] as f32];
        let pb = [positions[bi][0] as f32, positions[bi][1] as f32, positions[bi][2] as f32];
        let pc = [positions[ci][0] as f32, positions[ci][1] as f32, positions[ci][2] as f32];
        let pd = [positions[di][0] as f32, positions[di][1] as f32, positions[di][2] as f32];
        let f = bonded::dihedral_forces_3d(pa, pb, pc, pd, params.k, params.n, params.delta);
        for dim in 0..3 {
            forces[ai][dim] += f[0][dim] as f64;
            forces[bi][dim] += f[1][dim] as f64;
            forces[ci][dim] += f[2][dim] as f64;
            forces[di][dim] += f[3][dim] as f64;
        }
    }
}

/// Total bonded potential energy (bonds + angles + dihedrals) in 3D.
pub fn bonded_potential_energy_3d(positions: &[[f64; 3]], topology: &Topology) -> f64 {
    let mut energy = 0.0_f64;

    for &(i, j, ref params) in &topology.bonds {
        let dx = positions[j as usize][0] - positions[i as usize][0];
        let dy = positions[j as usize][1] - positions[i as usize][1];
        let dz = positions[j as usize][2] - positions[i as usize][2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt() as f32;
        energy += bonded::harmonic_bond_energy(r, params.r0, params.k) as f64;
    }

    for &(a, b, c, ref params) in &topology.angles {
        let ba = [
            (positions[a as usize][0] - positions[b as usize][0]) as f32,
            (positions[a as usize][1] - positions[b as usize][1]) as f32,
            (positions[a as usize][2] - positions[b as usize][2]) as f32,
        ];
        let bc = [
            (positions[c as usize][0] - positions[b as usize][0]) as f32,
            (positions[c as usize][1] - positions[b as usize][1]) as f32,
            (positions[c as usize][2] - positions[b as usize][2]) as f32,
        ];
        let theta = bonded::angle_from_vectors_3d(ba, bc);
        energy += bonded::harmonic_angle_energy(theta, params.theta0, params.k) as f64;
    }

    for &(a, b, c, d, ref params) in &topology.dihedrals {
        let pa = [positions[a as usize][0] as f32, positions[a as usize][1] as f32, positions[a as usize][2] as f32];
        let pb = [positions[b as usize][0] as f32, positions[b as usize][1] as f32, positions[b as usize][2] as f32];
        let pc = [positions[c as usize][0] as f32, positions[c as usize][1] as f32, positions[c as usize][2] as f32];
        let pd = [positions[d as usize][0] as f32, positions[d as usize][1] as f32, positions[d as usize][2] as f32];
        let phi = bonded::dihedral_from_positions_3d(pa, pb, pc, pd);
        energy += bonded::dihedral_energy(phi, params.k, params.n, params.delta) as f64;
    }

    energy
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::topology::{AngleParams, BondParams};

    #[test]
    fn bond_forces_newton3_sum_zero() {
        let topo = Topology::linear_chain(3, BondParams { r0: 1.0, k: 100.0 });
        // Stretch: particles at 0, 1.2, 2.5 (both bonds stretched)
        let positions = [[0.0, 0.0], [1.2, 0.0], [2.5, 0.0]];
        let mut forces = [[0.0f64; 2]; 3];
        compute_bonded_forces_2d(&positions, &topo, &mut forces);
        let sum_x: f64 = forces.iter().map(|f| f[0]).sum();
        let sum_y: f64 = forces.iter().map(|f| f[1]).sum();
        assert!(sum_x.abs() < 1e-6, "Newton 3 x: {sum_x}");
        assert!(sum_y.abs() < 1e-6, "Newton 3 y: {sum_y}");
    }

    #[test]
    fn bond_forces_pull_stretched_particles() {
        let topo = Topology::linear_chain(2, BondParams { r0: 1.0, k: 100.0 });
        let positions = [[0.0, 0.0], [2.0, 0.0]]; // stretched: r=2 > r0=1
        let mut forces = [[0.0f64; 2]; 2];
        compute_bonded_forces_2d(&positions, &topo, &mut forces);
        assert!(forces[0][0] > 0.0, "particle 0 pulled toward 1");
        assert!(forces[1][0] < 0.0, "particle 1 pulled toward 0");
    }

    #[test]
    fn angle_forces_with_topology() {
        let mut topo = Topology::linear_chain(3, BondParams { r0: 1.0, k: 100.0 });
        topo.infer_angles_from_bonds(AngleParams {
            theta0: std::f32::consts::PI, // 180 degrees
            k: 50.0,
        });
        assert_eq!(topo.angles.len(), 1);
        // Right angle: should have restoring force toward 180
        let positions = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0]];
        let mut forces = [[0.0f64; 2]; 3];
        compute_bonded_forces_2d(&positions, &topo, &mut forces);
        let sum_x: f64 = forces.iter().map(|f| f[0]).sum();
        let sum_y: f64 = forces.iter().map(|f| f[1]).sum();
        assert!(sum_x.abs() < 1e-2, "angle forces sum x: {sum_x}");
        assert!(sum_y.abs() < 1e-2, "angle forces sum y: {sum_y}");
    }

    #[test]
    fn bonded_energy_zero_at_equilibrium() {
        let topo = Topology::linear_chain(3, BondParams { r0: 1.0, k: 100.0 });
        // All bonds at equilibrium length, straight line
        let positions = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let e = bonded_potential_energy_2d(&positions, &topo);
        assert!(e.abs() < 1e-6, "energy at equilibrium: {e}");
    }

    #[test]
    fn bonded_energy_positive_when_stretched() {
        let topo = Topology::linear_chain(2, BondParams { r0: 1.0, k: 100.0 });
        let positions = [[0.0, 0.0], [2.0, 0.0]]; // stretched
        let e = bonded_potential_energy_2d(&positions, &topo);
        assert!(e > 0.0, "stretched bond has positive energy: {e}");
    }

    // ── 3D bonded forces ────────────────────────────────────────────────

    #[test]
    fn bond_forces_3d_newton3_sum_zero() {
        let topo = Topology::linear_chain(3, BondParams { r0: 1.0, k: 100.0 });
        let positions = [[0.0, 0.0, 0.0], [1.2, 0.0, 0.3], [2.5, 0.1, 0.0]];
        let mut forces = [[0.0f64; 3]; 3];
        compute_bonded_forces_3d(&positions, &topo, &mut forces);
        let sum_x: f64 = forces.iter().map(|f| f[0]).sum();
        let sum_y: f64 = forces.iter().map(|f| f[1]).sum();
        let sum_z: f64 = forces.iter().map(|f| f[2]).sum();
        assert!(sum_x.abs() < 1e-2, "Newton 3 x: {sum_x}");
        assert!(sum_y.abs() < 1e-2, "Newton 3 y: {sum_y}");
        assert!(sum_z.abs() < 1e-2, "Newton 3 z: {sum_z}");
    }

    #[test]
    fn bonded_energy_3d_zero_at_equilibrium() {
        let topo = Topology::linear_chain(3, BondParams { r0: 1.0, k: 100.0 });
        let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let e = bonded_potential_energy_3d(&positions, &topo);
        assert!(e.abs() < 1e-6, "energy at equilibrium: {e}");
    }

    #[test]
    fn bonded_energy_3d_positive_when_stretched() {
        let topo = Topology::linear_chain(2, BondParams { r0: 1.0, k: 100.0 });
        let positions = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let e = bonded_potential_energy_3d(&positions, &topo);
        assert!(e > 0.0, "stretched bond has positive energy: {e}");
    }

    #[test]
    fn chain_dynamics_converges_to_equilibrium() {
        // 5-particle chain, initially stretched. Bonded forces + Verlet should
        // bring bond lengths toward r0.
        let r0 = 1.0;
        let k = 200.0;
        let mut topo = Topology::linear_chain(5, BondParams { r0, k });
        topo.infer_angles_from_bonds(AngleParams {
            theta0: std::f32::consts::PI,
            k: 20.0,
        });

        // Initially: evenly spaced at 1.5 (stretched from r0=1.0)
        let mut pos: Vec<[f32; 2]> = (0..5).map(|i| [i as f32 * 1.5, 0.0]).collect();
        let mut vel = vec![[0.0f32; 2]; 5];
        let mut acc = vec![[0.0f32; 2]; 5];
        let dt = 0.001_f32;

        // Run 5000 Verlet steps with damping
        for _ in 0..5000 {
            // Position step
            for i in 0..5 {
                pos[i][0] += vel[i][0] * dt + 0.5 * acc[i][0] * dt * dt;
                pos[i][1] += vel[i][1] * dt + 0.5 * acc[i][1] * dt * dt;
            }
            // Forces
            let mut forces = vec![[0.0f64; 2]; 5];
            compute_bonded_forces_2d(&pos, &topo, &mut forces);
            // Velocity step + light damping
            for i in 0..5 {
                let ax_new = forces[i][0] as f32;
                let ay_new = forces[i][1] as f32;
                vel[i][0] = (vel[i][0] + 0.5 * (acc[i][0] + ax_new) * dt) * 0.999;
                vel[i][1] = (vel[i][1] + 0.5 * (acc[i][1] + ay_new) * dt) * 0.999;
                acc[i] = [ax_new, ay_new];
            }
        }

        // Check bond lengths near r0
        for i in 0..4 {
            let dx = pos[i + 1][0] - pos[i][0];
            let dy = pos[i + 1][1] - pos[i][1];
            let r = (dx * dx + dy * dy).sqrt();
            assert!(
                (r - r0).abs() < 0.1,
                "bond {i}-{}: r={r:.3}, expected ~{r0}",
                i + 1,
            );
        }
    }
}
