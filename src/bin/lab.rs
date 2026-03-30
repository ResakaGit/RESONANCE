//! Laboratorio universal — todos los use cases en una interfaz.
//! Universal laboratory — all use cases in one interface.
//!
//! Selector de experimento + parámetros + ejecución + dashboard de resultados.
//! Todo vive en este binario. Zero modificaciones a src/.
//!
//! Usage:
//!   cargo run --release --bin lab

use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use egui_plot::{Line, Plot, PlotPoints};

use resonance::use_cases::cli::archetype_label;
use resonance::use_cases::experiments::{
    cambrian, cancer_therapy, convergence, debate, fermi, lab as lab_exp, speciation,
};
use resonance::use_cases::presets;

// ─── Experiment selection ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Experiment {
    #[default]
    Lab,
    Fermi,
    Speciation,
    Cambrian,
    Debate,
    Convergence,
    CancerTherapy,
}

const EXPERIMENT_NAMES: &[(Experiment, &str)] = &[
    (Experiment::Lab,           "Universe Lab"),
    (Experiment::Fermi,         "Fermi Paradox"),
    (Experiment::Speciation,    "Speciation"),
    (Experiment::Cambrian,      "Cambrian Explosion"),
    (Experiment::Debate,        "Debate (Cooperation)"),
    (Experiment::Convergence,   "Convergence"),
    (Experiment::CancerTherapy, "Cancer Therapy"),
];

// ─── Shared parameters ──────────────────────────────────────────────────────

#[derive(Resource)]
struct LabParams {
    experiment:   Experiment,
    preset_index: usize,
    seed:         u64,
    worlds:       usize,
    generations:  u32,
    ticks:        u32,
    // Cancer-specific
    drug_potency:   f32,
    drug_bandwidth: f32,
    treatment_start: u32,
}

impl Default for LabParams {
    fn default() -> Self {
        Self {
            experiment:      Experiment::default(),
            preset_index:    0,
            seed:            42,
            worlds:          100,
            generations:     100,
            ticks:           500,
            drug_potency:    2.0,
            drug_bandwidth:  50.0,
            treatment_start: 5,
        }
    }
}

const PRESET_NAMES: &[&str] = &["Earth", "Jupiter", "Mars", "Eden", "Hell"];

fn preset_by_index(i: usize) -> presets::UniversePreset {
    match i {
        1 => presets::JUPITER,
        2 => presets::MARS,
        3 => presets::EDEN,
        4 => presets::HELL,
        _ => presets::EARTH,
    }
}

// ─── Results (tagged union) ─────────────────────────────────────────────────

#[derive(Default)]
enum LabResult {
    #[default]
    None,
    Lab(Box<resonance::use_cases::ExperimentReport>),
    Fermi(Box<fermi::FermiReport>),
    Speciation(Box<speciation::SpeciationReport>),
    Cambrian(Box<cambrian::CambrianReport>),
    Debate(Box<debate::DebateReport>),
    Convergence(Box<convergence::ConvergenceReport>),
    Cancer(Box<cancer_therapy::TherapyReport>),
}

#[derive(Resource, Default)]
struct LabState {
    result:  LabResult,
    wall_ms: u64,
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resonance — Laboratory".into(),
                resolution: (1400.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_resource::<LabParams>()
        .init_resource::<LabState>()
        .add_systems(Update, lab_ui_system)
        .run();
}

// ─── UI system ──────────────────────────────────────────────────────────────

fn lab_ui_system(
    mut contexts: EguiContexts,
    mut params:   ResMut<LabParams>,
    mut state:    ResMut<LabState>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else { return };

    // ── Left panel: experiment selection + parameters ──
    egui::SidePanel::left("lab_controls").default_width(280.0).show(ctx, |ui| {
        render_experiment_selector(ui, &mut params);
        ui.separator();
        render_shared_params(ui, &mut params);
        ui.separator();
        render_experiment_params(ui, &mut params);
        ui.separator();
        render_run_button(ui, &params, &mut state);
    });

    // ── Central panel: results ──
    egui::CentralPanel::default().show(ctx, |ui| {
        render_results(ui, &state);
    });
}

// ─── Left panel sections ────────────────────────────────────────────────────

fn render_experiment_selector(ui: &mut egui::Ui, params: &mut LabParams) {
    ui.heading("Experiment");
    for &(exp, name) in EXPERIMENT_NAMES {
        ui.radio_value(&mut params.experiment, exp, name);
    }
}

fn render_shared_params(ui: &mut egui::Ui, params: &mut LabParams) {
    ui.heading("Parameters");

    if params.experiment != Experiment::CancerTherapy {
        ui.label("Preset");
        egui::ComboBox::from_id_salt("preset")
            .selected_text(PRESET_NAMES[params.preset_index.min(PRESET_NAMES.len() - 1)])
            .show_ui(ui, |ui| {
                for (i, name) in PRESET_NAMES.iter().enumerate() {
                    ui.selectable_value(&mut params.preset_index, i, *name);
                }
            });
    }

    ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
    ui.add(egui::Slider::new(&mut params.worlds, 10..=2000).text("Worlds"));
    ui.add(egui::Slider::new(&mut params.generations, 10..=1000).text("Gens"));
    ui.add(egui::Slider::new(&mut params.ticks, 50..=2000).text("Ticks/gen"));
}

fn render_experiment_params(ui: &mut egui::Ui, params: &mut LabParams) {
    match params.experiment {
        Experiment::CancerTherapy => {
            ui.heading("Cancer Therapy");
            ui.add(egui::Slider::new(&mut params.drug_potency, 0.1..=10.0).text("Drug potency"));
            ui.add(egui::Slider::new(&mut params.drug_bandwidth, 10.0..=200.0).text("Drug bandwidth (Hz)"));
            ui.add(egui::Slider::new(&mut params.treatment_start, 0..=50).text("Treatment start (gen)"));
        }
        _ => {
            ui.label("No additional parameters for this experiment.");
        }
    }
}

fn render_run_button(ui: &mut egui::Ui, params: &LabParams, state: &mut LabState) {
    if ui.button("Run Experiment").clicked() {
        run_experiment(params, state);
    }
    if state.wall_ms > 0 {
        ui.label(format!("Last run: {}ms", state.wall_ms));
    }
}

// ─── Experiment execution (stateless dispatch) ──────────────────────────────

fn run_experiment(params: &LabParams, state: &mut LabState) {
    let start = Instant::now();
    let preset = preset_by_index(params.preset_index);

    state.result = match params.experiment {
        Experiment::Lab => {
            let r = lab_exp::run(&preset, params.seed, params.worlds, params.generations, params.ticks);
            LabResult::Lab(Box::new(r))
        }
        Experiment::Fermi => {
            let r = fermi::run(params.worlds, params.generations, params.ticks);
            LabResult::Fermi(Box::new(r))
        }
        Experiment::Speciation => {
            let r = speciation::run(&preset, params.seed, params.seed.wrapping_add(7777), params.generations, params.ticks, 0.5);
            LabResult::Speciation(Box::new(r))
        }
        Experiment::Cambrian => {
            let r = cambrian::run(&preset, params.seed, params.worlds, params.generations, params.ticks, 0.3);
            LabResult::Cambrian(Box::new(r))
        }
        Experiment::Debate => {
            let r = debate::run(&preset, params.worlds.min(50), params.generations, params.ticks);
            LabResult::Debate(Box::new(r))
        }
        Experiment::Convergence => {
            let r = convergence::run(&preset, params.worlds.min(100), params.generations, params.ticks, 0.3);
            LabResult::Convergence(Box::new(r))
        }
        Experiment::CancerTherapy => {
            let cfg = cancer_therapy::TherapyConfig {
                drug_potency:        params.drug_potency,
                drug_bandwidth:      params.drug_bandwidth,
                treatment_start_gen: params.treatment_start,
                worlds:              params.worlds.min(200),
                generations:         params.generations,
                ticks_per_gen:       params.ticks.min(500),
                seed:                params.seed,
                ..Default::default()
            };
            let r = cancer_therapy::run(&cfg);
            LabResult::Cancer(Box::new(r))
        }
    };

    state.wall_ms = start.elapsed().as_millis() as u64;
}

// ─── Results rendering (stateless per variant) ──────────────────────────────

fn render_results(ui: &mut egui::Ui, state: &LabState) {
    match &state.result {
        LabResult::None => {
            ui.centered_and_justified(|ui| {
                ui.heading("Select an experiment and click Run");
            });
        }
        LabResult::Lab(r)          => render_lab_result(ui, r),
        LabResult::Fermi(r)        => render_fermi_result(ui, r),
        LabResult::Speciation(r)   => render_speciation_result(ui, r),
        LabResult::Cambrian(r)     => render_cambrian_result(ui, r),
        LabResult::Debate(r)       => render_debate_result(ui, r),
        LabResult::Convergence(r)  => render_convergence_result(ui, r),
        LabResult::Cancer(r)       => render_cancer_result(ui, r),
    }
}

fn render_lab_result(ui: &mut egui::Ui, r: &resonance::use_cases::ExperimentReport) {
    ui.heading(format!("Universe Lab — {}", r.preset_name));
    render_fitness_chart(ui, &r.history);
    render_top_genomes(ui, &r.top_genomes);
}

fn render_fermi_result(ui: &mut egui::Ui, r: &fermi::FermiReport) {
    ui.heading("Fermi Paradox");
    egui::Grid::new("fermi_grid").show(ui, |ui| {
        ui.label("Universes:");    ui.label(format!("{}", r.total_universes));   ui.end_row();
        ui.label("With life:");    ui.label(format!("{} ({:.1}%)", r.with_life, r.life_probability * 100.0)); ui.end_row();
        ui.label("Complex life:"); ui.label(format!("{} ({:.1}%)", r.with_complex_life, r.complex_probability * 100.0)); ui.end_row();
    });
}

fn render_speciation_result(ui: &mut egui::Ui, r: &speciation::SpeciationReport) {
    ui.heading(format!("Speciation — {}", r.preset_name));
    egui::Grid::new("spec_grid").show(ui, |ui| {
        ui.label("Pop A freq:"); ui.label(format!("{:.1} Hz", r.mean_freq_a)); ui.end_row();
        ui.label("Pop B freq:"); ui.label(format!("{:.1} Hz", r.mean_freq_b)); ui.end_row();
        ui.label("Interference:"); ui.label(format!("{:.3}", r.cross_interference)); ui.end_row();
        ui.label("Speciated:");  ui.label(if r.speciated { "YES" } else { "NO" }); ui.end_row();
    });
}

fn render_cambrian_result(ui: &mut egui::Ui, r: &cambrian::CambrianReport) {
    ui.heading(format!("Cambrian — {}", r.preset_name));
    match r.explosion_gen {
        Some(g) => { ui.label(format!("Explosion detected at generation {}", g)); }
        None    => { ui.label("No explosion detected."); }
    }
    let diversity: PlotPoints = r.diversity_curve.iter().enumerate()
        .map(|(i, &v)| [i as f64, v as f64]).collect();
    Plot::new("cambrian_diversity").height(300.0).show(ui, |plot_ui| {
        plot_ui.line(Line::new(diversity).name("diversity"));
    });
}

fn render_debate_result(ui: &mut egui::Ui, r: &debate::DebateReport) {
    ui.heading(format!("Debate — {}", r.preset_name));
    egui::Grid::new("debate_grid").show(ui, |ui| {
        ui.label("Life rate:");        ui.label(format!("{:.1}%", r.life_rate * 100.0)); ui.end_row();
        ui.label("Complexity rate:");  ui.label(format!("{:.1}%", r.complexity_rate * 100.0)); ui.end_row();
        ui.label("Cooperation signal:"); ui.label(format!("{:.3}", r.cooperation_signal)); ui.end_row();
    });
}

fn render_convergence_result(ui: &mut egui::Ui, r: &convergence::ConvergenceReport) {
    ui.heading("Convergence Analysis");
    egui::Grid::new("conv_grid").show(ui, |ui| {
        ui.label("Seeds:");       ui.label(format!("{}", r.n_seeds)); ui.end_row();
        ui.label("Mean dist:");   ui.label(format!("{:.3}", r.mean_distance)); ui.end_row();
        ui.label("Convergence:"); ui.label(format!("{:.1}%", r.convergence_rate * 100.0)); ui.end_row();
    });
    render_top_genomes(ui, &r.top_genomes);
}

fn render_cancer_result(ui: &mut egui::Ui, r: &cancer_therapy::TherapyReport) {
    ui.heading("Cancer Therapy");
    egui::Grid::new("cancer_summary").show(ui, |ui| {
        ui.label("Eliminated:"); ui.label(if r.tumor_eliminated { "YES" } else { "NO" }); ui.end_row();
        if let Some(g) = r.generations_to_resistance {
            ui.label("Resistance at:"); ui.label(format!("gen {g}")); ui.end_row();
        }
        if let Some(g) = r.relapse_gen {
            ui.label("Relapse at:"); ui.label(format!("gen {g}")); ui.end_row();
        }
    });
    ui.separator();

    // Cancer + Normal time series
    let cancer: PlotPoints = r.timeline.iter()
        .map(|s| [s.generation as f64, s.cancer_alive_mean as f64]).collect();
    let normal: PlotPoints = r.timeline.iter()
        .map(|s| [s.generation as f64, s.normal_alive_mean as f64]).collect();
    Plot::new("cancer_pop").height(250.0).show(ui, |plot_ui| {
        plot_ui.line(Line::new(cancer).name("cancer").color(egui::Color32::RED));
        plot_ui.line(Line::new(normal).name("normal").color(egui::Color32::GREEN));
    });

    // Resistance index
    let resist: PlotPoints = r.timeline.iter()
        .map(|s| [s.generation as f64, s.resistance_index as f64]).collect();
    Plot::new("cancer_resist").height(200.0).show(ui, |plot_ui| {
        plot_ui.line(Line::new(resist).name("resistance").color(egui::Color32::from_rgb(255, 180, 50)));
    });
}

// ─── Shared render helpers ──────────────────────────────────────────────────

fn render_fitness_chart(ui: &mut egui::Ui, history: &[resonance::batch::harness::GenerationStats]) {
    if history.is_empty() { return; }
    let best: PlotPoints = history.iter()
        .map(|s| [s.generation as f64, s.best_fitness as f64]).collect();
    let mean: PlotPoints = history.iter()
        .map(|s| [s.generation as f64, s.mean_fitness as f64]).collect();
    Plot::new("fitness_chart").height(250.0).show(ui, |plot_ui| {
        plot_ui.line(Line::new(best).name("best").color(egui::Color32::GREEN));
        plot_ui.line(Line::new(mean).name("mean").color(egui::Color32::YELLOW));
    });
}

fn render_top_genomes(ui: &mut egui::Ui, genomes: &[resonance::batch::genome::GenomeBlob]) {
    if genomes.is_empty() { return; }
    ui.separator();
    ui.label("Top Genomes");
    egui::Grid::new("genomes_grid").striped(true).show(ui, |ui| {
        ui.label("#"); ui.label("Arch"); ui.label("Growth"); ui.label("Mob"); ui.label("Branch"); ui.label("Resil"); ui.end_row();
        for (i, g) in genomes.iter().take(10).enumerate() {
            ui.label(format!("{}", i + 1));
            ui.label(archetype_label(g.archetype));
            ui.label(format!("{:.2}", g.growth_bias));
            ui.label(format!("{:.2}", g.mobility_bias));
            ui.label(format!("{:.2}", g.branching_bias));
            ui.label(format!("{:.2}", g.resilience));
            ui.end_row();
        }
    });
}
