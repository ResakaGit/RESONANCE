//! Bozic et al. 2013 validation — mono vs combination therapy.
//!
//! Tests the prediction: combination therapy has exponential advantage
//! over monotherapy for resistance prevention.
//!
//! Usage: cargo run --release --bin bozic_validation

use resonance::use_cases::experiments::pathway_inhibitor_exp::{
    self, BozicValidationConfig,
};

fn main() {
    let config = BozicValidationConfig::default();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Bozic et al. 2013 Validation — Mono vs Combo Therapy      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Tumor: {} cells @ {:.0} Hz (spread ±{:.0} Hz)",
        config.tumor_count, config.tumor_freq, config.tumor_spread);
    println!("  Drug A: {:.0} Hz, conc={}, Ki={:.2}",
        config.drug_a_freq, config.drug_a_conc, config.drug_a_ki);
    println!("  Drug B: {:.0} Hz, conc={}, Ki={:.2}",
        config.drug_b_freq, config.drug_b_conc, config.drug_b_ki);
    println!("  {} worlds × {} gens × {} ticks",
        config.worlds, config.generations, config.ticks_per_gen);
    println!();

    let result = pathway_inhibitor_exp::run_bozic_validation(&config);

    // Print timelines
    println!("gen | no_drug | mono_A | mono_B | combo  | 2×A");
    println!("----|---------|--------|--------|--------|------");
    let n = result.no_drug.efficiency_timeline.len();
    for i in 0..n {
        println!("{:>3} | {:>7.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3}",
            i,
            result.no_drug.efficiency_timeline[i],
            result.mono_a.efficiency_timeline[i],
            result.mono_b.efficiency_timeline[i],
            result.combo_ab.efficiency_timeline[i],
            result.double_a.efficiency_timeline[i],
        );
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Results                                                    ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    let arms = [&result.no_drug, &result.mono_a, &result.mono_b, &result.combo_ab, &result.double_a];
    println!("║  {:>10} | {:>6} | {:>5} | {:>4} | {:>3}               ║", "arm", "eff", "alive", "res?", "gen");
    println!("║  -----------|--------|-------|------|----               ║");
    for arm in &arms {
        println!("║  {:>10} | {:>6.3} | {:>5.1} | {:>4} | {:>3}               ║",
            arm.label, arm.final_efficiency, arm.final_alive,
            if arm.resistance_detected { "YES" } else { "no" },
            arm.resistance_gen.map(|g| format!("{g}")).unwrap_or_else(|| "-".to_string()));
    }
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Bozic predictions
    println!();
    println!("=== Bozic 2013 Predictions vs Resonance ===");
    println!();

    let mono_eff = result.mono_a.final_efficiency;
    let combo_eff = result.combo_ab.final_efficiency;
    let double_eff = result.double_a.final_efficiency;
    let no_drug_eff = result.no_drug.final_efficiency;

    let mono_suppression = 1.0 - mono_eff / no_drug_eff.max(0.001);
    let combo_suppression = 1.0 - combo_eff / no_drug_eff.max(0.001);
    let double_suppression = 1.0 - double_eff / no_drug_eff.max(0.001);

    println!("  Suppression (efficiency reduction vs no-drug):");
    println!("    mono_A:    {:.1}%", mono_suppression * 100.0);
    println!("    mono_B:    {:.1}%", (1.0 - result.mono_b.final_efficiency / no_drug_eff.max(0.001)) * 100.0);
    println!("    combo_AB:  {:.1}%", combo_suppression * 100.0);
    println!("    double_A:  {:.1}%", double_suppression * 100.0);
    println!();

    // Key test: combo > double dose?
    let combo_advantage = combo_suppression > double_suppression;
    println!("  Bozic prediction: combo > double_dose?  {}",
        if combo_advantage { "✓ CONFIRMED" } else { "✗ NOT CONFIRMED" });
    println!("    combo suppression:  {:.1}%", combo_suppression * 100.0);
    println!("    double suppression: {:.1}%", double_suppression * 100.0);
    println!();
    println!("  Wall time: {} ms", result.wall_time_ms);
}
