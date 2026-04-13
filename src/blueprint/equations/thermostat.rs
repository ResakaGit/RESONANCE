//! Langevin thermostat — pure math.
//!
//! MD-1: NVE → NVT ensemble. Temperature control via friction + stochastic noise.
//!
//! Axiom 4: friction IS dissipation. The random kick is heat bath coupling.
//! Fluctuation-dissipation theorem: friction and noise are balanced so that
//! <v²> = k_B T / m at equilibrium (Maxwell-Boltzmann).

/// Langevin friction force per component: F_friction = -gamma * mass * velocity.
///
/// Axiom 4: pure dissipation — removes kinetic energy proportionally to speed.
#[inline]
pub fn langevin_friction(gamma: f64, mass: f64, velocity: f64) -> f64 {
    -gamma * mass * velocity
}

/// Langevin noise force standard deviation.
///
/// sigma_F = sqrt(2 * gamma * mass * k_B * T / dt).
/// Satisfies fluctuation-dissipation theorem: noise balances friction.
#[inline]
pub fn langevin_noise_sigma(gamma: f64, mass: f64, kb_t: f64, dt: f64) -> f64 {
    (2.0 * gamma * mass * kb_t / dt).max(0.0).sqrt()
}

/// Langevin velocity noise standard deviation (direct velocity formulation).
///
/// sigma_v = sqrt(2 * gamma * k_B * T * dt / mass).
/// Used in: v_new = v * (1 - gamma*dt) + sigma_v * z.
#[inline]
pub fn langevin_velocity_sigma(gamma: f64, kb_t: f64, dt: f64, mass: f64) -> f64 {
    (2.0 * gamma * kb_t * dt / mass).max(0.0).sqrt()
}

/// Instantaneous kinetic temperature from masses and velocities.
///
/// T = (2 / (N_dof * k_B)) * Σ(½ m v²).
/// `n_dof` = number of degrees of freedom (2*N for 2D, 3*N for 3D).
pub fn kinetic_temperature(
    masses: &[f64],
    velocities: &[[f64; 2]],
    k_b: f64,
) -> f64 {
    let n = masses.len().min(velocities.len());
    if n == 0 || k_b <= 0.0 {
        return 0.0;
    }
    let ke: f64 = (0..n)
        .map(|i| {
            let vx = velocities[i][0];
            let vy = velocities[i][1];
            0.5 * masses[i] * (vx * vx + vy * vy)
        })
        .sum();
    let n_dof = (2 * n) as f64; // 2D: 2 DOF per particle
    2.0 * ke / (n_dof * k_b)
}

/// Maxwell-Boltzmann velocity for one component: v = sqrt(k_B T / m) * z.
///
/// `z`: standard normal sample (from deterministic RNG).
#[inline]
pub fn maxwell_boltzmann_velocity(mass: f64, kb_t: f64, z: f64) -> f64 {
    (kb_t / mass).max(0.0).sqrt() * z
}

/// Chi-squared statistic for velocity distribution vs Maxwell-Boltzmann.
///
/// Bins velocities and compares observed vs expected counts.
/// Lower = better fit. Used for validation, not runtime.
pub fn velocity_distribution_chi2(
    velocities: &[f64],
    mass: f64,
    kb_t: f64,
    n_bins: usize,
) -> f64 {
    let n = velocities.len();
    if n < 10 || n_bins < 2 || mass <= 0.0 || kb_t <= 0.0 {
        return f64::MAX;
    }
    let sigma = (kb_t / mass).sqrt();
    // Bin range: [-4σ, 4σ]
    let v_min = -4.0 * sigma;
    let v_max = 4.0 * sigma;
    let bin_width = (v_max - v_min) / n_bins as f64;

    let mut observed = vec![0usize; n_bins];
    for &v in velocities {
        let bin = ((v - v_min) / bin_width).floor() as isize;
        if bin >= 0 && (bin as usize) < n_bins {
            observed[bin as usize] += 1;
        }
    }

    // Expected: N * P(bin) where P(bin) = erf integral of Gaussian
    let inv_sqrt2_sigma = 1.0 / (core::f64::consts::SQRT_2 * sigma);
    let mut chi2 = 0.0;
    for i in 0..n_bins {
        let lo = v_min + i as f64 * bin_width;
        let hi = lo + bin_width;
        // P(lo < v < hi) for Gaussian(0, sigma)
        let p = 0.5 * (erf(hi * inv_sqrt2_sigma) - erf(lo * inv_sqrt2_sigma));
        let expected = n as f64 * p;
        if expected > 0.5 {
            let diff = observed[i] as f64 - expected;
            chi2 += diff * diff / expected;
        }
    }
    chi2
}

/// Error function approximation (Abramowitz & Stegun 7.1.26). Max error 1.5e-7.
fn erf(x: f64) -> f64 {
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    let poly = 0.254_829_592 * t - 0.284_496_736 * t2 + 1.421_413_741 * t3
        - 1.453_152_027 * t4
        + 1.061_405_429 * t5;
    sign * (1.0 - poly * (-x * x).exp())
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friction_zero_velocity_zero_force() {
        assert_eq!(langevin_friction(0.2, 1.0, 0.0), 0.0);
    }

    #[test]
    fn friction_proportional_to_velocity() {
        let f1 = langevin_friction(0.2, 1.0, 1.0);
        let f2 = langevin_friction(0.2, 1.0, 2.0);
        assert!((f2 / f1 - 2.0).abs() < 1e-10, "F ∝ v");
    }

    #[test]
    fn friction_opposes_motion() {
        assert!(langevin_friction(0.2, 1.0, 5.0) < 0.0, "positive v → negative F");
        assert!(langevin_friction(0.2, 1.0, -5.0) > 0.0, "negative v → positive F");
    }

    #[test]
    fn noise_sigma_scales_with_sqrt_temperature() {
        let s1 = langevin_noise_sigma(0.2, 1.0, 1.0, 0.01);
        let s2 = langevin_noise_sigma(0.2, 1.0, 2.0, 0.01);
        let ratio = s2 / s1;
        assert!(
            (ratio - core::f64::consts::SQRT_2).abs() < 1e-10,
            "sigma(2T)/sigma(T) = sqrt(2): got {ratio}",
        );
    }

    #[test]
    fn noise_sigma_zero_temperature_zero() {
        assert_eq!(langevin_noise_sigma(0.2, 1.0, 0.0, 0.01), 0.0);
    }

    #[test]
    fn kinetic_temperature_single_particle() {
        // 1 particle, m=1, v=[1,0] → KE = 0.5, T = 2*0.5/(2*1) = 0.5
        let t = kinetic_temperature(&[1.0], &[[1.0, 0.0]], 1.0);
        assert!((t - 0.5).abs() < 1e-10, "T={t}");
    }

    #[test]
    fn kinetic_temperature_zero_velocity() {
        let t = kinetic_temperature(&[1.0, 1.0], &[[0.0, 0.0], [0.0, 0.0]], 1.0);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn maxwell_boltzmann_zero_z_zero_velocity() {
        assert_eq!(maxwell_boltzmann_velocity(1.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn maxwell_boltzmann_scales_with_temperature() {
        let v1 = maxwell_boltzmann_velocity(1.0, 1.0, 1.0);
        let v2 = maxwell_boltzmann_velocity(1.0, 4.0, 1.0);
        assert!((v2 / v1 - 2.0).abs() < 1e-10, "v ∝ sqrt(T)");
    }

    #[test]
    fn erf_known_values() {
        assert!((erf(0.0)).abs() < 1e-7);
        assert!((erf(1.0) - 0.842_700_8).abs() < 1e-5);
        assert!((erf(-1.0) + 0.842_700_8).abs() < 1e-5);
    }

    #[test]
    fn chi2_perfect_gaussian_is_small() {
        // Generate samples from a known Gaussian and check chi2 is low.
        let sigma = 1.0;
        let n = 10_000;
        let mut velocities = Vec::with_capacity(n);
        let mut state = 42u64;
        for _ in 0..n {
            state = super::super::determinism::next_u64(state);
            let v = super::super::determinism::gaussian_f32(state, sigma as f32) as f64;
            velocities.push(v);
        }
        let chi2 = velocity_distribution_chi2(&velocities, 1.0, sigma * sigma, 20);
        // For 20 bins, chi2 < 40 is a reasonable threshold (p > 0.01 ≈ df=19, chi2<36)
        assert!(chi2 < 50.0, "chi2 = {chi2} too high for Gaussian samples");
    }
}
