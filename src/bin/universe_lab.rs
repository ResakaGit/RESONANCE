//! A2: Universe Lab — evolve life under different physical laws.
//!
//! Usage: `cargo run --release --bin universe_lab -- --preset jupiter --gens 200`
//! Presets: earth, jupiter, mars, eden, hell, random

use resonance::use_cases::cli::{parse_arg, find_arg};
use resonance::use_cases::presets;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let preset_name = find_arg(&args, "--preset").unwrap_or_else(|| "earth".to_string());
    let gens   = parse_arg(&args, "--gens", 200);
    let worlds = parse_arg(&args, "--worlds", 200);
    let ticks  = parse_arg(&args, "--ticks", 500);
    let seed   = parse_arg(&args, "--seed", 42);

    let preset = match preset_name.to_lowercase().as_str() {
        "jupiter" => presets::JUPITER,
        "mars"    => presets::MARS,
        "eden"    => presets::EDEN,
        "hell"    => presets::HELL,
        "random"  => presets::UniversePreset::from_seed(seed as u64),
        _         => presets::EARTH,
    };

    println!("\n  Universe Lab: {} universe\n", preset.name);
    println!("  Physical constants:");
    println!("    gravity:            {:.2}", preset.gravity);
    println!("    solar_flux:         {:.2}", preset.solar_flux);
    println!("    solar_frequency:    {:.1} Hz", preset.solar_frequency);
    println!("    season_rate:        {:.4}", preset.season_rate);
    println!("    season_amplitude:   {:.2}", preset.season_amplitude);
    println!("    asteroid_interval:  {}", preset.asteroid_interval);
    println!("    photosynthesis_eff: {:.2}", preset.photosynthesis_eff);
    println!();

    let report = resonance::use_cases::experiments::lab::run(
        &preset, seed as u64, worlds as usize, gens as u32, ticks as u32,
    );
    resonance::use_cases::presenters::terminal::print_report(&report);
    println!();
}
