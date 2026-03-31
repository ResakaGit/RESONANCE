//! Pathway inhibitor experiment — CLI binary.
//!
//! Runs pathway inhibition + ablation, prints timeline + resistance detection.
//! Default params: ~2-5 min on Mac (100 worlds × 80 gens × 200 ticks).
//!
//! Usage: cargo run --release --bin pathway_inhibitor [-- --worlds 50 --gens 40]

use resonance::use_cases::experiments::pathway_inhibitor_exp::{self, InhibitorConfig};
use resonance::use_cases::cli::parse_arg;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = InhibitorConfig {
        worlds:          parse_arg(&args, "--worlds", 50) as usize,
        generations:     parse_arg(&args, "--gens", 40) as u32,
        ticks_per_gen:   parse_arg(&args, "--ticks", 150) as u32,
        seed:            parse_arg(&args, "--seed", 42) as u64,
        wildtype_count:  parse_arg(&args, "--wt", 30) as u8,
        resistant_count: parse_arg(&args, "--res", 5) as u8,
        ..Default::default()
    };

    println!("=== Pathway Inhibitor Experiment ===");
    println!("  worlds={}, gens={}, ticks/gen={}, seed={}",
        config.worlds, config.generations, config.ticks_per_gen, config.seed);
    println!("  wildtype={} @ {} Hz, resistant={} @ {} Hz",
        config.wildtype_count, config.wildtype_freq, config.resistant_count, config.resistant_freq);
    println!("  drug: {:?} targeting {:?} @ {} Hz, conc={}, Ki={}",
        config.drug_mode, config.target_role, config.drug_frequency,
        config.drug_concentration, config.drug_ki);
    println!("  treatment starts gen {}", config.treatment_start_gen);
    println!();

    let report = pathway_inhibitor_exp::run(&config);

    println!("gen | alive | wildtype | resistant | efficiency | expr[0] | drug | cost");
    println!("----|-------|----------|-----------|------------|---------|------|-----");
    for s in &report.timeline {
        println!("{:>3} | {:>5.1} | {:>8.1} | {:>9.1} | {:>10.3} | {:>7.3} | {:>4} | {:.3}",
            s.generation, s.alive_mean, s.wildtype_alive_mean,
            s.resistant_alive_mean, s.mean_efficiency,
            s.mean_expression_dim0,
            if s.drug_active { "ON" } else { "off" },
            s.total_inhibition_cost);
    }

    println!();
    println!("=== Results ===");
    println!("  resistance detected: {}", report.resistance_detected);
    if let Some(g) = report.resistance_gen {
        println!("  resistance at gen:   {g}");
    }
    println!("  compensation:        {}", report.compensation_detected);
    println!("  wall time:           {} ms", report.wall_time_ms);

    // Ablation: 3 concentrations
    println!();
    println!("=== Ablation: concentration sweep ===");
    let ablation = pathway_inhibitor_exp::ablate_concentration(
        &config, &[0.0, 0.4, 0.8]);
    for (i, r) in ablation.iter().enumerate() {
        let last = r.timeline.last().unwrap();
        println!("  conc={:.1} → alive={:.1}, efficiency={:.3}, resistance={}",
            [0.0, 0.4, 0.8][i], last.alive_mean, last.mean_efficiency, r.resistance_detected);
    }
}
