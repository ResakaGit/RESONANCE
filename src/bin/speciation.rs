//! B2: Allopatric Speciation — does reproductive isolation emerge without programming it?
//!
//! Usage: `cargo run --release --bin speciation -- --gens 200 --ticks 500`

use resonance::use_cases::cli::{parse_arg, archetype_label};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gens   = parse_arg(&args, "--gens", 200);
    let ticks  = parse_arg(&args, "--ticks", 500);
    let seed_a = parse_arg(&args, "--seed-a", 42);
    let seed_b = parse_arg(&args, "--seed-b", 7777);

    println!("\n  Running Speciation Experiment: seed A={seed_a}, seed B={seed_b}...\n");

    let report = resonance::use_cases::experiments::speciation::run(
        &resonance::use_cases::presets::EARTH,
        seed_a as u64,
        seed_b as u64,
        gens as u32,
        ticks as u32,
        0.5,
    );
    resonance::use_cases::presenters::terminal::print_speciation(&report);

    let print_pop = |label: &str, genomes: &[resonance::batch::genome::GenomeBlob]| {
        println!("\n  Population {label} (top 5):");
        for (i, g) in genomes.iter().take(5).enumerate() {
            println!("    {i}: {:<5} g={:.2} m={:.2} b={:.2} r={:.2}",
                archetype_label(g.archetype),
                g.growth_bias, g.mobility_bias, g.branching_bias, g.resilience);
        }
    };
    print_pop("A", &report.pop_a_genomes);
    print_pop("B", &report.pop_b_genomes);
    println!();
}
