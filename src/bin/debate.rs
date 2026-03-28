//! B4: Debate Settler — does cooperation inevitably emerge?
//!
//! Usage: `cargo run --release --bin debate -- --seeds 100 --gens 100`

use resonance::use_cases::cli::{parse_arg, find_arg, resolve_preset};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds = parse_arg(&args, "--seeds", 50);
    let gens  = parse_arg(&args, "--gens", 100);
    let ticks = parse_arg(&args, "--ticks", 500);

    let preset_name = find_arg(&args, "--preset").unwrap_or_else(|| "earth".to_string());
    let preset = resolve_preset(&preset_name);

    println!("\n  Running Debate Settler: {seeds} seeds on {} preset...\n", preset.name);

    let report = resonance::use_cases::experiments::debate::run(
        &preset, seeds as usize, gens as u32, ticks as u32,
    );
    resonance::use_cases::presenters::terminal::print_debate(&report);
    println!();
}
