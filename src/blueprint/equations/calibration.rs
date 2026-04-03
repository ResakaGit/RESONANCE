//! Pure calibration functions — R4 Empirical Calibration sprint.
//! All functions are stateless; they only validate ranges or compute golden snapshots.

use crate::blueprint::constants::calibration::{
    DECAY_RATE_MAX, DECAY_RATE_MIN, GROWTH_RATE_MAX, GROWTH_RATE_MIN, INTAKE_RATE_MAX,
    INTAKE_RATE_MIN, INTAKE_RATE_NOMINAL, MAINTENANCE_RATE_MAX, MAINTENANCE_RATE_MIN,
    MAINTENANCE_RATE_NOMINAL,
};

/// Returns true if `rate` is within the empirically plausible intake-rate range.
pub fn is_intake_rate_plausible(rate: f32) -> bool {
    (INTAKE_RATE_MIN..=INTAKE_RATE_MAX).contains(&rate)
}

/// Returns true if `rate` is within the empirically plausible maintenance-rate range.
pub fn is_maintenance_rate_plausible(rate: f32) -> bool {
    (MAINTENANCE_RATE_MIN..=MAINTENANCE_RATE_MAX).contains(&rate)
}

/// Returns true if `rate` is within the empirically plausible growth-rate range.
pub fn is_growth_rate_plausible(rate: f32) -> bool {
    (GROWTH_RATE_MIN..=GROWTH_RATE_MAX).contains(&rate)
}

/// Returns true if `rate` is within the empirically plausible decay-rate range.
pub fn is_decay_rate_plausible(rate: f32) -> bool {
    (DECAY_RATE_MIN..=DECAY_RATE_MAX).contains(&rate)
}

/// Golden snapshot: energy after `n_ticks` with nominal intake and maintenance rates.
/// e_{t+1} = e_t + INTAKE_RATE_NOMINAL * intake_available - MAINTENANCE_RATE_NOMINAL * e_t
pub fn golden_energy_after_ticks(initial_energy: f32, intake_available: f32, n_ticks: u32) -> f32 {
    let mut energy = initial_energy;
    for _ in 0..n_ticks {
        let intake = INTAKE_RATE_NOMINAL * intake_available;
        let cost = MAINTENANCE_RATE_NOMINAL * energy;
        energy = (energy + intake - cost).max(0.0);
    }
    energy
}

/// Relative error between a calibrated value and its reference.
/// Returns 0.0 when `reference` is zero to avoid division by zero.
pub fn calibration_error(calibrated: f32, reference: f32) -> f32 {
    if reference == 0.0 {
        return 0.0;
    }
    ((calibrated - reference) / reference).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intake_nominal_is_in_range() {
        assert!(is_intake_rate_plausible(INTAKE_RATE_NOMINAL));
    }

    #[test]
    fn maintenance_nominal_is_in_range() {
        assert!(is_maintenance_rate_plausible(MAINTENANCE_RATE_NOMINAL));
    }

    #[test]
    fn calibration_error_zero_when_exact() {
        assert_eq!(calibration_error(42.0, 42.0), 0.0);
    }

    #[test]
    fn calibration_error_zero_when_reference_is_zero() {
        assert_eq!(calibration_error(1.0, 0.0), 0.0);
    }
}
