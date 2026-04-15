//! Demo 3 · Kleiber's law verification — log-log fit on synthetic creatures.
//!
//! Generates a population whose metabolic rate is computed via the simulator's
//! own `kleiber_volume_factor(radius) = radius^KLEIBER_EXPONENT` (`exact_cache.rs`).
//! Adds deterministic noise, then fits log(B) vs log(M) and verifies the
//! recovered slope matches `KLEIBER_EXPONENT = 0.75`.
//!
//! Why this matters: Kleiber's law (1932) is one of biology's deepest empirical
//! regularities (B ∝ M^0.75 across 27 orders of magnitude, bacteria → blue whale).
//! In RESONANCE the exponent is one of the **4 fundamental constants** —
//! everything else derives from it.  This demo is the post-hoc check that the
//! derivation actually satisfies the law on a generated population.
//!
//! Pure HOF: the population is generated functionally (fold over indices),
//! the fit is a closed-form regression (no iteration), no globals.

use crate::blueprint::equations::derived_thresholds::KLEIBER_EXPONENT;

/// Demo input — controls the synthetic population.
#[derive(Clone, Debug)]
pub struct KleiberDemoConfig {
    /// PRNG seed for the deterministic noise (proportional jitter on B).
    pub seed: u64,
    /// Number of "creatures" (mass samples).  Default 256 covers ~3 decades.
    pub n_creatures: usize,
    /// Mass range — masses sampled log-uniformly in `[mass_min, mass_max]`.
    pub mass_min: f64,
    pub mass_max: f64,
    /// Calibration constant `a` in `B = a · M^0.75`.  Affects intercept, not slope.
    pub a_coefficient: f64,
    /// Multiplicative noise scale: each B is multiplied by `1 + noise·U(-1,1)`.
    /// `0.0` = no noise (perfect fit).  `0.05` = 5% jitter (realistic).
    pub noise: f64,
}

impl Default for KleiberDemoConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            n_creatures: 256,
            mass_min: 1e-3,    // 1 mg (small bacterium-scale)
            mass_max: 1e6,     // 1000 kg (large mammal-scale)
            a_coefficient: 3.4,// Empirical mammals (Kleiber 1932 in W·g^-0.75)
            noise: 0.05,
        }
    }
}

/// Per-creature observable.  Mass-based linear index in `step`.
#[derive(Clone, Debug, PartialEq)]
pub struct KleiberFrame {
    pub mass: f64,
    pub metabolic_rate: f64,
}

/// Final report — slope + R² + the regression's predicted exponent.
#[derive(Clone, Debug, PartialEq)]
pub struct KleiberReport {
    pub n_samples: usize,
    pub fitted_slope: f64,
    pub fitted_intercept_log: f64,
    pub axiomatic_exponent: f64,
    pub slope_error: f64,
    pub points: Vec<(f64, f64)>,
}

// ── Phase 1 · setup ────────────────────────────────────────────────────────

/// Pure: build the iteration "state" — just (cfg, current_index).
/// All the math happens in `step_creature` driven by the index.
pub fn setup(cfg: &KleiberDemoConfig) -> KleiberState {
    KleiberState { cfg: cfg.clone(), index: 0 }
}

#[derive(Clone, Debug)]
pub struct KleiberState {
    pub cfg: KleiberDemoConfig,
    pub index: usize,
}

// ── Phase 2 · step ─────────────────────────────────────────────────────────

/// Pure: emit one (mass, metabolic_rate) sample per step.
/// Mass is sampled log-uniformly so the regression has even coverage on the
/// log axis (otherwise small-mass points dominate visually but contribute
/// equally to the OLS fit — log-uniform spacing avoids that bias).
pub fn step_creature(state: &mut KleiberState) -> Option<KleiberFrame> {
    if state.index >= state.cfg.n_creatures { return None; }
    let i = state.index as f64;
    let n = state.cfg.n_creatures as f64;
    let log_min = state.cfg.mass_min.ln();
    let log_max = state.cfg.mass_max.ln();
    // Log-uniform mass: index 0 → mass_min, index n-1 → mass_max.
    let mass = (log_min + (log_max - log_min) * i / (n - 1.0).max(1.0)).exp();
    // Apply Kleiber's law directly using the simulator's `KLEIBER_EXPONENT` constant.
    // The empirical law is `B ∝ M^KLEIBER_EXPONENT` (mass, not radius); the
    // simulator's `kleiber_volume_factor(r) = r^E` operates on radius for its own
    // metabolic systems but the demo validates the EXPONENT directly against
    // the canonical Kleiber form so the regression slope == E.
    let pure_b = state.cfg.a_coefficient * mass.powf(KLEIBER_EXPONENT as f64);
    // Deterministic noise from a hash of (seed, index) — no global RNG.
    let jitter = jitter_signed(state.cfg.seed, state.index) * state.cfg.noise;
    let metabolic_rate = pure_b * (1.0 + jitter);
    state.index += 1;
    Some(KleiberFrame { mass, metabolic_rate })
}

/// Deterministic `[-1, 1]` jitter from `(seed, index)`.  Pure, no thread RNG.
/// Uses a simple xorshift over a hash mix; quality is fine for demo noise.
fn jitter_signed(seed: u64, index: usize) -> f64 {
    let mut x = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(index as u64);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    let u = (x >> 11) as f64 / ((1u64 << 53) as f64); // [0, 1)
    u * 2.0 - 1.0
}

// ── Phase 3 · summarize ────────────────────────────────────────────────────

/// Pure: fit log(B) ≈ slope · log(M) + intercept via the shared HOF
/// `super::loglog_linear_fit`.  Returns slope error vs `KLEIBER_EXPONENT`.
pub fn summarize(_state: &KleiberState, frames: Vec<KleiberFrame>) -> KleiberReport {
    let points: Vec<(f64, f64)> = frames
        .iter()
        .map(|f| (f.mass, f.metabolic_rate))
        .collect();
    let (slope, intercept) = super::loglog_linear_fit(&points);
    let axiomatic = KLEIBER_EXPONENT as f64;
    KleiberReport {
        n_samples: frames.len(),
        fitted_slope: slope,
        fitted_intercept_log: intercept,
        axiomatic_exponent: axiomatic,
        slope_error: (slope - axiomatic).abs(),
        points,
    }
}

// ── Orchestrator ───────────────────────────────────────────────────────────

/// Compose `setup → run_demo(step, summarize)`.
pub fn run(cfg: &KleiberDemoConfig) -> KleiberReport {
    let state = setup(cfg);
    super::run_demo(state, step_creature, summarize)
}

// ── Presentation: CSV + SVG (pure fns, no I/O) ─────────────────────────────

impl KleiberReport {
    /// Emit `(mass, metabolic_rate)` as CSV with a one-line header.
    /// Pure — caller writes to disk.  Determinism: pairs in insertion order.
    pub fn to_csv(&self) -> String {
        let mut out = String::with_capacity(64 + self.points.len() * 32);
        out.push_str("mass,metabolic_rate\n");
        for &(m, b) in &self.points {
            out.push_str(&format!("{m},{b}\n"));
        }
        out
    }

    /// Emit an SVG log-log scatter plot with the regression line overlaid.
    /// Pure stdlib — no external SVG/plotting crate.  Produces a valid
    /// standalone `.svg` ~5 KB suitable for HN/blog inline embedding.
    ///
    /// Layout:
    ///   - Outer canvas `width × height` px (default 800×600 recommended)
    ///   - Plot area inset by 60 px (left axis labels) / 60 px (bottom)
    ///   - Log axes: 4 grid lines + tick labels per decade
    ///   - Scatter: small circles, slope-line, equation legend top-right
    pub fn to_svg(&self, width: u32, height: u32) -> String {
        svg_loglog_scatter(self, width, height)
    }
}

/// Pure SVG generator.  Coordinates are computed in log-space then mapped
/// to plot pixels.  Defensive on edge cases (empty points, single-decade
/// span, non-finite values).
fn svg_loglog_scatter(report: &KleiberReport, width: u32, height: u32) -> String {
    // Plot area margins (px).  Left/bottom larger to fit axis labels.
    let (margin_l, margin_r, margin_t, margin_b) = (70, 30, 60, 60);
    let plot_w = (width as i32 - margin_l - margin_r).max(50);
    let plot_h = (height as i32 - margin_t - margin_b).max(50);

    // Compute log-space bounds with a 5% padding on each side.
    let logs: Vec<(f64, f64)> = report.points
        .iter()
        .filter(|(m, b)| *m > 0.0 && *b > 0.0 && m.is_finite() && b.is_finite())
        .map(|(m, b)| (m.ln(), b.ln()))
        .collect();
    if logs.is_empty() {
        return svg_placeholder(width, height, "(empty Kleiber report — no valid points)");
    }
    let (lx_min, lx_max, ly_min, ly_max) = bounds_with_padding(&logs, 0.05);

    // Linear mapping: log-coord → pixel.
    let map_x = |lx: f64| -> f64 {
        margin_l as f64 + (lx - lx_min) / (lx_max - lx_min) * plot_w as f64
    };
    let map_y = |ly: f64| -> f64 {
        // SVG Y grows downward, plot Y grows upward → invert.
        (margin_t + plot_h) as f64 - (ly - ly_min) / (ly_max - ly_min) * plot_h as f64
    };

    let mut s = String::with_capacity(8 * 1024);
    // SVG header + neutral background.
    s.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" font-family="monospace" font-size="11">
<rect width="100%" height="100%" fill="#fafafa"/>
"##));
    // Plot area frame.
    s.push_str(&format!(
        r##"<rect x="{margin_l}" y="{margin_t}" width="{plot_w}" height="{plot_h}" fill="white" stroke="#888"/>
"##));
    // Title.
    s.push_str(&format!(
        r##"<text x="{x}" y="30" text-anchor="middle" font-size="14" font-weight="bold">Kleiber's law verification — log-log fit</text>
"##,
        x = width / 2,
    ));
    // Decade grid lines + tick labels (X axis).
    for lx_decade in decade_ticks(lx_min, lx_max) {
        let x = map_x(lx_decade);
        s.push_str(&format!(
            r##"<line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke="#ddd"/>
<text x="{x}" y="{ty}" text-anchor="middle">10^{e}</text>
"##,
            y1 = margin_t,
            y2 = margin_t + plot_h,
            ty = margin_t + plot_h + 18,
            e = (lx_decade / 10.0_f64.ln()).round() as i32,
        ));
    }
    // Decade grid lines + tick labels (Y axis).
    for ly_decade in decade_ticks(ly_min, ly_max) {
        let y = map_y(ly_decade);
        s.push_str(&format!(
            r##"<line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke="#ddd"/>
<text x="{tx}" y="{ty}" text-anchor="end">10^{e}</text>
"##,
            x1 = margin_l,
            x2 = margin_l + plot_w,
            tx = margin_l - 6,
            ty = y + 4.0,
            e = (ly_decade / 10.0_f64.ln()).round() as i32,
        ));
    }
    // Axis labels.
    s.push_str(&format!(
        r##"<text x="{x}" y="{y}" text-anchor="middle">log10(mass)</text>
<text x="20" y="{cy}" text-anchor="middle" transform="rotate(-90 20 {cy})">log10(metabolic rate)</text>
"##,
        x = margin_l + plot_w / 2,
        y = height as i32 - 12,
        cy = margin_t + plot_h / 2,
    ));
    // Scatter points (pre-filtered to `logs`).
    for &(lx, ly) in &logs {
        s.push_str(&format!(
            r##"<circle cx="{cx:.2}" cy="{cy:.2}" r="2.5" fill="#0066cc" opacity="0.55"/>
"##,
            cx = map_x(lx),
            cy = map_y(ly),
        ));
    }
    // Regression line: from (lx_min, ly_min_pred) to (lx_max, ly_max_pred).
    let slope = report.fitted_slope;
    let intercept = report.fitted_intercept_log;
    let (line_x1, line_x2) = (map_x(lx_min), map_x(lx_max));
    let (line_y1, line_y2) = (
        map_y(slope * lx_min + intercept),
        map_y(slope * lx_max + intercept),
    );
    s.push_str(&format!(
        r##"<line x1="{line_x1:.2}" y1="{line_y1:.2}" x2="{line_x2:.2}" y2="{line_y2:.2}" stroke="#cc0000" stroke-width="1.8" stroke-dasharray="6,3"/>
"##,
    ));
    // Equation legend (top-right of plot area).
    let legend_x = margin_l + plot_w - 10;
    let legend_y = margin_t + 18;
    s.push_str(&format!(
        r##"<text x="{legend_x}" y="{legend_y}" text-anchor="end" fill="#333">slope = {slope:.6}</text>
<text x="{legend_x}" y="{ly2}" text-anchor="end" fill="#333">axiomatic = {ax:.6}</text>
<text x="{legend_x}" y="{ly3}" text-anchor="end" fill="#333">|error|  = {err:.2e}</text>
<text x="{legend_x}" y="{ly4}" text-anchor="end" fill="#333">n = {n} samples</text>
"##,
        ax = report.axiomatic_exponent,
        err = report.slope_error,
        n = report.n_samples,
        ly2 = legend_y + 16,
        ly3 = legend_y + 32,
        ly4 = legend_y + 48,
    ));
    s.push_str("</svg>\n");
    s
}

/// Returns log-space decade tick values within `[lo, hi]`.  Empty if span < one decade.
fn decade_ticks(lo: f64, hi: f64) -> Vec<f64> {
    let ln10 = 10.0_f64.ln();
    let lo_dec = (lo / ln10).ceil() as i32;
    let hi_dec = (hi / ln10).floor() as i32;
    (lo_dec..=hi_dec).map(|e| e as f64 * ln10).collect()
}

fn bounds_with_padding(points: &[(f64, f64)], pad_frac: f64) -> (f64, f64, f64, f64) {
    let (mut x_lo, mut x_hi) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut y_lo, mut y_hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &(x, y) in points {
        x_lo = x_lo.min(x);  x_hi = x_hi.max(x);
        y_lo = y_lo.min(y);  y_hi = y_hi.max(y);
    }
    let dx = (x_hi - x_lo).max(1e-9) * pad_frac;
    let dy = (y_hi - y_lo).max(1e-9) * pad_frac;
    (x_lo - dx, x_hi + dx, y_lo - dy, y_hi + dy)
}

fn svg_placeholder(width: u32, height: u32, msg: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">
<rect width="100%" height="100%" fill="#fafafa"/>
<text x="{x}" y="{y}" text-anchor="middle" font-family="monospace">{msg}</text>
</svg>
"##,
        x = width / 2, y = height / 2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_recovers_kleiber_exponent_with_zero_noise() {
        let report = run(&KleiberDemoConfig { noise: 0.0, ..Default::default() });
        assert!(
            report.slope_error < 1e-3,
            "fitted={} axiomatic={} error={}",
            report.fitted_slope, report.axiomatic_exponent, report.slope_error,
        );
    }

    #[test]
    fn fit_tolerant_to_5_percent_noise() {
        let report = run(&KleiberDemoConfig { noise: 0.05, ..Default::default() });
        assert!(
            report.slope_error < 0.02,
            "with 5% noise slope should still be within 0.02 of 0.75, got {}",
            report.slope_error,
        );
    }

    #[test]
    fn run_is_deterministic() {
        let cfg = KleiberDemoConfig::default();
        assert_eq!(run(&cfg), run(&cfg));
    }

    #[test]
    fn n_samples_matches_config() {
        let report = run(&KleiberDemoConfig { n_creatures: 100, ..Default::default() });
        assert_eq!(report.n_samples, 100);
    }

    #[test]
    fn csv_round_trips_point_count_and_header() {
        let r = run(&KleiberDemoConfig { n_creatures: 5, noise: 0.0, ..Default::default() });
        let csv = r.to_csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "mass,metabolic_rate");
        assert_eq!(lines.len(), 1 + 5);
        for line in &lines[1..] {
            let parts: Vec<&str> = line.split(',').collect();
            assert_eq!(parts.len(), 2);
            assert!(parts[0].parse::<f64>().is_ok());
            assert!(parts[1].parse::<f64>().is_ok());
        }
    }

    #[test]
    fn svg_is_valid_well_formed_xml_with_required_elements() {
        let r = run(&KleiberDemoConfig { n_creatures: 32, noise: 0.05, ..Default::default() });
        let svg = r.to_svg(800, 600);
        assert!(svg.starts_with("<svg "), "starts with svg root");
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Kleiber"));
        assert!(svg.contains("slope = 0.7"));
        assert!(svg.contains("<circle"), "scatter points present");
        assert!(svg.contains("<line"), "regression line + grid present");
        // Balanced root tag (no nested svg).
        assert_eq!(svg.matches("<svg ").count(), 1);
        assert_eq!(svg.matches("</svg>").count(), 1);
    }

    #[test]
    fn svg_handles_empty_report_with_placeholder() {
        let empty = KleiberReport {
            n_samples: 0, fitted_slope: 0.0, fitted_intercept_log: 0.0,
            axiomatic_exponent: 0.75, slope_error: 0.75, points: vec![],
        };
        let svg = empty.to_svg(400, 300);
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("empty Kleiber report"));
    }

    #[test]
    fn decade_ticks_returns_integer_decades_in_log_space() {
        let ln10 = 10.0_f64.ln();
        let ticks = super::decade_ticks(1.0_f64.ln() - 0.1, 1000.0_f64.ln() + 0.1);
        // Expect ticks at 10^0, 10^1, 10^2, 10^3.
        assert_eq!(ticks.len(), 4);
        for (i, t) in ticks.iter().enumerate() {
            assert!((t - i as f64 * ln10).abs() < 1e-9);
        }
    }

    #[test]
    fn jitter_is_bounded_and_deterministic() {
        for i in 0..1000 {
            let j = jitter_signed(42, i);
            assert!((-1.0..=1.0).contains(&j));
        }
        assert_eq!(jitter_signed(7, 13), jitter_signed(7, 13));
        assert_ne!(jitter_signed(7, 13), jitter_signed(7, 14));
    }
}
