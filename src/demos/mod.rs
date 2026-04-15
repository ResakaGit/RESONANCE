//! Stateless demo orchestration via higher-order composition.
//!
//! Each demo follows the same three-phase pattern:
//!
//!   1. **setup**     — build initial state from a config (pure, deterministic)
//!   2. **step**      — advance state by one tick, optionally emit a Frame
//!   3. **summarize** — fold observed frames into a typed Report
//!
//! The `run_demo` HOF below wires the three phases together with **zero
//! shared mutable state** beyond the local `state` value.  Side effects
//! (printing, file I/O) are LIFTED INTO THE CALLER via the report — the
//! orchestration itself is referentially transparent.
//!
//! # Why HOF?
//!
//! - The same orchestrator drives every demo (autopoiesis, papers, kleiber).
//! - Each demo only declares its 3 pure pieces; orchestration is shared.
//! - Tests can drive `step` independently of `summarize`, or assert on
//!   the report without exercising the stepper at all.
//!
//! # Determinism contract
//!
//! For a given `Config`, `run_demo(setup(cfg), step, summarize)` MUST be
//! byte-identical across runs.  Each demo's tests verify this.

pub mod autopoiesis;
pub mod kleiber;
pub mod papers;

/// Generic state-machine driver.  Pure HOF — no globals, no I/O.
///
/// Drives `state` through `step` until `step` returns `None`, then folds
/// the accumulated frames into a `Report` via `summarize`.  The closures
/// receive `&mut state` (step) and `&state` (summarize) — there is no
/// hidden state outside the local binding.
///
/// # Arguments
///
/// * `state`     — the initial state object (typically built by the demo's `setup`).
/// * `step`      — advances state by one tick; returns `Some(frame)` while running, `None` when done.
/// * `summarize` — folds the list of emitted frames + final state into a typed report.
///
/// # Returns
///
/// The Report computed by `summarize`.  Frames are dropped after summary.
pub fn run_demo<State, Frame, Report>(
    mut state: State,
    mut step: impl FnMut(&mut State) -> Option<Frame>,
    summarize: impl FnOnce(&State, Vec<Frame>) -> Report,
) -> Report {
    let mut frames: Vec<Frame> = Vec::new();
    while let Some(frame) = step(&mut state) {
        frames.push(frame);
    }
    summarize(&state, frames)
}

/// Linear regression on `(x, y)` pairs in log-log space.
/// Returns `(slope, intercept_log)` such that `log(y) ≈ slope · log(x) + intercept`.
///
/// Pure fn used by `kleiber.rs` to validate Kleiber's law (slope ≈ 0.75).
/// Lives here because it's small, generic, and a future demo may also need it.
///
/// # Edge cases
///
/// - Empty or single point ⇒ `(0.0, 0.0)`.
/// - Non-finite or non-positive coords skipped.
/// - All `x` equal ⇒ `(0.0, mean(log y))` (avoids div-by-zero).
pub fn loglog_linear_fit(points: &[(f64, f64)]) -> (f64, f64) {
    let mut xs: Vec<f64> = Vec::with_capacity(points.len());
    let mut ys: Vec<f64> = Vec::with_capacity(points.len());
    for &(x, y) in points {
        if x > 0.0 && y > 0.0 && x.is_finite() && y.is_finite() {
            xs.push(x.ln());
            ys.push(y.ln());
        }
    }
    let n = xs.len();
    if n < 2 { return (0.0, 0.0); }
    let mean_x = xs.iter().sum::<f64>() / n as f64;
    let mean_y = ys.iter().sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        let dx = xs[i] - mean_x;
        num += dx * (ys[i] - mean_y);
        den += dx * dx;
    }
    if den.abs() < f64::EPSILON { return (0.0, mean_y); }
    let slope = num / den;
    let intercept = mean_y - slope * mean_x;
    (slope, intercept)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── run_demo HOF ────────────────────────────────────────────────────────

    #[test]
    fn run_demo_drives_state_until_none() {
        // Counts 0..5 via mutable counter; summarize sums frames.
        let total = run_demo(
            0u32,
            |state| {
                if *state >= 5 { return None; }
                *state += 1;
                Some(*state)
            },
            |_state, frames| frames.iter().sum::<u32>(),
        );
        assert_eq!(total, 1 + 2 + 3 + 4 + 5);
    }

    #[test]
    fn run_demo_with_zero_steps_summarizes_empty_frames() {
        // Type annotation on the closure return makes Vec<u32> inferable.
        let n = run_demo(
            ("done", 99u32),
            |_| -> Option<u32> { None }, // never produces frames
            |state, frames: Vec<u32>| (state.0.to_string(), state.1, frames.len()),
        );
        assert_eq!(n, ("done".to_string(), 99, 0));
    }

    #[test]
    fn run_demo_is_pure_no_external_state_needed() {
        // Two runs with same closure produce same result.
        let run = || run_demo(
            10i32,
            |s| if *s == 0 { None } else { let v = *s; *s -= 1; Some(v) },
            |_, f| f.iter().sum::<i32>(),
        );
        assert_eq!(run(), run());
    }

    // ── loglog_linear_fit ───────────────────────────────────────────────────

    #[test]
    fn fit_recovers_slope_for_perfect_power_law() {
        // y = 2 * x^0.75  ⇒  log y = 0.75 log x + log 2
        let points: Vec<(f64, f64)> = (1..=20)
            .map(|i| { let x = i as f64; (x, 2.0 * x.powf(0.75)) })
            .collect();
        let (slope, intercept) = loglog_linear_fit(&points);
        assert!((slope - 0.75).abs() < 1e-9, "slope = {slope}");
        assert!((intercept - 2.0_f64.ln()).abs() < 1e-9, "intercept = {intercept}");
    }

    #[test]
    fn fit_handles_empty_input() {
        assert_eq!(loglog_linear_fit(&[]), (0.0, 0.0));
    }

    #[test]
    fn fit_skips_non_positive_coords() {
        let points = vec![(1.0, 1.0), (0.0, 5.0), (-1.0, 3.0), (10.0, 10.0_f64.powf(0.5))];
        let (slope, _) = loglog_linear_fit(&points);
        assert!((slope - 0.5).abs() < 1e-9);
    }

    #[test]
    fn fit_handles_single_point() {
        assert_eq!(loglog_linear_fit(&[(1.0, 1.0)]), (0.0, 0.0));
    }
}
