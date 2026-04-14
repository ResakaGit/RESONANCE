//! Dinámica estelar — Salpeter IMF, nucleosíntesis emergente, órbitas keplerianas.
//! Stellar dynamics — Salpeter IMF, emergent nucleosynthesis, Keplerian orbits.
//!
//! Pure math. CT-4 / ADR-036 §D4 (S0→S1).
//!
//! **No se modelan reacciones nucleares.** La frecuencia sube con la edad porque
//! la energía remanente se concentra en modos más altos: `f' = f · (E_0/E)^0.25`.
//! Derivado de la relación termodinámica E ∝ T⁴ (Stefan-Boltzmann) invertida.

use crate::blueprint::equations::determinism;

// ─── Salpeter IMF (initial mass function) ───────────────────────────────────

/// Exponente clásico de Salpeter (1955): `dN/dM ∝ M^-2.35`.
/// Classical Salpeter exponent.
pub const SALPETER_EXPONENT: f64 = 2.35;

/// Rangos de masa estelar (en unidades de qe arbitrarias).
/// Stellar mass range (arbitrary qe units).
pub const STELLAR_MASS_MIN: f64 = 1.0;
pub const STELLAR_MASS_MAX: f64 = 100.0;

/// Muestra una masa desde Salpeter IMF por inverse-CDF sampling.
/// Samples a stellar mass from Salpeter IMF via inverse CDF.
///
/// CDF: `P(M≤m) = (m^(1-α) - M_min^(1-α)) / (M_max^(1-α) - M_min^(1-α))`.
/// Para α=2.35: `M = (M_min^-1.35 - u·(M_min^-1.35 - M_max^-1.35))^(-1/1.35)`.
pub fn salpeter_mass_sample(u01: f64, m_min: f64, m_max: f64) -> f64 {
    let a = 1.0 - SALPETER_EXPONENT;
    let lo = m_min.powf(a);
    let hi = m_max.powf(a);
    let mixed = lo + u01.clamp(0.0, 1.0) * (hi - lo);
    mixed.powf(1.0 / a).clamp(m_min, m_max)
}

/// Genera N masas Salpeter-distribuidas, normalizadas para sumar `total_qe`.
/// Generates N Salpeter-distributed masses, rescaled to sum to `total_qe`.
///
/// **Garantía:** `sum(result) == total_qe` (bit-exacto modulo f64 rounding).
pub fn salpeter_mass_distribution(
    n: usize,
    total_qe: f64,
    seed: u64,
) -> Vec<f64> {
    if n == 0 || total_qe <= 0.0 { return Vec::new(); }
    let mut rng = seed.wrapping_add(0xD1B54A32D192ED03);
    let mut raw = Vec::with_capacity(n);
    let mut sum = 0.0;
    for _ in 0..n {
        rng = determinism::next_u64(rng);
        let u = (rng as f64) / (u64::MAX as f64);
        let m = salpeter_mass_sample(u, STELLAR_MASS_MIN, STELLAR_MASS_MAX);
        raw.push(m);
        sum += m;
    }
    if sum <= 0.0 { return raw; }
    let k = total_qe / sum;
    for m in raw.iter_mut() { *m *= k; }
    raw
}

// ─── Nucleosynthesis as frequency shift ─────────────────────────────────────

/// Desplazamiento emergente de frecuencia por edad estelar.
/// Emergent frequency shift from stellar ageing.
///
/// `f' = f · (E_initial / E_current)^0.25`. Al perder energía, la frecuencia
/// sube (blue shift). Derivado de E ∝ T⁴ con T ∝ f.
/// Clamped abajo por `f` (nunca cae) y arriba por un factor 10× (evita explosión numérica).
#[inline]
pub fn nucleosynthesis_shift(frequency_hz: f64, qe_initial: f64, qe_current: f64) -> f64 {
    if qe_current <= 0.0 || qe_initial <= 0.0 { return frequency_hz; }
    let ratio = (qe_initial / qe_current).max(1.0);
    let shift = ratio.powf(0.25).min(10.0);
    frequency_hz * shift
}

// ─── Keplerian disk ─────────────────────────────────────────────────────────

/// Velocidad orbital circular kepleriana: `v = sqrt(G·M/r)`.
/// Circular Keplerian orbital velocity.
#[inline]
pub fn keplerian_speed(g: f64, host_mass: f64, radius: f64) -> f64 {
    if radius <= 0.0 || host_mass <= 0.0 || g <= 0.0 { return 0.0; }
    (g * host_mass / radius).sqrt()
}

/// Vector velocidad tangencial en el plano xy dado un offset radial en 3D.
/// Tangential velocity vector in xy-plane given a 3D radial offset.
///
/// Usa eje-z como eje de rotación canónico. Suficiente para disco 2D inicial
/// (la gravedad 3D emergente rompe la coplanaridad si se quiere).
pub fn tangential_velocity_xy(offset: [f64; 3], speed: f64) -> [f64; 3] {
    let r2 = offset[0] * offset[0] + offset[1] * offset[1];
    if r2 <= 1e-18 { return [0.0; 3]; }
    let r = r2.sqrt();
    // Tangent = rotate radial (x,y) by +90° → (-y, x) / r.
    [-offset[1] / r * speed, offset[0] / r * speed, 0.0]
}

/// Momento angular de una partícula alrededor del origen: `L = r × (m·v)`.
/// Angular momentum of a particle about the origin.
#[inline]
pub fn angular_momentum(position: [f64; 3], velocity: [f64; 3], mass: f64) -> [f64; 3] {
    [
        mass * (position[1] * velocity[2] - position[2] * velocity[1]),
        mass * (position[2] * velocity[0] - position[0] * velocity[2]),
        mass * (position[0] * velocity[1] - position[1] * velocity[0]),
    ]
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salpeter_sample_within_bounds() {
        for u in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let m = salpeter_mass_sample(u, STELLAR_MASS_MIN, STELLAR_MASS_MAX);
            assert!((STELLAR_MASS_MIN..=STELLAR_MASS_MAX).contains(&m), "m={m} u={u}");
        }
    }

    #[test]
    fn salpeter_prefers_low_mass() {
        // IMF is heavily weighted toward low masses — 50% quantile should be well below geometric mean.
        let median = salpeter_mass_sample(0.5, STELLAR_MASS_MIN, STELLAR_MASS_MAX);
        let geomean = (STELLAR_MASS_MIN * STELLAR_MASS_MAX).sqrt();
        assert!(median < geomean, "median {median} not below geomean {geomean}");
    }

    #[test]
    fn salpeter_distribution_conserves_total() {
        let total = 10_000.0;
        let masses = salpeter_mass_distribution(100, total, 42);
        let sum: f64 = masses.iter().sum();
        assert!((sum - total).abs() / total < 1e-9, "sum {sum} vs total {total}");
    }

    #[test]
    fn salpeter_distribution_deterministic() {
        let a = salpeter_mass_distribution(50, 1000.0, 7);
        let b = salpeter_mass_distribution(50, 1000.0, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn salpeter_sampler_power_law_shape() {
        // Test the raw sampler (no renormalization) to verify IMF shape.
        let mut rng = 42u64;
        let mut low_count = 0;
        let mut high_count = 0;
        for _ in 0..5000 {
            rng = determinism::next_u64(rng);
            let u = (rng as f64) / (u64::MAX as f64);
            let m = salpeter_mass_sample(u, STELLAR_MASS_MIN, STELLAR_MASS_MAX);
            if (STELLAR_MASS_MIN..2.0).contains(&m) { low_count += 1; }
            if (10.0..20.0).contains(&m) { high_count += 1; }
        }
        // For α=2.35 ratio ≈ (15/1.5)^(-2.35) ≈ 0.0045 → low dominates ≥5×.
        assert!(
            low_count >= high_count * 5,
            "low={low_count} high={high_count} — sampler not Salpeter-shaped",
        );
    }

    #[test]
    fn salpeter_distribution_preserves_shape_after_normalization() {
        // After sum-normalization, the median should still be well below the mean
        // (heavy tail to the right is the Salpeter signature).
        let mut masses = salpeter_mass_distribution(1000, 1000.0 * 5.0, 11);
        masses.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = masses[masses.len() / 2];
        let mean: f64 = masses.iter().sum::<f64>() / masses.len() as f64;
        assert!(median < mean * 0.6, "median {median} not heavy-tail below mean {mean}");
    }

    #[test]
    fn nucleosynthesis_shifts_upward_as_mass_decreases() {
        let f0 = 100.0;
        let older = nucleosynthesis_shift(f0, 1000.0, 500.0);
        let very_old = nucleosynthesis_shift(f0, 1000.0, 100.0);
        assert!(older > f0);
        assert!(very_old > older);
    }

    #[test]
    fn nucleosynthesis_no_shift_when_unchanged() {
        let f = nucleosynthesis_shift(100.0, 500.0, 500.0);
        assert!((f - 100.0).abs() < 1e-9);
    }

    #[test]
    fn nucleosynthesis_never_drops_frequency() {
        // Increase in qe (shouldn't happen physically, defensive) stays ≥ original.
        let f = nucleosynthesis_shift(100.0, 500.0, 1000.0);
        assert!(f >= 100.0);
    }

    #[test]
    fn nucleosynthesis_zero_qe_is_neutral() {
        assert_eq!(nucleosynthesis_shift(100.0, 0.0, 10.0), 100.0);
        assert_eq!(nucleosynthesis_shift(100.0, 10.0, 0.0), 100.0);
    }

    #[test]
    fn keplerian_speed_scales_correctly() {
        let v_near = keplerian_speed(1.0, 100.0, 1.0);
        let v_far = keplerian_speed(1.0, 100.0, 4.0);
        assert!((v_near / v_far - 2.0).abs() < 1e-9, "v ∝ r^-1/2");
    }

    #[test]
    fn tangential_velocity_is_perpendicular_to_radius() {
        let offset = [3.0, 4.0, 0.0];
        let v = tangential_velocity_xy(offset, 5.0);
        let dot = offset[0] * v[0] + offset[1] * v[1] + offset[2] * v[2];
        assert!(dot.abs() < 1e-9, "r·v = {dot}, should be 0");
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((speed - 5.0).abs() < 1e-9);
    }

    #[test]
    fn angular_momentum_nonzero_for_tangential_motion() {
        let pos = [3.0, 0.0, 0.0];
        let vel = [0.0, 4.0, 0.0];
        let l = angular_momentum(pos, vel, 2.0);
        // L_z = m * (x * vy - y * vx) = 2 * (3*4 - 0) = 24
        assert_eq!(l, [0.0, 0.0, 24.0]);
    }

    #[test]
    fn angular_momentum_zero_for_radial_motion() {
        let pos = [3.0, 4.0, 0.0];
        let vel = [6.0, 8.0, 0.0]; // parallel to pos
        let l = angular_momentum(pos, vel, 1.0);
        assert!(l[0].abs() < 1e-9);
        assert!(l[1].abs() < 1e-9);
        assert!(l[2].abs() < 1e-9);
    }
}
