//! Adaptive therapy controller — the loop that controls tumor growth.
//!
//! Usage: cargo run --release --bin adaptive_therapy

use resonance::use_cases::experiments::pathway_inhibitor_exp::{
    self, BozicValidationConfig,
};

fn main() {
    let config = BozicValidationConfig {
        worlds: 20, generations: 30, ticks_per_gen: 80,
        ..Default::default()
    };

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Adaptive Therapy Controller                                ║");
    println!("║  Profile → Attack → Predict Escape → Close → Adapt          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let report = pathway_inhibitor_exp::run_adaptive(&config);

    println!("gen | alive | efficiency | growth | drugs | decision");
    println!("----|-------|------------|--------|-------|----------");
    for (i, (snap, dec)) in report.snapshots.iter().zip(report.decisions.iter()).enumerate() {
        println!("{:>3} | {:>5.1} | {:>10.3} | {:>+6.3} | {:>5} | {}",
            i, snap.alive_count, snap.mean_efficiency, snap.growth_rate,
            report.drug_count_timeline[i], dec.rationale);
    }

    println!();
    println!("=== Summary ===");
    println!("  final stability: {}", report.final_stability);
    if let Some(g) = report.stability_gen {
        println!("  stability at gen: {g}");
    }
    println!("  max drugs used:  {}", report.drug_count_timeline.iter().max().unwrap_or(&0));
    println!("  wall time:       {} ms", report.wall_time_ms);

    // Show drug protocol
    println!();
    println!("=== Drug Protocol (decisions with changes) ===");
    let mut prev_count = 0;
    for (i, dec) in report.decisions.iter().enumerate() {
        if dec.inhibitors.len() != prev_count || i == 0 {
            if !dec.inhibitors.is_empty() {
                let drugs: Vec<String> = dec.inhibitors.iter()
                    .map(|(f, c)| format!("{:.0}Hz@{:.2}", f, c))
                    .collect();
                println!("  gen {:>2}: [{}] — {}", i, drugs.join(", "), dec.rationale);
            } else {
                println!("  gen {:>2}: [none] — {}", i, dec.rationale);
            }
            prev_count = dec.inhibitors.len();
        }
    }
}
