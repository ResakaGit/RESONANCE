//! Validación **headless** end-to-end del pipeline multi-escala cósmico.
//! End-to-end **headless** validation of the multi-scale cosmic pipeline.
//!
//! Atraviesa S0→S1→S2→S3→S4 y regresa S4→S0 imprimiendo HUD por paso.
//! Ejerce `seed_universe` + `zoom_via_bridge` (observer.rs) + coarsening.
//! La versión gráfica 3D vive en `cosmic_telescope` (CT-8).
//!
//! Uso: `cargo run --release --bin cosmic_telescope_headless -- [seed]`.

use bevy::prelude::*;

use resonance::cosmic::bridges::ecological_to_molecular::{fold_proteome, proteome_health};
use resonance::blueprint::equations::proteome_inference::infer_proteome;
use resonance::cosmic::scales::coarsening::{
    background_coarsening_system, CosmicBackgroundClock,
};
use resonance::cosmic::{
    largest_entity_in, scale_short, seed_universe, zoom_via_bridge, BigBangParams, CosmicPlugin,
    ScaleLevel, ScaleManager, ZoomOutEvent, ALL_SCALES,
};

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    print_header(seed);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CosmicPlugin)
        .init_resource::<CosmicBackgroundClock>()
        .add_systems(
            Update,
            (
                resonance::cosmic::zoom::zoom_out_system,
                background_coarsening_system,
            )
                .chain(),
        );

    {
        let mut mgr = app.world_mut().resource_mut::<ScaleManager>();
        seed_universe(&mut mgr, &BigBangParams::interactive(seed));
    }
    println!("{}", hud_line(&app, "S0 init"));

    traverse_zoom_in(&mut app);
    println!("{}", hud_line(&app, "S4 reached"));

    demonstrate_molecular(&app, seed);

    traverse_zoom_out(&mut app);
    println!("{}", hud_line(&app, "S0 restored"));

    print_footer(&app);
}

// ─── Zoom traversal ─────────────────────────────────────────────────────────

fn traverse_zoom_in(app: &mut App) {
    let mut from = ScaleLevel::Cosmological;
    while let Some(child) = step_zoom_in(app, from) {
        println!("{}", hud_line(app, &format!("{from:?} → {child:?}")));
        from = child;
    }
}

fn step_zoom_in(app: &mut App, from: ScaleLevel) -> Option<ScaleLevel> {
    let parent_id = largest_entity_in(app.world().resource::<ScaleManager>(), from)?;
    let child = {
        let mut mgr = app.world_mut().resource_mut::<ScaleManager>();
        zoom_via_bridge(&mut mgr, parent_id, from)?
    };
    app.update();
    Some(child)
}

fn traverse_zoom_out(app: &mut App) {
    for _ in 0..4 {
        app.world_mut().send_event(ZoomOutEvent);
        app.update();
    }
}

// ─── Molecular demo ─────────────────────────────────────────────────────────

fn demonstrate_molecular(app: &App, seed: u64) {
    let mgr = app.world().resource::<ScaleManager>();
    let Some(inst) = mgr.get(ScaleLevel::Molecular) else { return; };
    if inst.world.n_alive() == 0 { return; }

    let Some(parent_id) = inst.parent_entity_id else { return; };
    let Some(ecological) = mgr.get(ScaleLevel::Ecological) else { return; };
    let Some(organism) = ecological
        .world
        .entities
        .iter()
        .find(|e| e.entity_id == parent_id)
    else { return; };

    let proteome = infer_proteome(organism.qe, organism.frequency_hz, organism.age_ticks, seed);
    if proteome.is_empty() { return; }

    let results = fold_proteome(&proteome[..proteome.len().min(2)], seed);
    let health = proteome_health(&results);
    println!(
        "  │ Molecular fold: {} proteins, health={:.4}",
        results.len(),
        health,
    );
    for (i, r) in results.iter().enumerate() {
        println!(
            "  │   [{i}] n_res={} best_q={:.3} coherence={:.3}",
            r.n_residues, r.best_q, r.best_coherence,
        );
    }
}

// ─── HUD ────────────────────────────────────────────────────────────────────

fn hud_line(app: &App, step: &str) -> String {
    let mgr = app.world().resource::<ScaleManager>();
    let observed = mgr.observed;
    let total_universe: f64 = mgr.total_qe_across_scales();
    let breadcrumb = breadcrumb_str(mgr);
    let status = scale_status(mgr);
    format!(
        "  [{step:>24}] obs={:?}  qe_universe={:>10.2}  path={breadcrumb}  {status}",
        observed, total_universe,
    )
}

fn breadcrumb_str(mgr: &ScaleManager) -> String {
    let mut parts = Vec::new();
    for lvl in ALL_SCALES {
        if !mgr.has(lvl) { continue; }
        let marker = if lvl == mgr.observed { "●" } else { "◉" };
        parts.push(format!("{marker}{lvl:?}"));
    }
    parts.join("→")
}

fn scale_status(mgr: &ScaleManager) -> String {
    let mut out = String::new();
    for lvl in ALL_SCALES {
        let marker = scale_marker(mgr, lvl);
        out.push_str(&format!("[{}{marker}]", scale_short(lvl)));
    }
    out
}

fn scale_marker(mgr: &ScaleManager, lvl: ScaleLevel) -> &'static str {
    if lvl == mgr.observed { "●" }
    else if mgr.has(lvl) { "◉" }
    else { "○" }
}

fn print_header(seed: u64) {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  COSMIC TELESCOPE (headless validation, seed={seed:>6})                 ║");
    println!("║  Big Bang → Stars → Planets → Life → Proteins → (return)            ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
}

fn print_footer(app: &App) {
    let mgr = app.world().resource::<ScaleManager>();
    println!();
    println!("=== Final cosmic state ===");
    println!("  observed = {:?}", mgr.observed);
    println!("  instances active = {}", mgr.instances.len());
    println!("  universe_seed = {}", mgr.universe_seed);
    println!("  qe_across_scales = {:.2}", mgr.total_qe_across_scales());
    println!();
    println!("Traversal S0→S4→S0 completed without crash.");
}
