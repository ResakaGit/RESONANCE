/// Non-linear radiation pressure: excess energy above threshold pushes outward.
///
/// `transfer = rate × max(source_qe - threshold, 0) × (1 / n_neighbors)`
///
/// Positive result = source loses energy (flows outward to neighbors).
/// Zero when source below threshold (passive diffusion handles low-density flow).
#[inline]
pub fn radiation_pressure_transfer(
    source_qe: f32,
    threshold: f32,
    transfer_rate: f32,
    n_neighbors: u32,
) -> f32 {
    let excess = (source_qe - threshold).max(0.0);
    if excess <= 0.0 || n_neighbors == 0 {
        return 0.0;
    }
    let rate = transfer_rate.clamp(0.0, 1.0);
    (excess * rate / n_neighbors as f32).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_returns_zero() {
        assert_eq!(radiation_pressure_transfer(50.0, 100.0, 0.05, 4), 0.0);
    }

    #[test]
    fn at_threshold_returns_zero() {
        assert_eq!(radiation_pressure_transfer(100.0, 100.0, 0.05, 4), 0.0);
    }

    #[test]
    fn above_threshold_transfers_proportionally() {
        let t = radiation_pressure_transfer(200.0, 100.0, 0.1, 4);
        // excess=100, rate=0.1, /4 neighbors = 2.5 per neighbor
        assert!((t - 2.5).abs() < 1e-5, "got {t}");
    }

    #[test]
    fn zero_neighbors_returns_zero() {
        assert_eq!(radiation_pressure_transfer(500.0, 100.0, 0.1, 0), 0.0);
    }

    #[test]
    fn transfer_rate_clamped_to_unit() {
        let a = radiation_pressure_transfer(200.0, 100.0, 2.0, 4);
        let b = radiation_pressure_transfer(200.0, 100.0, 1.0, 4);
        assert!((a - b).abs() < 1e-5, "rate should clamp to 1.0");
    }

    #[test]
    fn higher_excess_means_more_transfer() {
        let low = radiation_pressure_transfer(150.0, 100.0, 0.05, 4);
        let high = radiation_pressure_transfer(500.0, 100.0, 0.05, 4);
        assert!(high > low);
    }
}
