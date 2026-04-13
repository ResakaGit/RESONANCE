//! MD-12: Ewald summation — long-range electrostatics for periodic systems.
//!
//! Splits Coulomb 1/r into three fast-converging parts:
//!   E_coulomb = E_real + E_recip - E_self
//!
//! Real space: short-range, erfc-screened, O(N) with cell list.
//! Reciprocal space: Fourier series, O(N * k_max^3) bare Ewald.
//! Self correction: removes spurious self-interaction.
//!
//! Axiom 7: distance attenuation — real-space screening enforces finite range.
//! Axiom 8: oscillatory nature — reciprocal-space k-vectors are periodic modes.
//!
//! Units: caller-defined. Typically kcal/mol·Å or reduced. The Coulomb constant
//! `k_e` must be passed by the caller (not hardcoded).

use core::f64::consts::PI;

// ─── Complementary error function approximation ───────────────────────────

/// erfc(x) approximation (Abramowitz & Stegun, 7.1.26). Max error ~1.5e-7.
///
/// Good enough for MD force computation. No external crate needed.
fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let poly = t * (0.254_829_592
        + t * (-0.284_496_736
            + t * (1.421_413_741
                + t * (-1.453_152_027
                    + t * 1.061_405_429))));
    let result = poly * (-x * x).exp();
    if x >= 0.0 { result } else { 2.0 - result }
}

// ─── Real space ───────────────────────────────────────────────────────────

/// Real-space Ewald pair energy: k_e * q_i * q_j * erfc(alpha * r) / r.
///
/// `k_e`: Coulomb constant in caller's unit system.
/// `alpha`: Ewald splitting parameter (typically 5/L_box).
/// Returns 0 if r <= 0.
#[inline]
pub fn ewald_real_energy_pair(k_e: f64, q_i: f64, q_j: f64, r: f64, alpha: f64) -> f64 {
    if r <= 1e-15 {
        return 0.0;
    }
    k_e * q_i * q_j * erfc_approx(alpha * r) / r
}

/// Real-space Ewald pair force vector (on particle i, from j).
///
/// F_real = k_e * q_i * q_j * [erfc(α*r)/r² + 2α/(√π) * exp(-α²r²)/r] * (d/r)
///
/// `d`: displacement vector j→i (d = pos_i - pos_j, after minimum image).
/// Returns force ON particle i.
#[inline]
pub fn ewald_real_force_3d(
    k_e: f64, q_i: f64, q_j: f64, d: [f64; 3], alpha: f64,
) -> [f64; 3] {
    let r_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if r_sq < 1e-30 {
        return [0.0; 3];
    }
    let r = r_sq.sqrt();
    let alpha_r = alpha * r;

    // Scalar force magnitude / r: F_scalar = erfc(αr)/r² + 2α/√π * exp(-α²r²)/r
    let erfc_term = erfc_approx(alpha_r) / (r * r);
    let gauss_term = 2.0 * alpha / PI.sqrt() * (-alpha_r * alpha_r).exp() / r;
    let f_over_r = k_e * q_i * q_j * (erfc_term + gauss_term) / r;

    [f_over_r * d[0], f_over_r * d[1], f_over_r * d[2]]
}

// ─── Self correction ──────────────────────────────────────────────────────

/// Self-energy correction: E_self = -k_e * α/√π * Σ q_i².
///
/// Must be subtracted from (E_real + E_recip) to get true Coulomb energy.
/// Always positive (subtracted), so this function returns a positive value.
pub fn ewald_self_correction(k_e: f64, charges: &[f64], alpha: f64) -> f64 {
    let sum_q2: f64 = charges.iter().map(|q| q * q).sum();
    k_e * alpha / PI.sqrt() * sum_q2
}

// ─── Reciprocal space ─────────────────────────────────────────────────────

/// Reciprocal-space Ewald energy for a 3D periodic box.
///
/// E_recip = (1/2V) Σ_{k≠0} (4π/k²) * exp(-k²/(4α²)) * |S(k)|²
///
/// where S(k) = Σ_i q_i * exp(i k·r_i) is the structure factor.
///
/// `box_lengths`: [Lx, Ly, Lz].
/// `k_max`: maximum integer k-vector magnitude per dimension.
///
/// Bare Ewald: O(N * (2*k_max+1)³). Acceptable for N < 5000.
pub fn ewald_reciprocal_energy(
    k_e: f64,
    positions: &[[f64; 3]],
    charges: &[f64],
    box_lengths: [f64; 3],
    alpha: f64,
    k_max: u32,
) -> f64 {
    let volume = box_lengths[0] * box_lengths[1] * box_lengths[2];
    let four_alpha_sq = 4.0 * alpha * alpha;
    let prefactor = k_e * 2.0 * PI / volume;
    let mut energy = 0.0;

    let km = k_max as i32;
    for nx in -km..=km {
        let kx = 2.0 * PI * nx as f64 / box_lengths[0];
        for ny in -km..=km {
            let ky = 2.0 * PI * ny as f64 / box_lengths[1];
            for nz in -km..=km {
                if nx == 0 && ny == 0 && nz == 0 {
                    continue;
                }
                let kz = 2.0 * PI * nz as f64 / box_lengths[2];
                let k_sq = kx * kx + ky * ky + kz * kz;

                let gauss = (-k_sq / four_alpha_sq).exp() / k_sq;

                // Structure factor S(k) = Σ q_i * exp(i k·r_i)
                let (mut s_re, mut s_im) = (0.0, 0.0);
                for (pos, &q) in positions.iter().zip(charges.iter()) {
                    let kr = kx * pos[0] + ky * pos[1] + kz * pos[2];
                    s_re += q * kr.cos();
                    s_im += q * kr.sin();
                }
                let s_sq = s_re * s_re + s_im * s_im;

                energy += gauss * s_sq;
            }
        }
    }

    prefactor * energy
}

/// Reciprocal-space Ewald forces for a 3D periodic box.
///
/// F_i = -∂E_recip/∂r_i = -(1/V) Σ_{k≠0} (4π k/k²) * exp(-k²/(4α²)) * q_i * Im[S(k)*exp(-ik·r_i)]
///
/// Returns force vector per particle.
pub fn ewald_reciprocal_forces(
    k_e: f64,
    positions: &[[f64; 3]],
    charges: &[f64],
    box_lengths: [f64; 3],
    alpha: f64,
    k_max: u32,
) -> Vec<[f64; 3]> {
    let n = positions.len();
    let volume = box_lengths[0] * box_lengths[1] * box_lengths[2];
    let four_alpha_sq = 4.0 * alpha * alpha;
    let prefactor = k_e * 4.0 * PI / volume;
    let mut forces = vec![[0.0; 3]; n];

    let km = k_max as i32;
    for nx in -km..=km {
        let kx = 2.0 * PI * nx as f64 / box_lengths[0];
        for ny in -km..=km {
            let ky = 2.0 * PI * ny as f64 / box_lengths[1];
            for nz in -km..=km {
                if nx == 0 && ny == 0 && nz == 0 {
                    continue;
                }
                let kz = 2.0 * PI * nz as f64 / box_lengths[2];
                let k_sq = kx * kx + ky * ky + kz * kz;
                let kv = [kx, ky, kz];

                let gauss = (-k_sq / four_alpha_sq).exp() / k_sq;

                // Structure factor
                let (mut s_re, mut s_im) = (0.0, 0.0);
                for (pos, &q) in positions.iter().zip(charges.iter()) {
                    let kr = kx * pos[0] + ky * pos[1] + kz * pos[2];
                    s_re += q * kr.cos();
                    s_im += q * kr.sin();
                }

                // Force on each particle
                for (i, (pos, &q)) in positions.iter().zip(charges.iter()).enumerate() {
                    let kr = kv[0] * pos[0] + kv[1] * pos[1] + kv[2] * pos[2];
                    // -dE/dr_i = prefactor * gauss * q * (S_re*sin(kr) - S_im*cos(kr)) * k
                    let coeff = prefactor * gauss * q * (s_re * kr.sin() - s_im * kr.cos());
                    for d in 0..3 {
                        forces[i][d] += coeff * kv[d];
                    }
                }
            }
        }
    }

    forces
}

// ─── Combined interface ───────────────────────────────────────────────────

/// Full Ewald energy: E_real + E_recip - E_self.
///
/// `r_cut`: real-space cutoff. Pairs beyond r_cut are handled by reciprocal space.
/// Caller must pass displacement vectors (minimum-image) for real-space pairs.
///
/// For convenience, this function computes all three terms given full position data.
/// In production, real-space should use cell lists (O(N) with cutoff).
pub fn ewald_total_energy(
    k_e: f64,
    positions: &[[f64; 3]],
    charges: &[f64],
    box_lengths: [f64; 3],
    alpha: f64,
    k_max: u32,
    r_cut: f64,
) -> f64 {
    let n = positions.len();

    // Real space (brute-force, O(N²) — production should use cell list)
    let mut e_real = 0.0;
    for i in 0..n {
        for j in (i + 1)..n {
            let mut d = [0.0; 3];
            for k in 0..3 {
                d[k] = positions[i][k] - positions[j][k];
                d[k] -= (d[k] / box_lengths[k]).round() * box_lengths[k];
            }
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if r < r_cut {
                e_real += ewald_real_energy_pair(k_e, charges[i], charges[j], r, alpha);
            }
        }
    }

    // Reciprocal space
    let e_recip = ewald_reciprocal_energy(k_e, positions, charges, box_lengths, alpha, k_max);

    // Self correction
    let e_self = ewald_self_correction(k_e, charges, alpha);

    e_real + e_recip - e_self
}

/// Full Ewald forces: F_real + F_recip.
///
/// Self-correction has no force contribution (position-independent).
pub fn ewald_total_forces(
    k_e: f64,
    positions: &[[f64; 3]],
    charges: &[f64],
    box_lengths: [f64; 3],
    alpha: f64,
    k_max: u32,
    r_cut: f64,
) -> Vec<[f64; 3]> {
    let n = positions.len();
    let mut forces = vec![[0.0; 3]; n];

    // Real space (brute-force)
    for i in 0..n {
        for j in (i + 1)..n {
            let mut d = [0.0; 3];
            for k in 0..3 {
                d[k] = positions[i][k] - positions[j][k];
                d[k] -= (d[k] / box_lengths[k]).round() * box_lengths[k];
            }
            let r_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if r_sq < r_cut * r_cut {
                let f = ewald_real_force_3d(k_e, charges[i], charges[j], d, alpha);
                for k in 0..3 {
                    forces[i][k] += f[k];
                    forces[j][k] -= f[k];
                }
            }
        }
    }

    // Reciprocal space
    let f_recip = ewald_reciprocal_forces(k_e, positions, charges, box_lengths, alpha, k_max);
    for i in 0..n {
        for k in 0..3 {
            forces[i][k] += f_recip[i][k];
        }
    }

    forces
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Madelung constant for NaCl (rocksalt) structure.
    /// Literature value: ~1.747565.
    const NACL_MADELUNG: f64 = 1.747_565;

    #[test]
    fn erfc_matches_known_values() {
        // erfc(0) = 1
        assert!((erfc_approx(0.0) - 1.0).abs() < 1e-6);
        // erfc(∞) → 0
        assert!(erfc_approx(5.0) < 1e-10);
        // erfc(1) ≈ 0.1573
        assert!((erfc_approx(1.0) - 0.157_299_207).abs() < 1e-5);
        // erf(0) = 1 - erfc(0) = 0
        assert!((1.0 - erfc_approx(0.0)).abs() < 1e-6);
    }

    #[test]
    fn real_energy_positive_for_like_charges() {
        let e = ewald_real_energy_pair(1.0, 1.0, 1.0, 2.0, 1.0);
        assert!(e > 0.0, "like charges repel: E = {e}");
    }

    #[test]
    fn real_energy_negative_for_unlike_charges() {
        let e = ewald_real_energy_pair(1.0, 1.0, -1.0, 2.0, 1.0);
        assert!(e < 0.0, "unlike charges attract: E = {e}");
    }

    #[test]
    fn real_energy_decays_with_alpha() {
        // Higher alpha → faster real-space decay
        let e1 = ewald_real_energy_pair(1.0, 1.0, 1.0, 2.0, 1.0);
        let e2 = ewald_real_energy_pair(1.0, 1.0, 1.0, 2.0, 3.0);
        assert!(
            e2.abs() < e1.abs(),
            "higher alpha should screen more: |{e2}| < |{e1}|",
        );
    }

    #[test]
    fn real_force_repels_like_charges() {
        // Two positive charges along x-axis, i at origin, j at x=2
        let d = [2.0, 0.0, 0.0]; // pos_i - pos_j = i is to the right of j
        let f = ewald_real_force_3d(1.0, 1.0, 1.0, d, 1.0);
        assert!(f[0] > 0.0, "like charges push apart: f_x = {}", f[0]);
    }

    #[test]
    fn real_force_attracts_unlike_charges() {
        let d = [2.0, 0.0, 0.0];
        let f = ewald_real_force_3d(1.0, 1.0, -1.0, d, 1.0);
        assert!(f[0] < 0.0, "unlike charges attract: f_x = {}", f[0]);
    }

    #[test]
    fn self_correction_positive() {
        let charges = vec![1.0, -1.0, 0.5];
        let e_self = ewald_self_correction(1.0, &charges, 1.0);
        assert!(e_self > 0.0, "self correction must be positive: {e_self}");
    }

    #[test]
    fn self_correction_scales_with_sum_q_squared() {
        let c1 = vec![1.0, 1.0];
        let c2 = vec![2.0, 2.0];
        let e1 = ewald_self_correction(1.0, &c1, 1.0);
        let e2 = ewald_self_correction(1.0, &c2, 1.0);
        // sum(q²) for c2 is 4x c1
        assert!((e2 / e1 - 4.0).abs() < 1e-10);
    }

    #[test]
    fn reciprocal_energy_zero_for_neutral_uniform() {
        // Single neutral pair at same position → structure factor collapses
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let bl = [10.0; 3];
        let e = ewald_reciprocal_energy(1.0, &pos, &charges, bl, 0.5, 2);
        // When both charges are at the same position, S(k)=0 for all k (charges cancel)
        assert!(e.abs() < 1e-10, "E_recip should be ~0 for co-located +/-: {e}");
    }

    #[test]
    fn total_energy_nacl_unit_cell() {
        // NaCl unit cell: 8 ions in a cube of side a.
        // Na+ at (0,0,0), (a/2,a/2,0), (a/2,0,a/2), (0,a/2,a/2)
        // Cl- at (a/2,0,0), (0,a/2,0), (0,0,a/2), (a/2,a/2,a/2)
        let a = 5.64; // NaCl lattice constant in Angstrom
        let positions = vec![
            [0.0, 0.0, 0.0],
            [a / 2.0, a / 2.0, 0.0],
            [a / 2.0, 0.0, a / 2.0],
            [0.0, a / 2.0, a / 2.0],
            [a / 2.0, 0.0, 0.0],
            [0.0, a / 2.0, 0.0],
            [0.0, 0.0, a / 2.0],
            [a / 2.0, a / 2.0, a / 2.0],
        ];
        let charges = vec![1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0];
        let bl = [a; 3];
        let alpha = 5.0 / a;
        let k_max = 5;
        let r_cut = a / 2.0 - 0.01; // just under half-box

        let e_total = ewald_total_energy(1.0, &positions, &charges, bl, alpha, k_max, r_cut);

        // Madelung energy per ion pair: E = -M * k_e * q² / r_nn
        // r_nn = a/2 (nearest neighbor distance)
        // For 4 pairs: E_total = -4 * M / (a/2) = -8M/a
        // With k_e = 1, q = 1: E per pair = -M/(a/2)
        let r_nn = a / 2.0;
        let e_madelung_per_pair = -NACL_MADELUNG / r_nn;
        let e_expected = 4.0 * e_madelung_per_pair; // 4 formula units

        // Accept within 5% — bare Ewald with k_max=5 has finite convergence error
        let relative_error = (e_total - e_expected).abs() / e_expected.abs();
        assert!(
            relative_error < 0.05,
            "Madelung: E_total={e_total:.4}, expected={e_expected:.4}, error={:.1}%",
            relative_error * 100.0,
        );
    }

    #[test]
    fn forces_newton_third_law() {
        // Two charges: total force must be zero
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let bl = [10.0; 3];
        let forces = ewald_total_forces(1.0, &pos, &charges, bl, 0.5, 3, 4.5);
        for d in 0..3 {
            let sum = forces[0][d] + forces[1][d];
            assert!(
                sum.abs() < 1e-8,
                "Newton 3 violated: dim {d}, sum = {sum}",
            );
        }
    }

    #[test]
    fn forces_direction_unlike_charges() {
        // +q at origin, -q at (3,0,0) → should attract
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let bl = [10.0; 3];
        let forces = ewald_total_forces(1.0, &pos, &charges, bl, 0.5, 3, 4.5);
        // Force on +q should point toward -q (positive x)
        assert!(forces[0][0] > 0.0, "attraction: f[0].x = {}", forces[0][0]);
        // Force on -q should point toward +q (negative x)
        assert!(forces[1][0] < 0.0, "attraction: f[1].x = {}", forces[1][0]);
    }

    #[test]
    fn ewald_reduces_to_coulomb_at_large_box() {
        // In a very large box with well-separated charges, Ewald ≈ direct Coulomb
        let r = 2.0;
        let pos = vec![[0.0, 0.0, 0.0], [r, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let bl = [100.0; 3]; // huge box → minimal periodic image effects
        let alpha = 5.0 / 100.0;
        let e_ewald = ewald_total_energy(1.0, &pos, &charges, bl, alpha, 3, 49.0);
        let e_coulomb = -1.0 / r; // k_e * q1 * q2 / r = 1 * 1 * (-1) / 2
        let error = (e_ewald - e_coulomb).abs() / e_coulomb.abs();
        assert!(
            error < 0.01,
            "Ewald should match Coulomb in large box: {e_ewald:.6} vs {e_coulomb:.6}, error={:.2}%",
            error * 100.0,
        );
    }

    #[test]
    fn neutral_system_energy_independent_of_alpha() {
        // For a neutral system, changing alpha should give same total energy
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]];
        let charges = vec![1.0, -1.0];
        let bl = [10.0; 3];
        let e1 = ewald_total_energy(1.0, &pos, &charges, bl, 0.3, 5, 4.5);
        let e2 = ewald_total_energy(1.0, &pos, &charges, bl, 0.8, 5, 4.5);
        // Should converge to same value (within reciprocal-space truncation error)
        let diff = (e1 - e2).abs();
        assert!(
            diff < 0.02,
            "neutral system: energy should be alpha-independent: {e1:.6} vs {e2:.6}, diff={diff:.6}",
        );
    }
}
