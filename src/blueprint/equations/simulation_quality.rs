// ═══════════════════════════════════════════════
// SM-8C: Fórmulas extraídas de sistemas para calidad de código.
// ═══════════════════════════════════════════════

/// Perception signal strength: qe weighted by visibility, purity, and inverse square distance.
/// `S = qe * visibility * purity / dist²`
#[inline]
pub fn perception_signal_weighted(qe: f32, visibility: f32, purity: f32, distance_sq: f32) -> f32 {
    let d2 = distance_sq.max(1.0);
    qe * visibility * purity / d2
}

/// Apply bond weakening: scale bond energy by a weakening factor.
/// `eb_new = eb * weakening_factor`
#[inline]
pub fn bond_weakening(bond_energy: f32, weakening_factor: f32) -> f32 {
    bond_energy * weakening_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-6;

    // ── perception_signal_weighted ──────────────────────────────────────
    #[test]
    fn perception_signal_weighted_basic_computation() {
        let s = perception_signal_weighted(100.0, 1.0, 1.0, 4.0);
        assert!((s - 25.0).abs() < EPS, "got {s}");
    }

    #[test]
    fn perception_signal_weighted_zero_qe_returns_zero() {
        assert_eq!(perception_signal_weighted(0.0, 1.0, 1.0, 4.0), 0.0);
    }

    #[test]
    fn perception_signal_weighted_clamps_distance_to_one() {
        let s = perception_signal_weighted(10.0, 1.0, 0.5, 0.0);
        assert!((s - 5.0).abs() < EPS, "got {s}");
    }

    #[test]
    fn perception_signal_weighted_visibility_scales_linearly() {
        let full = perception_signal_weighted(100.0, 1.0, 1.0, 1.0);
        let half = perception_signal_weighted(100.0, 0.5, 1.0, 1.0);
        assert!((half / full - 0.5).abs() < EPS);
    }

    #[test]
    fn perception_signal_weighted_purity_scales_linearly() {
        let full = perception_signal_weighted(100.0, 1.0, 1.0, 1.0);
        let quarter = perception_signal_weighted(100.0, 1.0, 0.25, 1.0);
        assert!((quarter / full - 0.25).abs() < EPS);
    }

    #[test]
    fn perception_signal_weighted_inverse_square_law() {
        let near = perception_signal_weighted(100.0, 1.0, 1.0, 1.0);
        let far = perception_signal_weighted(100.0, 1.0, 1.0, 4.0);
        assert!((near / far - 4.0).abs() < EPS);
    }

    // ── bond_weakening ─────────────────────────────────────────────────
    #[test]
    fn bond_weakening_identity_factor() {
        assert!((bond_weakening(100.0, 1.0) - 100.0).abs() < EPS);
    }

    #[test]
    fn bond_weakening_half_factor() {
        assert!((bond_weakening(100.0, 0.5) - 50.0).abs() < EPS);
    }

    #[test]
    fn bond_weakening_zero_energy_returns_zero() {
        assert_eq!(bond_weakening(0.0, 0.8), 0.0);
    }

    #[test]
    fn bond_weakening_zero_factor_returns_zero() {
        assert_eq!(bond_weakening(100.0, 0.0), 0.0);
    }
}
