//! MD-11: SHAKE + RATTLE constraint solver — pure math.
//!
//! Enforces rigid bond lengths in molecular dynamics with holonomic constraints.
//! SHAKE corrects positions after Verlet position step.
//! RATTLE corrects velocities after Verlet velocity step.
//!
//! Pipeline: Verlet position → SHAKE → forces → Verlet velocity → RATTLE
//!
//! Axiom 2 (Pool Invariant): constraint forces are perpendicular to the
//! constraint surface → do no work → no energy created or destroyed.

// ─── SHAKE (position correction) ──────────────────────────────────────────

/// One SHAKE iteration for a single distance constraint.
///
/// After unconstrained Verlet position update, the bond (i,j) has length != d_target.
/// SHAKE computes a Lagrange multiplier λ and corrects both positions along the
/// old bond direction to restore the target distance.
///
/// # Arguments
/// * `r_i`, `r_j` — current (unconstrained) positions after Verlet step
/// * `r_i_old`, `r_j_old` — positions from previous timestep (before Verlet)
/// * `d_target` — target bond length
/// * `m_i`, `m_j` — masses
///
/// # Returns
/// Position corrections `(delta_i, delta_j)` to add to `r_i` and `r_j`.
pub fn shake_pair(
    r_i: [f64; 3], r_j: [f64; 3],
    r_i_old: [f64; 3], r_j_old: [f64; 3],
    d_target: f64, m_i: f64, m_j: f64,
) -> ([f64; 3], [f64; 3]) {
    // Current bond vector
    let mut d = [0.0; 3];
    for k in 0..3 {
        d[k] = r_i[k] - r_j[k];
    }
    let d_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];

    // Old bond vector (reference direction)
    let mut d_old = [0.0; 3];
    for k in 0..3 {
        d_old[k] = r_i_old[k] - r_j_old[k];
    }

    // Dot product: d_old · d_new
    let dot: f64 = d_old[0] * d[0] + d_old[1] * d[1] + d_old[2] * d[2];

    // Lagrange multiplier
    let inv_mass_sum = 1.0 / m_i + 1.0 / m_j;
    let lambda = (d_sq - d_target * d_target) / (2.0 * inv_mass_sum * dot);

    // Corrections along old bond direction
    let mut delta_i = [0.0; 3];
    let mut delta_j = [0.0; 3];
    for k in 0..3 {
        delta_i[k] = -lambda * d_old[k] / m_i;
        delta_j[k] = lambda * d_old[k] / m_j;
    }

    (delta_i, delta_j)
}

/// Iterative SHAKE solver for multiple constraints.
///
/// Iterates until all bond lengths are within `tolerance` of their target,
/// or `max_iter` iterations are reached.
///
/// # Arguments
/// * `positions` — mutable positions (corrected in-place)
/// * `old_positions` — positions before Verlet step
/// * `constraints` — list of `(atom_i, atom_j, d_target)`
/// * `masses` — per-atom masses
/// * `tolerance` — convergence criterion (relative to d_target)
/// * `max_iter` — maximum iterations
///
/// # Returns
/// Number of iterations used. If == max_iter, convergence not reached.
pub fn shake_solve(
    positions: &mut [[f64; 3]],
    old_positions: &[[f64; 3]],
    constraints: &[(u16, u16, f64)],
    masses: &[f64],
    tolerance: f64,
    max_iter: u32,
) -> u32 {
    for iter in 0..max_iter {
        let mut converged = true;

        for &(ai, aj, d_target) in constraints {
            let i = ai as usize;
            let j = aj as usize;

            // Check current distance
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = positions[i][k] - positions[j][k];
                d_sq += dk * dk;
            }
            let d_current = d_sq.sqrt();
            let error = (d_current - d_target).abs() / d_target;

            if error > tolerance {
                converged = false;
                let (delta_i, delta_j) = shake_pair(
                    positions[i], positions[j],
                    old_positions[i], old_positions[j],
                    d_target, masses[i], masses[j],
                );
                for k in 0..3 {
                    positions[i][k] += delta_i[k];
                    positions[j][k] += delta_j[k];
                }
            }
        }

        if converged {
            return iter + 1;
        }
    }

    // Non-convergence: log max residual for diagnostics
    #[cfg(debug_assertions)]
    {
        let mut max_err = 0.0_f64;
        for &(ai, aj, d_target) in constraints {
            let (i, j) = (ai as usize, aj as usize);
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = positions[i][k] - positions[j][k];
                d_sq += dk * dk;
            }
            let err = (d_sq.sqrt() - d_target).abs() / d_target;
            if err > max_err { max_err = err; }
        }
        eprintln!("SHAKE: did not converge in {max_iter} iterations (max residual: {max_err:.2e})");
    }

    max_iter
}

// ─── RATTLE (velocity correction) ─────────────────────────────────────────

/// RATTLE velocity correction for a single constrained pair.
///
/// After Verlet velocity update, enforces that the relative velocity is
/// perpendicular to the bond vector: v_ij · r_ij = 0.
///
/// # Arguments
/// * `r_i`, `r_j` — constrained positions (after SHAKE)
/// * `v_i`, `v_j` — unconstrained velocities (after Verlet velocity step)
/// * `d_target` — target bond length (for normalization)
/// * `m_i`, `m_j` — masses
///
/// # Returns
/// Velocity corrections `(delta_vi, delta_vj)` to add to `v_i` and `v_j`.
pub fn rattle_pair(
    r_i: [f64; 3], r_j: [f64; 3],
    v_i: [f64; 3], v_j: [f64; 3],
    _d_target: f64, m_i: f64, m_j: f64,
) -> ([f64; 3], [f64; 3]) {
    // Bond vector
    let mut d = [0.0; 3];
    for k in 0..3 {
        d[k] = r_i[k] - r_j[k];
    }
    let d_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];

    // Relative velocity
    let mut v_rel = [0.0; 3];
    for k in 0..3 {
        v_rel[k] = v_i[k] - v_j[k];
    }

    // Component of relative velocity along bond: v_rel · d
    let v_dot_d: f64 = v_rel[0] * d[0] + v_rel[1] * d[1] + v_rel[2] * d[2];

    // Lagrange multiplier
    let inv_mass_sum = 1.0 / m_i + 1.0 / m_j;
    let mu = v_dot_d / (inv_mass_sum * d_sq);

    // Velocity corrections
    let mut delta_vi = [0.0; 3];
    let mut delta_vj = [0.0; 3];
    for k in 0..3 {
        delta_vi[k] = -mu * d[k] / m_i;
        delta_vj[k] = mu * d[k] / m_j;
    }

    (delta_vi, delta_vj)
}

/// Apply RATTLE to all constraints.
///
/// Single pass — RATTLE converges in one iteration for independent constraints
/// (like water, where bonds share only the O atom).
pub fn rattle_solve(
    positions: &[[f64; 3]],
    velocities: &mut [[f64; 3]],
    constraints: &[(u16, u16, f64)],
    masses: &[f64],
) {
    for &(ai, aj, d_target) in constraints {
        let i = ai as usize;
        let j = aj as usize;
        let (dv_i, dv_j) = rattle_pair(
            positions[i], positions[j],
            velocities[i], velocities[j],
            d_target, masses[i], masses[j],
        );
        for k in 0..3 {
            velocities[i][k] += dv_i[k];
            velocities[j][k] += dv_j[k];
        }
    }
}

// ─── Constraint list builder ──────────────────────────────────────────────

/// Build constraint list from a Topology, selecting only bonds with k above threshold.
///
/// For TIP3P water with high-k harmonic bonds, all O-H bonds become rigid constraints.
/// Returns `(atom_i, atom_j, d_target)` tuples.
pub fn constraints_from_topology(
    topology: &crate::batch::topology::Topology,
    k_threshold: f64,
) -> Vec<(u16, u16, f64)> {
    topology.bonds
        .iter()
        .filter(|(_, _, params)| params.k >= k_threshold)
        .map(|&(i, j, params)| (i, j, params.r0))
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: distance between two 3D points.
    fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
        let mut sq = 0.0;
        for k in 0..3 {
            sq += (a[k] - b[k]).powi(2);
        }
        sq.sqrt()
    }

    #[test]
    fn shake_maintains_bond_length() {
        // Two equal-mass atoms, initially at correct distance, then perturbed
        let d_target = 1.0;
        let m = 1.0;
        let old_i = [0.0, 0.0, 0.0];
        let old_j = [d_target, 0.0, 0.0];

        // Simulate a Verlet step that stretches the bond
        let new_i = [-0.05, 0.01, 0.0]; // moved left
        let new_j = [1.08, -0.02, 0.0]; // moved right (bond too long)

        let mut pos = [new_i, new_j];
        let old_pos = [old_i, old_j];
        let constraints = [(0u16, 1u16, d_target)];
        let masses = [m, m];

        let iters = shake_solve(&mut pos, &old_pos, &constraints, &masses, 1e-10, 100);

        let d_final = dist(pos[0], pos[1]);
        assert!(
            (d_final - d_target).abs() < 1e-6,
            "bond length {d_final} != {d_target}, iters={iters}",
        );
    }

    #[test]
    fn shake_converges_quickly_for_water() {
        // TIP3P water: O at origin, H1 and H2 at correct geometry
        let r_oh = crate::batch::ff::water::TIP3P_R_OH;
        let half_angle = crate::batch::ff::water::TIP3P_ANGLE_HOH / 2.0;
        let h_dx = r_oh * half_angle.sin();
        let h_dy = r_oh * half_angle.cos();

        let old_o = [0.0, 0.0, 0.0];
        let old_h1 = [h_dx, h_dy, 0.0];
        let old_h2 = [-h_dx, h_dy, 0.0];

        // Perturb positions (simulate Verlet step)
        let new_o = [0.01, -0.02, 0.005];
        let new_h1 = [h_dx + 0.03, h_dy - 0.01, 0.01];
        let new_h2 = [-h_dx - 0.02, h_dy + 0.015, -0.005];

        let mut pos = [new_o, new_h1, new_h2];
        let old_pos = [old_o, old_h1, old_h2];
        let constraints = [
            (0u16, 1u16, r_oh), // O-H1
            (0u16, 2u16, r_oh), // O-H2
        ];
        let masses = [15.999, 1.008, 1.008];

        let iters = shake_solve(&mut pos, &old_pos, &constraints, &masses, 1e-8, 100);

        assert!(iters < 10, "water SHAKE should converge in <10 iters, got {iters}");

        let d_oh1 = dist(pos[0], pos[1]);
        let d_oh2 = dist(pos[0], pos[2]);
        assert!((d_oh1 - r_oh).abs() < 1e-6, "O-H1: {d_oh1} != {r_oh}");
        assert!((d_oh2 - r_oh).abs() < 1e-6, "O-H2: {d_oh2} != {r_oh}");
    }

    #[test]
    fn shake_preserves_center_of_mass() {
        let d_target = 2.0;
        let m_i = 2.0;
        let m_j = 3.0;
        let old_i = [0.0, 0.0, 0.0];
        let old_j = [d_target, 0.0, 0.0];
        let new_i = [0.1, 0.05, 0.0];
        let new_j = [2.2, -0.03, 0.0];

        let com_before = [
            (m_i * new_i[0] + m_j * new_j[0]) / (m_i + m_j),
            (m_i * new_i[1] + m_j * new_j[1]) / (m_i + m_j),
            (m_i * new_i[2] + m_j * new_j[2]) / (m_i + m_j),
        ];

        let mut pos = [new_i, new_j];
        let old_pos = [old_i, old_j];
        let constraints = [(0u16, 1u16, d_target)];
        let masses = [m_i, m_j];

        shake_solve(&mut pos, &old_pos, &constraints, &masses, 1e-10, 100);

        let com_after = [
            (m_i * pos[0][0] + m_j * pos[1][0]) / (m_i + m_j),
            (m_i * pos[0][1] + m_j * pos[1][1]) / (m_i + m_j),
            (m_i * pos[0][2] + m_j * pos[1][2]) / (m_i + m_j),
        ];

        for k in 0..3 {
            assert!(
                (com_after[k] - com_before[k]).abs() < 1e-10,
                "COM shifted in dim {k}: {} -> {}",
                com_before[k], com_after[k],
            );
        }
    }

    #[test]
    fn rattle_zero_constraint_velocity() {
        // After RATTLE, v_ij · r_ij = 0
        let r_i = [0.0, 0.0, 0.0];
        let r_j = [1.0, 0.0, 0.0]; // bond along x
        let v_i = [0.5, 0.3, -0.1]; // has x-component (violates constraint)
        let v_j = [-0.2, 0.1, 0.2];
        let d_target = 1.0;

        let (dv_i, dv_j) = rattle_pair(r_i, r_j, v_i, v_j, d_target, 1.0, 1.0);

        let vi_new = [v_i[0] + dv_i[0], v_i[1] + dv_i[1], v_i[2] + dv_i[2]];
        let vj_new = [v_j[0] + dv_j[0], v_j[1] + dv_j[1], v_j[2] + dv_j[2]];

        // v_rel · r should be ~0
        let d = [r_i[0] - r_j[0], r_i[1] - r_j[1], r_i[2] - r_j[2]];
        let v_rel = [vi_new[0] - vj_new[0], vi_new[1] - vj_new[1], vi_new[2] - vj_new[2]];
        let dot: f64 = v_rel[0] * d[0] + v_rel[1] * d[1] + v_rel[2] * d[2];

        assert!(dot.abs() < 1e-12, "v_rel · r = {dot}, should be ~0");
    }

    #[test]
    fn rattle_preserves_perpendicular_velocity() {
        // Velocity purely perpendicular to bond should be unchanged
        let r_i = [0.0, 0.0, 0.0];
        let r_j = [1.0, 0.0, 0.0]; // bond along x
        let v_i = [0.0, 0.5, 0.3]; // no x-component
        let v_j = [0.0, -0.2, 0.1];

        let (dv_i, dv_j) = rattle_pair(r_i, r_j, v_i, v_j, 1.0, 1.0, 1.0);

        for k in 0..3 {
            assert!(dv_i[k].abs() < 1e-12, "delta_vi[{k}] = {}", dv_i[k]);
            assert!(dv_j[k].abs() < 1e-12, "delta_vj[{k}] = {}", dv_j[k]);
        }
    }

    #[test]
    fn water_geometry_rigid_over_steps() {
        // Simulate 1000 Verlet steps with SHAKE — water geometry must stay rigid
        let r_oh = crate::batch::ff::water::TIP3P_R_OH;
        let half_angle = crate::batch::ff::water::TIP3P_ANGLE_HOH / 2.0;
        let h_dx = r_oh * half_angle.sin();
        let h_dy = r_oh * half_angle.cos();

        let mut pos = [
            [0.0, 0.0, 0.0],
            [h_dx, h_dy, 0.0],
            [-h_dx, h_dy, 0.0],
        ];
        let mut vel = [
            [0.01, -0.02, 0.005],
            [-0.03, 0.01, -0.01],
            [0.02, 0.01, 0.005],
        ];
        let masses = [15.999, 1.008, 1.008];
        let constraints = [
            (0u16, 1u16, r_oh),
            (0u16, 2u16, r_oh),
        ];
        let dt = 0.001;

        for _ in 0..1_000 {
            // Save old positions
            let old_pos = pos;

            // Verlet position step (no forces, just kinematic)
            for i in 0..3 {
                for k in 0..3 {
                    pos[i][k] += vel[i][k] * dt;
                }
            }

            // SHAKE
            shake_solve(&mut pos, &old_pos, &constraints, &masses, 1e-10, 100);

            // Update velocities from constrained position change
            for i in 0..3 {
                for k in 0..3 {
                    vel[i][k] = (pos[i][k] - old_pos[i][k]) / dt;
                }
            }

            // RATTLE
            rattle_solve(&pos, &mut vel, &constraints, &masses);
        }

        let d_oh1 = dist(pos[0], pos[1]);
        let d_oh2 = dist(pos[0], pos[2]);
        assert!(
            (d_oh1 - r_oh).abs() < 1e-6,
            "O-H1 drift after 1K steps: {d_oh1} vs {r_oh}",
        );
        assert!(
            (d_oh2 - r_oh).abs() < 1e-6,
            "O-H2 drift after 1K steps: {d_oh2} vs {r_oh}",
        );
    }

    #[test]
    fn constraints_from_topology_filters_by_k() {
        use crate::batch::ff::water::create_water_topology;
        let topo = create_water_topology(2);
        // TIP3P bonds have k=10000, well above threshold
        let constraints = constraints_from_topology(&topo, 5000.0);
        assert_eq!(constraints.len(), 4, "2 waters * 2 O-H bonds = 4 constraints");
        // Low threshold should also capture them
        let constraints_low = constraints_from_topology(&topo, 1.0);
        assert_eq!(constraints_low.len(), 4);
        // Very high threshold should capture none
        let constraints_high = constraints_from_topology(&topo, 50_000.0);
        assert_eq!(constraints_high.len(), 0);
    }
}
