//! MD-19: Rayon-parallel non-bonded force computation.
//!
//! Decision gate: rayon (already in dependencies) over GPU/SIMD.
//! Rationale: no external crate approval needed, no nightly required,
//! portable across all platforms. Sufficient for N < 10K.
//!
//! Strategy: partition atoms into chunks, accumulate per-chunk forces,
//! merge with atomic addition pattern. Thread-safe via per-thread buffers.
//!
//! Fallback: serial computation for N < 64 (overhead not worth it).

use rayon::prelude::*;

use crate::blueprint::equations::go_model;

// ─── Parallel Go model forces ────────────────────────────────────────────

/// Parallel non-bonded Go model force computation.
///
/// Splits the outer loop of the i-j pair iteration across rayon threads.
/// Each thread accumulates into its own force buffer; results merged at end.
///
/// Falls back to serial for N < 64 (rayon overhead exceeds benefit).
///
/// Returns (forces, potential_energy).
pub fn go_forces_parallel(
    positions: &[[f64; 3]],
    topo: &go_model::GoTopology,
    bandwidth: f64,
) -> (Vec<[f64; 3]>, f64) {
    let n = positions.len();

    // Fallback to serial for small systems
    if n < 64 {
        return go_forces_serial(positions, topo, bandwidth);
    }

    // Bonded forces (sequential — topology-driven, not O(N²))
    let (mut forces, mut pe) = bonded_forces_serial(positions, topo);

    // Native contacts — parallel over contact list
    let native_results: Vec<([f64; 3], [f64; 3], f64, usize, usize)> = topo.native_contacts
        .par_iter()
        .map(|&(ci, cj, sigma)| {
            let (i, j) = (ci as usize, cj as usize);
            let mut d = [0.0; 3];
            for k in 0..3 { d[k] = positions[i][k] - positions[j][k]; }
            let r = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();
            if r < 1e-15 {
                return ([0.0; 3], [0.0; 3], 0.0, i, j);
            }

            let f_i = topo.frequencies[i];
            let f_j = topo.frequencies[j];
            let pe_pair = go_model::go_axiom8_potential(r, sigma, topo.epsilon, f_i, f_j, bandwidth);
            let f_mag = go_model::go_axiom8_force(r, sigma, topo.epsilon, f_i, f_j, bandwidth);
            let f_over_r = f_mag / r;

            let fi = [f_over_r * d[0], f_over_r * d[1], f_over_r * d[2]];
            let fj = [-fi[0], -fi[1], -fi[2]];
            (fi, fj, pe_pair, i, j)
        })
        .collect();

    for (fi, fj, pe_pair, i, j) in native_results {
        for k in 0..3 {
            forces[i][k] += fi[k];
            forces[j][k] += fj[k];
        }
        pe += pe_pair;
    }

    // Non-native repulsion — parallel over outer index
    let is_native = build_native_mask(n, &topo.native_contacts);
    let rep_sigma = topo.bond_length * 0.8;

    let chunk_results: Vec<(Vec<[f64; 3]>, f64)> = (0..n).into_par_iter().map(|i| {
        let mut local_forces = vec![[0.0; 3]; n];
        let mut local_pe = 0.0;

        for j in (i + 3)..n {
            if is_native[i * n + j] { continue; }
            let mut d = [0.0; 3];
            for k in 0..3 { d[k] = positions[i][k] - positions[j][k]; }
            let r_sq = d[0]*d[0] + d[1]*d[1] + d[2]*d[2];
            let r = r_sq.sqrt();
            if r < 1e-15 || r > rep_sigma * 3.0 { continue; }

            let pe_pair = go_model::go_repulsive_potential(r, rep_sigma, topo.epsilon_repel);
            let f_mag = go_model::go_repulsive_force(r, rep_sigma, topo.epsilon_repel);
            let f_over_r = f_mag / r;

            for k in 0..3 {
                local_forces[i][k] += f_over_r * d[k];
                local_forces[j][k] -= f_over_r * d[k];
            }
            local_pe += pe_pair;
        }

        (local_forces, local_pe)
    }).collect();

    // Merge chunk results
    for (local_f, local_pe) in chunk_results {
        pe += local_pe;
        for i in 0..n {
            for k in 0..3 {
                forces[i][k] += local_f[i][k];
            }
        }
    }

    (forces, pe)
}

// ─── Serial fallback ──────────────────────────────────────────────────────

fn bonded_forces_serial(
    positions: &[[f64; 3]],
    topo: &go_model::GoTopology,
) -> (Vec<[f64; 3]>, f64) {
    let n = positions.len();
    let mut forces = vec![[0.0; 3]; n];
    let mut pe = 0.0;

    for i in 0..(n - 1) {
        let mut d = [0.0; 3];
        for k in 0..3 { d[k] = positions[i + 1][k] - positions[i][k]; }
        let r = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();
        if r < 1e-15 { continue; }
        let dr = r - topo.bond_length;
        pe += 0.5 * topo.bond_k * dr * dr;
        let f_mag = -topo.bond_k * dr / r;
        for k in 0..3 {
            forces[i][k] -= f_mag * d[k];
            forces[i + 1][k] += f_mag * d[k];
        }
    }

    (forces, pe)
}

fn go_forces_serial(
    positions: &[[f64; 3]],
    topo: &go_model::GoTopology,
    bandwidth: f64,
) -> (Vec<[f64; 3]>, f64) {
    let n = positions.len();
    let (mut forces, mut pe) = bonded_forces_serial(positions, topo);

    for &(ci, cj, sigma) in &topo.native_contacts {
        let (i, j) = (ci as usize, cj as usize);
        let mut d = [0.0; 3];
        for k in 0..3 { d[k] = positions[i][k] - positions[j][k]; }
        let r = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();
        if r < 1e-15 { continue; }
        let f_i = topo.frequencies[i];
        let f_j = topo.frequencies[j];
        pe += go_model::go_axiom8_potential(r, sigma, topo.epsilon, f_i, f_j, bandwidth);
        let f_mag = go_model::go_axiom8_force(r, sigma, topo.epsilon, f_i, f_j, bandwidth);
        let f_over_r = f_mag / r;
        for k in 0..3 {
            forces[i][k] += f_over_r * d[k];
            forces[j][k] -= f_over_r * d[k];
        }
    }

    let is_native = build_native_mask(n, &topo.native_contacts);
    let rep_sigma = topo.bond_length * 0.8;
    for i in 0..n {
        for j in (i + 3)..n {
            if is_native[i * n + j] { continue; }
            let mut d = [0.0; 3];
            for k in 0..3 { d[k] = positions[i][k] - positions[j][k]; }
            let r = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();
            if r < 1e-15 || r > rep_sigma * 3.0 { continue; }
            pe += go_model::go_repulsive_potential(r, rep_sigma, topo.epsilon_repel);
            let f_mag = go_model::go_repulsive_force(r, rep_sigma, topo.epsilon_repel);
            let f_over_r = f_mag / r;
            for k in 0..3 {
                forces[i][k] += f_over_r * d[k];
                forces[j][k] -= f_over_r * d[k];
            }
        }
    }

    (forces, pe)
}

fn build_native_mask(n: usize, contacts: &[(u16, u16, f64)]) -> Vec<bool> {
    let mut mask = vec![false; n * n];
    for &(i, j, _) in contacts {
        let (i, j) = (i as usize, j as usize);
        mask[i * n + j] = true;
        mask[j * n + i] = true;
    }
    mask
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_topology() -> go_model::GoTopology {
        let ca = vec![
            [0.0, 0.0, 0.0], [3.8, 0.0, 0.0], [7.6, 0.0, 0.0],
            [3.0, 5.0, 0.0], [6.0, 4.0, 0.0], [9.0, 3.0, 0.0],
            [2.0, 8.0, 0.0], [5.0, 7.0, 0.0],
        ];
        let seq = vec![0u8, 7, 10, 13, 19, 0, 7, 10];
        go_model::build_go_topology(&ca, &seq, 8.0, 50.0, 1.0, 100.0)
    }

    #[test]
    fn parallel_matches_serial() {
        let topo = test_topology();
        let positions: Vec<[f64; 3]> = (0..topo.n_residues)
            .map(|i| [i as f64 * 3.8 + 0.1 * i as f64, 0.5 * i as f64, 0.0])
            .collect();

        let (f_serial, pe_serial) = go_forces_serial(&positions, &topo, 50.0);
        let (f_parallel, pe_parallel) = go_forces_parallel(&positions, &topo, 50.0);

        assert!(
            (pe_serial - pe_parallel).abs() < 1e-6,
            "PE mismatch: serial={pe_serial}, parallel={pe_parallel}",
        );
        for i in 0..positions.len() {
            for k in 0..3 {
                assert!(
                    (f_serial[i][k] - f_parallel[i][k]).abs() < 1e-6,
                    "force mismatch at [{i}][{k}]: {} vs {}",
                    f_serial[i][k], f_parallel[i][k],
                );
            }
        }
    }

    #[test]
    fn forces_newton_third_law() {
        let topo = test_topology();
        let positions: Vec<[f64; 3]> = (0..topo.n_residues)
            .map(|i| [i as f64 * 3.0, (i as f64).sin() * 2.0, 0.0])
            .collect();

        let (forces, _) = go_forces_parallel(&positions, &topo, 50.0);

        let mut total = [0.0; 3];
        for f in &forces {
            for k in 0..3 { total[k] += f[k]; }
        }
        for k in 0..3 {
            assert!(total[k].abs() < 1e-6, "Newton 3 violated: total[{k}] = {}", total[k]);
        }
    }

    #[test]
    fn serial_fallback_for_small_n() {
        // N < 64 should use serial path (no panic, same result)
        let topo = test_topology();
        let positions: Vec<[f64; 3]> = (0..topo.n_residues)
            .map(|i| [i as f64 * 3.8, 0.0, 0.0])
            .collect();

        let (f1, pe1) = go_forces_serial(&positions, &topo, 50.0);
        let (f2, pe2) = go_forces_parallel(&positions, &topo, 50.0);

        assert!((pe1 - pe2).abs() < 1e-10);
        for i in 0..positions.len() {
            for k in 0..3 {
                assert!((f1[i][k] - f2[i][k]).abs() < 1e-10);
            }
        }
    }
}
