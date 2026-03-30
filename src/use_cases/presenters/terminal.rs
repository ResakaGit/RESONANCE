//! Terminal presenter — print ExperimentReport to stdout.

use crate::use_cases::ExperimentReport;
use crate::use_cases::cli::archetype_label;
use crate::use_cases::experiments::fermi::FermiReport;
use crate::use_cases::experiments::convergence::ConvergenceReport;
use crate::use_cases::experiments::speciation::SpeciationReport;
use crate::use_cases::experiments::cambrian::CambrianReport;
use crate::use_cases::experiments::debate::DebateReport;
use crate::use_cases::experiments::versus::VersusResult;
use crate::use_cases::experiments::fossil::FossilRecord;

/// Print a standard experiment report.
pub fn print_report(report: &ExperimentReport) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  Universe: {:<29}║", report.preset_name);
    println!("╠══════════════════════════════════════════╣");
    println!("║  Seed: {:<10} Worlds: {:<6} Gens: {:<4}║", report.seed, report.world_count, report.generations);
    println!("║  Ticks/gen: {:<6} Time: {:<6}ms        ║", report.ticks_per_gen, report.wall_time_ms);
    println!("╚══════════════════════════════════════════╝");
    println!();

    if let Some(last) = report.history.last() {
        println!("  Final: fitness={:.3} diversity={:.3} species={:.1}",
            last.best_fitness, last.diversity, last.species_mean);
        println!("  Complexity: genes={:.1} metabolic={:.0}% protein={:.0}%",
            last.gene_count_mean,
            last.metabolic_graph_rate * 100.0,
            last.protein_function_rate * 100.0);
    }
    println!("  {} genomes evolved.\n", report.top_genomes.len());

    for (i, g) in report.top_genomes.iter().enumerate() {
        println!("  #{i:>2}: {:<5} g={:.2} m={:.2} b={:.2} r={:.2}",
            archetype_label(g.archetype),
            g.growth_bias, g.mobility_bias, g.branching_bias, g.resilience);
    }
}

/// Print a Fermi Paradox report.
pub fn print_fermi(report: &FermiReport) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  FERMI PARADOX RESULTS                   ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Universes tested: {:<21}║", report.total_universes);
    println!("║  With life:        {:<4} ({:.1}%)          ║",
        report.with_life, report.life_probability * 100.0);
    println!("║  With complex life:{:<4} ({:.1}%)          ║",
        report.with_complex_life, report.complex_probability * 100.0);
    println!("╚══════════════════════════════════════════╝");
}

/// Print a convergence report.
pub fn print_convergence(report: &ConvergenceReport) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  CONVERGENT EVOLUTION ANALYSIS            ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Seeds tested:     {:<21}║", report.n_seeds);
    println!("║  Mean distance:    {:<21.3}║", report.mean_distance);
    println!("║  Min distance:     {:<21.3}║", report.min_distance);
    println!("║  Max distance:     {:<21.3}║", report.max_distance);
    println!("║  Convergence rate: {:.1}%                  ║", report.convergence_rate * 100.0);
    println!("╚══════════════════════════════════════════╝");
}

/// Print a speciation report.
pub fn print_speciation(report: &SpeciationReport) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  ALLOPATRIC SPECIATION ANALYSIS           ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Preset:     {:<27}║", report.preset_name);
    println!("║  Generations:{:<27}║", report.generations);
    println!("║  Freq A:     {:<27.1}║", report.mean_freq_a);
    println!("║  Freq B:     {:<27.1}║", report.mean_freq_b);
    println!("║  Interference:{:<26.3}║", report.cross_interference);
    println!("║  Speciated:  {:<27}║", if report.speciated { "YES" } else { "NO" });
    println!("╚══════════════════════════════════════════╝");
}

/// Print a Cambrian Explosion report.
pub fn print_cambrian(report: &CambrianReport) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  CAMBRIAN EXPLOSION ANALYSIS              ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Preset:       {:<25}║", report.preset_name);
    println!("║  Generations:  {:<25}║", report.generations);
    println!("║  Max Δdiversity:{:<24.3}║", report.max_diversity_delta);
    if let Some(explosion_at) = report.explosion_gen {
        println!("║  Explosion gen:{:<25}║", explosion_at);
    }
    println!("║  Detected:    {:<26}║",
        if report.explosion_detected { "YES — Cambrian event!" } else { "NO" });
    println!("╚══════════════════════════════════════════╝");

    // Mini diversity curve
    if report.diversity_curve.len() >= 2 {
        println!("\n  Diversity curve (sampled):");
        let step = (report.diversity_curve.len() / 10).max(1);
        for (i, &d) in report.diversity_curve.iter().enumerate() {
            if i % step == 0 || i == report.diversity_curve.len() - 1 {
                let bar_len = (d * 30.0) as usize;
                println!("  gen {i:>4}: {d:.3} {}", "█".repeat(bar_len.min(30)));
            }
        }
    }
}

/// Print a Debate Settler report.
pub fn print_debate(report: &DebateReport) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  COOPERATION EMERGENCE ANALYSIS           ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Preset:         {:<23}║", report.preset_name);
    println!("║  Seeds tested:   {:<23}║", report.n_seeds);
    println!("║  Generations:    {:<23}║", report.generations);
    println!("║  Life emerged:   {:.1}%{:<19}║", report.life_rate * 100.0, "");
    println!("║  Complexity grew:{:.1}%{:<19}║", report.complexity_rate * 100.0, "");
    println!("║  Cooperation:   {:.1}%{:<20}║", report.cooperation_signal * 100.0, "");
    println!("╚══════════════════════════════════════════╝");
}

/// Print a Fossil Record summary.
pub fn print_fossil(record: &FossilRecord) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  FOSSIL RECORD                            ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Preset:      {:<26}║", record.preset_name);
    println!("║  Seed:        {:<26}║", record.seed);
    println!("║  Generations: {:<26}║", record.fossils.len());
    println!("║  Time:        {:<22}ms  ║", record.wall_time_ms);
    println!("╚══════════════════════════════════════════╝");

    let step = (record.fossils.len() / 15).max(1);
    println!("\n  Timeline (sampled):");
    for (i, f) in record.fossils.iter().enumerate() {
        if i % step == 0 || i == record.fossils.len() - 1 {
            println!("  gen {:>4}: {:<5} fit={:.3} div={:.3} spp={:.1}",
                f.generation, archetype_label(f.genome.archetype),
                f.fitness, f.diversity, f.species);
        }
    }
}

/// Print a Versus Arena result.
pub fn print_versus(result: &VersusResult) {
    println!("╔══════════════════════════════════════════╗");
    println!("║  VERSUS ARENA RESULTS                     ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Team A: {} genomes   qe={:<14.1}║", result.survivors_a, result.qe_a);
    println!("║  Team B: {} genomes   qe={:<14.1}║", result.survivors_b, result.qe_b);
    println!("║  Winner: {:<31}║", result.winner);
    println!("╚══════════════════════════════════════════╝");
}
