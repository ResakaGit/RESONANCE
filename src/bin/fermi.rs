//! B1: Fermi Paradox — how many random universes develop complex life?
//!
//! Usage: `cargo run --release --bin fermi -- --universes 100 --gens 100 --ticks 500`

use resonance::use_cases::cli::parse_arg;

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
    println!();
}
