//! Transferencia cruzada — reproducción mediada por entidad transportadora.
//! Cross-transfer — reproduction mediated by carrier entity.
//!
//! Agnostic: works between any pair of entities, not just flora↔fauna.
//! Compatibility determined by frequency alignment (Axiom 8).

use crate::blueprint::constants::plant_physiology::TRANSFER_THRESHOLD;
use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;

/// Compatibility between energy tag source and target, via frequency alignment.
/// Compatibilidad entre fuente y destino via alineamiento de frecuencia.
///
/// Returns [0, 1]. Above TRANSFER_THRESHOLD = compatible for cross-reproduction.
#[inline]
pub fn transfer_compatibility(source_freq: f32, target_freq: f32) -> f32 {
    let df = source_freq - target_freq;
    let bw = COHERENCE_BANDWIDTH;
    (-0.5 * (df * df) / (bw * bw)).exp()
}

/// Mix two inference profiles for cross-reproduction offspring.
/// Mezclar dos perfiles de inferencia para descendencia cruzada.
///
/// `weight` ∈ [0, 1] determines contribution of each parent.
/// weight=0.5 gives equal mix.
pub fn mix_profiles(
    parent_a: [f32; 4],
    parent_b: [f32; 4],
    weight: f32,
) -> [f32; 4] {
    let w = weight.clamp(0.0, 1.0);
    let iw = 1.0 - w;
    [
        (parent_a[0] * w + parent_b[0] * iw).clamp(0.0, 1.0),
        (parent_a[1] * w + parent_b[1] * iw).clamp(0.0, 1.0),
        (parent_a[2] * w + parent_b[2] * iw).clamp(0.0, 1.0),
        (parent_a[3] * w + parent_b[3] * iw).clamp(0.0, 1.0),
    ]
}

/// Whether a transfer is compatible (above threshold).
/// Si una transferencia es compatible (sobre umbral).
#[inline]
pub fn is_transfer_compatible(source_freq: f32, target_freq: f32) -> bool {
    transfer_compatibility(source_freq, target_freq) >= TRANSFER_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_frequency_full_compatibility() {
        let c = transfer_compatibility(400.0, 400.0);
        assert!((c - 1.0).abs() < 1e-5);
    }

    #[test]
    fn distant_frequency_low_compatibility() {
        let c = transfer_compatibility(400.0, 700.0); // 6 bandwidths away
        assert!(c < 0.01);
    }

    #[test]
    fn compatible_within_bandwidth() {
        assert!(is_transfer_compatible(400.0, 410.0)); // 10 Hz apart
    }

    #[test]
    fn incompatible_far_apart() {
        assert!(!is_transfer_compatible(400.0, 700.0));
    }

    #[test]
    fn mix_equal_weight_averages() {
        let a = [1.0, 0.0, 0.5, 0.8];
        let b = [0.0, 1.0, 0.5, 0.2];
        let m = mix_profiles(a, b, 0.5);
        assert!((m[0] - 0.5).abs() < 1e-5);
        assert!((m[1] - 0.5).abs() < 1e-5);
        assert!((m[2] - 0.5).abs() < 1e-5);
        assert!((m[3] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn mix_full_weight_a() {
        let a = [1.0, 0.8, 0.6, 0.4];
        let b = [0.0, 0.0, 0.0, 0.0];
        let m = mix_profiles(a, b, 1.0);
        assert!((m[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn mix_clamped_to_unit() {
        let a = [1.5, -0.5, 0.5, 0.5];
        let b = [1.5, -0.5, 0.5, 0.5];
        let m = mix_profiles(a, b, 0.5);
        assert!(m[0] <= 1.0);
        assert!(m[1] >= 0.0);
    }
}
