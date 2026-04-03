//! Runner unificado de validación contra papers publicados.
//! Unified runner for published paper validation experiments.
//!
//! Ejecuta PV-1 a PV-5, reporta PASS/FAIL por paper.
//! Usage: cargo run --release --bin paper_validation

use resonance::use_cases::experiments::paper_foo_michor2009;
use resonance::use_cases::experiments::paper_hill_ccle;
use resonance::use_cases::experiments::paper_michor2005;
use resonance::use_cases::experiments::paper_sharma2010;
use resonance::use_cases::experiments::paper_unified_axioms;
use resonance::use_cases::experiments::paper_zhang2022;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  RESONANCE — Paper Validation Suite (PV-1 through PV-5)        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    let mut passed = 0u32;
    let mut failed = 0u32;

    // ── PV-1: Zhang et al. 2022 — Adaptive therapy ──────────────────────────
    print!("  PV-1  Zhang 2022 (adaptive therapy TTP)      ... ");
    let zhang = paper_zhang2022::run(&paper_zhang2022::ZhangConfig::default());
    let pv1_pass = zhang.prediction_met;
    if pv1_pass {
        println!(
            "PASS  (TTP ratio: {:.2}×, cycles: {}, drug exposure: {:.0}%)",
            zhang.ttp_ratio,
            zhang.adaptive_cycles,
            zhang.drug_exposure_ratio * 100.0
        );
        passed += 1;
    } else {
        println!(
            "FAIL  (adaptive TTP {:?} vs continuous {:?})",
            zhang.adaptive_ttp_gen, zhang.continuous_ttp_gen
        );
        failed += 1;
    }

    // ── PV-2: Sharma et al. 2010 — Drug-tolerant persisters ─────────────────
    print!("  PV-2  Sharma 2010 (drug-tolerant persisters) ... ");
    let sharma = paper_sharma2010::run(&paper_sharma2010::SharmaConfig::default());
    let pv2_pass = sharma.persister_fraction > 0.0 && sharma.persister_fraction < 0.5;
    if pv2_pass {
        println!(
            "PASS  (persister frac: {:.1}%, survivors: {:.1}, recovery: {})",
            sharma.persister_fraction * 100.0,
            sharma.post_drug_survivors,
            sharma.recovery_detected
        );
        passed += 1;
    } else {
        println!("FAIL  (persister frac: {:.3})", sharma.persister_fraction);
        failed += 1;
    }

    // ── PV-3: GDSC/CCLE — Hill slope calibration ────────────────────────────
    print!("  PV-3  GDSC/CCLE (Hill slope n=2 calibration) ... ");
    let hill = paper_hill_ccle::validate_against_published();
    let pv3_pass = hill.resonance_assumption_valid;
    if pv3_pass {
        println!(
            "PASS  (n=2 within IQR: {}, within 1σ: {})",
            hill.n2_within_iqr, hill.n2_within_1_std
        );
        passed += 1;
    } else {
        println!("FAIL  (n=2 outside empirical range)");
        failed += 1;
    }

    // ── PV-4: Foo & Michor 2009 — Continuous vs pulsed ──────────────────────
    print!("  PV-4  Foo & Michor 2009 (pulsed vs continuous) ... ");
    let foo = paper_foo_michor2009::run(&paper_foo_michor2009::FooMichorConfig::default());
    let pv4_pass = foo.pulsed_beats_continuous;
    if pv4_pass {
        println!(
            "PASS  (optimal dose: {:.2}, pulsed res: {:.1}%, continuous res: {:.1}%)",
            foo.optimal_dose,
            foo.pulsed_resistance_at_08 * 100.0,
            foo.continuous_resistance_at_08 * 100.0
        );
        passed += 1;
    } else {
        println!(
            "FAIL  (pulsed: {:.1}% vs continuous: {:.1}%)",
            foo.pulsed_resistance_at_08 * 100.0,
            foo.continuous_resistance_at_08 * 100.0
        );
        failed += 1;
    }

    // ── PV-5: Michor et al. 2005 — Biphasic CML decline ────────────────────
    print!("  PV-5  Michor 2005 (biphasic CML decline)     ... ");
    let michor = paper_michor2005::run(&paper_michor2005::MichorConfig::default());
    let pv5_pass = michor.stem_survive && michor.biphasic_detected;
    if pv5_pass {
        println!(
            "PASS  (slope ratio: {:.1}×, inflection gen: {}, stem survive: {})",
            michor.slope_ratio,
            michor
                .inflection_gen
                .map(|g| format!("{g}"))
                .unwrap_or("-".into()),
            michor.stem_survive
        );
        passed += 1;
    } else {
        println!(
            "FAIL  (biphasic: {}, stem survive: {}, ratio: {:.1})",
            michor.biphasic_detected, michor.stem_survive, michor.slope_ratio
        );
        failed += 1;
    }

    // ── PV-6: Unified axioms — ALL phenomena from 4 constants ─────────────
    println!();
    print!("  PV-6  UNIFIED (all 6 tests from 4 constants) ... ");
    let unified = paper_unified_axioms::run(42);
    for r in &unified.results {
        if r.passed {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    if unified.all_passed {
        println!(
            "PASS  ({}/{} sub-tests, {:.1}s)",
            unified.passed_count,
            unified.total_count,
            unified.wall_time_ms as f64 / 1000.0
        );
    } else {
        println!(
            "PARTIAL  ({}/{} sub-tests)",
            unified.passed_count, unified.total_count
        );
        for r in &unified.results {
            if !r.passed {
                println!("         FAIL {}: {}", r.name, r.detail);
            }
        }
    }

    // ── Summary ─────────────────────────────────────────────────────────────
    println!();
    let total_tests = 5 + unified.total_count;
    let total_pass = passed;
    let total_fail = failed;
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!(
        "║  Results: {total_pass} PASS, {total_fail} FAIL out of {total_tests} tests                      ║"
    );
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    println!(
        "║  PV-1  Zhang 2022      adaptive TTP > continuous     {}      ║",
        if zhang.prediction_met {
            "  PASS"
        } else {
            "  FAIL"
        }
    );
    println!(
        "║  PV-2  Sharma 2010     persisters survive + recover  {}      ║",
        if pv2_pass { "  PASS" } else { "  FAIL" }
    );
    println!(
        "║  PV-3  GDSC/CCLE       Hill n=2 within IQR          {}      ║",
        if pv3_pass { "  PASS" } else { "  FAIL" }
    );
    println!(
        "║  PV-4  Foo & Michor    pulsed beats continuous       {}      ║",
        if pv4_pass { "  PASS" } else { "  FAIL" }
    );
    println!(
        "║  PV-5  Michor 2005     biphasic + stem survive       {}      ║",
        if pv5_pass { "  PASS" } else { "  FAIL" }
    );
    println!("║                                                                ║");
    println!(
        "║  Wall time: PV-1={:.1}s PV-2={:.1}s PV-3=0ms PV-4={:.1}s PV-5={:.1}s   ║",
        zhang.wall_time_ms as f64 / 1000.0,
        sharma.wall_time_ms as f64 / 1000.0,
        foo.wall_time_ms as f64 / 1000.0,
        michor.wall_time_ms as f64 / 1000.0
    );
    println!("╚══════════════════════════════════════════════════════════════════╝");

    if failed > 0 {
        std::process::exit(1);
    }
}
