//! A1: Versus Arena — two evolved genome files compete.
//!
//! Usage: `cargo run --release --bin versus -- --a assets/evolved/seed_42.bin --b assets/evolved/seed_99.bin`
//! Or evolve fresh: `cargo run --release --bin versus -- --seed-a 42 --seed-b 99`

use resonance::use_cases::cli::{find_arg, parse_arg};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path_a = find_arg(&args, "--a");
    let path_b = find_arg(&args, "--b");

    if let (Some(a), Some(b)) = (&path_a, &path_b) {
        println!("\n  Loading genomes from files...\n");
        if let Some((ga, gb)) =
            resonance::use_cases::experiments::versus::load_competitors(Path::new(a), Path::new(b))
        {
            let result = resonance::use_cases::experiments::versus::compare_potential(&ga, &gb);
            resonance::use_cases::presenters::terminal::print_versus(&result);
            println!();
            return;
        }
        println!("  Failed to load genome files. Falling back to fresh evolution.\n");
    }

    let seed_a = parse_arg(&args, "--seed-a", 42);
    let seed_b = parse_arg(&args, "--seed-b", 99);
    let gens = parse_arg(&args, "--gens", 100);
    let ticks = parse_arg(&args, "--ticks", 500);

    println!("  Evolving Team A (seed={seed_a})...");
    let report_a = resonance::use_cases::evolve_with(
        &resonance::use_cases::presets::EARTH,
        seed_a as u64,
        100,
        gens as u32,
        ticks as u32,
        12,
    );
    println!("  Evolving Team B (seed={seed_b})...");
    let report_b = resonance::use_cases::evolve_with(
        &resonance::use_cases::presets::EARTH,
        seed_b as u64,
        100,
        gens as u32,
        ticks as u32,
        12,
    );

    let result = resonance::use_cases::experiments::versus::compare_potential(
        &report_a.top_genomes,
        &report_b.top_genomes,
    );
    println!();
    resonance::use_cases::presenters::terminal::print_versus(&result);
    println!();
}
