//! Velocity Verlet integrator — pure math.
//!
//! Symplectic: conserves phase-space volume => bounded energy drift.
//! O(dt^2) per step vs Euler O(dt). Standard for molecular dynamics.
//!
//! Axiom 1: energy is the only quantity — Verlet preserves Hamiltonian.
//! Axiom 4: dissipation external to integrator (separate system).

/// Position half-step: x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt^2.
#[inline]
pub fn position_step(x: f32, v: f32, a: f32, dt: f32) -> f32 {
    x + v * dt + 0.5 * a * dt * dt
}

/// Velocity full step: v(t+dt) = v(t) + 0.5*(a_old + a_new)*dt.
#[inline]
pub fn velocity_step(v: f32, a_old: f32, a_new: f32, dt: f32) -> f32 {
    v + 0.5 * (a_old + a_new) * dt
}

// ─── 3D f64 variants (MD-7) ─────────────────────────────────────────────────

/// Position half-step in 3D: x_i(t+dt) = x_i(t) + v_i(t)*dt + 0.5*a_i(t)*dt^2.
#[inline]
pub fn position_step_3d(x: [f64; 3], v: [f64; 3], a: [f64; 3], dt: f64) -> [f64; 3] {
    let half_dt2 = 0.5 * dt * dt;
    [
        x[0] + v[0] * dt + a[0] * half_dt2,
        x[1] + v[1] * dt + a[1] * half_dt2,
        x[2] + v[2] * dt + a[2] * half_dt2,
    ]
}

/// Velocity full step in 3D: v_i(t+dt) = v_i(t) + 0.5*(a_old_i + a_new_i)*dt.
#[inline]
pub fn velocity_step_3d(v: [f64; 3], a_old: [f64; 3], a_new: [f64; 3], dt: f64) -> [f64; 3] {
    let half_dt = 0.5 * dt;
    [
        v[0] + (a_old[0] + a_new[0]) * half_dt,
        v[1] + (a_old[1] + a_new[1]) * half_dt,
        v[2] + (a_old[2] + a_new[2]) * half_dt,
    ]
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_exact_for_constant_force() {
        let (x0, v0, a, dt) = (1.0, 2.0, 3.0, 0.1);
        let x1 = position_step(x0, v0, a, dt);
        let expected = x0 + v0 * dt + 0.5 * a * dt * dt;
        assert!((x1 - expected).abs() < 1e-6, "x1={x1}, expected={expected}");
    }

    #[test]
    fn velocity_exact_for_constant_force() {
        let (v0, a, dt) = (2.0, 3.0, 0.1);
        let v1 = velocity_step(v0, a, a, dt);
        let expected = v0 + a * dt;
        assert!((v1 - expected).abs() < 1e-6);
    }

    #[test]
    fn zero_force_uniform_motion() {
        let x = position_step(1.0, 5.0, 0.0, 0.1);
        assert!((x - 1.5).abs() < 1e-6);
        let v = velocity_step(5.0, 0.0, 0.0, 0.1);
        assert!((v - 5.0).abs() < 1e-6);
    }

    #[test]
    fn reversibility_constant_force() {
        let (x0, v0, a, dt) = (5.0, 3.0, -2.0, 0.05);
        let x1 = position_step(x0, v0, a, dt);
        let v1 = velocity_step(v0, a, a, dt);
        let x_back = position_step(x1, v1, a, -dt);
        let v_back = velocity_step(v1, a, a, -dt);
        assert!((x_back - x0).abs() < 1e-5, "x: {x_back} vs {x0}");
        assert!((v_back - v0).abs() < 1e-5, "v: {v_back} vs {v0}");
    }

    #[test]
    fn harmonic_oscillator_energy_bounded() {
        // Spring: a = -k*x, k=1. Verlet should bound energy drift.
        let k = 1.0_f32;
        let dt = 0.01_f32;
        let (mut x, mut v) = (1.0_f32, 0.0_f32);
        let e0 = 0.5 * k * x * x + 0.5 * v * v;

        let mut max_drift: f32 = 0.0;
        for _ in 0..10_000 {
            let a_old = -k * x;
            x = position_step(x, v, a_old, dt);
            let a_new = -k * x;
            v = velocity_step(v, a_old, a_new, dt);
            let drift = ((0.5 * k * x * x + 0.5 * v * v - e0) / e0).abs();
            if drift > max_drift {
                max_drift = drift;
            }
        }
        assert!(max_drift < 1e-4, "energy drift {max_drift} exceeds 1e-4");
    }

    #[test]
    fn euler_drifts_more_than_verlet() {
        let k = 1.0_f32;
        let dt = 0.01_f32;
        let e0 = 0.5_f32; // x=1, v=0 → E = 0.5*k*x²

        // Verlet
        let (mut xv, mut vv) = (1.0_f32, 0.0_f32);
        for _ in 0..10_000 {
            let a_old = -k * xv;
            xv = position_step(xv, vv, a_old, dt);
            vv = velocity_step(vv, a_old, -k * xv, dt);
        }
        let verlet_drift = ((0.5 * k * xv * xv + 0.5 * vv * vv - e0) / e0).abs();

        // Euler
        let (mut xe, mut ve) = (1.0_f32, 0.0_f32);
        for _ in 0..10_000 {
            ve += -k * xe * dt;
            xe += ve * dt;
        }
        let euler_drift = ((0.5 * k * xe * xe + 0.5 * ve * ve - e0) / e0).abs();

        assert!(
            verlet_drift < euler_drift,
            "verlet {verlet_drift} should drift less than euler {euler_drift}",
        );
    }

    // ── 3D f64 tests (MD-7) ─────────────────────────────────────────────

    #[test]
    fn position_3d_exact_for_constant_force() {
        let x = [1.0, 2.0, 3.0];
        let v = [0.1, -0.2, 0.3];
        let a = [0.5, -0.5, 1.0];
        let dt = 0.01;
        let result = position_step_3d(x, v, a, dt);
        for i in 0..3 {
            let expected = x[i] + v[i] * dt + 0.5 * a[i] * dt * dt;
            assert!((result[i] - expected).abs() < 1e-12, "dim {i}");
        }
    }

    #[test]
    fn velocity_3d_exact_for_constant_force() {
        let v = [1.0, -2.0, 3.0];
        let a = [0.5, -0.5, 1.0];
        let dt = 0.01;
        let result = velocity_step_3d(v, a, a, dt);
        for i in 0..3 {
            let expected = v[i] + a[i] * dt;
            assert!((result[i] - expected).abs() < 1e-12, "dim {i}");
        }
    }

    #[test]
    fn harmonic_oscillator_3d_energy_bounded() {
        let k = 1.0_f64;
        let dt = 0.001_f64;
        let mut x = [1.0, 0.5, -0.3];
        let mut v = [0.0; 3];
        let e0: f64 = (0..3).map(|i| 0.5 * k * x[i] * x[i] + 0.5 * v[i] * v[i]).sum();

        let mut max_drift = 0.0_f64;
        for _ in 0..50_000 {
            let a_old = [- k * x[0], -k * x[1], -k * x[2]];
            x = position_step_3d(x, v, a_old, dt);
            let a_new = [-k * x[0], -k * x[1], -k * x[2]];
            v = velocity_step_3d(v, a_old, a_new, dt);
            let e: f64 = (0..3).map(|i| 0.5 * k * x[i] * x[i] + 0.5 * v[i] * v[i]).sum();
            let drift = ((e - e0) / e0).abs();
            if drift > max_drift {
                max_drift = drift;
            }
        }
        assert!(max_drift < 1e-6, "3D energy drift {max_drift} exceeds 1e-6");
    }

    #[test]
    fn reversibility_3d() {
        let x0 = [5.0, -3.0, 1.0];
        let v0 = [0.1, -0.2, 0.3];
        let a = [0.5, -1.0, 0.25];
        let dt = 0.05;
        let x1 = position_step_3d(x0, v0, a, dt);
        let v1 = velocity_step_3d(v0, a, a, dt);
        let x_back = position_step_3d(x1, v1, a, -dt);
        let v_back = velocity_step_3d(v1, a, a, -dt);
        for i in 0..3 {
            assert!((x_back[i] - x0[i]).abs() < 1e-10, "x dim {i}");
            assert!((v_back[i] - v0[i]).abs() < 1e-10, "v dim {i}");
        }
    }

    #[test]
    fn zero_force_3d_uniform_motion() {
        let x = [1.0, 2.0, 3.0];
        let v = [5.0, -5.0, 10.0];
        let a = [0.0; 3];
        let dt = 0.1;
        let x1 = position_step_3d(x, v, a, dt);
        let v1 = velocity_step_3d(v, a, a, dt);
        for i in 0..3 {
            assert!((x1[i] - (x[i] + v[i] * dt)).abs() < 1e-12);
            assert!((v1[i] - v[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn verlet_3d_matches_2d_in_plane() {
        let dt = 0.01_f32;
        let (x2d, v2d, a2d) = (1.0_f32, 2.0_f32, -0.5_f32);
        let x2d_result = position_step(x2d, v2d, a2d, dt);
        let v2d_result = velocity_step(v2d, a2d, a2d, dt);

        let dt64 = dt as f64;
        let x3d = position_step_3d([x2d as f64, 0.0, 0.0], [v2d as f64, 0.0, 0.0], [a2d as f64, 0.0, 0.0], dt64);
        let v3d = velocity_step_3d([v2d as f64, 0.0, 0.0], [a2d as f64, 0.0, 0.0], [a2d as f64, 0.0, 0.0], dt64);

        assert!((x3d[0] - x2d_result as f64).abs() < 1e-5, "position mismatch");
        assert!((v3d[0] - v2d_result as f64).abs() < 1e-5, "velocity mismatch");
        assert!(x3d[1].abs() < 1e-15, "y should be zero");
        assert!(x3d[2].abs() < 1e-15, "z should be zero");
    }

    #[test]
    fn momentum_conserved_two_body() {
        let (k, dt) = (1.0_f32, 0.01_f32);
        let (m1, m2) = (1.0_f32, 2.0_f32);
        let (mut x1, mut x2) = (0.0_f32, 2.0_f32);
        let (mut v1, mut v2) = (1.0_f32, -0.5_f32);
        let p0 = m1 * v1 + m2 * v2;

        for _ in 0..1_000 {
            let f = k * (x2 - x1 - 1.0); // equilibrium at dx=1
            let (a1, a2) = (f / m1, -f / m2);
            x1 = position_step(x1, v1, a1, dt);
            x2 = position_step(x2, v2, a2, dt);
            let f_new = k * (x2 - x1 - 1.0);
            v1 = velocity_step(v1, a1, f_new / m1, dt);
            v2 = velocity_step(v2, a2, -f_new / m2, dt);
        }
        let p_final = m1 * v1 + m2 * v2;
        assert!(
            (p_final - p0).abs() < 1e-4,
            "momentum: {p0} -> {p_final}",
        );
    }
}
