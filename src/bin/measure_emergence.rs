//! measure_emergence — Prueba de emergencia del Axioma 6 (EM-1.4).
//!
//! Corre el experimento de sincronización de Kuramoto y reporta el order parameter
//! global en la condición acoplada vs ablada (regla local apagada). La emergencia
//! queda probada si `R_final ≫ R_ablated`.
//!
//! Usage: cargo run --release --bin measure_emergence [seed] [n_oscillators] [ticks]

use resonance::blueprint::constants::SYNC_SUPERCRITICAL_RATIO;
use resonance::use_cases::experiments::emergence_sync::{run_sync_experiment, SyncConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0u64);
    let n = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(256usize);
    let ticks = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(2000u64);

    let freq_spread = 0.10;
    let cfg = SyncConfig {
        seed,
        n_oscillators: n,
        freq_spread,
        coupling: SYNC_SUPERCRITICAL_RATIO * freq_spread,
        ticks,
    };
    let r = run_sync_experiment(&cfg);
    let gap = r.r_final - r.r_final_ablated;

    println!("=== Axiom 6: Emergence at Scale (Kuramoto sync) ===");
    println!("N={}  spread={:.3}  K={:.3}  ticks={}", r.n_oscillators, r.freq_spread, r.coupling, r.n_ticks);
    println!();
    println!("  R_final (coupled, rule ON)  = {:.4}", r.r_final);
    println!("  R_final (ablated, rule OFF) = {:.4}", r.r_final_ablated);
    println!("  emergence gap ΔR            = {:.4}", gap);
    println!();
    if r.synchronized {
        println!("VERDICT: PASS — global order emerges from the local rule (N-1 → N).");
        println!("         Ablating the coupling collapses it → order is not top-down.");
    } else {
        println!("VERDICT: FAIL — no decisive emergence gap for this configuration.");
    }
}
