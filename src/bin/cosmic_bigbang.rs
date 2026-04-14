//! CT-2 validation — Big Bang y formación de clusters S0.
//! CT-2 validation — Big Bang and cluster formation at S0.
//!
//! Uso: cargo run --release --bin cosmic_bigbang -- [seed] [ticks] [N]

use resonance::cosmic::scale_manager::CosmicWorld;
use resonance::cosmic::scales::cosmological::{
    cosmo_tick, detect_clusters, init_big_bang, CosmoConfig,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42);
    let ticks: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2000);
    let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(256);

    let mut cfg = CosmoConfig::default_with_seed(seed);
    cfg.n_initial_clusters = n;

    let mut world = init_big_bang(&cfg);
    let qe_initial = world.total_qe();

    println!("=== CT-2 Big Bang ===");
    println!("  seed       = {seed}");
    println!("  N          = {n}");
    println!("  total_qe   = {qe_initial:.2}");
    println!("  expansion  = {:.4}", cfg.expansion_rate);
    println!("  dissipate  = {:.4}", cfg.dissipation_rate);
    println!();

    let sample_every = ticks.max(10) / 10;
    for t in 0..ticks {
        cosmo_tick(&mut world, &cfg);
        if t % sample_every == 0 || t == ticks - 1 {
            let alive = world.n_alive();
            let qe = world.total_qe();
            let clusters = detect_clusters(&world, adaptive_link(&world));
            let cons = qe / qe_initial;
            println!(
                "  t={t:>5}  alive={alive:>4}  qe={qe:>10.2}  cons={cons:.4}  clusters={:>3}",
                clusters.len(),
            );
        }
    }

    let final_clusters = detect_clusters(&world, adaptive_link(&world));
    println!();
    println!("=== Final cluster stats ===");
    let mut report: Vec<_> = final_clusters.iter().collect();
    report.sort_by(|a, b| b.total_qe.partial_cmp(&a.total_qe).unwrap_or(std::cmp::Ordering::Equal));
    for (i, c) in report.iter().take(10).enumerate() {
        println!(
            "  #{i:>2}  members={:>4}  qe={:>9.2}  freq_var={:>8.2}",
            c.n_members, c.total_qe, c.freq_variance,
        );
    }
    println!();
    println!(
        "  conservation_ratio = {:.6} (Axiom 5: monotone ≤ 1.0)",
        world.total_qe() / qe_initial,
    );
}

/// Link distance adaptativo: 1/4 de la mediana de distancias al centroide.
/// Observational scale, not physical — evita hardcodear escala espacial.
fn adaptive_link(world: &CosmicWorld) -> f64 {
    let alive: Vec<_> = world.entities.iter().filter(|e| e.alive).collect();
    if alive.len() < 2 { return 0.0; }
    let mut cx = 0.0; let mut cy = 0.0; let mut cz = 0.0;
    for e in &alive {
        cx += e.position[0]; cy += e.position[1]; cz += e.position[2];
    }
    let n = alive.len() as f64;
    let (cx, cy, cz) = (cx / n, cy / n, cz / n);
    let mut dists: Vec<f64> = alive
        .iter()
        .map(|e| {
            let dx = e.position[0] - cx;
            let dy = e.position[1] - cy;
            let dz = e.position[2] - cz;
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .collect();
    dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dists[dists.len() / 2] * 0.25
}
