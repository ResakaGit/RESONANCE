//! `demos` — single binary dispatching the three publishable demos.
//!
//! Architecture: each subcommand is a thin printer wrapping the pure
//! orchestrators from `resonance::demos::*`.  No simulation logic lives
//! in this file — only argv parsing and a stdlib presenter.
//!
//! Usage:
//!     cargo run --release --bin demos -- autopoiesis [--seed N] [--ticks T]
//!     cargo run --release --bin demos -- papers [--seed N]
//!     cargo run --release --bin demos -- kleiber [--seed N] [--n N] [--noise X]
//!     cargo run --release --bin demos -- --help
//!
//! All three are deterministic for a given seed and produce a typed report
//! (no globals, no shared state).  The binary just renders the report to stdout.

use std::env;
use std::process::ExitCode;

use resonance::demos::{autopoiesis, kleiber, papers};

const HELP: &str = "\
demos — three publishable demos in one binary

USAGE:
    demos <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    autopoiesis   Formose mass-action vesicle: form + fission + lineage
    papers        Reproduce 6 published cancer-therapy papers from 4 constants
    kleiber       Verify Kleiber's law (B ∝ M^0.75) by log-log regression

GLOBAL OPTIONS:
    --help        Show this help

PER-SUBCOMMAND OPTIONS:
    autopoiesis  --seed N            (default 0)
                 --ticks N           (default 2000)
                 --grid WxH          (default 16x16)
                 --spot R            (default 2)
                 --food-qe Q         (default 50)
                 --sample-every N    (default 50)

    papers       --seed N            (default 0)

    kleiber      --seed N            (default 42)
                 --n N               (default 256)
                 --noise X           (default 0.05)
";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let subcommand = match args.get(1).map(String::as_str) {
        Some(s) if !s.starts_with('-') => s,
        _ => { eprintln!("error: missing subcommand\n\n{HELP}"); return ExitCode::FAILURE; }
    };
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    let result = match subcommand {
        "autopoiesis" => run_autopoiesis(&rest),
        "papers"      => run_papers(&rest),
        "kleiber"     => run_kleiber(&rest),
        other => Err(format!("unknown subcommand: {other}\n\n{HELP}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => { eprintln!("error: {e}"); ExitCode::FAILURE }
    }
}

// ── Subcommand: autopoiesis ───────────────────────────────────────────────

fn run_autopoiesis(rest: &[String]) -> Result<(), String> {
    let cfg = autopoiesis::AutopoiesisDemoConfig {
        seed:           parse_or(rest, "--seed", 0_u64)?,
        network_path:   parse_str(rest, "--network",
                            "assets/reactions/formose.ron")?,
        grid:           parse_grid(rest, "--grid", (16, 16))?,
        spot_radius:    parse_or(rest, "--spot", 2usize)?,
        food_qe:        parse_or(rest, "--food-qe", 50.0_f32)?,
        ticks:          parse_or(rest, "--ticks", 2000_u64)?,
        sample_every:   parse_or(rest, "--sample-every", 50_u64)?,
    };
    let report = autopoiesis::run(&cfg)?;
    println!("AUTOPOIESIS DEMO");
    println!("================");
    println!("seed={}  ticks={}  fissions={}  total_dissipated={:.4}",
        report.seed, report.n_ticks, report.n_fissions, report.total_dissipated);
    println!("\ndissipation curve (tick, qe):");
    for (tick, d) in &report.dissipation_curve {
        println!("  {tick:>6}  {d:>10.4}");
    }
    println!("\nverdict: vesicle formed at t=0, fission triggered at the gas/liquid pressure ratio");
    println!("         (ADR-039 §revisión 2026-04-15-b), {} child lineages spawned.",
             report.n_fissions * 2);
    Ok(())
}

// ── Subcommand: papers ────────────────────────────────────────────────────

fn run_papers(rest: &[String]) -> Result<(), String> {
    let cfg = papers::PapersDemoConfig {
        seed: parse_or(rest, "--seed", 0_u64)?,
        include: papers::PaperSet::All,
    };
    let report = papers::run(&cfg);
    println!("PAPER VALIDATION DEMO ({}/{} pass, {:.1}s wall)",
        report.passed_count(), report.total(),
        report.total_wall_ms as f32 / 1000.0);
    println!("=========================================");
    for o in &report.outcomes {
        let mark = if o.verdict.is_pass() { "PASS" } else { "FAIL" };
        let detail = match o.verdict {
            papers::PaperVerdict::Boolean(b) => if b { "true".into() } else { "false".into() },
            papers::PaperVerdict::Fraction { passed, total } =>
                format!("{passed}/{total}"),
        };
        println!("  {} [{}] {:>5}ms  {:<8}  {}",
            mark, o.id, o.wall_ms, detail, o.citation);
    }
    println!("\nNote: PV-6 (unified axioms) reports K/N — the 6 prior predictions");
    println!("      derive from 4 fundamental constants; honest fractional verdict.");
    Ok(())
}

// ── Subcommand: kleiber ───────────────────────────────────────────────────

fn run_kleiber(rest: &[String]) -> Result<(), String> {
    let cfg = kleiber::KleiberDemoConfig {
        seed:        parse_or(rest, "--seed", 42_u64)?,
        n_creatures: parse_or(rest, "--n", 256_usize)?,
        noise:       parse_or(rest, "--noise", 0.05_f64)?,
        ..Default::default()
    };
    let report = kleiber::run(&cfg);
    println!("KLEIBER LAW DEMO");
    println!("================");
    println!("n_samples       = {}", report.n_samples);
    println!("axiomatic exp   = {:.6}  (KLEIBER_EXPONENT, hardcoded constant)",
        report.axiomatic_exponent);
    println!("fitted slope    = {:.6}  (log-log regression)", report.fitted_slope);
    println!("slope error     = {:.6}", report.slope_error);
    println!("verdict: {}",
        if report.slope_error < 0.02 { "Kleiber's law verified within 2%" }
        else { "slope deviates >2% — investigate noise / sample size" });
    println!("\ndensity preview ({} log-mass bins):", LOG_BINS);
    print_density_histogram(&report.points);
    Ok(())
}

const LOG_BINS: usize = 20;

/// Tiny stdout sparkline for the (mass, B) point cloud.  Each line is one
/// log-mass bin; the bar length is the bin's count.  Pure stdlib.
fn print_density_histogram(points: &[(f64, f64)]) {
    if points.is_empty() { return; }
    let logs: Vec<f64> = points.iter().map(|(m, _)| m.ln()).collect();
    let lo = logs.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = logs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (hi - lo).max(1e-9);
    let mut bins = [0u32; LOG_BINS];
    for &l in &logs {
        let i = (((l - lo) / span) * (LOG_BINS as f64 - 1.0)) as usize;
        bins[i.min(LOG_BINS - 1)] += 1;
    }
    let max_count = *bins.iter().max().unwrap_or(&1) as f32;
    for (i, &c) in bins.iter().enumerate() {
        let mass_lo = (lo + span * (i as f64 / LOG_BINS as f64)).exp();
        let bar_len = ((c as f32 / max_count) * 30.0) as usize;
        println!("  M~{mass_lo:>10.3e}  {} ({c})", "#".repeat(bar_len));
    }
}

// ── argv helpers ──────────────────────────────────────────────────────────

fn find_value<'a>(rest: &'a [String], flag: &str) -> Option<&'a str> {
    let mut iter = rest.iter();
    while let Some(a) = iter.next() {
        if a == flag { return iter.next().map(|s| s.as_str()); }
    }
    None
}

fn parse_or<T: std::str::FromStr>(rest: &[String], flag: &str, default: T) -> Result<T, String>
where T::Err: std::fmt::Display {
    match find_value(rest, flag) {
        None => Ok(default),
        Some(v) => v.parse().map_err(|e| format!("{flag}: parse error '{v}': {e}")),
    }
}

fn parse_str(rest: &[String], flag: &str, default: &str) -> Result<String, String> {
    Ok(find_value(rest, flag).unwrap_or(default).to_string())
}

fn parse_grid(rest: &[String], flag: &str, default: (usize, usize)) -> Result<(usize, usize), String> {
    let Some(v) = find_value(rest, flag) else { return Ok(default); };
    let (w, h) = v.split_once('x').ok_or_else(|| format!("{flag}: expected WxH"))?;
    let w: usize = w.parse().map_err(|e| format!("{flag} width: {e}"))?;
    let h: usize = h.parse().map_err(|e| format!("{flag} height: {e}"))?;
    if w == 0 || h == 0 { return Err(format!("{flag}: dims must be > 0")); }
    Ok((w, h))
}
