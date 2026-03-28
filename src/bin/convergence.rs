//! D2: Convergent Evolution — do different seeds find the same optimal morphology?
//!
//! Usage: `cargo run --release --bin convergence -- --seeds 20 --gens 100 --ticks 500`

use resonance::use_cases::cli::{parse_arg, archetype_label};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds  = parse_arg(&args, "--seeds", 20);
    let gens   = parse_arg(&args, "--gens", 100);
    let ticks  = parse_arg(&args, "--ticks", 500);

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
        println!("  seed {i:>2}: {:<5} g={:.2} m={:.2} b={:.2} r={:.2}",
            archetype_label(g.archetype),
            g.growth_bias, g.mobility_bias, g.branching_bias, g.resilience);
    }
    println!();
}
