//! Cancer Therapy Simulator — resistance dynamics under chemotherapy.
//!
//! Usage:
//!   cargo run --release --bin cancer_therapy
//!   cargo run --release --bin cancer_therapy -- --potency 5 --bandwidth 30 --gens 200
//!   cargo run --release --bin cancer_therapy -- --intermittent 10 --worlds 500

use resonance::use_cases::cli::{parse_arg, parse_arg_f32, find_arg};
use resonance::use_cases::experiments::cancer_therapy::{self, TherapyConfig, TherapySnapshot};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = TherapyConfig {
        normal_count:        parse_arg(&args, "--normals", 30) as u8,
        cancer_count:        parse_arg(&args, "--cancers", 15) as u8,
        normal_freq:         parse_arg(&args, "--normal-freq", 250) as f32,
        cancer_freq:         parse_arg(&args, "--cancer-freq", 400) as f32,
        drug_target_freq:    parse_arg(&args, "--target-freq", 400) as f32,
        drug_potency:        parse_arg_f32(&args, "--potency", 2.0),
        drug_bandwidth:      parse_arg_f32(&args, "--bandwidth", 50.0),
        treatment_start_gen: parse_arg(&args, "--start", 5) as u32,
        treatment_pause_gens: parse_arg(&args, "--intermittent", 0) as u32,
        worlds:              parse_arg(&args, "--worlds", 100) as usize,
        generations:         parse_arg(&args, "--gens", 100) as u32,
        ticks_per_gen:       parse_arg(&args, "--ticks", 300) as u32,
        seed:                parse_arg(&args, "--seed", 42) as u64,
        ..Default::default()
    };

    println!("╔══════════════════════════════════════════════════╗");
    println!("║  RESONANCE — Cancer Therapy Simulation            ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!("║  Normal cells: {:<6} freq: {:<6.0} Hz            ║", config.normal_count, config.normal_freq);
    println!("║  Cancer cells: {:<6} freq: {:<6.0} Hz            ║", config.cancer_count, config.cancer_freq);
    println!("║  Drug target:  {:<6.0} Hz  potency: {:<4.1} qe/tick ║", config.drug_target_freq, config.drug_potency);
    println!("║  Bandwidth:    {:<6.0} Hz  ({})      ║",
        config.drug_bandwidth,
        if config.drug_bandwidth < 30.0 { "targeted" } else { "cytotoxic" });
    println!("║  Treatment:    gen {:<4} {}           ║",
        config.treatment_start_gen,
        if config.treatment_pause_gens == 0 { "continuous".to_string() }
        else { format!("{} on / {} off", config.treatment_pause_gens, config.treatment_pause_gens) });
    println!("║  Worlds: {:<6} Gens: {:<6} Ticks: {:<6}       ║",
        config.worlds, config.generations, config.ticks_per_gen);
    println!("╚══════════════════════════════════════════════════╝\n");

    let report = cancer_therapy::run(&config);

    // Timeline
    println!("  Gen │ Cancer │ Normal │ Freq mean │ Freq σ │ Resist │ Divers │ Drug");
    println!("──────┼────────┼────────┼───────────┼────────┼────────┼────────┼─────");

    for snap in &report.timeline {
        let g = snap.generation;
        if g < 10 || g % 10 == 0 || g == config.generations - 1 {
            println!(
                " {:>4} │ {:>6.1} │ {:>6.1} │   {:>6.1}  │ {:>5.1}  │ {:>5.2}  │ {:>5.1}  │  {}",
                g,
                snap.cancer_alive_mean,
                snap.normal_alive_mean,
                snap.cancer_freq_mean,
                snap.cancer_freq_std,
                snap.resistance_index,
                snap.clonal_diversity,
                if snap.drug_active { "ON" } else { "off" },
            );
        }
    }

    // Summary
    println!("\n══════════════════════════════════════════════════════");
    println!("  Time: {}ms", report.wall_time_ms);

    if report.tumor_eliminated {
        println!("  TUMOR ELIMINATED (cancer < 1.0 at some point)");
    } else {
        println!("  TUMOR PERSISTED");
    }

    if let Some(g) = report.generations_to_resistance {
        println!("  RESISTANCE EMERGED at generation {g}");
        println!("  (frequency drifted beyond drug bandwidth)");
    } else {
        println!("  No full resistance detected in {} generations", config.generations);
    }

    if let Some(g) = report.relapse_gen {
        println!("  RELAPSE at generation {g} (tumor returned after response)");
    }

    // CSV export
    if let Some(out_path) = find_arg(&args, "--out") {
        let csv = therapy_timeline_to_csv(&report.timeline);
        match std::fs::write(&out_path, &csv) {
            Ok(()) => println!("  Exported {} rows to {out_path}", report.timeline.len()),
            Err(e) => eprintln!("  Export failed: {e}"),
        }
    }

    // Calibration guidance
    println!("\n  CALIBRATION NOTES:");
    println!("  ─────────────────────────────────────────────────");
    println!("  1 generation ≈ 1 cell doubling time (~24-72h for most tumors)");
    println!("  drug_potency=2.0 ≈ IC50 (half-maximal inhibitory concentration)");
    println!("  bandwidth=50 Hz ≈ moderate selectivity (TKI-like)");
    println!("  bandwidth=20 Hz ≈ high selectivity (antibody-like)");
    println!("  bandwidth=100 Hz ≈ low selectivity (alkylating agent-like)");
    println!("  resistance_index > 1.0 ≈ clinically resistant");
    println!();
}

/// Serializa timeline de terapia como CSV. Stateless, zero IO.
fn therapy_timeline_to_csv(timeline: &[TherapySnapshot]) -> String {
    let mut out = String::with_capacity(timeline.len() * 120);
    out.push_str("gen,cancer_alive,normal_alive,freq_mean,freq_std,resistance,diversity,drug_active,drug_drain,potency\n");
    for s in timeline {
        out.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.4},{:.2},{},{:.4},{:.4}\n",
            s.generation, s.cancer_alive_mean, s.normal_alive_mean,
            s.cancer_freq_mean, s.cancer_freq_std, s.resistance_index,
            s.clonal_diversity, s.drug_active as u8, s.total_drug_drain, s.effective_potency,
        ));
    }
    out
}
