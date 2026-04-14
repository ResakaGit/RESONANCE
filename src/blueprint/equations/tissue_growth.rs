//! Crecimiento diferencial de tejido — curvatura desde gradiente de nutrientes.
//! Differential tissue growth — curvature from nutrient gradient.
//!
//! Inner face (closer to stem) receives more nutrients → grows faster → curves outward.
//! Curvature derived from distance attenuation (Axiom 7), not from organ role.

/// Differential growth rate between inner and outer face of an organ.
/// Tasa de crecimiento diferencial entre cara interna y externa.
///
/// Inner face is closer to the nutrient source (stem), outer is farther.
/// Returns ratio > 1.0 when inner grows faster (outward curvature).
#[inline]
pub fn differential_growth_rate(
    organ_qe: f32,
    distance_inner: f32,
    distance_outer: f32,
) -> f32 {
    if organ_qe <= 0.0 || distance_inner <= 0.0 || distance_outer <= 0.0 {
        return 1.0; // no curvature
    }
    let flux_inner = organ_qe / (1.0 + distance_inner * distance_inner);
    let flux_outer = organ_qe / (1.0 + distance_outer * distance_outer);
    if flux_outer <= 1e-6 {
        return 1.0;
    }
    (flux_inner / flux_outer).clamp(0.5, 4.0)
}

/// Curvature value from growth ratio. Logarithmic to prevent extreme bending.
/// Valor de curvatura desde ratio de crecimiento. Log para evitar flexión extrema.
///
/// Returns 0 when ratio = 1 (no differential), positive for outward curl.
#[inline]
pub fn curvature_from_gradient(growth_ratio: f32, scale: f32) -> f32 {
    if growth_ratio <= 0.0 {
        return 0.0;
    }
    growth_ratio.ln().clamp(-1.0, 1.0) * scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_distances_no_differential() {
        let r = differential_growth_rate(10.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn inner_closer_grows_faster() {
        let r = differential_growth_rate(10.0, 0.5, 2.0);
        assert!(r > 1.0, "inner should grow faster, got {r}");
    }

    #[test]
    fn zero_qe_no_curvature() {
        let r = differential_growth_rate(0.0, 0.5, 2.0);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn curvature_zero_at_ratio_one() {
        let c = curvature_from_gradient(1.0, 0.3);
        assert!((c).abs() < 1e-5);
    }

    #[test]
    fn curvature_positive_for_ratio_above_one() {
        let c = curvature_from_gradient(2.0, 0.3);
        assert!(c > 0.0);
    }

    #[test]
    fn curvature_clamped() {
        let c = curvature_from_gradient(100.0, 1.0);
        assert!(c <= 1.0);
    }
}
