//! R2: Determinism — hashing de estado del simulador.
//! Funciones puras para verificar que misma configuración → mismo resultado final.
//! Tests en `tests/r2_determinism.rs`.

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Hash determinista de un slice de f32 (orden importa).
/// Usa `to_bits()` para garantizar bit-exacto incluyendo +0.0/-0.0/NaN.
#[inline]
pub fn hash_f32_slice(values: &[f32]) -> u64 {
    let mut h = DefaultHasher::new();
    for v in values {
        v.to_bits().hash(&mut h);
    }
    h.finish()
}

/// Hash de snapshot de energías de un conjunto de entidades.
/// `energy_snapshot` debe estar en orden canónico (e.g. ordenado por Entity index).
#[inline]
pub fn snapshot_hash(energy_snapshot: &[f32]) -> u64 {
    hash_f32_slice(energy_snapshot)
}

/// Verifica que dos snapshots son idénticos bit a bit (contrato de determinismo).
#[inline]
pub fn snapshots_match(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && hash_f32_slice(a) == hash_f32_slice(b)
}

// ─── Deterministic RNG (batch simulator) ────────────────────────────────────

/// PCG-like state step: deterministic, no external deps.
/// `state' = state × 6364136223846793005 + 1442695040888963407`
#[inline]
pub fn next_u64(state: u64) -> u64 {
    state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407)
}

/// Uniform f32 in [0, 1) from state (uses top 24 bits for mantissa quality).
#[inline]
pub fn unit_f32(state: u64) -> f32 {
    (state >> 40) as f32 / ((1u64 << 24) as f32)
}

/// Uniform f32 in [min, max) from state.
#[inline]
pub fn range_f32(state: u64, min: f32, max: f32) -> f32 {
    min + unit_f32(state) * (max - min)
}

/// Gaussian f32 via Box-Muller (uses two next_u64 calls internally).
#[inline]
pub fn gaussian_f32(state: u64, sigma: f32) -> f32 {
    let u1 = unit_f32(state).max(1e-10);
    let u2 = unit_f32(next_u64(state));
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    z * sigma
}

// ─── Shared math: Gaussian frequency alignment (Axiom 8) ────────────────────

/// Gaussian frequency alignment. Axiom 8: oscillatory interaction modulated by Δf.
///
/// `alignment = exp(-Δf² / (2 × bandwidth²))`.
/// Same frequency → 1.0. Far frequencies → 0.0. Bandwidth = coherence window.
///
/// Centralized: used by protein_fold, metabolic_genome, multicellular.
/// Each caller passes their domain-specific bandwidth constant.
#[inline]
pub fn gaussian_frequency_alignment(f_a: f32, f_b: f32, bandwidth: f32) -> f32 {
    if !f_a.is_finite() || !f_b.is_finite() || bandwidth <= 0.0 { return 0.0; }
    let delta = (f_a - f_b).abs();
    (-delta * delta / (2.0 * bandwidth * bandwidth)).exp()
}

/// Sanitize f32 to unit range [0,1]. NaN/Inf → 0.0.
#[inline]
pub fn sanitize_unit(v: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaussian_same_freq_is_one() {
        assert!((gaussian_frequency_alignment(400.0, 400.0, 50.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn gaussian_different_freq_less_than_one() {
        assert!(gaussian_frequency_alignment(400.0, 600.0, 50.0) < 1.0);
    }

    #[test]
    fn gaussian_nan_safe() {
        assert_eq!(gaussian_frequency_alignment(f32::NAN, 400.0, 50.0), 0.0);
    }

    #[test]
    fn gaussian_zero_bandwidth_safe() {
        assert_eq!(gaussian_frequency_alignment(400.0, 400.0, 0.0), 0.0);
    }

    #[test]
    fn sanitize_unit_nan_zero() { assert_eq!(sanitize_unit(f32::NAN), 0.0); }

    #[test]
    fn sanitize_unit_clamps() { assert_eq!(sanitize_unit(2.0), 1.0); assert_eq!(sanitize_unit(-1.0), 0.0); }

    #[test]
    fn hash_f32_slice_same_input_same_output() {
        let vals = [1.0f32, 2.5, 100.0, 0.001];
        assert_eq!(hash_f32_slice(&vals), hash_f32_slice(&vals));
    }

    #[test]
    fn hash_f32_slice_different_order_different_hash() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [3.0f32, 2.0, 1.0];
        assert_ne!(hash_f32_slice(&a), hash_f32_slice(&b));
    }

    #[test]
    fn hash_f32_slice_empty_is_stable() {
        let h1 = hash_f32_slice(&[]);
        let h2 = hash_f32_slice(&[]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_f32_slice_single_zero_is_stable() {
        assert_eq!(hash_f32_slice(&[0.0]), hash_f32_slice(&[0.0]));
    }

    #[test]
    fn hash_f32_slice_different_values_different_hash() {
        let a = [100.0f32, 200.0, 300.0];
        let b = [100.0f32, 200.0, 301.0];
        assert_ne!(hash_f32_slice(&a), hash_f32_slice(&b));
    }

    #[test]
    fn snapshot_hash_delegates_to_hash_f32_slice() {
        let vals = [42.0f32, 7.0, 0.5];
        assert_eq!(snapshot_hash(&vals), hash_f32_slice(&vals));
    }

    #[test]
    fn snapshots_match_identical_slices_returns_true() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [1.0f32, 2.0, 3.0];
        assert!(snapshots_match(&a, &b));
    }

    #[test]
    fn snapshots_match_different_values_returns_false() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [1.0f32, 2.0, 4.0];
        assert!(!snapshots_match(&a, &b));
    }

    #[test]
    fn snapshots_match_different_length_returns_false() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [1.0f32, 2.0];
        assert!(!snapshots_match(&a, &b));
    }

    #[test]
    fn snapshots_match_empty_slices_returns_true() {
        assert!(snapshots_match(&[], &[]));
    }

    #[test]
    fn hash_f32_slice_nan_bits_are_stable() {
        let nan = f32::NAN;
        let a = [nan];
        let b = [nan];
        assert_eq!(hash_f32_slice(&a), hash_f32_slice(&b));
    }

    // ── RNG ─────────────────────────────────────────────────────────────────

    #[test]
    fn next_u64_deterministic() {
        assert_eq!(next_u64(42), next_u64(42));
    }

    #[test]
    fn next_u64_different_seeds_differ() {
        assert_ne!(next_u64(0), next_u64(1));
    }

    #[test]
    fn unit_f32_in_range() {
        for i in 0..1000 {
            let v = unit_f32(next_u64(i));
            assert!(v >= 0.0 && v < 1.0, "unit_f32({i}) = {v}");
        }
    }

    #[test]
    fn range_f32_in_bounds() {
        for i in 0..1000 {
            let v = range_f32(next_u64(i), 10.0, 20.0);
            assert!(v >= 10.0 && v < 20.0, "range_f32({i}) = {v}");
        }
    }

    #[test]
    fn gaussian_f32_is_finite() {
        for i in 0..1000 {
            let v = gaussian_f32(next_u64(i), 1.0);
            assert!(v.is_finite(), "gaussian({i}) = {v}");
        }
    }

    #[test]
    fn gaussian_f32_approximate_mean_zero() {
        let n = 10_000u64;
        let sum: f32 = (0..n).map(|i| gaussian_f32(next_u64(i * 7 + 13), 1.0)).sum();
        let mean = sum / n as f32;
        assert!(mean.abs() < 0.1, "mean={mean} should be near 0");
    }
}
