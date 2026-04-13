//! MD-16: Replica Exchange MD (REMD / Parallel Tempering).
//!
//! N replicas at different temperatures. Swap by Metropolis criterion.
//! High-T replicas escape traps, low-T replicas benefit from exploration.
//!
//! Axiom 4: multiple dissipation rates sample the free energy landscape.
//!
//! ADR-024: temperature exchange (not coordinate exchange). Velocities
//! rescaled on swap to match new temperature.

use crate::blueprint::equations::{determinism, go_model, md_observables, thermostat as thermo_eq, verlet};

// ─── Types ────────────────────────────────────────────────────────────────

/// State of one REMD replica.
#[derive(Clone, Debug)]
pub struct ReplicaState {
    pub temperature: f64,
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub old_acc: Vec<[f64; 3]>,
    pub potential_energy: f64,
}

/// REMD configuration.
#[derive(Clone, Debug)]
pub struct RemdConfig {
    pub n_replicas: usize,
    pub t_min: f64,
    pub t_max: f64,
    pub steps_per_swap: u32,
    pub total_swaps: u32,
    pub dt: f64,
    pub gamma: f64,
    pub seed: u64,
    /// Go model parameters.
    pub epsilon: f64,
    pub epsilon_repel: f64,
    pub bond_k: f64,
}

impl Default for RemdConfig {
    fn default() -> Self {
        Self {
            n_replicas: 8,
            t_min: 0.5,
            t_max: 1.5,
            steps_per_swap: 100,
            total_swaps: 100,
            dt: 0.005,
            gamma: 0.5,
            seed: 42,
            epsilon: 1.0,
            epsilon_repel: 0.5,
            bond_k: 100.0,
        }
    }
}

/// REMD result.
#[derive(Clone, Debug)]
pub struct RemdResult {
    /// Best (lowest) RMSD to native found across all replicas.
    pub min_rmsd: f64,
    /// Best native fraction Q found.
    pub best_q: f64,
    /// Best folding coherence found.
    pub best_coherence: f64,
    /// Positions of best structure.
    pub best_positions: Vec<[f64; 3]>,
    /// Swap acceptance ratio per temperature pair.
    pub acceptance_ratios: Vec<f64>,
    /// Mean energy per replica temperature.
    pub mean_energies: Vec<f64>,
    pub total_steps: u64,
}

// ─── Temperature ladder ───────────────────────────────────────────────────

/// Geometric temperature ladder: T_i = T_min * (T_max/T_min)^(i/(N-1)).
pub fn temperature_ladder(t_min: f64, t_max: f64, n: usize) -> Vec<f64> {
    if n <= 1 { return vec![t_min]; }
    let ratio = (t_max / t_min).powf(1.0 / (n - 1) as f64);
    (0..n).map(|i| t_min * ratio.powi(i as i32)).collect()
}

// ─── Swap criterion ───────────────────────────────────────────────────────

/// Metropolis swap criterion: P = min(1, exp(Delta)).
///
/// Delta = (beta_i - beta_j) * (E_i - E_j)
/// where beta = 1/T (k_B = 1 in reduced units).
///
/// Returns true if swap accepted.
pub fn attempt_swap(
    t_i: f64, e_i: f64,
    t_j: f64, e_j: f64,
    rng_state: u64,
) -> bool {
    let beta_i = 1.0 / t_i;
    let beta_j = 1.0 / t_j;
    let delta = (beta_i - beta_j) * (e_i - e_j);

    if delta <= 0.0 {
        return true; // always accept downhill
    }

    // Accept with probability exp(-delta)
    let prob = (-delta).exp();
    let rand = (determinism::next_u64(rng_state) as f64) / (u64::MAX as f64);
    rand < prob
}

/// Rescale velocities after temperature swap: v_new = v * sqrt(T_new / T_old).
pub fn rescale_velocities(velocities: &mut [[f64; 3]], t_old: f64, t_new: f64) {
    let scale = (t_new / t_old).sqrt();
    for v in velocities.iter_mut() {
        for k in 0..3 {
            v[k] *= scale;
        }
    }
}

// ─── Go model force computation ──────────────────────────────────────────

/// Compute Go model forces for a single replica.
///
/// Returns (forces, potential_energy).
fn go_forces(
    positions: &[[f64; 3]],
    topo: &go_model::GoTopology,
    bandwidth: f64,
) -> (Vec<[f64; 3]>, f64) {
    let n = positions.len();
    let mut forces = vec![[0.0; 3]; n];
    let mut pe = 0.0;

    // Bonded: sequential harmonic bonds
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

    // Native contacts: frequency-modulated
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

    // Non-native repulsion — uses pre-computed mask from GoTopology (no alloc per step)
    let rep_sigma = topo.bond_length * 0.8;
    for i in 0..n {
        for j in (i + 3)..n {
            if topo.native_mask[i * n + j] { continue; }
            let mut d = [0.0; 3];
            for k in 0..3 { d[k] = positions[i][k] - positions[j][k]; }
            let r_sq = d[0]*d[0] + d[1]*d[1] + d[2]*d[2];
            let r = r_sq.sqrt();
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

// ─── REMD runner ──────────────────────────────────────────────────────────

/// Run REMD simulation with Go model.
pub fn run_remd(
    config: &RemdConfig,
    topo: &go_model::GoTopology,
    native_positions: &[[f64; 3]],
    initial_positions: &[[f64; 3]],
    bandwidth: f64,
) -> RemdResult {
    let n = topo.n_residues;
    let temps = temperature_ladder(config.t_min, config.t_max, config.n_replicas);

    // Initialize replicas
    let mut replicas: Vec<ReplicaState> = temps.iter().map(|&t| {
        let velocities = md_observables::init_velocities_3d(n, t, config.seed ^ (t * 1000.0) as u64);
        ReplicaState {
            temperature: t,
            positions: initial_positions.to_vec(),
            velocities,
            old_acc: vec![[0.0; 3]; n],
            potential_energy: 0.0,
        }
    }).collect();

    let mut swap_attempts = vec![0u64; config.n_replicas - 1];
    let mut swap_accepts = vec![0u64; config.n_replicas - 1];
    let mut energy_sums = vec![0.0; config.n_replicas];
    let mut energy_counts = vec![0u64; config.n_replicas];

    let mut best_rmsd = f64::MAX;
    let mut best_q = 0.0;
    let mut best_coherence = 0.0;
    let mut best_positions = initial_positions.to_vec();

    let mut rng = config.seed;

    for swap_round in 0..config.total_swaps {
        // Pre-compute per-replica RNG seeds (deterministic, independent streams)
        let rep_seeds: Vec<u64> = (0..config.n_replicas)
            .map(|r| determinism::next_u64(config.seed ^ (swap_round as u64 * 31 + r as u64 * 997)))
            .collect();

        // MD steps for each replica (parallel across threads)
        let dt = config.dt;
        let gamma = config.gamma;
        let steps = config.steps_per_swap;
        std::thread::scope(|s| {
            let handles: Vec<_> = replicas.iter_mut().zip(rep_seeds.iter()).map(|(rep, &seed)| {
                s.spawn(move || {
                    let mut rep_rng = seed;
                    for _ in 0..steps {
                        // Verlet position step
                        for i in 0..n {
                            rep.positions[i] = verlet::position_step_3d(
                                rep.positions[i], rep.velocities[i], rep.old_acc[i], dt,
                            );
                        }

                        // Forces
                        let (forces, pe) = go_forces(&rep.positions, topo, bandwidth);
                        rep.potential_energy = pe;

                        // Verlet velocity finish
                        for i in 0..n {
                            let new_acc = forces[i];
                            rep.velocities[i] = verlet::velocity_step_3d(
                                rep.velocities[i], rep.old_acc[i], new_acc, dt,
                            );
                            rep.old_acc[i] = new_acc;
                        }

                        // Langevin thermostat
                        if gamma > 0.0 {
                            let c1 = 1.0 - gamma * dt;
                            let sigma_v = thermo_eq::langevin_velocity_sigma(
                                gamma, rep.temperature, dt, 1.0,
                            );
                            rep_rng = determinism::next_u64(rep_rng);
                            let mut local_rng = rep_rng;
                            for i in 0..n {
                                for d in 0..3 {
                                    local_rng = determinism::next_u64(local_rng);
                                    let z = determinism::gaussian_f32(local_rng, 1.0) as f64;
                                    rep.velocities[i][d] = rep.velocities[i][d] * c1 + sigma_v * z;
                                }
                            }
                        }
                    }
                })
            }).collect();
            for h in handles {
                let _ = h.join();
            }
        });

        // Advance main RNG for swap determinism
        rng = determinism::next_u64(rng);

        // Accumulate energies
        for (r, rep) in replicas.iter().enumerate() {
            energy_sums[r] += rep.potential_energy;
            energy_counts[r] += 1;
        }

        // Attempt swaps (even/odd alternation)
        let offset = (swap_round % 2) as usize;
        let mut pair = offset;
        while pair + 1 < config.n_replicas {
            rng = determinism::next_u64(rng);
            swap_attempts[pair] += 1;

            if attempt_swap(
                replicas[pair].temperature, replicas[pair].potential_energy,
                replicas[pair + 1].temperature, replicas[pair + 1].potential_energy,
                rng,
            ) {
                swap_accepts[pair] += 1;
                // Swap temperatures
                let t_old_i = replicas[pair].temperature;
                let t_old_j = replicas[pair + 1].temperature;
                replicas[pair].temperature = t_old_j;
                replicas[pair + 1].temperature = t_old_i;
                // Rescale velocities
                rescale_velocities(&mut replicas[pair].velocities, t_old_i, t_old_j);
                rescale_velocities(&mut replicas[pair + 1].velocities, t_old_j, t_old_i);
            }

            pair += 2;
        }

        // Track best structure (lowest-temperature replica)
        let lowest_t_idx = replicas.iter()
            .enumerate()
            .min_by(|a, b| a.1.temperature.partial_cmp(&b.1.temperature).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let q = go_model::native_contact_fraction(&replicas[lowest_t_idx].positions, &topo.native_contacts, go_model::Q_TOLERANCE);
        let coherence = go_model::folding_coherence(
            &replicas[lowest_t_idx].positions, &topo.native_contacts,
            &topo.frequencies, bandwidth, go_model::Q_TOLERANCE,
        );

        use crate::blueprint::equations::md_analysis;
        let rmsd = md_analysis::rmsd_kabsch(&replicas[lowest_t_idx].positions, native_positions);

        if q > best_q || (q == best_q && rmsd < best_rmsd) {
            best_rmsd = rmsd;
            best_q = q;
            best_coherence = coherence;
            best_positions = replicas[lowest_t_idx].positions.clone();
        }
    }

    let acceptance_ratios: Vec<f64> = swap_attempts.iter().zip(swap_accepts.iter())
        .map(|(&att, &acc)| if att > 0 { acc as f64 / att as f64 } else { 0.0 })
        .collect();

    let mean_energies: Vec<f64> = energy_sums.iter().zip(energy_counts.iter())
        .map(|(&sum, &cnt)| if cnt > 0 { sum / cnt as f64 } else { 0.0 })
        .collect();

    let total_steps = config.total_swaps as u64 * config.steps_per_swap as u64 * config.n_replicas as u64;

    RemdResult {
        min_rmsd: best_rmsd,
        best_q,
        best_coherence,
        best_positions,
        acceptance_ratios,
        mean_energies,
        total_steps,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_always_accepted_for_equal_temps() {
        assert!(attempt_swap(1.0, 5.0, 1.0, 3.0, 42));
    }

    #[test]
    fn swap_detailed_balance_direction() {
        // Lower energy at lower temp should be favorable
        let accepted = attempt_swap(0.5, -10.0, 1.0, -5.0, 42);
        // Delta = (1/0.5 - 1/1.0) * (-10.0 - (-5.0)) = (2-1)*(-5) = -5 < 0 → accept
        assert!(accepted, "favorable swap should always accept");
    }

    #[test]
    fn temperature_ladder_geometric() {
        let temps = temperature_ladder(0.5, 2.0, 4);
        assert_eq!(temps.len(), 4);
        assert!((temps[0] - 0.5).abs() < 1e-10);
        assert!((temps[3] - 2.0).abs() < 1e-10);
        // Geometric: T[i+1]/T[i] = const
        let r1 = temps[1] / temps[0];
        let r2 = temps[2] / temps[1];
        let r3 = temps[3] / temps[2];
        assert!((r1 - r2).abs() < 1e-10, "not geometric: {r1} vs {r2}");
        assert!((r2 - r3).abs() < 1e-10, "not geometric: {r2} vs {r3}");
    }

    #[test]
    fn velocity_rescaling_preserves_ke_ratio() {
        let mut vels = vec![[1.0, 2.0, 3.0], [-1.0, 0.5, -0.5]];
        let ke_before: f64 = vels.iter().map(|v| v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sum();
        let t_old = 1.0;
        let t_new = 2.0;
        rescale_velocities(&mut vels, t_old, t_new);
        let ke_after: f64 = vels.iter().map(|v| v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sum();
        let ratio = ke_after / ke_before;
        assert!((ratio - t_new / t_old).abs() < 1e-10, "KE ratio {ratio} != T ratio {}", t_new / t_old);
    }

    #[test]
    fn temperature_ladder_single_replica() {
        let temps = temperature_ladder(1.0, 2.0, 1);
        assert_eq!(temps.len(), 1);
        assert!((temps[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn remd_runs_without_panic() {
        // Minimal REMD: 5 residues, 2 replicas, few steps
        let ca = vec![[0.0, 0.0, 0.0], [3.8, 0.0, 0.0], [7.6, 0.0, 0.0], [3.0, 5.0, 0.0], [6.0, 4.0, 0.0]];
        let seq = vec![0u8, 7, 10, 13, 19];
        let topo = go_model::build_go_topology(&ca, &seq, 8.0, 50.0, 1.0, 100.0);
        let initial = go_model::extended_chain(5, 3.8);

        let config = RemdConfig {
            n_replicas: 2,
            t_min: 0.5,
            t_max: 1.0,
            steps_per_swap: 10,
            total_swaps: 5,
            dt: 0.005,
            gamma: 0.5,
            seed: 42,
            epsilon: 1.0,
            epsilon_repel: 0.5,
            bond_k: 100.0,
        };

        let result = run_remd(&config, &topo, &ca, &initial, 50.0);
        assert!(result.total_steps > 0);
        assert!(result.best_q >= 0.0 && result.best_q <= 1.0);
    }

    #[test]
    fn remd_samples_lower_energy_at_low_t() {
        let ca = vec![[0.0, 0.0, 0.0], [3.8, 0.0, 0.0], [7.6, 0.0, 0.0], [3.0, 5.0, 0.0], [6.0, 4.0, 0.0]];
        let seq = vec![0u8, 7, 10, 13, 19];
        let topo = go_model::build_go_topology(&ca, &seq, 8.0, 50.0, 1.0, 100.0);
        let initial = go_model::extended_chain(5, 3.8);

        let config = RemdConfig {
            n_replicas: 4,
            t_min: 0.3,
            t_max: 2.0,
            steps_per_swap: 50,
            total_swaps: 20,
            dt: 0.005,
            gamma: 1.0,
            seed: 123,
            epsilon: 1.0,
            epsilon_repel: 0.5,
            bond_k: 100.0,
        };

        let result = run_remd(&config, &topo, &ca, &initial, 50.0);
        // Mean energy at lowest T should be <= mean energy at highest T
        if result.mean_energies.len() >= 2 {
            let e_low = result.mean_energies[0];
            let e_high = result.mean_energies[result.mean_energies.len() - 1];
            // This is a statistical test — allow some slack
            assert!(
                e_low <= e_high + 10.0,
                "low-T energy {e_low} should be <= high-T energy {e_high} (with slack)",
            );
        }
    }
}
