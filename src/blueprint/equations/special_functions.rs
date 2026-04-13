//! Shared math special functions — single source of truth.
//!
//! Avoids duplication of approximation polynomials across modules.
//! Reference: Abramowitz & Stegun, Handbook of Mathematical Functions, 1964.

// ─── erfc approximation (A&S 7.1.26) ─────────────────────────────────────

/// Polynomial coefficients for erfc approximation.
/// Abramowitz & Stegun, formula 7.1.26. Max error: 1.5 × 10⁻⁷.
const ERFC_P: f64 = 0.327_591_1;
const ERFC_A1: f64 = 0.254_829_592;
const ERFC_A2: f64 = -0.284_496_736;
const ERFC_A3: f64 = 1.421_413_741;
const ERFC_A4: f64 = -1.453_152_027;
const ERFC_A5: f64 = 1.061_405_429;

/// Complementary error function erfc(x) = 1 - erf(x).
///
/// Approximation valid for all x ≥ 0. For x < 0: erfc(-x) = 2 - erfc(x).
/// Max absolute error: 1.5 × 10⁻⁷ (sufficient for MD force computation).
#[inline]
pub fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + ERFC_P * x.abs());
    let poly = t * (ERFC_A1 + t * (ERFC_A2 + t * (ERFC_A3 + t * (ERFC_A4 + t * ERFC_A5))));
    let result = poly * (-x * x).exp();
    if x >= 0.0 { result } else { 2.0 - result }
}

// ─── Unit conversion constants ────────────────────────────────────────────

/// Degrees to radians conversion factor.
pub const DEG_TO_RAD: f64 = core::f64::consts::PI / 180.0;

/// Radians to degrees conversion factor.
pub const RAD_TO_DEG: f64 = 180.0 / core::f64::consts::PI;

/// kcal/mol to kJ/mol.
pub const KCAL_TO_KJ: f64 = 4.184;

/// Angstrom to nanometer.
pub const ANGSTROM_TO_NM: f64 = 0.1;

// ─── Named thresholds for MD algorithms ───────────────────────────────────

/// Step size for numerical gradient (central differences) in bonded forces.
pub const NUMERICAL_GRAD_STEP: f32 = 1e-4;

/// Barnes-Hut opening angle: cells with size/distance < theta are approximated.
/// 0.5 is standard. Lower = more accurate but slower.
pub const BARNES_HUT_THETA: f64 = 0.5;

/// Particle count below which brute-force O(N²) is faster than tree O(N log N).
pub const BRUTE_FORCE_THRESHOLD: usize = 64;

/// Softening radius for LJ: clamp r_sq to (0.5σ)² to prevent singularity.
pub const LJ_SOFTENING_RADIUS_SQ: f64 = 0.25;

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erfc_zero_is_one() {
        assert!((erfc_approx(0.0) - 1.0).abs() < 1e-7);
    }

    #[test]
    fn erfc_large_is_zero() {
        assert!(erfc_approx(5.0) < 1e-10);
    }

    #[test]
    fn erfc_one_matches_reference() {
        // erfc(1) ≈ 0.157299207
        assert!((erfc_approx(1.0) - 0.157_299_207).abs() < 1e-6);
    }

    #[test]
    fn erfc_negative_symmetric() {
        // erfc(-x) = 2 - erfc(x)
        let x = 0.7;
        let sum = erfc_approx(x) + erfc_approx(-x);
        assert!((sum - 2.0).abs() < 1e-7);
    }

    #[test]
    fn deg_to_rad_consistency() {
        assert!((90.0 * DEG_TO_RAD - core::f64::consts::FRAC_PI_2).abs() < 1e-15);
        assert!((180.0 * DEG_TO_RAD - core::f64::consts::PI).abs() < 1e-15);
    }
}
