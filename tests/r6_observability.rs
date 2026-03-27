//! R6 — Observability: unit tests for pure-math functions and CSV export.
//! Tests are deterministic and require no Bevy runtime.

use resonance::blueprint::equations::observability::{
    drift_rate, is_conservation_violation, is_critical_drift, saturation_index, system_cost_ratio,
};
use resonance::simulation::observability::{export_dashboard_csv_row, SimulationHealthDashboard};

// ─── drift_rate ───────────────────────────────────────────────────────────────

#[test]
fn drift_rate_no_change_returns_zero() {
    assert_eq!(drift_rate(100.0, 100.0), 0.0);
}

#[test]
fn drift_rate_increase_returns_positive() {
    let d = drift_rate(100.0, 110.0);
    assert!(d > 0.0, "expected positive drift, got {d}");
    assert!((d - 0.1).abs() < 1e-5, "expected 0.1, got {d}");
}

#[test]
fn drift_rate_zero_prev_returns_zero() {
    assert_eq!(drift_rate(0.0, 500.0), 0.0);
}

// ─── saturation_index ────────────────────────────────────────────────────────

#[test]
fn saturation_index_full_returns_one() {
    assert_eq!(saturation_index(1000.0, 1000.0), 1.0);
}

#[test]
fn saturation_index_empty_returns_zero() {
    assert_eq!(saturation_index(0.0, 1000.0), 0.0);
}

#[test]
fn saturation_index_half_returns_half() {
    let s = saturation_index(500.0, 1000.0);
    assert!((s - 0.5).abs() < 1e-6, "expected 0.5, got {s}");
}

// ─── system_cost_ratio ───────────────────────────────────────────────────────

#[test]
fn system_cost_ratio_zero_budget_returns_zero() {
    assert_eq!(system_cost_ratio(10, 0), 0.0);
}

#[test]
fn system_cost_ratio_full_returns_one() {
    assert_eq!(system_cost_ratio(64, 64), 1.0);
}

// ─── is_critical_drift ───────────────────────────────────────────────────────

#[test]
fn is_critical_drift_above_threshold_returns_true() {
    assert!(is_critical_drift(0.10, 0.05));
}

#[test]
fn is_critical_drift_at_threshold_returns_false() {
    assert!(!is_critical_drift(0.05, 0.05));
}

#[test]
fn is_critical_drift_negative_above_threshold_returns_true() {
    // Absolute value: -0.1 exceeds threshold 0.05
    assert!(is_critical_drift(-0.10, 0.05));
}

// ─── is_conservation_violation ───────────────────────────────────────────────

#[test]
fn is_conservation_violation_above_tolerance_returns_true() {
    // CONSERVATION_ERROR_TOLERANCE = 1e-3
    assert!(is_conservation_violation(0.002));
}

#[test]
fn is_conservation_violation_below_tolerance_returns_false() {
    assert!(!is_conservation_violation(0.0005));
}

#[test]
fn is_conservation_violation_zero_returns_false() {
    assert!(!is_conservation_violation(0.0));
}

// ─── export_dashboard_csv_row ────────────────────────────────────────────────

#[test]
fn export_csv_row_format_contains_tick() {
    let dashboard = SimulationHealthDashboard {
        tick_count: 42,
        conservation_error: 0.001,
        drift_rate: 0.002,
        saturation_index: 0.75,
    };
    let row = export_dashboard_csv_row(&dashboard);
    assert!(row.starts_with("42,"), "expected row to start with tick '42,', got: {row}");
}

#[test]
fn export_csv_row_has_four_fields() {
    let dashboard = SimulationHealthDashboard {
        tick_count: 1,
        conservation_error: 0.0,
        drift_rate: 0.0,
        saturation_index: 0.0,
    };
    let row = export_dashboard_csv_row(&dashboard);
    let fields: Vec<&str> = row.split(',').collect();
    assert_eq!(fields.len(), 4, "expected 4 CSV fields, got {}: '{row}'", fields.len());
}

#[test]
fn export_csv_row_default_dashboard_starts_with_zero() {
    let dashboard = SimulationHealthDashboard::default();
    let row = export_dashboard_csv_row(&dashboard);
    assert!(row.starts_with("0,"), "default tick should be 0, got: {row}");
}
