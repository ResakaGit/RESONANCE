//! CLI: Run batch evolution and print live progress.
//!
//! Usage: `cargo run --bin evolve`
//! Or with args: `cargo run --bin evolve -- --worlds 1000 --gens 500 --ticks 500`

use resonance::batch::batch::BatchConfig;
use resonance::batch::bridge;
use resonance::batch::harness::GeneticHarness;
use resonance::use_cases::cli::{parse_arg, archetype_label, trophic_label};
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let worlds = parse_arg(&args, "--worlds", 1000);
    let gens   = parse_arg(&args, "--gens", 500);
    let ticks  = parse_arg(&args, "--ticks", 500);
    let seed   = parse_arg(&args, "--seed", 42);

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  RESONANCE — Batch Evolution                           ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Worlds: {worlds:<8} Ticks/eval: {ticks:<8} Seed: {seed:<8} ║");
    println!("║  Generations: {gens:<6} Elite: 10%   Mutation σ: 0.05   ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let config = BatchConfig {
        world_count: worlds as usize,
        ticks_per_eval: ticks as u32,
        initial_entities: 12,
        max_generations: gens as u32,
        seed: seed as u64,
        ..Default::default()
    };

    let mut harness = GeneticHarness::new(config);
    let start = Instant::now();

    println!("  Gen │  Best  │  Mean  │ Worst  │ Diversity │ Surv │ Spp  │ Genes │ Graph │ Prot │ Time");
    println!("──────┼────────┼────────┼────────┼───────────┼──────┼──────┼───────┼───────┼──────┼──────");

    for g in 0..gens as u32 {
        let gen_start = Instant::now();
        let stats = harness.step();
        let gen_ms = gen_start.elapsed().as_millis();

        if g < 10 || g % 10 == 0 || g == gens as u32 - 1 {
            println!(
                " {:>4} │ {:>6.3} │ {:>6.3} │ {:>6.3} │   {:>6.3}   │{:>5.1} │{:>5.1} │ {:>5.1} │{:>5.0}% │{:>4.0}% │{:>4}ms",
                stats.generation,
                stats.best_fitness,
                stats.mean_fitness,
                stats.worst_fitness,
                stats.diversity,
                stats.survivors_mean,
                stats.species_mean,
                stats.gene_count_mean,
                stats.metabolic_graph_rate * 100.0,
                stats.protein_function_rate * 100.0,
                gen_ms,
            );
        }
    }

    let elapsed = start.elapsed();
    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  Total time: {:.1}s ({:.1} gen/s)",
        elapsed.as_secs_f32(),
        gens as f32 / elapsed.as_secs_f32(),
    );
    println!();

    // Top 10 genomes
    let top = harness.top_genomes(10);
    println!("  TOP 10 GENOMES");
    println!("  ─────────────────────────────────────────────────────────");
    println!("   # │ Arch │ Troph │ Growth │ Mobile │ Branch │ Resil");
    println!("  ───┼──────┼───────┼────────┼────────┼────────┼───────");
    for (i, g) in top.iter().enumerate() {
        println!(
            "  {:>2} │ {:<5} │ {:<5} │ {:>6.3} │ {:>6.3} │ {:>6.3} │ {:>5.3}",
            i + 1, archetype_label(g.archetype), trophic_label(g.trophic_class),
            g.growth_bias, g.mobility_bias, g.branching_bias, g.resilience,
        );
    }

    // Genome distribution analysis
    println!();
    println!("  GENOME ANALYSIS");
    println!("  ─────────────────────────────────────────────────────────");
    if !top.is_empty() {
        let avg_growth: f32   = top.iter().map(|g| g.growth_bias).sum::<f32>() / top.len() as f32;
        let avg_mobility: f32 = top.iter().map(|g| g.mobility_bias).sum::<f32>() / top.len() as f32;
        let avg_branch: f32   = top.iter().map(|g| g.branching_bias).sum::<f32>() / top.len() as f32;
        let avg_resil: f32    = top.iter().map(|g| g.resilience).sum::<f32>() / top.len() as f32;

        println!("  Avg growth:    {:>5.3}  {}", avg_growth, bar(avg_growth));
        println!("  Avg mobility:  {:>5.3}  {}", avg_mobility, bar(avg_mobility));
        println!("  Avg branching: {:>5.3}  {}", avg_branch, bar(avg_branch));
        println!("  Avg resilience:{:>5.3}  {}", avg_resil, bar(avg_resil));

        // Archetype distribution
        let mut arch_counts = [0u32; 5];
        for g in &top { arch_counts[g.archetype.min(4) as usize] += 1; }
        println!();
        println!("  Archetype distribution:");
        for (i, &c) in arch_counts.iter().enumerate() {
            let pct = c as f32 / top.len() as f32 * 100.0;
            println!("    {}: {c} ({pct:.0}%)", archetype_label(i as u8));
        }
    }

    // Evolution curve summary
    println!();
    println!("  EVOLUTION CURVE");
    println!("  ─────────────────────────────────────────────────────────");
    let hist = &harness.history;
    if hist.len() >= 2 {
        let first = &hist[0];
        let last = &hist[hist.len() - 1];
        let improvement = if first.best_fitness > 0.0 {
            (last.best_fitness - first.best_fitness) / first.best_fitness * 100.0
        } else if last.best_fitness > 0.0 {
            f32::INFINITY
        } else {
            0.0
        };
        println!("  Gen 1:    best={:.3} mean={:.3} diversity={:.3}",
            first.best_fitness, first.mean_fitness, first.diversity);
        println!("  Gen {}:  best={:.3} mean={:.3} diversity={:.3}",
            last.generation, last.best_fitness, last.mean_fitness, last.diversity);
        println!("  Improvement: {:.1}%", improvement);

        let diversity_drop = if first.diversity > 0.0 {
            (1.0 - last.diversity / first.diversity) * 100.0
        } else { 0.0 };
        println!("  Diversity reduction: {:.1}% (convergence)", diversity_drop);
    }

    // Save genomes
    let out_path = format!("assets/evolved/seed_{seed}.bin");
    let path = Path::new(&out_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    match bridge::save_genomes(&top, path) {
        Ok(()) => println!("\n  Saved {} genomes to {out_path}", top.len()),
        Err(e) => println!("\n  Failed to save: {e}"),
    }

    println!();
}

fn bar(value: f32) -> String {
    let width = (value * 30.0) as usize;
    format!("[{}{}]", "█".repeat(width), "░".repeat(30 - width))
}

