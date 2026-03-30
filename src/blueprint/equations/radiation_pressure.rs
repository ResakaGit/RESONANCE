/// Frequency alignment factor for radiation pressure transfer.
/// Same frequency = easy transfer (alignment ≈ 1). Different frequency = resists mixing.
///
/// `alignment = exp(-Δf² / (2 × bandwidth²))`
///
/// Axiom 8: interaction modulated by frequency difference.
#[inline]
pub fn pressure_frequency_alignment(source_freq: f32, target_freq: f32, bandwidth: f32) -> f32 {
    let bw = bandwidth.max(1.0);
    let df = source_freq - target_freq;
    (-df * df / (2.0 * bw * bw)).exp()
}

/// Non-linear radiation pressure: excess energy pushes outward, modulated by frequency alignment.
///
/// `transfer = rate × excess × alignment / n_neighbors`
///
/// - Excess = `max(source_qe - threshold, 0)` (Axiom 4: only surplus does work)
/// - Alignment = `exp(-Δf²/2σ²)` (Axiom 8: coherent neighbors transfer easily)
/// - Result ≥ 0 (Axiom 5: no energy creation)
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

/// Frequency-modulated radiation pressure: transfers more between coherent cells.
///
/// Combines excess-based pressure with frequency alignment.
/// Biomes (same frequency) redistribute internally; cross-biome transfer is suppressed.
#[inline]
pub fn radiation_pressure_transfer_coherent(
    source_qe: f32,
    target_freq: f32,
    source_freq: f32,
    threshold: f32,
    transfer_rate: f32,
    bandwidth: f32,
    n_neighbors: u32,
) -> f32 {
    let base = radiation_pressure_transfer(source_qe, threshold, transfer_rate, n_neighbors);
    if base <= 0.0 {
        return 0.0;
    }
    base * pressure_frequency_alignment(source_freq, target_freq, bandwidth)
}

/// Default bandwidth for pressure frequency alignment (Hz).
/// Controls how far in frequency space pressure can reach.
/// Alias for COHERENCE_BANDWIDTH (4th fundamental constant).
pub use super::derived_thresholds::COHERENCE_BANDWIDTH as PRESSURE_FREQUENCY_BANDWIDTH;

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
        assert!((a - b).abs() < 1e-5);
    }

    #[test]
    fn higher_excess_means_more_transfer() {
        let low = radiation_pressure_transfer(150.0, 100.0, 0.05, 4);
        let high = radiation_pressure_transfer(500.0, 100.0, 0.05, 4);
        assert!(high > low);
    }

    // ── Frequency alignment tests ────────────────────────────────────────

    #[test]
    fn alignment_same_frequency_is_one() {
        let a = pressure_frequency_alignment(85.0, 85.0, 50.0);
        assert!((a - 1.0).abs() < 1e-5);
    }

    #[test]
    fn alignment_decreases_with_frequency_distance() {
        let near = pressure_frequency_alignment(85.0, 90.0, 50.0);
        let far = pressure_frequency_alignment(85.0, 440.0, 50.0);
        assert!(near > far, "near={near} far={far}");
    }

    #[test]
    fn alignment_distant_frequencies_near_zero() {
        let a = pressure_frequency_alignment(85.0, 1000.0, 50.0);
        assert!(a < 0.01, "got {a}");
    }

    #[test]
    fn alignment_symmetric() {
        let ab = pressure_frequency_alignment(85.0, 200.0, 50.0);
        let ba = pressure_frequency_alignment(200.0, 85.0, 50.0);
        assert!((ab - ba).abs() < 1e-5);
    }

    // ── Coherent transfer tests ──────────────────────────────────────────

    #[test]
    fn coherent_same_freq_equals_base() {
        let base = radiation_pressure_transfer(200.0, 100.0, 0.05, 4);
        let coherent = radiation_pressure_transfer_coherent(200.0, 85.0, 85.0, 100.0, 0.05, 50.0, 4);
        assert!((base - coherent).abs() < 1e-5);
    }

    #[test]
    fn coherent_different_freq_less_than_base() {
        let base = radiation_pressure_transfer(200.0, 100.0, 0.05, 4);
        let coherent = radiation_pressure_transfer_coherent(200.0, 440.0, 85.0, 100.0, 0.05, 50.0, 4);
        assert!(coherent < base, "coherent={coherent} base={base}");
    }

    #[test]
    fn coherent_very_different_freq_near_zero() {
        let t = radiation_pressure_transfer_coherent(500.0, 1000.0, 85.0, 100.0, 0.05, 50.0, 4);
        assert!(t < 0.1, "got {t}");
    }
}
