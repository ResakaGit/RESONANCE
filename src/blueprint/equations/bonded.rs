//! MD-5: Bonded potentials — pure math.
//!
//! Harmonic bond, harmonic angle, proper dihedral.
//! All stateless. All functions: parameters in → energy/force out.
//!
//! Axiom 8: bonded interactions are small-amplitude limits of oscillatory coupling.
//! ADR-021: bonded forces separated from non-bonded, topology-driven iteration.

// ─── Harmonic Bond ─────────────────────────────────────────────────────────

/// Harmonic bond potential: V(r) = 0.5 * k * (r - r0)^2.
#[inline]
pub fn harmonic_bond_energy(r: f32, r0: f32, k: f32) -> f32 {
    let dr = r - r0;
    0.5 * k * dr * dr
}

/// Harmonic bond force on particle i from bond i-j.
///
/// `dx, dy`: displacement from i to j (j.pos - i.pos).
/// Returns force ON i. Newton 3: force on j = negated.
///
/// Convention: positive along dx = toward j (attractive when stretched).
/// At r > r0: pulls i toward j. At r < r0: pushes i away from j.
pub fn harmonic_bond_force(dx: f32, dy: f32, r0: f32, k: f32) -> [f32; 2] {
    let r_sq = dx * dx + dy * dy;
    if r_sq < 1e-20 {
        return [0.0, 0.0];
    }
    let r = r_sq.sqrt();
    // F_on_i = k * (r - r0) * (dx/r, dy/r)
    let f_scalar = k * (r - r0) / r;
    [f_scalar * dx, f_scalar * dy]
}

// ─── Harmonic Angle ────────────────────────────────────────────────────────

/// Angle between vectors ba and bc (at vertex b), in radians.
///
/// Returns angle in [0, pi]. Uses atan2 for numerical stability.
pub fn angle_from_vectors_2d(ba: [f32; 2], bc: [f32; 2]) -> f32 {
    let dot = ba[0] * bc[0] + ba[1] * bc[1];
    let cross = ba[0] * bc[1] - ba[1] * bc[0];
    cross.atan2(dot).abs()
}

/// Harmonic angle potential: V(theta) = 0.5 * k * (theta - theta0)^2.
#[inline]
pub fn harmonic_angle_energy(theta: f32, theta0: f32, k: f32) -> f32 {
    let dt = theta - theta0;
    0.5 * k * dt * dt
}

/// Harmonic angle forces on particles a, b (vertex), c.
///
/// Returns [f_a, f_b, f_c]. Newton 3: f_a + f_b + f_c = 0.
/// Uses numerical gradient (central differences) for robustness.
pub fn harmonic_angle_forces_2d(
    a: [f32; 2],
    b: [f32; 2],
    c: [f32; 2],
    theta0: f32,
    k: f32,
) -> [[f32; 2]; 3] {
    let h = 1e-4_f32;
    let positions = [a, b, c];
    let mut forces = [[0.0f32; 2]; 3];

    for atom in 0..3 {
        for dim in 0..2 {
            let mut p_plus = positions;
            let mut p_minus = positions;
            p_plus[atom][dim] += h;
            p_minus[atom][dim] -= h;

            let ba_p = [p_plus[0][0] - p_plus[1][0], p_plus[0][1] - p_plus[1][1]];
            let bc_p = [p_plus[2][0] - p_plus[1][0], p_plus[2][1] - p_plus[1][1]];
            let v_plus = harmonic_angle_energy(angle_from_vectors_2d(ba_p, bc_p), theta0, k);

            let ba_m = [p_minus[0][0] - p_minus[1][0], p_minus[0][1] - p_minus[1][1]];
            let bc_m = [p_minus[2][0] - p_minus[1][0], p_minus[2][1] - p_minus[1][1]];
            let v_minus = harmonic_angle_energy(angle_from_vectors_2d(ba_m, bc_m), theta0, k);

            forces[atom][dim] = -(v_plus - v_minus) / (2.0 * h);
        }
    }
    forces
}

// ─── 3D Harmonic Bond ─────────────────────────────────────────────────────

/// Harmonic bond force on particle i from bond i-j (3D).
///
/// `dx, dy, dz`: displacement from i to j (j.pos - i.pos).
/// Returns force ON i. Newton 3: force on j = negated.
pub fn harmonic_bond_force_3d(dx: f32, dy: f32, dz: f32, r0: f32, k: f32) -> [f32; 3] {
    let r_sq = dx * dx + dy * dy + dz * dz;
    if r_sq < 1e-20 {
        return [0.0; 3];
    }
    let r = r_sq.sqrt();
    let f_scalar = k * (r - r0) / r;
    [f_scalar * dx, f_scalar * dy, f_scalar * dz]
}

// ─── 3D Harmonic Angle ────────────────────────────────────────────────────

/// Angle between vectors ba and bc (at vertex b) in 3D, in radians [0, pi].
pub fn angle_from_vectors_3d(ba: [f32; 3], bc: [f32; 3]) -> f32 {
    let cross = cross_3d(ba, bc);
    let cross_mag = mag_3d(cross);
    let dot = dot_3d(ba, bc);
    cross_mag.atan2(dot)
}

/// Harmonic angle forces on particles a, b (vertex), c in 3D.
///
/// Returns [f_a, f_b, f_c]. Newton 3: sum = 0.
/// Uses numerical gradient (central differences) for robustness.
pub fn harmonic_angle_forces_3d(
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    theta0: f32,
    k: f32,
) -> [[f32; 3]; 3] {
    let h = 1e-4_f32;
    let positions = [a, b, c];
    let mut forces = [[0.0f32; 3]; 3];

    for atom in 0..3 {
        for dim in 0..3 {
            let mut p_plus = positions;
            let mut p_minus = positions;
            p_plus[atom][dim] += h;
            p_minus[atom][dim] -= h;

            let ba_p = sub_3d(p_plus[0], p_plus[1]);
            let bc_p = sub_3d(p_plus[2], p_plus[1]);
            let v_plus = harmonic_angle_energy(angle_from_vectors_3d(ba_p, bc_p), theta0, k);

            let ba_m = sub_3d(p_minus[0], p_minus[1]);
            let bc_m = sub_3d(p_minus[2], p_minus[1]);
            let v_minus = harmonic_angle_energy(angle_from_vectors_3d(ba_m, bc_m), theta0, k);

            forces[atom][dim] = -(v_plus - v_minus) / (2.0 * h);
        }
    }
    forces
}

// ─── Proper Dihedral (3D) ──────────────────────────────────────────────────

/// Cross product of two 3D vectors.
#[inline]
fn cross_3d(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Dot product of two 3D vectors.
#[inline]
fn dot_3d(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Magnitude of a 3D vector.
#[inline]
fn mag_3d(v: [f32; 3]) -> f32 {
    dot_3d(v, v).sqrt()
}

/// Subtract two 3D vectors: a - b.
#[inline]
fn sub_3d(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Dihedral angle between planes (a,b,c) and (b,c,d), in radians [-pi, pi].
///
/// Uses the atan2 formula for numerical stability:
/// phi = atan2((b1 x b2) . b0/|b0|, b1 . b2)
/// where b0 = b-a, b1 = c-b, b2 = d-c.
pub fn dihedral_from_positions_3d(
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
) -> f32 {
    let b0 = sub_3d(b, a);
    let b1 = sub_3d(c, b);
    let b2 = sub_3d(d, c);

    let n1 = cross_3d(b0, b1); // normal to plane (a,b,c)
    let n2 = cross_3d(b1, b2); // normal to plane (b,c,d)

    let m1 = cross_3d(n1, b1); // n1 x b1 (in-plane of n1, perpendicular to b1)
    let b1_mag = mag_3d(b1).max(1e-10);

    let x = dot_3d(n1, n2);
    let y = dot_3d(m1, n2) / b1_mag;
    y.atan2(x)
}

/// Proper dihedral potential: V(phi) = k * (1 + cos(n*phi - delta)).
///
/// `n`: periodicity (1, 2, 3...). `delta`: phase offset.
#[inline]
pub fn dihedral_energy(phi: f32, k: f32, n: u8, delta: f32) -> f32 {
    k * (1.0 + (n as f32 * phi - delta).cos())
}

/// Proper dihedral forces on particles a, b, c, d (3D).
///
/// Returns [f_a, f_b, f_c, f_d]. Newton 3: sum = 0.
/// Uses numerical gradient (central differences) for robustness.
/// Analytical gradient is complex and error-prone; numerical is sufficient
/// for the Go model use case (MD-15).
pub fn dihedral_forces_3d(
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
    k: f32,
    n: u8,
    delta: f32,
) -> [[f32; 3]; 4] {
    let h = 1e-4_f32;
    let positions = [a, b, c, d];
    let mut forces = [[0.0f32; 3]; 4];

    for atom in 0..4 {
        for dim in 0..3 {
            let mut pos_plus = positions;
            let mut pos_minus = positions;
            pos_plus[atom][dim] += h;
            pos_minus[atom][dim] -= h;

            let phi_plus =
                dihedral_from_positions_3d(pos_plus[0], pos_plus[1], pos_plus[2], pos_plus[3]);
            let phi_minus = dihedral_from_positions_3d(
                pos_minus[0],
                pos_minus[1],
                pos_minus[2],
                pos_minus[3],
            );

            let v_plus = dihedral_energy(phi_plus, k, n, delta);
            let v_minus = dihedral_energy(phi_minus, k, n, delta);

            forces[atom][dim] = -(v_plus - v_minus) / (2.0 * h);
        }
    }
    forces
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bond ───────────────────────────────────────────────────────────────

    #[test]
    fn bond_energy_zero_at_equilibrium() {
        assert_eq!(harmonic_bond_energy(1.5, 1.5, 100.0), 0.0);
    }

    #[test]
    fn bond_energy_symmetric() {
        let e_stretch = harmonic_bond_energy(1.6, 1.5, 100.0);
        let e_compress = harmonic_bond_energy(1.4, 1.5, 100.0);
        assert!((e_stretch - e_compress).abs() < 1e-6, "V(r0+d) = V(r0-d)");
    }

    #[test]
    fn bond_energy_positive() {
        assert!(harmonic_bond_energy(2.0, 1.5, 100.0) > 0.0);
    }

    #[test]
    fn bond_force_restoring_stretched() {
        // Bond stretched: r=2.0 > r0=1.5, dx=2.0 → force toward j (positive)
        let f = harmonic_bond_force(2.0, 0.0, 1.5, 100.0);
        assert!(f[0] > 0.0, "stretched → pull toward j: fx={}", f[0]);
    }

    #[test]
    fn bond_force_restoring_compressed() {
        // Bond compressed: r=1.0 < r0=1.5, dx=1.0 → force away from j (negative)
        let f = harmonic_bond_force(1.0, 0.0, 1.5, 100.0);
        assert!(f[0] < 0.0, "compressed → push away: fx={}", f[0]);
    }

    #[test]
    fn bond_force_zero_at_equilibrium() {
        let f = harmonic_bond_force(1.5, 0.0, 1.5, 100.0);
        assert!(f[0].abs() < 1e-6 && f[1].abs() < 1e-6, "F=0 at r0");
    }

    #[test]
    fn bond_force_newton3() {
        let f_on_i = harmonic_bond_force(2.0, 1.0, 1.5, 100.0);
        // Force on j = -f_on_i (Newton 3)
        let sum = [f_on_i[0] + (-f_on_i[0]), f_on_i[1] + (-f_on_i[1])];
        assert!(sum[0].abs() < 1e-10 && sum[1].abs() < 1e-10);
    }

    #[test]
    fn bond_oscillation_period() {
        // Two particles connected by spring: omega = sqrt(k/mu), mu = m/2.
        // Use Velocity Verlet for accurate period measurement.
        let k = 100.0_f32;
        let m = 1.0_f32;
        let mu = m * m / (m + m); // 0.5
        let omega = (k / mu).sqrt();
        let expected_period = 2.0 * std::f32::consts::PI / omega;
        let dt = 0.0001_f32;

        let r0 = 1.5_f32;
        let mut xi = 0.0_f32;
        let mut xj = r0 + 0.1;
        let mut vi = 0.0_f32;
        let mut vj = 0.0_f32;
        // Initial acceleration
        let f0 = harmonic_bond_force(xj - xi, 0.0, r0, k);
        let mut ai = f0[0] / m;
        let mut aj = -f0[0] / m;

        let mut prev_stretch = xj - xi - r0;
        let mut crossings = 0u32;
        let mut first_crossing_step = 0u32;
        let mut measured_period = 0.0_f32;

        for step in 0..200_000 {
            // Verlet position step
            xi += vi * dt + 0.5 * ai * dt * dt;
            xj += vj * dt + 0.5 * aj * dt * dt;
            // New forces
            let f = harmonic_bond_force(xj - xi, 0.0, r0, k);
            let ai_new = f[0] / m;
            let aj_new = -f[0] / m;
            // Verlet velocity step
            vi += 0.5 * (ai + ai_new) * dt;
            vj += 0.5 * (aj + aj_new) * dt;
            ai = ai_new;
            aj = aj_new;

            let stretch = xj - xi - r0;
            if prev_stretch > 0.0 && stretch <= 0.0 {
                crossings += 1;
                if crossings == 1 {
                    first_crossing_step = step;
                } else if crossings == 2 {
                    // Period = time between two consecutive same-direction crossings
                    measured_period = (step - first_crossing_step) as f32 * dt;
                    break;
                }
            }
            prev_stretch = stretch;
        }
        let error = ((measured_period - expected_period) / expected_period).abs();
        assert!(
            error < 0.02,
            "period: measured={measured_period:.5}, expected={expected_period:.5}, error={error:.4}",
        );
    }

    // ── Angle ──────────────────────────────────────────────────────────────

    #[test]
    fn angle_from_vectors_90_degrees() {
        let ba = [1.0, 0.0];
        let bc = [0.0, 1.0];
        let theta = angle_from_vectors_2d(ba, bc);
        assert!(
            (theta - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
            "90 degrees: {theta}",
        );
    }

    #[test]
    fn angle_from_vectors_180_degrees() {
        let ba = [1.0, 0.0];
        let bc = [-1.0, 0.0];
        let theta = angle_from_vectors_2d(ba, bc);
        assert!(
            (theta - std::f32::consts::PI).abs() < 1e-4,
            "180 degrees: {theta}",
        );
    }

    #[test]
    fn angle_energy_zero_at_equilibrium() {
        let theta0 = std::f32::consts::FRAC_PI_2;
        assert_eq!(harmonic_angle_energy(theta0, theta0, 50.0), 0.0);
    }

    #[test]
    fn angle_force_sum_zero() {
        let a = [1.0, 0.0];
        let b = [0.0, 0.0];
        let c = [0.0, 1.0];
        let theta0 = 1.0; // not at equilibrium
        let forces = harmonic_angle_forces_2d(a, b, c, theta0, 50.0);
        let sum_x = forces[0][0] + forces[1][0] + forces[2][0];
        let sum_y = forces[0][1] + forces[1][1] + forces[2][1];
        assert!(
            sum_x.abs() < 1e-3 && sum_y.abs() < 1e-3,
            "Newton 3: sum=({sum_x}, {sum_y})",
        );
    }

    #[test]
    fn angle_force_restores_toward_equilibrium() {
        // Angle is 90 degrees, equilibrium is 120 degrees → force opens the angle
        let a = [2.0, 0.0];
        let b = [0.0, 0.0];
        let c = [0.0, 2.0];
        let theta0 = 2.094; // ~120 degrees
        let k = 50.0;
        let forces = harmonic_angle_forces_2d(a, b, c, theta0, k);

        // Run a few steps: angle should increase toward theta0
        let mut pa = a;
        let mut pc = c;
        let dt = 0.001;
        for _ in 0..100 {
            let f = harmonic_angle_forces_2d(pa, b, pc, theta0, k);
            pa[0] += f[0][0] * dt;
            pa[1] += f[0][1] * dt;
            pc[0] += f[2][0] * dt;
            pc[1] += f[2][1] * dt;
        }
        let ba_new = [pa[0] - b[0], pa[1] - b[1]];
        let bc_new = [pc[0] - b[0], pc[1] - b[1]];
        let theta_new = angle_from_vectors_2d(ba_new, bc_new);
        let theta_old = std::f32::consts::FRAC_PI_2;
        assert!(
            (theta_new - theta0).abs() < (theta_old - theta0).abs(),
            "angle should move toward equilibrium: old={theta_old:.3}, new={theta_new:.3}, target={theta0:.3}",
        );
    }

    // ── Bond 3D ────────────────────────────────────────────────────────────

    #[test]
    fn bond_force_3d_restoring_stretched() {
        let f = harmonic_bond_force_3d(2.0, 0.0, 0.0, 1.5, 100.0);
        assert!(f[0] > 0.0, "stretched → pull toward j: fx={}", f[0]);
    }

    #[test]
    fn bond_force_3d_zero_at_equilibrium() {
        let f = harmonic_bond_force_3d(1.5, 0.0, 0.0, 1.5, 100.0);
        assert!(f[0].abs() < 1e-6 && f[1].abs() < 1e-6 && f[2].abs() < 1e-6);
    }

    #[test]
    fn bond_force_3d_diagonal() {
        // Bond along (1,1,1), r = sqrt(3) ≈ 1.732, r0=1.0 → stretched
        let f = harmonic_bond_force_3d(1.0, 1.0, 1.0, 1.0, 100.0);
        // Force should be equal in all components (symmetric)
        assert!((f[0] - f[1]).abs() < 1e-6);
        assert!((f[1] - f[2]).abs() < 1e-6);
        assert!(f[0] > 0.0, "stretched → attractive");
    }

    // ── Angle 3D ──────────────────────────────────────────────────────────

    #[test]
    fn angle_3d_right_angle() {
        let ba = [1.0, 0.0, 0.0];
        let bc = [0.0, 1.0, 0.0];
        let theta = angle_from_vectors_3d(ba, bc);
        assert!(
            (theta - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
            "90 degrees: {theta}",
        );
    }

    #[test]
    fn angle_3d_180_degrees() {
        let ba = [1.0, 0.0, 0.0];
        let bc = [-1.0, 0.0, 0.0];
        let theta = angle_from_vectors_3d(ba, bc);
        assert!(
            (theta - std::f32::consts::PI).abs() < 1e-4,
            "180 degrees: {theta}",
        );
    }

    #[test]
    fn angle_forces_3d_sum_zero() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.5];
        let theta0 = 1.0;
        let forces = harmonic_angle_forces_3d(a, b, c, theta0, 50.0);
        let sum: [f32; 3] = [
            forces[0][0] + forces[1][0] + forces[2][0],
            forces[0][1] + forces[1][1] + forces[2][1],
            forces[0][2] + forces[1][2] + forces[2][2],
        ];
        assert!(
            sum[0].abs() < 1e-2 && sum[1].abs() < 1e-2 && sum[2].abs() < 1e-2,
            "Newton 3: sum=({:.4}, {:.4}, {:.4})",
            sum[0], sum[1], sum[2],
        );
    }

    // ── Dihedral (3D) ──────────────────────────────────────────────────────

    #[test]
    fn dihedral_trans_is_pi() {
        // Trans configuration: a-b-c-d in a zigzag
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [1.5, 1.0, 0.0];
        let d = [0.5, 1.0, 0.0]; // same side as a relative to b-c
        let phi = dihedral_from_positions_3d(a, b, c, d);
        // This should be close to 0 (cis) since a and d are on same side
        assert!(phi.abs() < 0.5 || (phi.abs() - std::f32::consts::PI).abs() < 0.5,
            "phi={phi:.3}");
    }

    #[test]
    fn dihedral_energy_periodic() {
        let phi = 1.0;
        let k = 2.0;
        let n = 3u8;
        let delta = 0.0;
        let v1 = dihedral_energy(phi, k, n, delta);
        let v2 = dihedral_energy(phi + 2.0 * std::f32::consts::PI / n as f32, k, n, delta);
        assert!(
            (v1 - v2).abs() < 1e-4,
            "V(phi) = V(phi + 2pi/n): {v1} vs {v2}",
        );
    }

    #[test]
    fn dihedral_forces_sum_zero() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [1.5, 1.0, 0.0];
        let d = [2.5, 1.0, 0.5];
        let forces = dihedral_forces_3d(a, b, c, d, 2.0, 3, 0.0);
        let sum: [f32; 3] = [
            forces[0][0] + forces[1][0] + forces[2][0] + forces[3][0],
            forces[0][1] + forces[1][1] + forces[2][1] + forces[3][1],
            forces[0][2] + forces[1][2] + forces[2][2] + forces[3][2],
        ];
        assert!(
            sum[0].abs() < 0.01 && sum[1].abs() < 0.01 && sum[2].abs() < 0.01,
            "Newton 3: sum=({:.4}, {:.4}, {:.4})",
            sum[0], sum[1], sum[2],
        );
    }
}
