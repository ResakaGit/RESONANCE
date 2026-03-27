//! R6 / SF-1: Observability — pure math for simulation health and field metrics.
//! No side effects. All functions are deterministic.

use crate::blueprint::constants::units::CONSERVATION_ERROR_TOLERANCE;
use crate::layers::MatterState;
use crate::worldgen::EnergyCell;

// ─── Existing health metrics ─────────────────────────────────────────────────

/// Fractional change in total energy between two ticks.
/// Returns 0.0 when `energy_prev` is zero (no drift measurable).
/// Equation: (current - prev) / prev.
#[inline]
pub fn drift_rate(energy_prev: f32, energy_current: f32) -> f32 {
    if energy_prev == 0.0 { return 0.0; }
    (energy_current - energy_prev) / energy_prev
}

/// Saturation index: how close a value is to its maximum (0.0 ..= 1.0).
/// Clamped to [0, 1]. Returns 0.0 when `maximum` is zero.
#[inline]
pub fn saturation_index(current: f32, maximum: f32) -> f32 {
    if maximum == 0.0 { return 0.0; }
    (current / maximum).clamp(0.0, 1.0)
}

/// System cost ratio: fraction of budget spent (0.0 ..= 1.0).
/// Returns 0.0 when `total_budget` is zero.
#[inline]
pub fn system_cost_ratio(ticks_spent: u32, total_budget: u32) -> f32 {
    if total_budget == 0 { return 0.0; }
    (ticks_spent as f32 / total_budget as f32).clamp(0.0, 1.0)
}

/// Returns true if `drift` exceeds the critical threshold (absolute value).
#[inline]
pub fn is_critical_drift(drift: f32, threshold: f32) -> bool {
    drift.abs() > threshold
}

/// Returns true if `conservation_error` exceeds `CONSERVATION_ERROR_TOLERANCE`.
#[inline]
pub fn is_conservation_violation(error: f32) -> bool {
    error > CONSERVATION_ERROR_TOLERANCE
}

// ─── SF-1A: Field & ecology metrics ─────────────────────────────────────────

/// Total accumulated energy across all cells in the field.
/// Equation: sum(max(cell.accumulated_qe, 0.0) for cell in cells).
#[inline]
pub fn field_total_qe(cells: &[EnergyCell]) -> f32 {
    cells.iter().map(|c| c.accumulated_qe.max(0.0)).sum()
}

/// Fraction of cells with accumulated_qe above `threshold` (0.0 ..= 1.0).
/// Returns 0.0 for an empty slice.
#[inline]
pub fn field_occupancy(cells: &[EnergyCell], threshold: f32) -> f32 {
    if cells.is_empty() { return 0.0; }
    let active = cells.iter().filter(|c| c.accumulated_qe > threshold).count();
    active as f32 / cells.len() as f32
}

/// Fraction of cells per MatterState: [Solid, Liquid, Gas, Plasma], normalized to sum 1.0.
/// Returns [0.0; 4] for an empty slice.
pub fn field_matter_distribution(cells: &[EnergyCell]) -> [f32; 4] {
    if cells.is_empty() { return [0.0; 4]; }
    let mut counts = [0u32; 4];
    for cell in cells {
        let idx = match cell.matter_state {
            MatterState::Solid  => 0,
            MatterState::Liquid => 1,
            MatterState::Gas    => 2,
            MatterState::Plasma => 3,
        };
        counts[idx] += 1;
    }
    let total = cells.len() as f32;
    [
        counts[0] as f32 / total,
        counts[1] as f32 / total,
        counts[2] as f32 / total,
        counts[3] as f32 / total,
    ]
}

/// Instantaneous population growth rate: (current - previous) / max(previous, 1).
/// Positive = growth, negative = decline.
#[inline]
pub fn population_growth_rate(current: u32, previous: u32) -> f32 {
    let prev = previous.max(1) as f32;
    (current as f32 - previous as f32) / prev
}

/// Shannon diversity index over frequency band counts.
/// H = -sum(p_i * ln(p_i)) for p_i > 0, where p_i = band_i / total.
/// Returns 0.0 when total is zero or only one band has entries.
pub fn frequency_diversity_index(band_counts: &[u32]) -> f32 {
    let total: u32 = band_counts.iter().sum();
    if total == 0 { return 0.0; }
    let total_f = total as f32;
    let mut h: f32 = 0.0;
    for &count in band_counts {
        if count == 0 { continue; }
        let p = count as f32 / total_f;
        h -= p * p.ln();
    }
    h
}

/// Assigns each cell's dominant_frequency_hz to one of `num_bands` equal-width bands
/// spanning [0, max_freq] and returns the band counts.
/// Cells with dominant_frequency_hz <= 0.0 are ignored.
/// Returns a Vec of length `num_bands` (stack-friendly for small band counts).
pub fn frequency_band_histogram(cells: &[EnergyCell], num_bands: usize) -> Vec<u32> {
    let mut counts = vec![0u32; num_bands.max(1)];
    if cells.is_empty() || num_bands == 0 { return counts; }
    let max_freq = cells
        .iter()
        .map(|c| c.dominant_frequency_hz)
        .fold(0.0f32, f32::max);
    if max_freq <= 0.0 { return counts; }
    let band_width = max_freq / num_bands as f32;
    for cell in cells {
        if cell.dominant_frequency_hz <= 0.0 { continue; }
        let band = ((cell.dominant_frequency_hz / band_width) as usize).min(num_bands - 1);
        counts[band] += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::EnergyCell;

    fn make_cell(qe: f32, state: MatterState, freq_hz: f32) -> EnergyCell {
        let mut cell = EnergyCell::default();
        cell.accumulated_qe = qe;
        cell.matter_state = state;
        cell.dominant_frequency_hz = freq_hz;
        cell
    }

    // ─── drift_rate ──────────────────────────────────────────────────────────

    #[test]
    fn drift_rate_no_change_returns_zero() {
        assert_eq!(drift_rate(100.0, 100.0), 0.0);
    }

    #[test]
    fn drift_rate_increase_returns_positive() {
        let d = drift_rate(100.0, 110.0);
        assert!((d - 0.1).abs() < 1e-6, "expected 0.1, got {d}");
    }

    #[test]
    fn drift_rate_decrease_returns_negative() {
        let d = drift_rate(100.0, 90.0);
        assert!((d - (-0.1)).abs() < 1e-6, "expected -0.1, got {d}");
    }

    #[test]
    fn drift_rate_zero_prev_returns_zero() {
        assert_eq!(drift_rate(0.0, 50.0), 0.0);
    }

    // ─── saturation_index ────────────────────────────────────────────────────

    #[test]
    fn saturation_index_full_returns_one() {
        assert_eq!(saturation_index(100.0, 100.0), 1.0);
    }

    #[test]
    fn saturation_index_empty_returns_zero() {
        assert_eq!(saturation_index(0.0, 100.0), 0.0);
    }

    #[test]
    fn saturation_index_over_maximum_clamps_to_one() {
        assert_eq!(saturation_index(150.0, 100.0), 1.0);
    }

    #[test]
    fn saturation_index_zero_maximum_returns_zero() {
        assert_eq!(saturation_index(50.0, 0.0), 0.0);
    }

    // ─── system_cost_ratio ───────────────────────────────────────────────────

    #[test]
    fn system_cost_ratio_zero_budget_returns_zero() {
        assert_eq!(system_cost_ratio(10, 0), 0.0);
    }

    #[test]
    fn system_cost_ratio_full_budget_returns_one() {
        assert_eq!(system_cost_ratio(5, 5), 1.0);
    }

    // ─── is_critical_drift ───────────────────────────────────────────────────

    #[test]
    fn is_critical_drift_above_threshold_returns_true() {
        assert!(is_critical_drift(0.1, 0.05));
    }

    #[test]
    fn is_critical_drift_below_threshold_returns_false() {
        assert!(!is_critical_drift(0.02, 0.05));
    }

    // ─── is_conservation_violation ───────────────────────────────────────────

    #[test]
    fn is_conservation_violation_above_tolerance_returns_true() {
        assert!(is_conservation_violation(CONSERVATION_ERROR_TOLERANCE + 0.001));
    }

    #[test]
    fn is_conservation_violation_at_tolerance_returns_false() {
        assert!(!is_conservation_violation(CONSERVATION_ERROR_TOLERANCE));
    }

    // ─── field_total_qe ──────────────────────────────────────────────────────

    #[test]
    fn field_total_qe_sums_accumulated() {
        let cells = [
            make_cell(10.0, MatterState::Solid, 0.0),
            make_cell(20.0, MatterState::Solid, 0.0),
            make_cell(5.0, MatterState::Liquid, 0.0),
        ];
        let total = field_total_qe(&cells);
        assert!((total - 35.0).abs() < 1e-6, "expected 35.0, got {total}");
    }

    #[test]
    fn field_total_qe_negative_clamped_to_zero() {
        let cells = [
            make_cell(-5.0, MatterState::Solid, 0.0),
            make_cell(10.0, MatterState::Solid, 0.0),
        ];
        let total = field_total_qe(&cells);
        assert!((total - 10.0).abs() < 1e-6, "expected 10.0, got {total}");
    }

    #[test]
    fn field_total_qe_empty_returns_zero() {
        assert_eq!(field_total_qe(&[]), 0.0);
    }

    #[test]
    fn field_total_qe_4x4_grid_correct_sum() {
        let cells: Vec<EnergyCell> = (0..16)
            .map(|i| make_cell(i as f32 * 2.0, MatterState::Solid, 0.0))
            .collect();
        let total = field_total_qe(&cells);
        let expected: f32 = (0..16).map(|i| i as f32 * 2.0).sum();
        assert!((total - expected).abs() < 1e-4, "expected {expected}, got {total}");
    }

    // ─── field_occupancy ─────────────────────────────────────────────────────

    #[test]
    fn field_occupancy_threshold_filters_correctly() {
        let cells = [
            make_cell(0.5, MatterState::Solid, 0.0),
            make_cell(1.5, MatterState::Solid, 0.0),
            make_cell(2.0, MatterState::Solid, 0.0),
            make_cell(0.1, MatterState::Solid, 0.0),
        ];
        let occ = field_occupancy(&cells, 1.0);
        assert!((occ - 0.5).abs() < 1e-6, "expected 0.5, got {occ}");
    }

    #[test]
    fn field_occupancy_all_above_returns_one() {
        let cells = [
            make_cell(5.0, MatterState::Solid, 0.0),
            make_cell(10.0, MatterState::Solid, 0.0),
        ];
        let occ = field_occupancy(&cells, 1.0);
        assert!((occ - 1.0).abs() < 1e-6, "expected 1.0, got {occ}");
    }

    #[test]
    fn field_occupancy_none_above_returns_zero() {
        let cells = [
            make_cell(0.1, MatterState::Solid, 0.0),
            make_cell(0.5, MatterState::Solid, 0.0),
        ];
        let occ = field_occupancy(&cells, 1.0);
        assert!((occ).abs() < 1e-6, "expected 0.0, got {occ}");
    }

    #[test]
    fn field_occupancy_empty_returns_zero() {
        assert_eq!(field_occupancy(&[], 1.0), 0.0);
    }

    // ─── field_matter_distribution ───────────────────────────────────────────

    #[test]
    fn field_matter_distribution_single_state_returns_one() {
        let cells = [
            make_cell(1.0, MatterState::Gas, 0.0),
            make_cell(1.0, MatterState::Gas, 0.0),
        ];
        let dist = field_matter_distribution(&cells);
        assert_eq!(dist, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn field_matter_distribution_sums_to_one() {
        let cells = [
            make_cell(1.0, MatterState::Solid, 0.0),
            make_cell(1.0, MatterState::Liquid, 0.0),
            make_cell(1.0, MatterState::Gas, 0.0),
            make_cell(1.0, MatterState::Plasma, 0.0),
        ];
        let dist = field_matter_distribution(&cells);
        let sum: f32 = dist.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "expected sum 1.0, got {sum}");
        assert!((dist[0] - 0.25).abs() < 1e-6);
        assert!((dist[1] - 0.25).abs() < 1e-6);
        assert!((dist[2] - 0.25).abs() < 1e-6);
        assert!((dist[3] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn field_matter_distribution_empty_returns_zeros() {
        assert_eq!(field_matter_distribution(&[]), [0.0; 4]);
    }

    #[test]
    fn field_matter_distribution_mixed_ratios() {
        let cells = [
            make_cell(1.0, MatterState::Solid, 0.0),
            make_cell(1.0, MatterState::Solid, 0.0),
            make_cell(1.0, MatterState::Solid, 0.0),
            make_cell(1.0, MatterState::Plasma, 0.0),
        ];
        let dist = field_matter_distribution(&cells);
        assert!((dist[0] - 0.75).abs() < 1e-6, "solid expected 0.75, got {}", dist[0]);
        assert!((dist[3] - 0.25).abs() < 1e-6, "plasma expected 0.25, got {}", dist[3]);
    }

    // ─── population_growth_rate ──────────────────────────────────────────────

    #[test]
    fn population_growth_rate_net_growth() {
        let rate = population_growth_rate(105, 100);
        assert!((rate - 0.05).abs() < 1e-6, "expected 0.05, got {rate}");
    }

    #[test]
    fn population_growth_rate_net_decline() {
        let rate = population_growth_rate(90, 100);
        assert!((rate - (-0.1)).abs() < 1e-6, "expected -0.1, got {rate}");
    }

    #[test]
    fn population_growth_rate_no_change() {
        let rate = population_growth_rate(50, 50);
        assert!((rate).abs() < 1e-6, "expected 0.0, got {rate}");
    }

    #[test]
    fn population_growth_rate_previous_zero_uses_one() {
        let rate = population_growth_rate(5, 0);
        assert!((rate - 5.0).abs() < 1e-6, "expected 5.0, got {rate}");
    }

    #[test]
    fn population_growth_rate_both_zero_returns_zero() {
        let rate = population_growth_rate(0, 0);
        assert!((rate).abs() < 1e-6, "expected 0.0, got {rate}");
    }

    // ─── frequency_diversity_index ───────────────────────────────────────────

    #[test]
    fn frequency_diversity_uniform_returns_max_entropy() {
        let bands = [10u32; 8];
        let h = frequency_diversity_index(&bands);
        let expected = (8.0f32).ln();
        assert!((h - expected).abs() < 1e-5, "expected {expected}, got {h}");
    }

    #[test]
    fn frequency_diversity_single_band_returns_zero() {
        let bands = [100, 0, 0, 0, 0, 0, 0, 0];
        let h = frequency_diversity_index(&bands);
        assert!((h).abs() < 1e-6, "expected 0.0, got {h}");
    }

    #[test]
    fn frequency_diversity_empty_returns_zero() {
        let bands = [0u32; 8];
        let h = frequency_diversity_index(&bands);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn frequency_diversity_two_equal_bands() {
        let bands = [50, 50, 0, 0, 0, 0, 0, 0];
        let h = frequency_diversity_index(&bands);
        let expected = (2.0f32).ln();
        assert!((h - expected).abs() < 1e-5, "expected {expected}, got {h}");
    }

    #[test]
    fn frequency_diversity_deterministic() {
        let bands = [10, 20, 30, 5, 15, 8, 12, 0];
        let h1 = frequency_diversity_index(&bands);
        let h2 = frequency_diversity_index(&bands);
        assert_eq!(h1, h2, "must be deterministic");
    }

    // ─── frequency_band_histogram ────────────────────────────────────────────

    #[test]
    fn frequency_band_histogram_distributes_correctly() {
        let cells = [
            make_cell(1.0, MatterState::Solid, 1.0),
            make_cell(1.0, MatterState::Solid, 3.0),
            make_cell(1.0, MatterState::Solid, 5.0),
            make_cell(1.0, MatterState::Solid, 7.0),
        ];
        let hist = frequency_band_histogram(&cells, 4);
        // max_freq = 7.0, band_width = 1.75
        // band(1.0) = floor(1.0/1.75) = 0
        // band(3.0) = floor(3.0/1.75) = 1
        // band(5.0) = floor(5.0/1.75) = 2
        // band(7.0) = min(floor(7.0/1.75),3) = 3
        assert_eq!(hist, vec![1, 1, 1, 1]);
    }

    #[test]
    fn frequency_band_histogram_zero_freq_ignored() {
        let cells = [
            make_cell(1.0, MatterState::Solid, 0.0),
            make_cell(1.0, MatterState::Solid, 4.0),
        ];
        let hist = frequency_band_histogram(&cells, 2);
        let total: u32 = hist.iter().sum();
        assert_eq!(total, 1, "only one cell has freq > 0");
    }

    #[test]
    fn frequency_band_histogram_empty_cells() {
        let hist = frequency_band_histogram(&[], 4);
        assert_eq!(hist, vec![0, 0, 0, 0]);
    }

    #[test]
    fn frequency_band_histogram_all_same_freq() {
        let cells = [
            make_cell(1.0, MatterState::Solid, 5.0),
            make_cell(1.0, MatterState::Solid, 5.0),
            make_cell(1.0, MatterState::Solid, 5.0),
        ];
        let hist = frequency_band_histogram(&cells, 4);
        // max = 5.0, band_width = 1.25
        // band(5.0) = min(floor(5.0/1.25), 3) = min(4, 3) = 3
        assert_eq!(hist[3], 3);
    }
}
