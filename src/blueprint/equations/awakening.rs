/// Awakening potential: can an inert entity become alive?
///
/// Uses the same axiomatic criteria as abiogenesis:
/// `potential = (coherence_gain - dissipation) / (coherence_gain + qe)`
/// But applied to existing entities, not empty cells.
///
/// Axiom 1: everything is energy — state transitions are continuous.
/// Axiom 6: emergence at scale — life is not a binary flag, it's a threshold.
#[inline]
pub fn awakening_potential(qe: f32, coherence_gain: f32, dissipation_rate: f32) -> f32 {
    let diss = qe * dissipation_rate.max(0.0);
    let net = coherence_gain - diss;
    let denom = coherence_gain + qe;
    if denom <= 0.0 {
        return 0.0;
    }
    (net / denom).clamp(0.0, 1.0)
}

/// Minimum awakening potential to gain behavioral capabilities.
pub const AWAKENING_THRESHOLD: f32 = 0.3;

/// Minimum qe for an entity to awaken (below this, not enough energy to sustain behavior).
pub const AWAKENING_MIN_QE: f32 = 20.0;

/// How many entities can awaken per tick (budget to prevent frame spikes).
pub const AWAKENING_BUDGET_PER_TICK: usize = 4;

/// Ticks between awakening scans (not every tick — expensive query).
pub const AWAKENING_SCAN_INTERVAL: u64 = 8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_coherence_returns_zero() {
        assert_eq!(awakening_potential(100.0, 0.0, 0.01), 0.0);
    }

    #[test]
    fn high_coherence_low_dissipation_high_potential() {
        let p = awakening_potential(50.0, 200.0, 0.005);
        assert!(p > AWAKENING_THRESHOLD, "p={p}");
    }

    #[test]
    fn dissipation_exceeds_coherence_returns_zero() {
        let p = awakening_potential(100.0, 10.0, 0.5);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn potential_clamped_to_unit() {
        let p = awakening_potential(1.0, 1000.0, 0.0);
        assert!(p <= 1.0);
    }

    #[test]
    fn zero_qe_zero_coherence_returns_zero() {
        let p = awakening_potential(0.0, 0.0, 0.01);
        assert_eq!(p, 0.0);
    }
}
