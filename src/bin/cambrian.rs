//! B3: Cambrian Explosion — detect morphological innovation spikes.
//!
//! Usage: `cargo run --release --bin cambrian -- --gens 300 --worlds 200`

use resonance::use_cases::cli::parse_arg;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gens   = parse_arg(&args, "--gens", 300);
    let worlds = parse_arg(&args, "--worlds", 200);
    let ticks  = parse_arg(&args, "--ticks", 500);
    let seed   = parse_arg(&args, "--seed", 42);

    println!("\n  Running Cambrian Explosion Analysis...\n");
    println!("  worlds={worlds} gens={gens} ticks={ticks} seed={seed}\n");

    let report = resonance::use_cases::experiments::cambrian::run(
        &resonance::use_cases::presets::EARTH,
        seed as u64,
        worlds as usize,
        gens as u32,
        ticks as u32,
        0.05,
    );
    resonance::use_cases::presenters::terminal::print_cambrian(&report);
    println!();
}
