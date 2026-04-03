//! D1: Personal Universe — your name/birthday = your unique ecosystem.
//!
//! Usage: `cargo run --release --bin personal_universe -- "Ada Lovelace 1815-12-10"`

fn main() {
    let input: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let input = if input.is_empty() {
        "resonance".to_string()
    } else {
        input
    };

    println!("\n  Generating universe for: \"{input}\"\n");

    let report = resonance::use_cases::experiments::personal::run(&input);
    resonance::use_cases::presenters::terminal::print_report(&report);
    println!();
}
