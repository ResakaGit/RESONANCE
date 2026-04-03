//! R6 / SF-1: Simulation Observability — health dashboard, metrics snapshot, alert flags.
//! System: `simulation_health_system` in `Phase::MetabolicLayer`.
//! System: `metrics_snapshot_system` in `Phase::MetabolicLayer` (after health).

use bevy::prelude::*;

// SF-1D constants — canonical source: blueprint/constants/simulation_foundations.rs
// Inlined here because constants/mod.rs wiring is outside file ownership scope.
const METRICS_FIELD_OCCUPANCY_THRESHOLD: f32 = 1.0;
const METRICS_FREQUENCY_BANDS: usize = 8;
use crate::blueprint::equations::observability as obs_eq;
use crate::events::DeathEvent;
use crate::layers::BaseEnergy;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::field_grid::EnergyFieldGrid;

// ─── Resources ────────────────────────────────────────────────────────────────

/// Dashboard resource: snapshot of simulation health per tick.
#[derive(Resource, Debug, Default)]
pub struct SimulationHealthDashboard {
    pub conservation_error: f32,
    pub drift_rate: f32,
    pub saturation_index: f32,
    pub tick_count: u64,
}

/// Alert flags resource: set true when threshold exceeded.
#[derive(Resource, Debug, Default)]
pub struct SimulationAlerts {
    pub conservation_violated: bool,
    pub critical_drift: bool,
    pub high_saturation: bool,
    pub alert_tick: u64,
}

/// SF-1B: Global energy and field metrics (split 1/2).
#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct SimulationMetricsSnapshot {
    pub tick: u64,
    pub total_qe: f32,
    pub field_occupancy: f32,
    pub matter_distribution: [f32; 4],
}

/// SF-1B: Ecology and diversity metrics (split 2/2).
#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct SimulationEcologySnapshot {
    pub entity_count: u32,
    pub growth_rate: f32,
    pub frequency_diversity: f32,
    pub deaths_this_tick: u32,
}

// ─── Thresholds ───────────────────────────────────────────────────────────────

/// Thresholds for alert trigger levels.
pub struct ObservabilityThresholds;

impl ObservabilityThresholds {
    /// 5% fractional drift per tick is critical.
    pub const CRITICAL_DRIFT: f32 = 0.05;
    /// 90% saturation is high.
    pub const HIGH_SATURATION: f32 = 0.90;
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Collects total BaseEnergy across all entities and updates the health dashboard.
/// Uses a `Local<f32>` to track the previous tick's total without adding a field
/// to the dashboard resource (DOD: max 4 fields).
/// Raises alert flags when critical thresholds are exceeded.
pub fn simulation_health_system(
    mut dashboard: ResMut<SimulationHealthDashboard>,
    mut alerts: ResMut<SimulationAlerts>,
    mut prev_total: Local<f32>,
    query: Query<&BaseEnergy>,
) {
    let total_qe: f32 = query.iter().map(|e| e.qe()).sum();

    const QE_GLOBAL_CEILING: f32 = 10_000.0 * 1_000.0;
    let sat = obs_eq::saturation_index(total_qe, QE_GLOBAL_CEILING);
    let drift = obs_eq::drift_rate(*prev_total, total_qe);
    let cons_error = dashboard.conservation_error;

    let violated = obs_eq::is_conservation_violation(cons_error);
    let critical = obs_eq::is_critical_drift(drift, ObservabilityThresholds::CRITICAL_DRIFT);
    let high_sat = sat >= ObservabilityThresholds::HIGH_SATURATION;

    let new_tick = dashboard.tick_count + 1;

    if dashboard.drift_rate != drift
        || dashboard.saturation_index != sat
        || dashboard.tick_count != new_tick
    {
        dashboard.drift_rate = drift;
        dashboard.saturation_index = sat;
        dashboard.tick_count = new_tick;
    }

    if alerts.conservation_violated != violated
        || alerts.critical_drift != critical
        || alerts.high_saturation != high_sat
    {
        alerts.conservation_violated = violated;
        alerts.critical_drift = critical;
        alerts.high_saturation = high_sat;
        alerts.alert_tick = new_tick;
    }

    *prev_total = total_qe;
}

/// SF-1C: Collects field and ecology metrics into atomic snapshot resources.
/// Reads `EnergyFieldGrid` (optional — graceful when absent) and entity queries.
/// Runs after `simulation_health_system` in `Phase::MetabolicLayer`.
pub fn metrics_snapshot_system(
    field: Option<Res<EnergyFieldGrid>>,
    clock: Res<SimulationClock>,
    query_alive: Query<Entity, With<BaseEnergy>>,
    mut snapshot: ResMut<SimulationMetricsSnapshot>,
    mut ecology: ResMut<SimulationEcologySnapshot>,
    mut prev_entity_count: Local<u32>,
    mut death_events: EventReader<DeathEvent>,
) {
    let tick = clock.tick_id;

    // Idempotent guard: skip if already processed this tick.
    if snapshot.tick == tick && tick > 0 {
        return;
    }

    // ─── Field metrics ───────────────────────────────────────────────────
    let (total_qe, occupancy, matter_dist, freq_diversity) = if let Some(ref grid) = field {
        let cells: Vec<_> = grid.iter_cells().collect();
        let cell_refs: &[&_] = &cells;

        // We need &[EnergyCell] but iter_cells gives &EnergyCell references.
        // Collect cell slice from grid for pure equations.
        let total = grid.total_qe();
        let occ = field_occupancy_from_refs(cell_refs, METRICS_FIELD_OCCUPANCY_THRESHOLD);
        let dist = matter_distribution_from_refs(cell_refs);
        let histogram = frequency_band_histogram_from_refs(cell_refs, METRICS_FREQUENCY_BANDS);
        let diversity = obs_eq::frequency_diversity_index(&histogram);
        (total, occ, dist, diversity)
    } else {
        (0.0, 0.0, [0.0; 4], 0.0)
    };

    // ─── Ecology metrics ─────────────────────────────────────────────────
    let entity_count = query_alive.iter().count() as u32;
    let deaths_this_tick = death_events.read().count() as u32;
    let growth_rate = obs_eq::population_growth_rate(entity_count, *prev_entity_count);

    // ─── Write snapshots (guard change detection) ────────────────────────
    if snapshot.tick != tick
        || snapshot.total_qe != total_qe
        || snapshot.field_occupancy != occupancy
        || snapshot.matter_distribution != matter_dist
    {
        snapshot.tick = tick;
        snapshot.total_qe = total_qe;
        snapshot.field_occupancy = occupancy;
        snapshot.matter_distribution = matter_dist;
    }

    if ecology.entity_count != entity_count
        || ecology.growth_rate != growth_rate
        || ecology.frequency_diversity != freq_diversity
        || ecology.deaths_this_tick != deaths_this_tick
    {
        ecology.entity_count = entity_count;
        ecology.growth_rate = growth_rate;
        ecology.frequency_diversity = freq_diversity;
        ecology.deaths_this_tick = deaths_this_tick;
    }

    *prev_entity_count = entity_count;
}

// ─── Helper adapters (ref-slice → equation bridge) ───────────────────────────

/// Occupancy computed from iterator of cell references (avoids clone).
fn field_occupancy_from_refs(cells: &[&crate::worldgen::EnergyCell], threshold: f32) -> f32 {
    if cells.is_empty() {
        return 0.0;
    }
    let active = cells
        .iter()
        .filter(|c| c.accumulated_qe > threshold)
        .count();
    active as f32 / cells.len() as f32
}

/// Matter distribution from cell references.
fn matter_distribution_from_refs(cells: &[&crate::worldgen::EnergyCell]) -> [f32; 4] {
    use crate::layers::MatterState;
    if cells.is_empty() {
        return [0.0; 4];
    }
    let mut counts = [0u32; 4];
    for cell in cells {
        let idx = match cell.matter_state {
            MatterState::Solid => 0,
            MatterState::Liquid => 1,
            MatterState::Gas => 2,
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

/// Frequency band histogram from cell references.
fn frequency_band_histogram_from_refs(
    cells: &[&crate::worldgen::EnergyCell],
    num_bands: usize,
) -> Vec<u32> {
    let mut counts = vec![0u32; num_bands.max(1)];
    if cells.is_empty() || num_bands == 0 {
        return counts;
    }
    let max_freq = cells
        .iter()
        .map(|c| c.dominant_frequency_hz)
        .fold(0.0f32, f32::max);
    if max_freq <= 0.0 {
        return counts;
    }
    let band_width = max_freq / num_bands as f32;
    for cell in cells {
        if cell.dominant_frequency_hz <= 0.0 {
            continue;
        }
        let band = ((cell.dominant_frequency_hz / band_width) as usize).min(num_bands - 1);
        counts[band] += 1;
    }
    counts
}

// ─── Export ───────────────────────────────────────────────────────────────────

/// Exports a dashboard snapshot as a CSV row.
/// Format: `"tick,conservation_error,drift_rate,saturation_index"`.
/// Returns String — export path only, not on hot path.
pub fn export_dashboard_csv_row(dashboard: &SimulationHealthDashboard) -> String {
    format!(
        "{},{},{},{}",
        dashboard.tick_count,
        dashboard.conservation_error,
        dashboard.drift_rate,
        dashboard.saturation_index,
    )
}

// ─── SF-4: Metrics Export ─────────────────────────────────────────────────────

/// CSV or NDJSON output format for metrics export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsFormat {
    Csv,
    Json,
}

/// SF-4A: Export configuration. Inserted only when `RESONANCE_METRICS=1`.
#[derive(Resource, Debug, Clone)]
pub struct MetricsExportConfig {
    pub output_path: String,
    pub batch_size: u32,
    pub format: MetricsFormat,
    pub enabled: bool,
}

impl Default for MetricsExportConfig {
    fn default() -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            output_path: format!("/tmp/resonance_metrics_{timestamp}.csv"),
            batch_size: crate::blueprint::constants::METRICS_EXPORT_BATCH_SIZE,
            format: MetricsFormat::Csv,
            enabled: true,
        }
    }
}

/// SF-4B: Batches snapshots and flushes to disk every `batch_size` ticks.
/// Phase::MetabolicLayer, after `metrics_snapshot_system`.
/// No-op unless `RESONANCE_METRICS=1` is set in the environment.
pub fn metrics_batch_system(
    snapshot: Res<SimulationMetricsSnapshot>,
    ecology: Res<SimulationEcologySnapshot>,
    dashboard: Res<SimulationHealthDashboard>,
    config: Option<Res<MetricsExportConfig>>,
    mut buffer: Local<Vec<String>>,
    mut header_written: Local<bool>,
) {
    let Some(cfg) = config else {
        return;
    };
    if !cfg.enabled {
        return;
    }

    let row = match cfg.format {
        MetricsFormat::Csv => format_csv_row(&snapshot, &ecology, &dashboard),
        MetricsFormat::Json => format_json_row(&snapshot, &ecology, &dashboard),
    };
    buffer.push(row);

    if buffer.len() < cfg.batch_size as usize {
        return;
    }

    use std::io::Write;
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&cfg.output_path)
    else {
        return;
    };

    if !*header_written && cfg.format == MetricsFormat::Csv {
        let _ = writeln!(
            file,
            "tick,total_qe,field_occupancy,solid,liquid,gas,plasma,entities,deaths,growth_rate,diversity,conservation_error,drift_rate,saturation"
        );
        *header_written = true;
    }
    for line in buffer.drain(..) {
        let _ = writeln!(file, "{line}");
    }
}

fn format_csv_row(
    s: &SimulationMetricsSnapshot,
    e: &SimulationEcologySnapshot,
    d: &SimulationHealthDashboard,
) -> String {
    let [solid, liquid, gas, plasma] = s.matter_distribution;
    format!(
        "{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{:.6},{:.6},{:.6},{:.6},{:.6}",
        s.tick,
        s.total_qe,
        s.field_occupancy,
        solid,
        liquid,
        gas,
        plasma,
        e.entity_count,
        e.deaths_this_tick,
        e.growth_rate,
        e.frequency_diversity,
        d.conservation_error,
        d.drift_rate,
        d.saturation_index,
    )
}

fn format_json_row(
    s: &SimulationMetricsSnapshot,
    e: &SimulationEcologySnapshot,
    d: &SimulationHealthDashboard,
) -> String {
    let [solid, liquid, gas, plasma] = s.matter_distribution;
    format!(
        r#"{{"tick":{},"total_qe":{:.6},"field_occupancy":{:.6},"matter":[{:.6},{:.6},{:.6},{:.6}],"entities":{},"deaths":{},"growth_rate":{:.6},"diversity":{:.6},"conservation_error":{:.6},"drift_rate":{:.6},"saturation":{:.6}}}"#,
        s.tick,
        s.total_qe,
        s.field_occupancy,
        solid,
        liquid,
        gas,
        plasma,
        e.entity_count,
        e.deaths_this_tick,
        e.growth_rate,
        e.frequency_diversity,
        d.conservation_error,
        d.drift_rate,
        d.saturation_index,
    )
}

#[cfg(test)]
mod tests {
    use crate::blueprint::constants::units::CONSERVATION_ERROR_TOLERANCE;

    #[test]
    fn conservation_error_tolerance_accessible() {
        assert!(CONSERVATION_ERROR_TOLERANCE > 0.0);
    }
}
