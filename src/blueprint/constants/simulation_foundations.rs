//! SF-1: Simulation Foundations — observability metrics constants.

/// Minimum accumulated_qe for a cell to count as "active" in occupancy.
pub const METRICS_FIELD_OCCUPANCY_THRESHOLD: f32 = 1.0;

/// Number of frequency bands for Shannon diversity index.
pub const METRICS_FREQUENCY_BANDS: usize = 8;

/// Snapshot collection interval in ticks (1 = every tick).
pub const METRICS_SNAPSHOT_INTERVAL: u32 = 1;

// ─── SF-4: Metrics Export ─────────────────────────────────────────────────────

/// Default flush interval: every 60 ticks = 3 seconds at 20Hz.
pub const METRICS_EXPORT_BATCH_SIZE: u32 = 60;
/// Default output path prefix (timestamp + ".csv" appended at runtime).
pub const METRICS_EXPORT_DEFAULT_PATH: &str = "/tmp/resonance_metrics";
