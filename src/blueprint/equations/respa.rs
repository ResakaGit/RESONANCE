//! R5d: r-RESPA 2-level multiple timestepping — pure math.
//!
//! Reversible Reference System Propagator Algorithm (Tuckerman et al., JCP 1992).
//!
//! Inner loop (dt_inner): fast-varying forces (bonded).
//! Outer loop (dt_outer = n_inner * dt_inner): slow-varying forces (non-bonded).
//!
//! ~2x speedup: non-bonded computed every n_inner steps instead of every step.
//!
//! Axiom 4: each timescale has its own dissipation rate.
//! Axiom 7: force split along distance ranges (short=fast, long=slow).

/// r-RESPA velocity Verlet: outer step with n_inner substeps.
///
/// Pipeline per outer step:
///   1. v += 0.5 * dt_outer * a_slow
///   2. for _ in 0..n_inner:
///      a. v += 0.5 * dt_inner * a_fast
///      b. x += dt_inner * v
///      c. recompute a_fast from x
///      d. v += 0.5 * dt_inner * a_fast
///   3. recompute a_slow from x
///   4. v += 0.5 * dt_outer * a_slow
///
/// `positions`, `velocities`: modified in-place.
/// `forces_fast`: closure that computes fast forces from positions.
/// `forces_slow`: closure that computes slow forces from positions.
///
/// Returns (total_fast_evals, total_slow_evals).
pub fn respa_step<F, S>(
    positions: &mut [[f64; 3]],
    velocities: &mut [[f64; 3]],
    masses: &[f64],
    dt_inner: f64,
    n_inner: usize,
    forces_fast: &F,
    forces_slow: &S,
) -> (usize, usize)
where
    F: Fn(&[[f64; 3]]) -> Vec<[f64; 3]>,
    S: Fn(&[[f64; 3]]) -> Vec<[f64; 3]>,
{
    let n = positions.len();

    // 1. Half-kick from slow forces
    let f_slow = forces_slow(positions);
    for i in 0..n {
        let inv_m = 1.0 / masses[i];
        let half_dt_outer = 0.5 * dt_inner * n_inner as f64;
        for d in 0..3 {
            velocities[i][d] += half_dt_outer * f_slow[i][d] * inv_m;
        }
    }

    // 2. Inner loop: n_inner fast substeps
    let mut fast_evals = 0;
    for _ in 0..n_inner {
        // 2a. Half-kick from fast forces
        let f_fast = forces_fast(positions);
        fast_evals += 1;
        let half_dt_inner = 0.5 * dt_inner;
        for i in 0..n {
            let inv_m = 1.0 / masses[i];
            for d in 0..3 {
                velocities[i][d] += half_dt_inner * f_fast[i][d] * inv_m;
            }
        }

        // 2b. Drift
        for i in 0..n {
            for d in 0..3 {
                positions[i][d] += dt_inner * velocities[i][d];
            }
        }

        // 2c-d. Recompute fast forces + second half-kick
        let f_fast2 = forces_fast(positions);
        fast_evals += 1;
        for i in 0..n {
            let inv_m = 1.0 / masses[i];
            for d in 0..3 {
                velocities[i][d] += half_dt_inner * f_fast2[i][d] * inv_m;
            }
        }
    }

    // 3-4. Recompute slow forces + half-kick
    let f_slow2 = forces_slow(positions);
    let half_dt_outer = 0.5 * dt_inner * n_inner as f64;
    for i in 0..n {
        let inv_m = 1.0 / masses[i];
        for d in 0..3 {
            velocities[i][d] += half_dt_outer * f_slow2[i][d] * inv_m;
        }
    }

    (fast_evals, 2) // 2 slow evaluations per outer step
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respa_harmonic_energy_bounded() {
        // Two-particle harmonic oscillator: fast = spring, slow = none
        let k = 100.0;
        let r0 = 1.0;
        let mut pos = vec![[0.0, 0.0, 0.0], [1.1, 0.0, 0.0]]; // slightly stretched
        let mut vel = vec![[0.0; 3]; 2];
        let masses = vec![1.0, 1.0];

        let fast = |p: &[[f64; 3]]| -> Vec<[f64; 3]> {
            let dx = p[1][0] - p[0][0];
            let r = dx.abs();
            let f = k * (r - r0) * dx.signum(); // positive when stretched
            vec![[f, 0.0, 0.0], [-f, 0.0, 0.0]] // pull toward each other
        };
        let slow = |_p: &[[f64; 3]]| -> Vec<[f64; 3]> { vec![[0.0; 3]; 2] };

        let e0 = {
            let dx = pos[1][0] - pos[0][0];
            0.5 * k * (dx - r0) * (dx - r0) + 0.5 * (vel[0][0] * vel[0][0] + vel[1][0] * vel[1][0])
        };

        for _ in 0..1000 {
            respa_step(&mut pos, &mut vel, &masses, 0.001, 4, &fast, &slow);
        }

        let dx = pos[1][0] - pos[0][0];
        let e_final = 0.5 * k * (dx - r0) * (dx - r0) + 0.5 * (vel[0][0] * vel[0][0] + vel[1][0] * vel[1][0]);
        let drift = ((e_final - e0) / e0).abs();
        assert!(drift < 0.01, "RESPA energy drift {drift:.4} exceeds 1%");
    }

    #[test]
    fn respa_eval_counts() {
        let pos = &mut vec![[0.0; 3]; 2];
        let vel = &mut vec![[0.0; 3]; 2];
        let masses = vec![1.0, 1.0];
        let fast = |_: &[[f64; 3]]| vec![[0.0; 3]; 2];
        let slow = |_: &[[f64; 3]]| vec![[0.0; 3]; 2];

        let (fast_evals, slow_evals) = respa_step(pos, vel, &masses, 0.001, 5, &fast, &slow);
        assert_eq!(fast_evals, 10, "n_inner=5 → 2*5=10 fast evals");
        assert_eq!(slow_evals, 2, "always 2 slow evals per outer step");
    }

    #[test]
    fn respa_n_inner_1_matches_verlet() {
        // n_inner=1 should be equivalent to standard Verlet (fast+slow combined)
        let k = 50.0;
        let r0 = 2.0;
        let dt = 0.001;
        let mut pos_r = vec![[0.0, 0.0, 0.0], [2.3, 0.0, 0.0]];
        let mut vel_r = vec![[0.1, 0.0, 0.0], [-0.1, 0.0, 0.0]];
        let masses = vec![1.0, 1.0];

        let force_fn = |p: &[[f64; 3]]| -> Vec<[f64; 3]> {
            let dx = p[1][0] - p[0][0];
            let r = dx.abs();
            let f = k * (r - r0) * dx.signum();
            vec![[-f, 0.0, 0.0], [f, 0.0, 0.0]]
        };
        let zero_fn = |_: &[[f64; 3]]| vec![[0.0; 3]; 2];

        // Run RESPA with n_inner=1 (all forces as "fast")
        for _ in 0..100 {
            respa_step(&mut pos_r, &mut vel_r, &masses, dt, 1, &force_fn, &zero_fn);
        }

        // Run standard Verlet
        let mut pos_v = vec![[0.0, 0.0, 0.0], [2.3, 0.0, 0.0]];
        let mut vel_v = vec![[0.1, 0.0, 0.0], [-0.1, 0.0, 0.0]];
        let mut acc_v = vec![[0.0; 3]; 2];
        for _ in 0..100 {
            for i in 0..2 {
                pos_v[i][0] += vel_v[i][0] * dt + 0.5 * acc_v[i][0] * dt * dt;
            }
            let f = force_fn(&pos_v);
            for i in 0..2 {
                let new_a = f[i][0];
                vel_v[i][0] += 0.5 * (acc_v[i][0] + new_a) * dt;
                acc_v[i][0] = new_a;
            }
        }

        // Should be very close (not exact due to different kick ordering)
        let dx_r = pos_r[1][0] - pos_r[0][0];
        let dx_v = pos_v[1][0] - pos_v[0][0];
        assert!(
            (dx_r - dx_v).abs() < 0.01,
            "RESPA(n=1) should match Verlet: {dx_r:.6} vs {dx_v:.6}",
        );
    }
}
