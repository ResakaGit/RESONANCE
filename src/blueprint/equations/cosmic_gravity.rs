//! Gravedad cosmológica — N-body 3D con modulación de frecuencia (Axiom 8).
//! Cosmological gravity — 3D N-body with frequency modulation (Axiom 8).
//!
//! Pure math. No ECS, no side effects.
//!
//! **Axiom 7 (Distance Attenuation):** F ∝ 1/r² (InverseSquare).
//! **Axiom 8 (Oscillatory):** atracción modulada por `cos(2π · Δf / bandwidth)`.
//! Integrador externo (Verlet), dissipation externa (system separado).

use crate::blueprint::equations::derived_thresholds::{
    COHERENCE_BANDWIDTH, DENSITY_SCALE, DISSIPATION_GAS,
};

/// Constante gravitacional derivada. `G = 1 / DENSITY_SCALE` — misma escala que
/// Coulomb `k_C`. Acopla masa (= qe) con geometría del espacio.
/// Gravitational constant. `G = 1 / DENSITY_SCALE` — same scale as Coulomb `k_C`.
pub const fn gravitational_constant() -> f64 {
    1.0 / DENSITY_SCALE as f64
}

/// Tasa de expansión tipo Hubble derivada de `DISSIPATION_GAS`. Universo temprano
/// es gas/plasma; dissipation determina el ritmo de expansión.
/// Hubble-like expansion rate derived from `DISSIPATION_GAS`.
pub const fn expansion_rate_default() -> f64 {
    DISSIPATION_GAS as f64
}

/// Softening de Plummer para evitar singularidad en r→0. `ε = 1 / DENSITY_SCALE`.
/// Plummer softening to avoid r→0 singularity. `ε = 1 / DENSITY_SCALE`.
#[inline]
pub const fn plummer_softening() -> f64 {
    1.0 / DENSITY_SCALE as f64
}

/// Modulador de frecuencia para la gravedad (Axiom 8). Resultado en `[1-α, 1+α]`.
/// Frequency modulator for gravity (Axiom 8). Result in `[1-α, 1+α]`.
///
/// `α = 0.5` — atracción hasta 50% más fuerte entre frecuencias alineadas,
/// hasta 50% más débil entre opuestas.
#[inline]
pub fn frequency_alignment(freq_a: f64, freq_b: f64) -> f64 {
    const ALPHA: f64 = 0.5;
    let bw = COHERENCE_BANDWIDTH as f64;
    let delta = (freq_a - freq_b).abs();
    1.0 + ALPHA * (std::f64::consts::TAU * delta / bw.max(1e-9)).cos()
}

/// Aceleración gravitacional sobre `i` por todas las masas `m_j` en posiciones `x_j`.
/// Naive O(N²). Suficiente hasta N~512 (escala cosmológica, pocos clusters).
/// Gravitational acceleration on `i` from all other masses. Naive O(N²).
///
/// `a_i = G · Σ_{j≠i} m_j · α(f_i,f_j) · (x_j - x_i) / (|x_j - x_i|² + ε²)^(3/2)`
pub fn gravity_accelerations(
    positions: &[[f64; 3]],
    masses: &[f64],
    frequencies: &[f64],
) -> Vec<[f64; 3]> {
    let n = positions.len();
    debug_assert_eq!(masses.len(), n);
    debug_assert_eq!(frequencies.len(), n);

    let g = gravitational_constant();
    let eps2 = plummer_softening().powi(2);
    let mut acc = vec![[0.0_f64; 3]; n];

    for i in 0..n {
        for j in 0..n {
            if i == j { continue; }
            let dx = positions[j][0] - positions[i][0];
            let dy = positions[j][1] - positions[i][1];
            let dz = positions[j][2] - positions[i][2];
            let r2 = dx * dx + dy * dy + dz * dz + eps2;
            let inv = 1.0 / (r2 * r2.sqrt());
            let modul = frequency_alignment(frequencies[i], frequencies[j]);
            let factor = g * masses[j] * modul * inv;
            acc[i][0] += factor * dx;
            acc[i][1] += factor * dy;
            acc[i][2] += factor * dz;
        }
    }
    acc
}

/// Aplica término de expansión tipo Hubble: `v += H · x · dt`. Aleja posiciones
/// del origen proporcional a la distancia (aceleración positiva radial).
/// Applies Hubble-like expansion: `v += H · x · dt`.
#[inline]
pub fn apply_expansion(velocity: &mut [f64; 3], position: [f64; 3], hubble: f64, dt: f64) {
    let factor = hubble * dt;
    velocity[0] += position[0] * factor;
    velocity[1] += position[1] * factor;
    velocity[2] += position[2] * factor;
}

/// Aplica dissipation multiplicativa a la energía: `qe *= (1 - rate·dt)`.
/// Acotada inferiormente a 0. Axiom 4.
/// Applies multiplicative dissipation to energy. Bounded at 0.
#[inline]
pub fn apply_dissipation_qe(qe: f64, rate: f64, dt: f64) -> f64 {
    (qe * (1.0 - rate * dt)).max(0.0)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_zero_for_single_body() {
        let acc = gravity_accelerations(&[[0.0; 3]], &[1.0], &[100.0]);
        assert_eq!(acc, vec![[0.0, 0.0, 0.0]]);
    }

    #[test]
    fn gravity_attractive_between_two_bodies() {
        let acc = gravity_accelerations(
            &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
            &[100.0, 100.0],
            &[50.0, 50.0],
        );
        // Body 0 accelerates toward +x, body 1 toward -x.
        assert!(acc[0][0] > 0.0);
        assert!(acc[1][0] < 0.0);
        // Symmetric magnitudes (same mass).
        assert!((acc[0][0] + acc[1][0]).abs() < 1e-9);
    }

    #[test]
    fn gravity_inverse_square_scaling() {
        // Doubling distance should reduce acceleration magnitude by 4× (ignoring softening at far range).
        let far = 100.0_f64;
        let a_near = gravity_accelerations(
            &[[0.0; 3], [far, 0.0, 0.0]],
            &[1.0, 1.0],
            &[50.0, 50.0],
        );
        let a_far = gravity_accelerations(
            &[[0.0; 3], [2.0 * far, 0.0, 0.0]],
            &[1.0, 1.0],
            &[50.0, 50.0],
        );
        let ratio = a_near[0][0] / a_far[0][0];
        assert!((ratio - 4.0).abs() < 0.1, "ratio {ratio} far from 4 (inverse-square)");
    }

    #[test]
    fn frequency_alignment_bounded() {
        for (a, b) in [(0.0, 0.0), (100.0, 100.0), (0.0, 25.0), (0.0, 50.0), (0.0, 500.0)] {
            let m = frequency_alignment(a, b);
            assert!(m >= 0.5 - 1e-9, "alignment {m} below 0.5");
            assert!(m <= 1.5 + 1e-9, "alignment {m} above 1.5");
        }
    }

    #[test]
    fn frequency_alignment_peaks_at_equal() {
        let aligned = frequency_alignment(100.0, 100.0);
        let off = frequency_alignment(100.0, 100.0 + COHERENCE_BANDWIDTH as f64 / 2.0);
        assert!(aligned > off, "aligned {aligned} <= off {off}");
    }

    #[test]
    fn expansion_moves_outward() {
        let mut v = [0.0, 0.0, 0.0];
        apply_expansion(&mut v, [5.0, -3.0, 2.0], 0.1, 1.0);
        assert!(v[0] > 0.0 && v[1] < 0.0 && v[2] > 0.0);
    }

    #[test]
    fn dissipation_reduces_monotonically() {
        let before = 1000.0;
        let after = apply_dissipation_qe(before, DISSIPATION_GAS as f64, 1.0);
        assert!(after < before);
        assert!(after > 0.0);
    }

    #[test]
    fn dissipation_cannot_go_negative() {
        let after = apply_dissipation_qe(10.0, 2.0, 1.0);
        assert_eq!(after, 0.0);
    }
}
