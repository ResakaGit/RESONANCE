//! Particle Lab — observe emergent molecules from charged particles.
//!
//! Usage:
//!   cargo run --release --bin particle_lab
//!   cargo run --release --bin particle_lab -- --positive 20 --negative 20 --snapshots 100

use resonance::use_cases::cli::parse_arg;
use resonance::use_cases::experiments::particle_lab::{self, ParticleLabConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = ParticleLabConfig {
        positive_count: parse_arg(&args, "--positive", 15) as u8,
        negative_count: parse_arg(&args, "--negative", 15) as u8,
        freq_spread: parse_arg(&args, "--freq-spread", 100) as f32,
        arena_size: parse_arg(&args, "--arena", 10) as f32,
        ticks_per_snapshot: parse_arg(&args, "--ticks", 20) as u32,
        snapshots: parse_arg(&args, "--snapshots", 50) as u32,
        seed: parse_arg(&args, "--seed", 42) as u64,
        ..Default::default()
    };

    println!("╔══════════════════════════════════════════════════╗");
    println!("║  RESONANCE — Particle Lab                         ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!("║  Positive: {:<6} Negative: {:<6}               ║", config.positive_count, config.negative_count);
    println!("║  Freq spread: {:<6.0} Hz  Arena: {:<4.0}             ║", config.freq_spread, config.arena_size);
    println!("║  Snapshots: {:<6} Ticks/snap: {:<6}            ║", config.snapshots, config.ticks_per_snapshot);
    println!("╚══════════════════════════════════════════════════╝\n");

    let report = particle_lab::run(&config);

    println!("  Step │ Bonds │ Types │  KE mean │  PE mean │ Σcharge");
    println!("───────┼───────┼───────┼──────────┼──────────┼────────");
    for s in &report.timeline {
        if s.step < 5 || s.step % 10 == 0 || s.step == config.snapshots - 1 {
            println!("  {:>4} │  {:>4} │  {:>4} │ {:>8.4} │ {:>8.4} │ {:>6.2}",
                s.step, s.bond_count, s.molecule_types,
                s.mean_kinetic_energy, s.mean_potential_energy, s.total_charge);
        }
    }

    println!("\n══════════════════════════════════════════════════════");
    println!("  Time: {}ms", report.wall_time_ms);
    println!("  Final bonds: {}", report.timeline.last().map(|s| s.bond_count).unwrap_or(0));
    println!("  Final molecule types: {}", report.timeline.last().map(|s| s.molecule_types).unwrap_or(0));
    println!("  Final molecules:");
    for (i, m) in report.final_molecules.iter().enumerate() {
        println!("    #{i}: {} particles, charge={:.1}, freq={:.0} Hz",
            m.particle_count, m.total_charge, m.mean_frequency);
    }
    println!();
}
