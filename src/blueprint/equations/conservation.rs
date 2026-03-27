//! R1: Invariantes de conservación energética.
//! Funciones de verificación de validez de qe y conservación global de pool.

/// Verifica que un valor de qe sea válido: finito y no negativo.
#[inline]
pub fn is_valid_qe(qe: f32) -> bool {
    qe.is_finite() && qe >= 0.0
}

/// Error de conservación global: `|Σ extracted − available.min(Σ extracted)|`.
/// Retorna 0.0 cuando el total extraído no supera el disponible.
#[inline]
pub fn global_conservation_error(available: f32, extracted: &[f32]) -> f32 {
    let total: f32 = extracted.iter().sum();
    (total - available.min(total)).abs()
}

/// Per-pool conservation error for a single tick.
/// `|pool_before + actual_intake - pool_after - total_extracted - total_dissipated|`.
#[inline]
pub fn conservation_error(
    pool_before: f32,
    pool_after: f32,
    actual_intake: f32,
    total_extracted: f32,
    total_dissipated: f32,
) -> f32 {
    (pool_before + actual_intake - pool_after - total_extracted - total_dissipated).abs()
}

/// Retorna `true` si algún valor del slice es NaN o Inf.
#[inline]
pub fn has_invalid_values(values: &[f32]) -> bool {
    values.iter().any(|v| !v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_qe_zero_is_valid() {
        assert!(is_valid_qe(0.0));
    }

    #[test]
    fn is_valid_qe_positive_is_valid() {
        assert!(is_valid_qe(9999.0));
    }

    #[test]
    fn is_valid_qe_negative_is_invalid() {
        assert!(!is_valid_qe(-0.1));
    }

    #[test]
    fn is_valid_qe_nan_is_invalid() {
        assert!(!is_valid_qe(f32::NAN));
    }

    #[test]
    fn is_valid_qe_inf_is_invalid() {
        assert!(!is_valid_qe(f32::INFINITY));
    }

    #[test]
    fn is_valid_qe_neg_inf_is_invalid() {
        assert!(!is_valid_qe(f32::NEG_INFINITY));
    }

    #[test]
    fn global_conservation_error_no_overshoot_returns_zero() {
        // total=300 ≤ available=1000 → error=0
        assert_eq!(global_conservation_error(1000.0, &[100.0, 100.0, 100.0]), 0.0);
    }

    #[test]
    fn global_conservation_error_exact_boundary_returns_zero() {
        assert_eq!(global_conservation_error(300.0, &[100.0, 100.0, 100.0]), 0.0);
    }

    #[test]
    fn global_conservation_error_overshoot_returns_excess() {
        // total=400, available=300 → excess=100
        let err = global_conservation_error(300.0, &[200.0, 200.0]);
        assert!((err - 100.0).abs() < 1e-5, "err={err}");
    }

    #[test]
    fn global_conservation_error_empty_slice_returns_zero() {
        assert_eq!(global_conservation_error(1000.0, &[]), 0.0);
    }

    #[test]
    fn has_invalid_values_all_finite_returns_false() {
        assert!(!has_invalid_values(&[0.0, 1.0, 9999.0]));
    }

    #[test]
    fn has_invalid_values_nan_returns_true() {
        assert!(has_invalid_values(&[1.0, f32::NAN, 3.0]));
    }

    #[test]
    fn has_invalid_values_inf_returns_true() {
        assert!(has_invalid_values(&[f32::INFINITY]));
    }

    #[test]
    fn has_invalid_values_neg_inf_returns_true() {
        assert!(has_invalid_values(&[f32::NEG_INFINITY, 0.0]));
    }

    #[test]
    fn has_invalid_values_empty_slice_returns_false() {
        assert!(!has_invalid_values(&[]));
    }
}
