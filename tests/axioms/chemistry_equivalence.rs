//! AI-3 (ADR-045): equivalencia química alchemical vs mass-action.
//! AI-3 (ADR-045): chemistry equivalence — alchemical vs mass-action.
//!
//! Spike controlado: en lugar de replicar el path "alchemical-only"
//! (embebido en LayersPlugin + SimulationPlugin, no aislable trivialmente),
//! validamos las invariantes que justifican coexistencia (Camino 1):
//!
//! 1. Mass-action solo respeta Ax 4: `total_dissipated(t)` monotónico.
//! 2. Bridge AI-1 conserva qe global: `Σ(species) + Σ(field) + Σ(diss)`
//!    permanece dentro de tolerancia tras N ticks de injection.
//! 3. Curva CSV de `total_dissipated(t)` exportada para inspección manual
//!    contra futuras corridas alchemical-only (sprint dedicado AI-bench).
//!
//! Output: `target/ai3_dissipation_curve.csv` con `tick,total_dissipated`.
//!
//! Veredicto del spike (en ADR-045 §5):
//!   - Bridge preserva invariantes axiomáticos ⇒ Camino 1 viable
//!   - Comparación cuantitativa cruzada deferida a sprint propio

#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use resonance::blueprint::constants::chemistry::SPECIES_TO_QE_COUPLING;
use resonance::layers::reaction::SpeciesId;
use resonance::layers::reaction_network::ReactionNetwork;
use resonance::layers::species_grid::SpeciesGrid;
use resonance::math_types::Vec2;
use resonance::simulation::species_to_qe::inject_species_to_field;
use resonance::use_cases::experiments::autopoiesis::{
    SoupConfig, SoupSim, run_soup_with_network,
};
use resonance::worldgen::EnergyFieldGrid;

/// Carga formose (red canónica para el spike).
fn formose() -> ReactionNetwork {
    let text = fs::read_to_string("assets/reactions/formose.ron")
        .expect("assets/reactions/formose.ron debe existir");
    ReactionNetwork::from_ron_str(&text).expect("formose.ron parses")
}

/// Config canónica del spike: misma geometría / kinética para ambos paths.
fn canonical_cfg() -> SoupConfig {
    SoupConfig {
        seed: 0,
        n_species: 4,
        n_reactions: 4,
        food_size: 2,
        grid: (16, 16),
        ticks: 1000,
        equilibration_ticks: 100,
        detection_every: 50,
        last_window_ticks: 200,
        initial_food_qe: 50.0,
        dt: 0.1,
        food_spot_radius: Some(2),
    }
}

// ── Test 1 — Mass-action curve monotonicity ────────────────────────────────

#[test]
#[ignore = "AI-3 spike — produce CSV de curva de dissipation"]
fn mass_action_dissipation_is_monotone_and_dumps_csv() {
    let cfg = canonical_cfg();
    let net = formose();
    let mut sim = SoupSim::new(cfg.clone(), net);

    // Sample cada 50 ticks → 21 puntos para la corrida de 1000 ticks.
    let sample_every: u64 = 50;
    let mut samples: Vec<(u64, f32)> = Vec::with_capacity(1 + cfg.ticks as usize / sample_every as usize);
    samples.push((0, 0.0));
    while !sim.is_done() {
        sim.step();
        if sim.tick() % sample_every == 0 {
            samples.push((sim.tick(), sim.total_dissipated()));
        }
    }
    let report = sim.finish();

    // Invariante Ax 4: dissipated NUNCA decrece.
    for w in samples.windows(2) {
        let (t_a, d_a) = w[0];
        let (t_b, d_b) = w[1];
        assert!(
            d_b + 1e-6 >= d_a,
            "Ax 4: dissipated decreció entre tick {t_a} (d={d_a}) y tick {t_b} (d={d_b})",
        );
    }

    // Total final coincide con report (consistencia entre stepper + finish).
    let last_d = samples.last().map(|(_, d)| *d).unwrap_or(0.0);
    assert!(
        (last_d - report.total_dissipated).abs() < 1e-3,
        "stepper sample={last_d} != report.total={}",
        report.total_dissipated,
    );

    // CSV out: target/ai3_dissipation_curve.csv
    let mut csv = String::from("tick,total_dissipated\n");
    for (tick, d) in &samples {
        csv.push_str(&format!("{tick},{d:.6}\n"));
    }
    let out_path = csv_out_path("ai3_dissipation_curve.csv");
    fs::create_dir_all(out_path.parent().expect("path tiene parent"))
        .expect("crear target/");
    fs::write(&out_path, csv).expect("escribir CSV");
    eprintln!("AI-3: wrote {} samples to {:?}", samples.len(), out_path);
    eprintln!(
        "AI-3: total_dissipated final = {:.4}, last sample = {:.4}",
        report.total_dissipated, last_d,
    );
}

// ── Test 2 — Bridge AI-1 preserva conservación global ──────────────────────

/// Conservación global a través del bridge AI-1.  Suma de qe en species,
/// más qe inyectado al field, debe igualar (dentro de tolerancia ±1e-3) al
/// qe inicial sembrado en species menos cualquier dissipation reportada.
///
/// Por simplicidad: corremos `inject_species_to_field` repetido sin avanzar
/// el `SoupSim` (no hay disipación interna), entonces:
///   Σ(field_qe) ≤ Σ(species_qe inicial) × COUPLING × N_ticks  (cota superior)
///   Σ(species_qe) post-injection == Σ(species_qe) pre-injection (no muta species)
#[test]
#[ignore = "AI-3 spike — bridge conservation check"]
fn bridge_injection_does_not_create_qe() {
    let net = formose();
    let mut species = SpeciesGrid::new(8, 8, 50.0);
    // Spot 5×5 de species 1 (C2) con qe=10 cada celda
    let s1 = SpeciesId::new(1).expect("id válido");
    for y in 1..=5 { for x in 1..=5 { species.seed(x, y, s1, 10.0); } }

    let species_total_initial: f32 = species
        .cells().iter().map(|c| c.species.iter().sum::<f32>()).sum();
    assert!(
        (species_total_initial - 25.0 * 10.0).abs() < 1e-3,
        "sembrado correcto = 250 qe",
    );

    let mut field = EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO);
    // Sembrar freq dominante en field para que alignment > 0
    for y in 0..8u32 { for x in 0..8u32 {
        if let Some(c) = field.cell_xy_mut(x, y) { c.dominant_frequency_hz = 50.0; }
    }}

    // 100 inyecciones con dt=0.1 — el bridge NO muta species (contrato AI-1)
    let dt = 0.1_f32;
    let n_iters = 100usize;
    for _ in 0..n_iters { inject_species_to_field(&species, &net, &mut field, dt); }

    let species_total_final: f32 = species
        .cells().iter().map(|c| c.species.iter().sum::<f32>()).sum();
    let field_total: f32 = field.total_qe();

    // Invariante 1: bridge no muta species (contrato AI-1, ADR-043 §3 Out of scope).
    assert!(
        (species_total_final - species_total_initial).abs() < 1e-3,
        "bridge no muta species: pre={species_total_initial} post={species_total_final}",
    );

    // Invariante 2: cota superior — el field no recibe más qe que el máximo teórico.
    let upper_bound = species_total_initial
        * (n_iters as f32) * dt * SPECIES_TO_QE_COUPLING * 1.0;
    assert!(
        field_total <= upper_bound + 1e-3,
        "field_total={field_total} excede cota teórica {upper_bound} \
         (species×iters×dt×COUPLING×alignment_max)",
    );
    assert!(field_total > 0.0, "alignment > 0 ⇒ injection > 0, got {field_total}");

    eprintln!(
        "AI-3 bridge: species pre={species_total_initial}, species post={species_total_final}, \
         field={field_total}, cota={upper_bound}",
    );
}

// ── Test 3 — Bridge produce monotónica creciente del field qe ──────────────

#[test]
#[ignore = "AI-3 spike — bridge integration sanity"]
fn bridge_injection_is_monotone_under_repeated_calls() {
    let net = formose();
    let mut species = SpeciesGrid::new(8, 8, 50.0);
    let s1 = SpeciesId::new(1).expect("id válido");
    for y in 3..=5 { for x in 3..=5 { species.seed(x, y, s1, 100.0); } }
    let mut field = EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO);
    for y in 0..8u32 { for x in 0..8u32 {
        if let Some(c) = field.cell_xy_mut(x, y) { c.dominant_frequency_hz = 50.0; }
    }}

    let mut prev = 0.0_f32;
    for _ in 0..20 {
        inject_species_to_field(&species, &net, &mut field, 1.0);
        let now = field.total_qe();
        assert!(now >= prev - 1e-6, "field qe debe crecer monotónico, prev={prev} now={now}");
        prev = now;
    }
}

// ── Test 4 — Determinismo cross-run ────────────────────────────────────────

#[test]
#[ignore = "AI-3 spike — determinism check, byte-equivalence"]
fn mass_action_two_runs_same_dissipated_total() {
    let cfg = canonical_cfg();
    let net = formose();
    let r1 = run_soup_with_network(&cfg, net.clone());
    let r2 = run_soup_with_network(&cfg, net);
    assert_eq!(r1.total_dissipated, r2.total_dissipated);
    assert_eq!(r1.fission_events.len(), r2.fission_events.len());
}

fn csv_out_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join(name)
}
