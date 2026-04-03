//! R4 Empirical Calibration — golden snapshot integration tests.
//! Pure-function tests; no Bevy App required.

use resonance::blueprint::{constants::calibration as cal, equations::calibration as eq_cal};

#[test]
fn intake_rate_nominal_is_plausible() {
    assert!(
        eq_cal::is_intake_rate_plausible(cal::INTAKE_RATE_NOMINAL),
        "INTAKE_RATE_NOMINAL {} is outside plausible range [{}, {}]",
        cal::INTAKE_RATE_NOMINAL,
        cal::INTAKE_RATE_MIN,
        cal::INTAKE_RATE_MAX,
    );
}

#[test]
fn maintenance_rate_nominal_is_plausible() {
    assert!(
        eq_cal::is_maintenance_rate_plausible(cal::MAINTENANCE_RATE_NOMINAL),
        "MAINTENANCE_RATE_NOMINAL {} is outside plausible range [{}, {}]",
        cal::MAINTENANCE_RATE_NOMINAL,
        cal::MAINTENANCE_RATE_MIN,
        cal::MAINTENANCE_RATE_MAX,
    );
}

#[test]
fn growth_rate_nominal_is_plausible() {
    assert!(
        eq_cal::is_growth_rate_plausible(cal::GROWTH_RATE_NOMINAL),
        "GROWTH_RATE_NOMINAL {} is outside plausible range [{}, {}]",
        cal::GROWTH_RATE_NOMINAL,
        cal::GROWTH_RATE_MIN,
        cal::GROWTH_RATE_MAX,
    );
}

#[test]
fn decay_rate_nominal_is_plausible() {
    assert!(
        eq_cal::is_decay_rate_plausible(cal::DECAY_RATE_NOMINAL),
        "DECAY_RATE_NOMINAL {} is outside plausible range [{}, {}]",
        cal::DECAY_RATE_NOMINAL,
        cal::DECAY_RATE_MIN,
        cal::DECAY_RATE_MAX,
    );
}

#[test]
fn golden_energy_100_ticks_within_tolerance() {
    // initial=500 qe, intake_available=100 qe, 100 ticks
    // nominal: intake_rate=0.08, maintenance_rate=0.01
    // Equilibrium = (0.08 * 100) / 0.01 = 800 qe.
    // Starting below equilibrium the system converges upward;
    // after 100 ticks from 500 qe the result is ~690 qe (within 600–780 tolerance band).
    let result = eq_cal::golden_energy_after_ticks(500.0, 100.0, 100);
    assert!(
        result >= 600.0 && result <= 780.0,
        "golden_energy_after_ticks(500, 100, 100) = {result:.2}, expected 600–780 qe"
    );
}

#[test]
fn calibration_error_zero_when_exact() {
    let err = eq_cal::calibration_error(42.0, 42.0);
    assert_eq!(
        err, 0.0,
        "identical values must yield zero calibration error"
    );
}
