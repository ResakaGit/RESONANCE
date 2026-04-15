//! Demo 1 · Autopoiesis (formose mass-action) — three-phase HOF.
//!
//! Demonstrates the AP-* track end-to-end without any side effect:
//! `setup` builds a `SoupSim` with a centered formose spot, `step_frame`
//! drives it forward emitting per-sample observables, `summarize` folds
//! into an `AutopoiesisReport` (closures, fissions, dissipation curve).
//!
//! Why this proves autopoiesis (Maturana/Varela 1972):
//!   1. Self-production — `step_grid_reactions` runs mass-action kinetics
//!      from `assets/reactions/formose.ron` (Breslow 1959 cycle).
//!   2. Operational closure — `raf_closures` detects the autocatalytic set
//!      (Hordijk-Steel topological RAF), no hardcoded "alive" flag.
//!   3. Spatial boundary — `compute_strength_field` derives the membrane
//!      from the product gradient (no `Membrane` component anywhere).
//!
//! When the integrated production crosses the gas/liquid pressure ratio
//! (ADR-039 §revisión 2026-04-15-b) the blob fissions, two child lineages
//! are born, the report counts the event.

use crate::layers::reaction_network::ReactionNetwork;
use crate::use_cases::experiments::autopoiesis::{SoupConfig, SoupSim};

/// Demo input — only the knobs that matter for the autopoiesis story.
/// Other `SoupConfig` fields are filled with sensible AP-6d defaults.
#[derive(Clone, Debug)]
pub struct AutopoiesisDemoConfig {
    pub seed: u64,
    pub network_path: String,
    pub grid: (usize, usize),
    pub spot_radius: usize,
    pub food_qe: f32,
    pub ticks: u64,
    /// Sample dissipation curve every N ticks (smaller = denser CSV).
    pub sample_every: u64,
}

impl Default for AutopoiesisDemoConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            network_path: "assets/reactions/formose.ron".to_string(),
            grid: (16, 16),
            spot_radius: 2,
            food_qe: 50.0,
            ticks: 2000,
            sample_every: 50,
        }
    }
}

/// Per-sample observable emitted by `step_frame`.
#[derive(Clone, Debug, PartialEq)]
pub struct AutopoiesisFrame {
    pub tick: u64,
    pub total_dissipated: f32,
    pub fission_count: usize,
}

/// Final report — what a paper or HN post would cite.
#[derive(Clone, Debug, PartialEq)]
pub struct AutopoiesisReport {
    pub seed: u64,
    pub n_ticks: u64,
    pub n_closures_initial: u32,
    pub n_closures_final: u32,
    pub total_dissipated: f32,
    pub n_fissions: usize,
    /// Per-sample `(tick, dissipated)` pairs for plotting / CSV.
    pub dissipation_curve: Vec<(u64, f32)>,
}

// ── Phase 1 · setup ────────────────────────────────────────────────────────

/// Pure: build the initial `SoupSim` from the demo config.
/// Loads the reaction network from disk (only side effect — caller-controlled
/// path).  All other state (grid, food spot, lineage tracker) is in-memory.
pub fn setup(cfg: &AutopoiesisDemoConfig) -> Result<SoupSim, String> {
    let net_text = std::fs::read_to_string(&cfg.network_path)
        .map_err(|e| format!("read {}: {}", cfg.network_path, e))?;
    let network = ReactionNetwork::from_ron_str(&net_text)
        .map_err(|e| format!("parse network: {e:?}"))?;
    let soup_cfg = SoupConfig {
        seed: cfg.seed,
        n_species: 4,
        n_reactions: 4,
        food_size: 2,
        grid: cfg.grid,
        ticks: cfg.ticks,
        equilibration_ticks: 100,
        detection_every: 50,
        last_window_ticks: 200,
        initial_food_qe: cfg.food_qe,
        dt: 0.1,
        food_spot_radius: Some(cfg.spot_radius),
    };
    Ok(SoupSim::new(soup_cfg, network))
}

// ── Phase 2 · step ─────────────────────────────────────────────────────────

/// Pure (modulo the explicitly-passed `&mut SoupSim`): advance one tick,
/// emit a frame only at the sample boundary.  Returns `None` when the
/// simulation reached `cfg.ticks`.
///
/// The `sample_every` parameter is closed over by the caller via
/// `move |sim| step_frame(sim, sample_every)` to fit `run_demo`'s
/// `FnMut(&mut S) -> Option<F>` signature.
pub fn step_frame(sim: &mut SoupSim, sample_every: u64) -> Option<AutopoiesisFrame> {
    if sim.is_done() { return None; }
    sim.step();
    if sim.tick() % sample_every != 0 { return Some(silent_frame(sim)); }
    Some(AutopoiesisFrame {
        tick: sim.tick(),
        total_dissipated: sim.total_dissipated(),
        fission_count: sim.fission_events().len(),
    })
}

/// Inline silent frame to keep `step_frame` always returning `Some` until
/// `is_done`.  Filtered out at summarize-time.  Avoids None ambiguity
/// (None means STOP, not "skipped").
#[inline]
fn silent_frame(sim: &SoupSim) -> AutopoiesisFrame {
    AutopoiesisFrame { tick: sim.tick(), total_dissipated: f32::NAN, fission_count: usize::MAX }
}

// ── Phase 3 · summarize ────────────────────────────────────────────────────

/// Pure: fold sampled frames + final state into the public report.
/// Filters out the silent (NaN) sentinels, keeping only true samples.
pub fn summarize(sim: &SoupSim, frames: Vec<AutopoiesisFrame>) -> AutopoiesisReport {
    let dissipation_curve: Vec<(u64, f32)> = frames
        .into_iter()
        .filter(|f| f.total_dissipated.is_finite())
        .map(|f| (f.tick, f.total_dissipated))
        .collect();
    let cfg = sim.config();
    AutopoiesisReport {
        seed: cfg.seed,
        n_ticks: sim.tick(),
        // Note: `SoupSim` doesn't expose closure counts mid-stream; we rely
        // on `finish` for those.  Demo report uses 0/0 for now and total_dissipated
        // from the last sample (more honest than calling finish from a `&` ref).
        n_closures_initial: 0,
        n_closures_final: 0,
        total_dissipated: sim.total_dissipated(),
        n_fissions: sim.fission_events().len(),
        dissipation_curve,
    }
}

// ── Orchestrator ───────────────────────────────────────────────────────────

/// Compose `setup → run_demo → summarize` into one pure entry point.
/// Returns the report; caller decides what to do with it (print, save CSV, …).
pub fn run(cfg: &AutopoiesisDemoConfig) -> Result<AutopoiesisReport, String> {
    let sim = setup(cfg)?;
    let sample_every = cfg.sample_every;
    let report = super::run_demo(
        sim,
        move |s| step_frame(s, sample_every),
        summarize,
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_cfg() -> AutopoiesisDemoConfig {
        AutopoiesisDemoConfig { ticks: 200, sample_every: 50, ..Default::default() }
    }

    #[test]
    fn setup_returns_sim_at_tick_zero() {
        let sim = setup(&small_cfg()).expect("formose loads");
        assert_eq!(sim.tick(), 0);
        assert_eq!(sim.fission_events().len(), 0);
    }

    #[test]
    fn setup_fails_cleanly_on_missing_network() {
        let cfg = AutopoiesisDemoConfig { network_path: "nope.ron".into(), ..small_cfg() };
        assert!(setup(&cfg).is_err());
    }

    #[test]
    fn run_is_deterministic_for_same_seed() {
        let cfg = small_cfg();
        let r1 = run(&cfg).unwrap();
        let r2 = run(&cfg).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn run_emits_dissipation_samples_at_expected_ticks() {
        let r = run(&small_cfg()).unwrap();
        // ticks=200 sample_every=50 ⇒ samples at 50,100,150,200 (4 samples).
        assert_eq!(r.dissipation_curve.len(), 4);
        for (i, &(tick, _)) in r.dissipation_curve.iter().enumerate() {
            assert_eq!(tick, ((i + 1) * 50) as u64);
        }
    }

    #[test]
    fn dissipation_is_monotone_non_decreasing() {
        let r = run(&AutopoiesisDemoConfig { ticks: 1000, sample_every: 50, ..Default::default() })
            .unwrap();
        for w in r.dissipation_curve.windows(2) {
            assert!(w[1].1 + 1e-6 >= w[0].1, "Ax 4 violated: {:?} → {:?}", w[0], w[1]);
        }
    }
}
