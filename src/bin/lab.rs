//! Laboratorio universal — todos los use cases en una interfaz composable.
//! Universal laboratory — all use cases in one composable interface.
//!
//! Arquitectura: LabMode state machine → dispatch controls + central view.
//! Cada experiment define qué controles muestra y cómo renderiza resultados.
//! Zero lógica condicional dispersa. Zero `if is_live` flags.
//!
//! Usage:
//!   cargo run --release --bin lab
//!   RESONANCE_MAP=earth cargo run --release --bin lab

use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use egui_plot::{Line, Plot, PlotPoints};

use resonance::layers::{BaseEnergy, OscillatorySignature, SpatialVolume};
use resonance::plugins::{LayersPlugin, SimulationPlugin, SimulationTickPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::SimulationClock;
use resonance::use_cases::cli::archetype_label;
use resonance::use_cases::experiments::{
    cambrian, cancer_therapy, convergence, debate, fermi, lab as lab_exp, speciation,
};
use resonance::use_cases::export;
use resonance::use_cases::orchestrators;
use resonance::use_cases::presets;
use resonance::worldgen::EnergyFieldGrid;

// ─── Constants (visual calibration, no physics) ─────────────────────────────

const CONTROL_PANEL_WIDTH: f32 = 280.0;
const CHART_HEIGHT_MAIN: f32 = 250.0;
const CHART_HEIGHT_SMALL: f32 = 200.0;
const CHART_HEIGHT_CAMBRIAN: f32 = 300.0;

const WORLDS_RANGE: std::ops::RangeInclusive<usize> = 10..=2000;
const GENS_RANGE: std::ops::RangeInclusive<u32> = 10..=1000;
const TICKS_RANGE: std::ops::RangeInclusive<u32> = 50..=2000;
const POTENCY_RANGE: std::ops::RangeInclusive<f32> = 0.1..=10.0;
const BANDWIDTH_RANGE: std::ops::RangeInclusive<f32> = 10.0..=200.0;
const TREATMENT_START_RANGE: std::ops::RangeInclusive<u32> = 0..=50;

const SPECIATION_SEED_OFFSET: u64 = 7777;
const SPECIATION_THRESHOLD: f32 = 0.5;
const CAMBRIAN_THRESHOLD: f32 = 0.3;
const CONVERGENCE_THRESHOLD: f32 = 0.3;
const DEBATE_MAX_SEEDS: usize = 50;
const CONVERGENCE_MAX_SEEDS: usize = 100;
const CANCER_MAX_WORLDS: usize = 200;
const CANCER_MAX_TICKS: u32 = 500;
const ABLATION_STEPS: usize = 8;
const ENSEMBLE_SEEDS: usize = 10;
const DEFAULT_EXPORT_PATH: &str = "lab_results.csv";
const FREQ_HUE_MAX: f32 = 800.0; // max frequency for hue normalization
const ENTITY_QE_BRIGHTNESS_REF: f32 = 50.0; // qe reference for entity brightness

const COLOR_BEST: egui::Color32 = egui::Color32::GREEN;
const COLOR_MEAN: egui::Color32 = egui::Color32::YELLOW;
const COLOR_CANCER: egui::Color32 = egui::Color32::RED;
const COLOR_NORMAL: egui::Color32 = egui::Color32::GREEN;
const COLOR_RESISTANCE: egui::Color32 = egui::Color32::from_rgb(255, 180, 50);
const COLOR_ABLATION: [egui::Color32; ABLATION_STEPS] = [
    egui::Color32::from_rgb(66, 133, 244),
    egui::Color32::from_rgb(234, 67, 53),
    egui::Color32::from_rgb(251, 188, 4),
    egui::Color32::from_rgb(52, 168, 83),
    egui::Color32::from_rgb(154, 66, 244),
    egui::Color32::from_rgb(244, 66, 154),
    egui::Color32::from_rgb(66, 244, 210),
    egui::Color32::from_rgb(244, 154, 66),
];

const PRESET_NAMES: &[&str] = &["Earth", "Jupiter", "Mars", "Eden", "Hell"];

// ─── LR-1: State Machine ───────────────────────────────────────────────────

/// Modo principal del lab. Determina qué controles y qué vista central se muestran.
/// Main lab mode. Determines which controls and central view are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LabMode {
    #[default]
    Batch,
    Live,
}

/// Experimento batch seleccionado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BatchExperiment {
    #[default]
    Lab,
    Fermi,
    Speciation,
    Cambrian,
    Debate,
    Convergence,
    CancerTherapy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RunMode {
    #[default]
    Single,
    Ablation,
    Ensemble,
}

/// Capa de visualización para Live 2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ViewLayer {
    #[default]
    FrequencyEnergy,
    EnergyOnly,
}

const BATCH_EXPERIMENTS: &[(BatchExperiment, &str)] = &[
    (BatchExperiment::Lab, "Universe Lab"),
    (BatchExperiment::Fermi, "Fermi Paradox"),
    (BatchExperiment::Speciation, "Speciation"),
    (BatchExperiment::Cambrian, "Cambrian Explosion"),
    (BatchExperiment::Debate, "Debate (Cooperation)"),
    (BatchExperiment::Convergence, "Convergence"),
    (BatchExperiment::CancerTherapy, "Cancer Therapy"),
];

// ─── Resources ──────────────────────────────────────────────────────────────

#[derive(Resource)]
struct LabParams {
    mode: LabMode,
    experiment: BatchExperiment,
    run_mode: RunMode,
    preset_index: usize,
    seed: u64,
    worlds: usize,
    generations: u32,
    ticks: u32,
    view_layer: ViewLayer,
}

impl Default for LabParams {
    fn default() -> Self {
        Self {
            mode: LabMode::default(),
            experiment: BatchExperiment::default(),
            run_mode: RunMode::default(),
            preset_index: 0,
            seed: 42,
            worlds: 100,
            generations: 100,
            ticks: 500,
            view_layer: ViewLayer::default(),
        }
    }
}

#[derive(Resource)]
struct CancerParams {
    drug_potency: f32,
    drug_bandwidth: f32,
    treatment_start: u32,
}

impl Default for CancerParams {
    fn default() -> Self {
        Self {
            drug_potency: 2.0,
            drug_bandwidth: 50.0,
            treatment_start: 5,
        }
    }
}

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
    Ablation(Vec<resonance::use_cases::ExperimentReport>),
    Ensemble(Box<orchestrators::EnsembleReport>),
}

#[derive(Resource, Default)]
struct LabState {
    result: LabResult,
    wall_ms: u64,
    last_csv: String,
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
        .add_plugins(SimulationTickPlugin)
        .init_resource::<PaletteRegistry>()
        .insert_resource(SimWorldTransformParams::default())
        .add_plugins(LayersPlugin)
        .add_plugins(SimulationPlugin)
        .init_resource::<LabParams>()
        .init_resource::<CancerParams>()
        .init_resource::<LabState>()
        .add_systems(Update, (controls_system, central_system).chain())
        .run();
}

// ─── LR-4: Composed dispatch (controls + central by mode) ──────────────────

fn controls_system(
    mut contexts: EguiContexts,
    mut params: ResMut<LabParams>,
    mut cancer: ResMut<CancerParams>,
    mut state: ResMut<LabState>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::SidePanel::left("lab_controls")
        .default_width(CONTROL_PANEL_WIDTH)
        .show(ctx, |ui| {
            // Mode selector (top)
            ui.heading("Mode");
            ui.radio_value(&mut params.mode, LabMode::Batch, "Batch Experiments");
            ui.radio_value(&mut params.mode, LabMode::Live, "Live 2D Simulation");
            ui.separator();

            // LR-3: Contextual controls by mode
            match params.mode {
                LabMode::Batch => {
                    render_batch_controls(ui, &mut params, &mut cancer, &mut state);
                }
                LabMode::Live => {
                    render_live_controls(ui, &mut params);
                }
            }
        });
}

fn central_system(
    mut contexts: EguiContexts,
    params: Res<LabParams>,
    state: Res<LabState>,
    grid: Option<Res<EnergyFieldGrid>>,
    clock: Option<Res<SimulationClock>>,
    entity_query: Query<(
        &Transform,
        &BaseEnergy,
        &SpatialVolume,
        &OscillatorySignature,
    )>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::CentralPanel::default().show(ctx, |ui| match params.mode {
        LabMode::Batch => render_results(ui, &state),
        LabMode::Live => render_live_2d(ui, &grid, &clock, &entity_query, &params),
    });
}

// ─── LR-3: Batch controls (contextual per experiment) ───────────────────────

fn render_batch_controls(
    ui: &mut egui::Ui,
    params: &mut LabParams,
    cancer: &mut CancerParams,
    state: &mut LabState,
) {
    // Experiment selector
    ui.heading("Experiment");
    for &(exp, name) in BATCH_EXPERIMENTS {
        ui.radio_value(&mut params.experiment, exp, name);
    }
    ui.separator();

    // Contextual params per experiment
    match params.experiment {
        BatchExperiment::CancerTherapy => {
            ui.heading("Cancer Therapy");
            ui.add(egui::Slider::new(&mut cancer.drug_potency, POTENCY_RANGE).text("Drug potency"));
            ui.add(
                egui::Slider::new(&mut cancer.drug_bandwidth, BANDWIDTH_RANGE)
                    .text("Bandwidth (Hz)"),
            );
            ui.add(
                egui::Slider::new(&mut cancer.treatment_start, TREATMENT_START_RANGE)
                    .text("Start (gen)"),
            );
            ui.separator();
            ui.add(egui::Slider::new(&mut params.worlds, 10..=CANCER_MAX_WORLDS).text("Worlds"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, 50..=CANCER_MAX_TICKS).text("Ticks/gen"));
        }
        BatchExperiment::Fermi => {
            ui.heading("Fermi Paradox");
            ui.add(egui::Slider::new(&mut params.worlds, WORLDS_RANGE).text("Universes"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, TICKS_RANGE).text("Ticks/gen"));
        }
        _ => {
            ui.heading("Parameters");
            ui.label("Preset");
            egui::ComboBox::from_id_salt("preset")
                .selected_text(PRESET_NAMES[params.preset_index.min(PRESET_NAMES.len() - 1)])
                .show_ui(ui, |ui| {
                    for (i, name) in PRESET_NAMES.iter().enumerate() {
                        ui.selectable_value(&mut params.preset_index, i, *name);
                    }
                });
            ui.add(
                egui::DragValue::new(&mut params.seed)
                    .prefix("Seed: ")
                    .speed(1.0),
            );
            ui.add(egui::Slider::new(&mut params.worlds, WORLDS_RANGE).text("Worlds"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, TICKS_RANGE).text("Ticks/gen"));
        }
    }

    ui.separator();
    ui.heading("Run Mode");
    ui.radio_value(&mut params.run_mode, RunMode::Single, "Single run");
    ui.radio_value(
        &mut params.run_mode,
        RunMode::Ablation,
        format!("Ablation ({ABLATION_STEPS} steps)"),
    );
    ui.radio_value(
        &mut params.run_mode,
        RunMode::Ensemble,
        format!("Ensemble ({ENSEMBLE_SEEDS} seeds)"),
    );

    ui.separator();
    let label = match params.run_mode {
        RunMode::Single => "Run Experiment",
        RunMode::Ablation => "Run Ablation",
        RunMode::Ensemble => "Run Ensemble",
    };
    if ui.button(label).clicked() {
        run_experiment(params, cancer, state);
    }
    if state.wall_ms > 0 {
        ui.label(format!("Last run: {}ms", state.wall_ms));
    }
    if !state.last_csv.is_empty() {
        if ui.button("Export CSV").clicked() {
            let _ = std::fs::write(DEFAULT_EXPORT_PATH, &state.last_csv);
            ui.label(format!("Saved to {}", DEFAULT_EXPORT_PATH));
        }
    }
}

// ─── LR-2: Live controls ────────────────────────────────────────────────────

fn render_live_controls(ui: &mut egui::Ui, params: &mut LabParams) {
    ui.heading("Live Simulation");
    ui.label("Map loaded from RESONANCE_MAP env var.");
    ui.label("Set before running: RESONANCE_MAP=earth cargo run ...");
    ui.separator();

    ui.heading("View Layer");
    ui.radio_value(
        &mut params.view_layer,
        ViewLayer::FrequencyEnergy,
        "Frequency + Energy",
    );
    ui.radio_value(&mut params.view_layer, ViewLayer::EnergyOnly, "Energy Only");
}

// ─── Experiment execution (stateless dispatch) ──────────────────────────────

fn preset_by_index(i: usize) -> presets::UniversePreset {
    match i {
        1 => presets::JUPITER,
        2 => presets::MARS,
        3 => presets::EDEN,
        4 => presets::HELL,
        _ => presets::EARTH,
    }
}

fn run_experiment(params: &LabParams, cancer: &CancerParams, state: &mut LabState) {
    let start = Instant::now();
    let preset = preset_by_index(params.preset_index);

    match params.run_mode {
        RunMode::Ablation => {
            run_ablation(params, &preset, state);
            state.wall_ms = start.elapsed().as_millis() as u64;
            return;
        }
        RunMode::Ensemble => {
            run_ensemble(params, &preset, state);
            state.wall_ms = start.elapsed().as_millis() as u64;
            return;
        }
        RunMode::Single => {}
    }

    state.result = match params.experiment {
        BatchExperiment::Lab => LabResult::Lab(Box::new(lab_exp::run(
            &preset,
            params.seed,
            params.worlds,
            params.generations,
            params.ticks,
        ))),
        BatchExperiment::Fermi => LabResult::Fermi(Box::new(fermi::run(
            params.worlds,
            params.generations,
            params.ticks,
        ))),
        BatchExperiment::Speciation => LabResult::Speciation(Box::new(speciation::run(
            &preset,
            params.seed,
            params.seed.wrapping_add(SPECIATION_SEED_OFFSET),
            params.generations,
            params.ticks,
            SPECIATION_THRESHOLD,
        ))),
        BatchExperiment::Cambrian => LabResult::Cambrian(Box::new(cambrian::run(
            &preset,
            params.seed,
            params.worlds,
            params.generations,
            params.ticks,
            CAMBRIAN_THRESHOLD,
        ))),
        BatchExperiment::Debate => LabResult::Debate(Box::new(debate::run(
            &preset,
            params.worlds.min(DEBATE_MAX_SEEDS),
            params.generations,
            params.ticks,
        ))),
        BatchExperiment::Convergence => LabResult::Convergence(Box::new(convergence::run(
            &preset,
            params.worlds.min(CONVERGENCE_MAX_SEEDS),
            params.generations,
            params.ticks,
            CONVERGENCE_THRESHOLD,
        ))),
        BatchExperiment::CancerTherapy => {
            let cfg = cancer_therapy::TherapyConfig {
                drug_potency: cancer.drug_potency,
                drug_bandwidth: cancer.drug_bandwidth,
                treatment_start_gen: cancer.treatment_start,
                worlds: params.worlds.min(CANCER_MAX_WORLDS),
                generations: params.generations,
                ticks_per_gen: params.ticks.min(CANCER_MAX_TICKS),
                seed: params.seed,
                ..Default::default()
            };
            LabResult::Cancer(Box::new(cancer_therapy::run(&cfg)))
        }
    };
    state.last_csv = result_to_csv(&state.result);
    state.wall_ms = start.elapsed().as_millis() as u64;
}

fn run_ablation(params: &LabParams, preset: &presets::UniversePreset, state: &mut LabState) {
    use resonance::batch::batch::BatchConfig;
    let base = BatchConfig {
        world_count: params.worlds,
        max_generations: params.generations,
        ticks_per_eval: params.ticks,
        seed: params.seed,
        initial_entities: 12,
        ..Default::default()
    };
    let reports = orchestrators::ablate(
        &base,
        preset,
        &(0..ABLATION_STEPS).map(|i| i as f32).collect::<Vec<_>>(),
        |cfg, step| {
            cfg.seed = params.seed.wrapping_add(step as u64 * 1000);
        },
    );
    state.result = LabResult::Ablation(reports);
    state.last_csv = result_to_csv(&state.result);
}

fn run_ensemble(params: &LabParams, preset: &presets::UniversePreset, state: &mut LabState) {
    use resonance::batch::batch::BatchConfig;
    let base = BatchConfig {
        world_count: params.worlds,
        max_generations: params.generations,
        ticks_per_eval: params.ticks,
        seed: params.seed,
        initial_entities: 12,
        ..Default::default()
    };
    let report = orchestrators::ensemble(&base, preset, ENSEMBLE_SEEDS);
    state.result = LabResult::Ensemble(Box::new(report));
    state.last_csv = result_to_csv(&state.result);
}

// ─── CSV export (stateless) ─────────────────────────────────────────────────

fn result_to_csv(result: &LabResult) -> String {
    use std::fmt::Write;
    match result {
        LabResult::None => String::new(),
        LabResult::Lab(r) => export::export_history_csv(&r.history),
        LabResult::Fermi(r) => {
            let mut csv = String::from("universe,species,fitness,diversity\n");
            for (i, rep) in r.reports.iter().enumerate() {
                let last = rep.history.last();
                let _ = write!(
                    csv,
                    "{},{:.2},{:.4},{:.4}\n",
                    i,
                    last.map(|s| s.species_mean).unwrap_or(0.0),
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0)
                );
            }
            csv
        }
        LabResult::Cancer(r) => {
            let mut csv =
                String::from("gen,cancer,normal,freq_mean,resistance,diversity,drug_active\n");
            for s in &r.timeline {
                let _ = write!(
                    csv,
                    "{},{:.2},{:.2},{:.2},{:.4},{:.2},{}\n",
                    s.generation,
                    s.cancer_alive_mean,
                    s.normal_alive_mean,
                    s.cancer_freq_mean,
                    s.resistance_index,
                    s.clonal_diversity,
                    s.drug_active as u8
                );
            }
            csv
        }
        LabResult::Ablation(reports) => {
            let mut csv = String::from("step,best_fitness,mean_fitness,diversity,species\n");
            for (i, r) in reports.iter().enumerate() {
                let last = r.history.last();
                let _ = write!(
                    csv,
                    "{},{:.4},{:.4},{:.4},{:.2}\n",
                    i,
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.mean_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0),
                    last.map(|s| s.species_mean).unwrap_or(0.0)
                );
            }
            csv
        }
        LabResult::Ensemble(e) => {
            let mut csv = String::from("seed,best_fitness,diversity,species\n");
            for (i, r) in e.reports.iter().enumerate() {
                let last = r.history.last();
                let _ = write!(
                    csv,
                    "{},{:.4},{:.4},{:.2}\n",
                    i,
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0),
                    last.map(|s| s.species_mean).unwrap_or(0.0)
                );
            }
            csv
        }
        LabResult::Speciation(_) => String::new(), // TODO: add speciation CSV
        LabResult::Cambrian(_) => String::new(),   // TODO: add cambrian CSV
        LabResult::Debate(_) => String::new(),     // TODO: add debate CSV
        LabResult::Convergence(_) => String::new(), // TODO: add convergence CSV
    }
}

// ─── Batch results rendering ────────────────────────────────────────────────

fn render_results(ui: &mut egui::Ui, state: &LabState) {
    match &state.result {
        LabResult::None => {
            ui.centered_and_justified(|ui| {
                ui.heading("Select an experiment and click Run");
            });
        }
        LabResult::Lab(r) => {
            ui.heading(format!("Universe Lab — {}", r.preset_name));
            render_fitness_chart(ui, &r.history);
            render_top_genomes(ui, &r.top_genomes);
        }
        LabResult::Fermi(r) => {
            ui.heading("Fermi Paradox");
            egui::Grid::new("fermi").show(ui, |ui| {
                ui.label("Universes:");
                ui.label(format!("{}", r.total_universes));
                ui.end_row();
                ui.label("With life:");
                ui.label(format!(
                    "{} ({:.1}%)",
                    r.with_life,
                    r.life_probability * 100.0
                ));
                ui.end_row();
                ui.label("Complex life:");
                ui.label(format!(
                    "{} ({:.1}%)",
                    r.with_complex_life,
                    r.complex_probability * 100.0
                ));
                ui.end_row();
            });
        }
        LabResult::Speciation(r) => {
            ui.heading(format!("Speciation — {}", r.preset_name));
            egui::Grid::new("spec").show(ui, |ui| {
                ui.label("Pop A freq:");
                ui.label(format!("{:.1} Hz", r.mean_freq_a));
                ui.end_row();
                ui.label("Pop B freq:");
                ui.label(format!("{:.1} Hz", r.mean_freq_b));
                ui.end_row();
                ui.label("Interference:");
                ui.label(format!("{:.3}", r.cross_interference));
                ui.end_row();
                ui.label("Speciated:");
                ui.label(if r.speciated { "YES" } else { "NO" });
                ui.end_row();
            });
        }
        LabResult::Cambrian(r) => {
            ui.heading(format!("Cambrian — {}", r.preset_name));
            match r.explosion_gen {
                Some(g) => {
                    ui.label(format!("Explosion at gen {}", g));
                }
                None => {
                    ui.label("No explosion detected.");
                }
            }
            let data: PlotPoints = r
                .diversity_curve
                .iter()
                .enumerate()
                .map(|(i, &v)| [i as f64, v as f64])
                .collect();
            Plot::new("cambrian")
                .height(CHART_HEIGHT_CAMBRIAN)
                .show(ui, |p| {
                    p.line(Line::new(data).name("diversity"));
                });
        }
        LabResult::Debate(r) => {
            ui.heading(format!("Debate — {}", r.preset_name));
            egui::Grid::new("debate").show(ui, |ui| {
                ui.label("Life rate:");
                ui.label(format!("{:.1}%", r.life_rate * 100.0));
                ui.end_row();
                ui.label("Complexity:");
                ui.label(format!("{:.1}%", r.complexity_rate * 100.0));
                ui.end_row();
                ui.label("Cooperation:");
                ui.label(format!("{:.3}", r.cooperation_signal));
                ui.end_row();
            });
        }
        LabResult::Convergence(r) => {
            ui.heading("Convergence");
            egui::Grid::new("conv").show(ui, |ui| {
                ui.label("Seeds:");
                ui.label(format!("{}", r.n_seeds));
                ui.end_row();
                ui.label("Mean dist:");
                ui.label(format!("{:.3}", r.mean_distance));
                ui.end_row();
                ui.label("Convergence:");
                ui.label(format!("{:.1}%", r.convergence_rate * 100.0));
                ui.end_row();
            });
            render_top_genomes(ui, &r.top_genomes);
        }
        LabResult::Cancer(r) => {
            ui.heading("Cancer Therapy");
            egui::Grid::new("cancer").show(ui, |ui| {
                ui.label("Eliminated:");
                ui.label(if r.tumor_eliminated { "YES" } else { "NO" });
                ui.end_row();
                if let Some(g) = r.generations_to_resistance {
                    ui.label("Resistance:");
                    ui.label(format!("gen {g}"));
                    ui.end_row();
                }
                if let Some(g) = r.relapse_gen {
                    ui.label("Relapse:");
                    ui.label(format!("gen {g}"));
                    ui.end_row();
                }
            });
            ui.separator();
            let cancer_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.cancer_alive_mean as f64])
                .collect();
            let normal_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.normal_alive_mean as f64])
                .collect();
            Plot::new("cancer_pop")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    p.line(Line::new(cancer_pts).name("cancer").color(COLOR_CANCER));
                    p.line(Line::new(normal_pts).name("normal").color(COLOR_NORMAL));
                });
            let resist: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.resistance_index as f64])
                .collect();
            Plot::new("cancer_resist")
                .height(CHART_HEIGHT_SMALL)
                .show(ui, |p| {
                    p.line(Line::new(resist).name("resistance").color(COLOR_RESISTANCE));
                });
        }
        LabResult::Ablation(runs) => {
            ui.heading(format!("Ablation — {} runs", runs.len()));
            Plot::new("ablation")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    for (i, r) in runs.iter().enumerate() {
                        let data: PlotPoints = r
                            .history
                            .iter()
                            .map(|s| [s.generation as f64, s.best_fitness as f64])
                            .collect();
                        p.line(
                            Line::new(data)
                                .name(format!("run {i}"))
                                .color(COLOR_ABLATION[i % ABLATION_STEPS]),
                        );
                    }
                });
        }
        LabResult::Ensemble(e) => {
            ui.heading(format!("Ensemble — {} seeds", e.reports.len()));
            egui::Grid::new("ens").show(ui, |ui| {
                ui.label("Mean fitness:");
                ui.label(format!("{:.3}", e.mean_fitness));
                ui.end_row();
                ui.label("Std:");
                ui.label(format!("{:.3}", e.std_fitness));
                ui.end_row();
            });
            Plot::new("ensemble")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    for (i, r) in e.reports.iter().enumerate() {
                        let data: PlotPoints = r
                            .history
                            .iter()
                            .map(|s| [s.generation as f64, s.best_fitness as f64])
                            .collect();
                        p.line(
                            Line::new(data)
                                .name(format!("seed {i}"))
                                .color(COLOR_ABLATION[i % ABLATION_STEPS]),
                        );
                    }
                });
        }
    }
}

// ─── Live 2D rendering ──────────────────────────────────────────────────────

fn render_live_2d(
    ui: &mut egui::Ui,
    grid: &Option<Res<EnergyFieldGrid>>,
    clock: &Option<Res<SimulationClock>>,
    entities: &Query<(
        &Transform,
        &BaseEnergy,
        &SpatialVolume,
        &OscillatorySignature,
    )>,
    params: &LabParams,
) {
    let tick = clock.as_ref().map(|c| c.tick_id).unwrap_or(0);
    let alive = entities.iter().filter(|(_, e, _, _)| e.qe() > 0.0).count();
    ui.label(format!("Tick: {} | Alive: {}", tick, alive));
    ui.separator();

    let Some(grid) = grid.as_ref() else {
        ui.label("Waiting for simulation (loading map)...");
        return;
    };

    let available = ui.available_size();
    let canvas_side = available.x.min(available.y).max(200.0);
    let cell_px = canvas_side / grid.width.max(1) as f32;

    let (response, painter) =
        ui.allocate_painter(egui::Vec2::splat(canvas_side), egui::Sense::hover());
    let origin = response.rect.min;

    // Heatmap
    let mut max_qe = 1.0_f32;
    for cell in grid.iter_cells() {
        max_qe = max_qe.max(cell.accumulated_qe);
    }

    let w = grid.width as usize;
    for (idx, cell) in grid.iter_cells().enumerate() {
        let gx = (idx % w) as f32;
        let gy = (idx / w) as f32;
        let t = (cell.accumulated_qe / max_qe).clamp(0.0, 1.0);

        let (r, g, b) = match params.view_layer {
            ViewLayer::FrequencyEnergy => {
                let hue = if cell.dominant_frequency_hz > 0.0 {
                    (cell.dominant_frequency_hz / FREQ_HUE_MAX).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                hsv_to_rgb(hue, cell.purity.clamp(0.1, 1.0), (t.sqrt() * 1.2).min(1.0))
            }
            ViewLayer::EnergyOnly => {
                let v = (t.sqrt() * 255.0).min(255.0) as u8;
                (v / 4, v, v / 3) // green-tinted energy
            }
        };

        let rect = egui::Rect::from_min_size(
            egui::Pos2::new(origin.x + gx * cell_px, origin.y + gy * cell_px),
            egui::Vec2::splat(cell_px + 1.0),
        );
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(r, g, b));
    }

    // Entities
    for (transform, energy, volume, osc) in entities {
        if energy.qe() <= 0.0 {
            continue;
        }
        let rel_x = transform.translation.x - grid.origin.x;
        let rel_y = transform.translation.z - grid.origin.y;
        let px = origin.x + (rel_x / grid.cell_size) * cell_px;
        let py = origin.y + (rel_y / grid.cell_size) * cell_px;
        let radius_px = (volume.radius * cell_px * 0.3).clamp(3.0, cell_px * 0.4);
        let hue = (osc.frequency_hz() / FREQ_HUE_MAX).clamp(0.0, 1.0);
        let brightness = (energy.qe() / ENTITY_QE_BRIGHTNESS_REF).clamp(0.4, 1.0);
        let (r, g, b) = hsv_to_rgb(hue, 0.9, brightness);
        let center = egui::Pos2::new(px, py);
        painter.circle_filled(center, radius_px, egui::Color32::from_rgb(r, g, b));
        painter.circle_stroke(
            center,
            radius_px,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
        );
    }

    // Border
    painter.rect_stroke(
        egui::Rect::from_min_size(origin, egui::Vec2::splat(canvas_side)),
        2.0,
        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 80, 80)),
    );
}

// ─── Shared helpers (pure, stateless) ───────────────────────────────────────

/// HSV to RGB. h in [0,1], s in [0,1], v in [0,1]. Pure, stateless.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.clamp(0.0, 1.0) * 6.0;
    let c = v.clamp(0.0, 1.0) * s.clamp(0.0, 1.0);
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v.clamp(0.0, 1.0) - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn render_fitness_chart(ui: &mut egui::Ui, history: &[resonance::batch::harness::GenerationStats]) {
    if history.is_empty() {
        return;
    }
    let best: PlotPoints = history
        .iter()
        .map(|s| [s.generation as f64, s.best_fitness as f64])
        .collect();
    let mean: PlotPoints = history
        .iter()
        .map(|s| [s.generation as f64, s.mean_fitness as f64])
        .collect();
    Plot::new("fitness")
        .height(CHART_HEIGHT_MAIN)
        .show(ui, |p| {
            p.line(Line::new(best).name("best").color(COLOR_BEST));
            p.line(Line::new(mean).name("mean").color(COLOR_MEAN));
        });
}

fn render_top_genomes(ui: &mut egui::Ui, genomes: &[resonance::batch::genome::GenomeBlob]) {
    if genomes.is_empty() {
        return;
    }
    ui.separator();
    ui.label("Top Genomes");
    egui::Grid::new("genomes").striped(true).show(ui, |ui| {
        ui.label("#");
        ui.label("Arch");
        ui.label("Growth");
        ui.label("Mob");
        ui.label("Branch");
        ui.label("Resil");
        ui.end_row();
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
