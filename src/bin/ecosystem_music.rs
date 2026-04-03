//! D3: Ecosystem Music — sonification of evolved creatures as WAV.
//!
//! Each creature = a sine wave at its frequency. Interference = chords.
//! Axiom 8 made literal: the ecosystem sounds.
//!
//! Usage: `cargo run --release --bin ecosystem_music -- --out ecosystem.wav`
//! Or with custom evolution: `cargo run --release --bin ecosystem_music -- --gens 200 --duration 15`

use resonance::batch::bridge;
use resonance::use_cases::cli::{archetype_label, find_arg, parse_arg};
use resonance::use_cases::experiments::sonification::{self, SonificationConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gens = parse_arg(&args, "--gens", 100);
    let worlds = parse_arg(&args, "--worlds", 200);
    let ticks = parse_arg(&args, "--ticks", 500);
    let seed = parse_arg(&args, "--seed", 42);
    let duration = parse_arg(&args, "--duration", 10);
    let out_path = find_arg(&args, "--out").unwrap_or_else(|| "ecosystem.wav".to_string());

    println!("╔══════════════════════════════════════════╗");
    println!("║  RESONANCE — Ecosystem Music              ║");
    println!("╚══════════════════════════════════════════╝\n");

    println!("  Evolving creatures (seed={seed}, gens={gens})...");
    let report = resonance::use_cases::evolve_with(
        &resonance::use_cases::presets::EARTH,
        seed as u64,
        worlds as usize,
        gens as u32,
        ticks as u32,
        12,
    );

    println!("  {} genomes evolved.\n", report.top_genomes.len());
    println!("  Voices:");
    for (i, g) in report.top_genomes.iter().enumerate() {
        let freq = bridge::genome_to_components(g).2.frequency_hz();
        let config = SonificationConfig::default();
        let audio_freq = freq / config.freq_divisor;
        println!(
            "    #{i}: {:<5} freq={:.0} Hz → audio={:.0} Hz  amp={:.2}",
            archetype_label(g.archetype),
            freq,
            audio_freq,
            g.resilience * 0.5 + 0.1
        );
    }

    println!("\n  Generating {duration}s WAV at 44100 Hz...");
    let config = SonificationConfig {
        duration_secs: duration as f32,
        ..Default::default()
    };
    let wav = sonification::genomes_to_wav(&report.top_genomes, &config);

    match std::fs::write(&out_path, &wav) {
        Ok(()) => {
            let size_kb = wav.len() / 1024;
            println!("  Saved {out_path} ({size_kb} KB, {duration}s mono 16-bit)\n");
            println!("  Play with: aplay {out_path}  (Linux)");
            println!("             afplay {out_path}  (macOS)");
        }
        Err(e) => println!("  Error: {e}"),
    }
    println!();
}
