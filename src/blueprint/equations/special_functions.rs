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

// ─── FFT (Cooley-Tukey radix-2, in-place) ─────────────────────────────────

/// In-place radix-2 FFT. Input length must be a power of 2.
///
/// `data`: interleaved [re0, im0, re1, im1, ...] of length 2*N.
/// `inverse`: true for inverse FFT (scales by 1/N).
///
/// Cooley-Tukey decimation-in-time. O(N log N). No external crate.
pub fn fft_radix2(data: &mut [f64], inverse: bool) {
    let n = data.len() / 2;
    if n <= 1 { return; }
    debug_assert!(n.is_power_of_two(), "FFT length must be power of 2, got {n}");

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 0..n {
        if i < j {
            data.swap(2 * i, 2 * j);
            data.swap(2 * i + 1, 2 * j + 1);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    // Butterfly passes
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = sign * core::f64::consts::PI / half as f64;
        let wn_re = angle.cos();
        let wn_im = angle.sin();

        let mut start = 0;
        while start < n {
            let mut w_re = 1.0;
            let mut w_im = 0.0;
            for k in 0..half {
                let i = start + k;
                let j = start + k + half;
                let tr = w_re * data[2 * j] - w_im * data[2 * j + 1];
                let ti = w_re * data[2 * j + 1] + w_im * data[2 * j];
                data[2 * j] = data[2 * i] - tr;
                data[2 * j + 1] = data[2 * i + 1] - ti;
                data[2 * i] += tr;
                data[2 * i + 1] += ti;
                let new_w_re = w_re * wn_re - w_im * wn_im;
                let new_w_im = w_re * wn_im + w_im * wn_re;
                w_re = new_w_re;
                w_im = new_w_im;
            }
            start += len;
        }
        len <<= 1;
    }

    if inverse {
        let inv_n = 1.0 / n as f64;
        for v in data.iter_mut() {
            *v *= inv_n;
        }
    }
}

/// 3D FFT on a grid of dimensions [nx, ny, nz].
///
/// `grid`: interleaved complex data [re, im] of size 2*nx*ny*nz.
/// All dimensions must be powers of 2.
pub fn fft_3d(grid: &mut [f64], nx: usize, ny: usize, nz: usize, inverse: bool) {
    // FFT along z (fastest varying)
    let mut buf = vec![0.0; 2 * nz];
    for ix in 0..nx {
        for iy in 0..ny {
            for iz in 0..nz {
                let idx = ((ix * ny + iy) * nz + iz) * 2;
                buf[2 * iz] = grid[idx];
                buf[2 * iz + 1] = grid[idx + 1];
            }
            fft_radix2(&mut buf, inverse);
            for iz in 0..nz {
                let idx = ((ix * ny + iy) * nz + iz) * 2;
                grid[idx] = buf[2 * iz];
                grid[idx + 1] = buf[2 * iz + 1];
            }
        }
    }

    // FFT along y
    let mut buf = vec![0.0; 2 * ny];
    for ix in 0..nx {
        for iz in 0..nz {
            for iy in 0..ny {
                let idx = ((ix * ny + iy) * nz + iz) * 2;
                buf[2 * iy] = grid[idx];
                buf[2 * iy + 1] = grid[idx + 1];
            }
            fft_radix2(&mut buf, inverse);
            for iy in 0..ny {
                let idx = ((ix * ny + iy) * nz + iz) * 2;
                grid[idx] = buf[2 * iy];
                grid[idx + 1] = buf[2 * iy + 1];
            }
        }
    }

    // FFT along x
    let mut buf = vec![0.0; 2 * nx];
    for iy in 0..ny {
        for iz in 0..nz {
            for ix in 0..nx {
                let idx = ((ix * ny + iy) * nz + iz) * 2;
                buf[2 * ix] = grid[idx];
                buf[2 * ix + 1] = grid[idx + 1];
            }
            fft_radix2(&mut buf, inverse);
            for ix in 0..nx {
                let idx = ((ix * ny + iy) * nz + iz) * 2;
                grid[idx] = buf[2 * ix];
                grid[idx + 1] = buf[2 * ix + 1];
            }
        }
    }
}

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

    #[test]
    fn fft_roundtrip_4() {
        // FFT then IFFT should return original data
        let original = [1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0]; // 4 real values
        let mut data = original;
        fft_radix2(&mut data, false);
        fft_radix2(&mut data, true);
        for i in 0..8 {
            assert!((data[i] - original[i]).abs() < 1e-12, "mismatch at {i}");
        }
    }

    #[test]
    fn fft_dc_component() {
        // DC component = sum of all values
        let mut data = [1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0];
        fft_radix2(&mut data, false);
        assert!((data[0] - 10.0).abs() < 1e-12, "DC = sum = 10, got {}", data[0]);
    }

    #[test]
    fn fft_parseval_theorem() {
        // Sum of |x|² in time = (1/N) * sum of |X|² in frequency
        let mut data = [1.0, 0.0, -0.5, 0.3, 2.0, -1.0, 0.0, 0.7];
        let n = 4;
        let time_energy: f64 = (0..n).map(|i| data[2*i]*data[2*i] + data[2*i+1]*data[2*i+1]).sum();
        fft_radix2(&mut data, false);
        let freq_energy: f64 = (0..n).map(|i| data[2*i]*data[2*i] + data[2*i+1]*data[2*i+1]).sum();
        assert!((freq_energy / n as f64 - time_energy).abs() < 1e-10, "Parseval violated");
    }
}
