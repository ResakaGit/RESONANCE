//! B1: Fermi Paradox — how many random universes develop complex life?
//!
//! Usage: `cargo run --release --bin fermi -- --universes 100 --gens 100 --ticks 500`

use resonance::use_cases::cli::{parse_arg, find_arg};
use resonance::use_cases::export;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n      = parse_arg(&args, "--universes", 50);
    let gens   = parse_arg(&args, "--gens", 100);
    let ticks  = parse_arg(&args, "--ticks", 500);

    println!("\n  Running Fermi Paradox: {n} random universes...\n");

    let report = resonance::use_cases::experiments::fermi::run(n as usize, gens as u32, ticks as u32);
    resonance::use_cases::presenters::terminal::print_fermi(&report);

    println!("\n  Per-universe summary:");
    for (i, r) in report.reports.iter().enumerate() {
        let species = r.history.last().map(|s| s.species_mean).unwrap_or(0.0);
        let fitness = r.history.last().map(|s| s.best_fitness).unwrap_or(0.0);
        let marker = if species > 3.0 { "***" } else if species > 1.0 { " + " } else { "   " };
        if i < 20 || species > 3.0 {
            println!("  {marker} U{i:>3}: species={species:.1} fitness={fitness:.3}");
        }
    }

    // CSV export: one row per universe with final stats
    if let Some(out_path) = find_arg(&args, "--out") {
        let mut csv = String::from("universe,seed,species,fitness,diversity,survivors\n");
        for (i, r) in report.reports.iter().enumerate() {
            let last = r.history.last();
            csv.push_str(&format!(
                "{},{},{:.2},{:.4},{:.4},{:.2}\n",
                i, r.seed,
                last.map(|s| s.species_mean).unwrap_or(0.0),
                last.map(|s| s.best_fitness).unwrap_or(0.0),
                last.map(|s| s.diversity).unwrap_or(0.0),
                last.map(|s| s.survivors_mean).unwrap_or(0.0),
            ));
        }
        // Also export per-universe generation histories
        if let Some(hist_path) = find_arg(&args, "--out-history") {
            let mut all_hist = Vec::new();
            for r in &report.reports { all_hist.extend_from_slice(&r.history); }
            let hist_csv = export::export_history_csv(&all_hist);
            match std::fs::write(&hist_path, &hist_csv) {
                Ok(()) => println!("  Exported history to {hist_path}"),
                Err(e) => eprintln!("  History export failed: {e}"),
            }
        }
        match std::fs::write(&out_path, &csv) {
            Ok(()) => println!("  Exported {n} universes to {out_path}"),
            Err(e) => eprintln!("  Export failed: {e}"),
        }
    }

    println!();
}
