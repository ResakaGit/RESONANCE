//! R5e: Generalized Born / Surface Area (GB/SA) implicit solvent — pure math.
//!
//! Replaces explicit water with a continuum dielectric model.
//! 10-100x faster than explicit TIP3P for folding scans.
//!
//! Theory:
//!   E_solv = E_GB + E_SA
//!   E_GB = -0.5 * (1/ε_in - 1/ε_out) * Σ_{i,j} q_i q_j / f_GB(r_ij, R_i, R_j)
//!   E_SA = γ * SASA (solvent-accessible surface area)
//!
//! Born radii from OBC model (Onufriev-Bashford-Case, Proteins 2004).
//!
//! Axiom 1: solvent energy captured as continuum qe contribution.
//! Axiom 4: solvent friction via Langevin γ (already in thermostat).
//! Axiom 7: Born screening decays with distance.
//! Axiom 8: optional frequency-dependent dielectric (novel extension).

// ─── Physical constants ───────────────────────────────────────────────────

/// Dielectric constant of water at 300K.
pub const EPSILON_WATER: f64 = 78.5;

/// Dielectric constant inside protein.
pub const EPSILON_PROTEIN: f64 = 1.0;

/// Surface tension coefficient for SA term (kcal/mol/Å²).
pub const GAMMA_SA: f64 = 0.005;

/// Probe radius for SASA calculation (Å). Standard water probe.
pub const PROBE_RADIUS: f64 = 1.4;

// ─── OBC Born radii ──────────────────────────────────────────────────────

/// Van der Waals radii for common atom types (Å).
/// Indexed by element: 0=C, 1=N, 2=O, 3=S, 4=H.
pub const VDW_RADII: [f64; 5] = [1.70, 1.55, 1.52, 1.80, 1.20];

/// OBC scaling factors (Onufriev-Bashford-Case model II).
const OBC_ALPHA: f64 = 1.0;
const OBC_BETA: f64 = 0.8;
const OBC_GAMMA: f64 = 4.85;

/// Compute effective Born radii using OBC model.
///
/// `positions`: atom coordinates.
/// `vdw_radii`: per-atom van der Waals radii (Å).
///
/// Born radius R_i = 1 / (1/ρ_i - tanh(ψ)/ρ_i)
/// where ψ = α*I - β*I² + γ*I³, I = volume integral of neighbor overlap.
pub fn compute_born_radii(
    positions: &[[f64; 3]],
    vdw_radii: &[f64],
) -> Vec<f64> {
    let n = positions.len();
    let offset = 0.09; // dielectric offset (Å)
    let mut radii = Vec::with_capacity(n);

    for i in 0..n {
        let rho_i = vdw_radii[i] - offset;
        if rho_i <= 0.0 {
            radii.push(vdw_radii[i]);
            continue;
        }

        // Compute pairwise volume integral
        let mut sum_integral = 0.0;
        for j in 0..n {
            if i == j { continue; }
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = positions[i][k] - positions[j][k];
                d_sq += dk * dk;
            }
            let r = d_sq.sqrt();
            let rho_j = vdw_radii[j] - offset;
            if rho_j <= 0.0 || r < 1e-10 { continue; }

            // Approximate overlap integral (Still et al. 1990)
            let l_ij = (rho_i.max(r - rho_j)).recip();
            let u_ij = (r + rho_j).recip();
            if l_ij > u_ij { continue; }

            let integral = 0.5 * (1.0 / l_ij - 1.0 / u_ij
                + 0.25 * r * (1.0 / (u_ij * u_ij) - 1.0 / (l_ij * l_ij))
                + 0.5 * (rho_j * rho_j / r).ln().abs() * (1.0 / (l_ij * l_ij) - 1.0 / (u_ij * u_ij)));

            sum_integral += integral.max(0.0);
        }

        let psi = OBC_ALPHA * sum_integral - OBC_BETA * sum_integral * sum_integral
            + OBC_GAMMA * sum_integral * sum_integral * sum_integral;
        let tanh_psi = psi.tanh();
        let born_r = 1.0 / (1.0 / rho_i - tanh_psi / rho_i);
        radii.push(born_r.max(vdw_radii[i] * 0.5)); // clamp to reasonable minimum
    }

    radii
}

// ─── Generalized Born energy ──────────────────────────────────────────────

/// GB pair interaction function: f_GB = sqrt(r² + R_i*R_j*exp(-r²/(4*R_i*R_j)))
///
/// Smooth interpolation between Coulomb (r >> R) and Born (r → 0).
#[inline]
pub fn f_gb(r: f64, r_i: f64, r_j: f64) -> f64 {
    let rr = r_i * r_j;
    (r * r + rr * (-r * r / (4.0 * rr)).exp()).sqrt()
}

/// GB solvation energy: E_GB = -0.5 * (1/ε_in - 1/ε_out) * Σ_{i<=j} q_i*q_j / f_GB.
///
/// `k_e`: Coulomb constant.
/// `charges`: per-atom charges.
/// `born_radii`: per-atom effective Born radii.
pub fn gb_energy(
    k_e: f64,
    positions: &[[f64; 3]],
    charges: &[f64],
    born_radii: &[f64],
    epsilon_in: f64,
    epsilon_out: f64,
) -> f64 {
    let n = positions.len();
    let factor = -0.5 * k_e * (1.0 / epsilon_in - 1.0 / epsilon_out);
    let mut energy = 0.0;

    for i in 0..n {
        // Self term (i == j)
        energy += factor * charges[i] * charges[i] / born_radii[i];

        for j in (i + 1)..n {
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = positions[i][k] - positions[j][k];
                d_sq += dk * dk;
            }
            let r = d_sq.sqrt();
            let f = f_gb(r, born_radii[i], born_radii[j]);
            energy += 2.0 * factor * charges[i] * charges[j] / f;
        }
    }

    energy
}

/// GB forces: F_i = -∂E_GB/∂r_i (pair contribution only, ignoring dR/dr for simplicity).
pub fn gb_forces(
    k_e: f64,
    positions: &[[f64; 3]],
    charges: &[f64],
    born_radii: &[f64],
    epsilon_in: f64,
    epsilon_out: f64,
) -> Vec<[f64; 3]> {
    let n = positions.len();
    let factor = -0.5 * k_e * (1.0 / epsilon_in - 1.0 / epsilon_out);
    let mut forces = vec![[0.0; 3]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let mut d = [0.0; 3];
            let mut d_sq = 0.0;
            for k in 0..3 {
                d[k] = positions[i][k] - positions[j][k];
                d_sq += d[k] * d[k];
            }
            let r = d_sq.sqrt();
            if r < 1e-15 { continue; }

            let rr = born_radii[i] * born_radii[j];
            let exp_term = (-d_sq / (4.0 * rr)).exp();
            let f_val = (d_sq + rr * exp_term).sqrt();
            let f_val_3 = f_val * f_val * f_val;

            // df_GB/dr = r * (1 + exp_term/(4*R_i*R_j) * r²/(4*R_i*R_j)... ≈ r / f_GB for large r
            // Simplified: F ≈ factor * q_i * q_j * r / f_GB³
            let f_mag = 2.0 * factor * charges[i] * charges[j] / f_val_3;

            for k in 0..3 {
                forces[i][k] += f_mag * d[k];
                forces[j][k] -= f_mag * d[k];
            }
        }
    }

    forces
}

// ─── Surface Area (SASA) ──────────────────────────────────────────────────

/// Approximate SASA per atom using LCPO method (linear combination of pairwise overlaps).
///
/// Simplified: SASA_i ≈ 4π(R_i + probe)² * (1 - Σ_j overlap_ij)
pub fn sasa_energy(
    positions: &[[f64; 3]],
    vdw_radii: &[f64],
    probe: f64,
    gamma: f64,
) -> f64 {
    let n = positions.len();
    let four_pi = 4.0 * core::f64::consts::PI;
    let mut total_sasa = 0.0;

    for i in 0..n {
        let r_i = vdw_radii[i] + probe;
        let sphere_area = four_pi * r_i * r_i;
        let mut burial = 0.0;

        for j in 0..n {
            if i == j { continue; }
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = positions[i][k] - positions[j][k];
                d_sq += dk * dk;
            }
            let r = d_sq.sqrt();
            let r_j = vdw_radii[j] + probe;

            if r < r_i + r_j {
                // Fractional burial
                let overlap = (r_i + r_j - r) / (2.0 * r_i);
                burial += overlap.clamp(0.0, 1.0);
            }
        }

        let exposed = (1.0 - burial).max(0.0);
        total_sasa += sphere_area * exposed;
    }

    gamma * total_sasa
}

// ─── Axiom 8: Frequency-dependent dielectric (novel) ─────────────────────

/// Frequency-modulated Born screening (Axiom 8 extension).
///
/// ε_eff = ε_water * (1 - alignment(f_water, f_avg) * screening_factor)
///
/// Residues with frequencies coherent with water's characteristic frequency
/// are better solvated (lower ε_eff → stronger screening).
pub fn frequency_dependent_epsilon(
    f_residue: f64,
    f_water: f64,
    bandwidth: f64,
    epsilon_base: f64,
) -> f64 {
    let df = (f_residue - f_water) / bandwidth;
    let alignment = (-0.5 * df * df).exp();
    // More coherent with water → higher effective dielectric → better solvated
    epsilon_base * (0.5 + 0.5 * alignment)
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f_gb_reduces_to_r_at_large_distance() {
        let r = 100.0;
        let f = f_gb(r, 1.5, 1.5);
        assert!((f - r).abs() < 0.1, "f_GB should ≈ r for large r: {f}");
    }

    #[test]
    fn f_gb_at_zero_is_sqrt_rr() {
        let r_i = 1.5;
        let r_j = 2.0;
        let f = f_gb(0.0, r_i, r_j);
        let expected = (r_i * r_j).sqrt();
        assert!((f - expected).abs() < 1e-10, "f_GB(0) = sqrt(R_i*R_j): {f} vs {expected}");
    }

    #[test]
    fn born_radii_positive() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [6.0, 0.0, 0.0]];
        let vdw = vec![1.7, 1.55, 1.52];
        let radii = compute_born_radii(&pos, &vdw);
        for (i, &r) in radii.iter().enumerate() {
            assert!(r > 0.0, "Born radius {i} should be positive: {r}");
        }
    }

    #[test]
    fn gb_energy_negative_for_unlike_charges() {
        let pos = vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let born_radii = vec![1.5, 1.5];
        let e = gb_energy(1.0, &pos, &charges, &born_radii, EPSILON_PROTEIN, EPSILON_WATER);
        assert!(e < 0.0, "GB should stabilize unlike charges: {e}");
    }

    #[test]
    fn gb_forces_newton_third() {
        let pos = vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let born_radii = vec![1.5, 1.5];
        let forces = gb_forces(1.0, &pos, &charges, &born_radii, EPSILON_PROTEIN, EPSILON_WATER);
        for d in 0..3 {
            let sum = forces[0][d] + forces[1][d];
            assert!(sum.abs() < 1e-10, "Newton 3: dim {d}, sum={sum}");
        }
    }

    #[test]
    fn sasa_single_atom_is_sphere() {
        let pos = vec![[0.0, 0.0, 0.0]];
        let vdw = vec![1.7];
        let e = sasa_energy(&pos, &vdw, PROBE_RADIUS, GAMMA_SA);
        let expected = GAMMA_SA * 4.0 * core::f64::consts::PI * (1.7 + PROBE_RADIUS).powi(2);
        assert!((e - expected).abs() < 1e-6, "single atom SASA = full sphere: {e} vs {expected}");
    }

    #[test]
    fn sasa_buried_atom_reduced() {
        // Two atoms overlapping → each has reduced SASA
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let vdw = vec![1.7, 1.7];
        let e_pair = sasa_energy(&pos, &vdw, PROBE_RADIUS, GAMMA_SA);
        let e_single = sasa_energy(&pos[..1], &vdw[..1], PROBE_RADIUS, GAMMA_SA);
        assert!(e_pair < 2.0 * e_single, "overlapping atoms should have less SASA");
    }

    #[test]
    fn frequency_epsilon_coherent_higher() {
        // Residue coherent with water → higher effective ε → better solvated
        let e_coherent = frequency_dependent_epsilon(100.0, 100.0, 50.0, EPSILON_WATER);
        let e_incoherent = frequency_dependent_epsilon(100.0, 300.0, 50.0, EPSILON_WATER);
        assert!(e_coherent > e_incoherent, "coherent should have higher ε: {e_coherent} vs {e_incoherent}");
    }
}
