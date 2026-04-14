//! Dissipation acumulada — matemática del coarsening multi-escala.
//! Accumulated dissipation — multi-scale coarsening math.
//!
//! CT-6 / ADR-036 §D3. Pure. Funciones `O(1)` que proyectan evolución
//! exponencial de N ticks sin iterar.

/// Aplica N ticks de dissipation en O(1): `qe × (1 - rate)^N`, bounded in [0, qe].
/// Applies N ticks of dissipation in closed form.
///
/// Equivalent to iterating `qe = qe · (1 - rate)` N times, bit-close modulo f64.
#[inline]
pub fn accumulated_dissipation(qe: f64, rate: f64, n_ticks: u64) -> f64 {
    if qe <= 0.0 || n_ticks == 0 { return qe.max(0.0); }
    let clamped = rate.clamp(0.0, 1.0);
    let factor = (1.0 - clamped).powi(n_ticks.min(i32::MAX as u64) as i32);
    (qe * factor).max(0.0)
}

/// Ratio de coarsening por distancia a la escala observada: `k^distance`.
/// Coarsening ratio by distance to observed: `k^distance`.
///
/// Retorna `None` si la distancia ≥ `max_distance` (frozen) o si es el nivel
/// observado (ratio 1:1 implícito, manejado por el caller).
pub fn coarsening_ratio(distance: u8, k: u64, max_distance: u8) -> Option<u64> {
    if distance == 0 { return Some(1); }
    if distance >= max_distance { return None; }
    Some(k.saturating_pow(distance as u32))
}

/// Error relativo entre dissipation iterada vs acumulada. Útil para tests.
/// Relative error between iterated vs accumulated dissipation.
pub fn iteration_vs_accumulated_error(qe: f64, rate: f64, n_ticks: u64) -> f64 {
    let mut iterated = qe;
    for _ in 0..n_ticks { iterated *= 1.0 - rate; }
    let accumulated = accumulated_dissipation(qe, rate, n_ticks);
    if iterated.abs() < 1e-18 { return (iterated - accumulated).abs(); }
    ((iterated - accumulated) / iterated).abs()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_ticks_is_identity() {
        assert_eq!(accumulated_dissipation(1000.0, 0.1, 0), 1000.0);
    }

    #[test]
    fn zero_rate_is_identity() {
        assert_eq!(accumulated_dissipation(1000.0, 0.0, 100), 1000.0);
    }

    #[test]
    fn full_rate_drains_to_zero() {
        assert_eq!(accumulated_dissipation(1000.0, 1.0, 1), 0.0);
        assert_eq!(accumulated_dissipation(1000.0, 1.0, 10), 0.0);
    }

    #[test]
    fn accumulated_equals_iterated_within_tolerance() {
        for rate in [0.001, 0.01, 0.05, 0.1] {
            for n in [1u64, 10, 100, 1000] {
                let err = iteration_vs_accumulated_error(1000.0, rate, n);
                assert!(err < 1e-9, "rate={rate} n={n} err={err}");
            }
        }
    }

    #[test]
    fn negative_qe_clamped_to_zero() {
        assert_eq!(accumulated_dissipation(-5.0, 0.1, 1), 0.0);
    }

    #[test]
    fn ratio_geometric_with_k_16() {
        assert_eq!(coarsening_ratio(0, 16, 5), Some(1));
        assert_eq!(coarsening_ratio(1, 16, 5), Some(16));
        assert_eq!(coarsening_ratio(2, 16, 5), Some(256));
        assert_eq!(coarsening_ratio(3, 16, 5), Some(4096));
        assert_eq!(coarsening_ratio(4, 16, 5), Some(65_536));
    }

    #[test]
    fn ratio_frozen_at_or_beyond_max_distance() {
        assert_eq!(coarsening_ratio(5, 16, 5), None);
        assert_eq!(coarsening_ratio(10, 16, 5), None);
    }
}
