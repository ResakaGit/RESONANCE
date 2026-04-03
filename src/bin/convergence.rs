//! D2: Convergent Evolution — do different seeds find the same optimal morphology?
//!
//! Usage: `cargo run --release --bin convergence -- --seeds 20 --gens 100 --ticks 500`

use resonance::use_cases::cli::{archetype_label, find_arg, parse_arg};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds = parse_arg(&args, "--seeds", 20);
    let gens = parse_arg(&args, "--gens", 100);
    let ticks = parse_arg(&args, "--ticks", 500);

    println!("\n  Running Convergence Analysis: {seeds} seeds on Earth preset...\n");

    let report = resonance::use_cases::experiments::convergence::run(
        &resonance::use_cases::presets::EARTH,
        seeds as usize,
        gens as u32,
        ticks as u32,
        0.3,
    );
    resonance::use_cases::presenters::terminal::print_convergence(&report);

    println!("\n  Top genome per seed:");
    for (i, g) in report.top_genomes.iter().enumerate() {
        println!(
            "  seed {i:>2}: {:<5} g={:.2} m={:.2} b={:.2} r={:.2}",
            archetype_label(g.archetype),
            g.growth_bias,
            g.mobility_bias,
            g.branching_bias,
            g.resilience
        );
    }

    // CSV export: genome biases per seed for clustering analysis
    if let Some(out_path) = find_arg(&args, "--out") {
        let mut csv = String::from("seed,archetype,trophic,growth,mobility,branching,resilience\n");
        for (i, g) in report.top_genomes.iter().enumerate() {
            csv.push_str(&format!(
                "{},{},{},{:.4},{:.4},{:.4},{:.4}\n",
                i,
                g.archetype,
                g.trophic_class,
                g.growth_bias,
                g.mobility_bias,
                g.branching_bias,
                g.resilience,
            ));
        }
        match std::fs::write(&out_path, &csv) {
            Ok(()) => println!("  Exported {seeds} genomes to {out_path}"),
            Err(e) => eprintln!("  Export failed: {e}"),
        }
    }

    println!();
}
