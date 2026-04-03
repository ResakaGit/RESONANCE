/// Planetary formation equations: gravity, fusion, angular momentum.
///
/// These derive from the 8 axioms + 4 fundamental constants:
/// - Gravity: Axiom 7 (distance attenuation) applied as attraction toward mass.
/// - Fusion: Axiom 4 (dissipation at plasma state → energy release).
/// - Angular momentum: Axiom 8 (oscillatory conservation during radial collapse).
///
/// No hardcoded constants. All rates from derived_thresholds.rs.
use super::derived_thresholds as dt;

/// Gravitational constant: `DISSIPATION_SOLID²` — gravity is the weakest interaction.
/// At 0.005² = 0.000025 per tick, gravitational collapse is very gradual.
#[inline]
pub fn gravitational_constant() -> f32 {
    dt::DISSIPATION_SOLID * dt::DISSIPATION_SOLID
}

/// Gravitational transfer: high-mass cells attract energy from neighbors.
/// `transfer = neighbor_qe × (source_qe × G) / n_neighbors`
/// Capped at 10% of neighbor's energy per tick to prevent instant collapse.
#[inline]
pub fn gravitational_transfer(source_qe: f32, neighbor_qe: f32, n_neighbors: u32) -> f32 {
    if source_qe <= neighbor_qe || neighbor_qe <= 0.0 {
        return 0.0;
    }
    let g = gravitational_constant();
    let pull = source_qe * g / n_neighbors.max(1) as f32;
    (neighbor_qe * pull).min(neighbor_qe * 0.1)
}

/// Fusion energy release rate when cell exceeds plasma density threshold.
/// `rate = DISSIPATION_PLASMA` — plasma-state energy converts mass to radiation.
/// Returns bonus qe to add to the cell (energy creation from mass-energy equivalence).
#[inline]
pub fn fusion_release(cell_qe: f32) -> f32 {
    let threshold = dt::plasma_density_threshold();
    if cell_qe < threshold {
        return 0.0;
    }
    let excess = cell_qe - threshold;
    excess * dt::DISSIPATION_PLASMA
}

/// Angular momentum conservation ratio during gravitational collapse.
/// When energy moves radially inward, this fraction is deflected tangentially.
/// `ratio = DISSIPATION_LIQUID / DISSIPATION_GAS` — liquid/gas viscosity boundary.
/// Higher viscosity (liquid) = more angular momentum preserved.
#[inline]
pub fn angular_conservation_ratio() -> f32 {
    dt::DISSIPATION_LIQUID / dt::DISSIPATION_GAS
}

/// Tangential deflection: fraction of radial transfer that becomes perpendicular flow.
#[inline]
pub fn tangential_deflection(radial_transfer: f32) -> f32 {
    radial_transfer * angular_conservation_ratio()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_constant_is_small() {
        let g = gravitational_constant();
        assert!(g > 0.0 && g < 0.001, "g={g}");
    }

    #[test]
    fn gravity_only_attracts_toward_higher_mass() {
        assert_eq!(gravitational_transfer(10.0, 50.0, 4), 0.0);
        assert!(gravitational_transfer(50.0, 10.0, 4) > 0.0);
    }

    #[test]
    fn gravity_transfer_capped_at_ten_percent() {
        let t = gravitational_transfer(10000.0, 100.0, 4);
        assert!(t <= 10.0, "transfer must be ≤ 10% of neighbor: {t}");
    }

    #[test]
    fn fusion_below_threshold_is_zero() {
        assert_eq!(fusion_release(1.0), 0.0);
    }

    #[test]
    fn fusion_above_threshold_is_positive() {
        let threshold = dt::plasma_density_threshold();
        assert!(fusion_release(threshold + 100.0) > 0.0);
    }

    #[test]
    fn angular_ratio_between_zero_and_one() {
        let r = angular_conservation_ratio();
        assert!(r > 0.0 && r < 1.0, "ratio={r}");
    }

    #[test]
    fn tangential_deflection_less_than_input() {
        let d = tangential_deflection(10.0);
        assert!(d < 10.0 && d > 0.0);
    }
}
