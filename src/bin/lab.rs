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
use resonance::use_cases::export;
use resonance::use_cases::orchestrators;
use resonance::use_cases::presets;
use resonance::plugins::{LayersPlugin, SimulationPlugin, SimulationTickPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::SimulationClock;
use resonance::worldgen::EnergyFieldGrid;
use resonance::layers::{BaseEnergy, OscillatorySignature, SpatialVolume};

// ─── Layout / range constants (visual calibration, no physics) ──────────────

const CONTROL_PANEL_WIDTH: f32 = 280.0;
const CHART_HEIGHT_MAIN: f32   = 250.0;
const CHART_HEIGHT_SMALL: f32  = 200.0;
const CHART_HEIGHT_CAMBRIAN: f32 = 300.0;

const WORLDS_RANGE: std::ops::RangeInclusive<usize> = 10..=2000;
const GENS_RANGE: std::ops::RangeInclusive<u32>     = 10..=1000;
const TICKS_RANGE: std::ops::RangeInclusive<u32>    = 50..=2000;
const POTENCY_RANGE: std::ops::RangeInclusive<f32>  = 0.1..=10.0;
const BANDWIDTH_RANGE: std::ops::RangeInclusive<f32> = 10.0..=200.0;
const TREATMENT_START_RANGE: std::ops::RangeInclusive<u32> = 0..=50;

const SPECIATION_SEED_OFFSET: u64 = 7777;
const SPECIATION_THRESHOLD: f32   = 0.5;
const CAMBRIAN_THRESHOLD: f32     = 0.3;
const CONVERGENCE_THRESHOLD: f32  = 0.3;
const DEBATE_MAX_SEEDS: usize     = 50;
const CONVERGENCE_MAX_SEEDS: usize = 100;
const CANCER_MAX_WORLDS: usize    = 200;
const CANCER_MAX_TICKS: u32       = 500;
const ABLATION_STEPS: usize       = 8;
const ENSEMBLE_SEEDS: usize       = 10;
const DEFAULT_EXPORT_PATH: &str   = "lab_results.csv";

// ─── Chart colors (visual identity) ─────────────────────────────────────────

const COLOR_BEST: egui::Color32       = egui::Color32::GREEN;
const COLOR_MEAN: egui::Color32       = egui::Color32::YELLOW;
const COLOR_CANCER: egui::Color32     = egui::Color32::RED;
const COLOR_NORMAL: egui::Color32     = egui::Color32::GREEN;
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
    Live2D,
}

const EXPERIMENT_NAMES: &[(Experiment, &str)] = &[
    (Experiment::Lab,           "Universe Lab"),
    (Experiment::Fermi,         "Fermi Paradox"),
    (Experiment::Speciation,    "Speciation"),
    (Experiment::Cambrian,      "Cambrian Explosion"),
    (Experiment::Debate,        "Debate (Cooperation)"),
    (Experiment::Convergence,   "Convergence"),
    (Experiment::CancerTherapy, "Cancer Therapy"),
    (Experiment::Live2D,        "Live 2D Sim"),
];

// ─── Shared parameters ──────────────────────────────────────────────────────

/// Modo de ejecución del lab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RunMode {
    #[default]
    Single,
    Ablation,
    Ensemble,
}

/// Parámetros compartidos entre todos los experimentos.
#[derive(Resource)]
struct LabParams {
    experiment:   Experiment,
    preset_index: usize,
    seed:         u64,
    worlds:       usize,
    generations:  u32,
    ticks:        u32,
    run_mode:     RunMode,
}

impl Default for LabParams {
    fn default() -> Self {
        Self {
            experiment:   Experiment::default(),
            preset_index: 0,
            seed:         42,
            worlds:       100,
            generations:  100,
            ticks:        500,
            run_mode:     RunMode::default(),
        }
    }
}

/// Parámetros específicos de cancer therapy.
#[derive(Resource)]
struct CancerParams {
    drug_potency:    f32,
    drug_bandwidth:  f32,
    treatment_start: u32,
}

impl Default for CancerParams {
    fn default() -> Self {
        Self { drug_potency: 2.0, drug_bandwidth: 50.0, treatment_start: 5 }
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
    Ablation(Vec<resonance::use_cases::ExperimentReport>),
    Ensemble(Box<orchestrators::EnsembleReport>),
}

#[derive(Resource, Default)]
struct LabState {
    result:   LabResult,
    wall_ms:  u64,
    last_csv: String,
}

/// Flag: indica si el runtime de simulación fue inicializado.
#[derive(Resource, Default)]
struct LiveSimInitialized(bool);

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
        // Simulation runtime (for Live 2D — uses real EnergyFieldGrid)
        .add_plugins(SimulationTickPlugin)
        .init_resource::<PaletteRegistry>()
        .insert_resource(SimWorldTransformParams::default())
        .add_plugins(LayersPlugin)
        .add_plugins(SimulationPlugin)
        // Lab state
        .init_resource::<LabParams>()
        .init_resource::<CancerParams>()
        .init_resource::<LabState>()
        .init_resource::<LiveSimInitialized>()
        .add_systems(Update, (
            lab_controls_system,
            lab_live2d_system,
        ).chain())
        .run();
}

// ─── UI system ──────────────────────────────────────────────────────────────

fn lab_controls_system(
    mut contexts: EguiContexts,
    mut params:   ResMut<LabParams>,
    mut cancer:   ResMut<CancerParams>,
    mut state:    ResMut<LabState>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else { return };
    let is_live = params.experiment == Experiment::Live2D;

    egui::SidePanel::left("lab_controls").default_width(CONTROL_PANEL_WIDTH).show(ctx, |ui| {
        render_experiment_selector(ui, &mut params);
        ui.separator();
        render_shared_params(ui, &mut params);
        ui.separator();
        render_experiment_params(ui, &params, &mut cancer);
        ui.separator();
        if !is_live {
            render_run_button(ui, &params, &cancer, &mut state);
        }
    });

    if !is_live {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_results(ui, &state);
        });
    }
}

fn lab_live2d_system(
    mut contexts:  EguiContexts,
    params:        Res<LabParams>,
    grid:          Option<Res<EnergyFieldGrid>>,
    clock:         Option<Res<SimulationClock>>,
    entity_query:  Query<(&Transform, &BaseEnergy, &SpatialVolume, &OscillatorySignature)>,
) {
    if params.experiment != Experiment::Live2D { return; }
    let Some(ctx) = contexts.try_ctx_mut() else { return };

    egui::CentralPanel::default().show(ctx, |ui| {
        let tick = clock.as_ref().map(|c| c.tick_id).unwrap_or(0);
        let alive = entity_query.iter().filter(|(_, e, _, _)| e.qe() > 0.0).count();

        ui.horizontal(|ui| {
            ui.label(format!("Tick: {} | Alive: {}", tick, alive));
        });
        ui.separator();

        let Some(grid) = grid.as_ref() else {
            ui.label("Waiting for simulation to initialize (loading map)...");
            return;
        };

        // Scale to fill panel
        let available = ui.available_size();
        let canvas_side = available.x.min(available.y).max(200.0);
        let cell_px = canvas_side / grid.width.max(1) as f32;

        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(canvas_side, canvas_side),
            egui::Sense::hover(),
        );
        let origin = response.rect.min;

        // Layer 1: Energy field heatmap from real EnergyFieldGrid
        let mut max_qe = 1.0_f32;
        for cell in grid.iter_cells() {
            max_qe = max_qe.max(cell.accumulated_qe);
        }

        let w = grid.width as usize;
        for (idx, cell) in grid.iter_cells().enumerate() {
            let gx = (idx % w) as f32;
            let gy = (idx / w) as f32;
            let t = (cell.accumulated_qe / max_qe).clamp(0.0, 1.0);

            // Color by frequency + energy: hue from dominant freq, brightness from qe
            let hue = if cell.dominant_frequency_hz > 0.0 {
                (cell.dominant_frequency_hz / 800.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let sat = cell.purity.clamp(0.1, 1.0);
            let val = (t.sqrt() * 1.2).min(1.0); // sqrt for better low-end visibility
            let (r, g, b) = hsv_to_rgb_tuple(hue, sat, val);

            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(origin.x + gx * cell_px, origin.y + gy * cell_px),
                egui::Vec2::splat(cell_px),
            );
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(r, g, b));
        }

        // Layer 2: Grid lines
        let grid_side = grid.width.max(grid.height);
        for i in 0..=grid_side {
            let offset = i as f32 * cell_px;
            let line_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, 12);
            painter.line_segment(
                [egui::Pos2::new(origin.x + offset, origin.y),
                 egui::Pos2::new(origin.x + offset, origin.y + canvas_side)],
                egui::Stroke::new(0.5, line_color),
            );
            painter.line_segment(
                [egui::Pos2::new(origin.x, origin.y + offset),
                 egui::Pos2::new(origin.x + canvas_side, origin.y + offset)],
                egui::Stroke::new(0.5, line_color),
            );
        }

        // Layer 3: Entities from real Bevy ECS
        let xz_ground = true; // Resonance uses XZ ground plane
        for (transform, energy, volume, osc) in &entity_query {
            if energy.qe() <= 0.0 { continue; }
            let pos = if xz_ground {
                (transform.translation.x, transform.translation.z)
            } else {
                (transform.translation.x, transform.translation.y)
            };
            // Convert world pos to grid pos
            let rel_x = pos.0 - grid.origin.x;
            let rel_y = pos.1 - grid.origin.y;
            let px = origin.x + (rel_x / grid.cell_size) * cell_px;
            let py = origin.y + (rel_y / grid.cell_size) * cell_px;

            let radius_px = (volume.radius * cell_px * 0.3).clamp(3.0, cell_px * 0.4);
            let hue = (osc.frequency_hz() / 800.0).clamp(0.0, 1.0);
            let brightness = (energy.qe() / 50.0).clamp(0.4, 1.0);
            let (r, g, b) = hsv_to_rgb_tuple(hue, 0.9, brightness);
            let center = egui::Pos2::new(px, py);

            painter.circle_filled(center, radius_px, egui::Color32::from_rgb(r, g, b));
            painter.circle_stroke(center, radius_px, egui::Stroke::new(1.0, egui::Color32::WHITE));
        }

        // Layer 4: Border
        painter.rect_stroke(
            egui::Rect::from_min_size(origin, egui::Vec2::splat(canvas_side)),
            2.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
        );
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
    ui.add(egui::Slider::new(&mut params.worlds, WORLDS_RANGE).text("Worlds"));
    ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
    ui.add(egui::Slider::new(&mut params.ticks, TICKS_RANGE).text("Ticks/gen"));

    ui.separator();
    ui.heading("Run Mode");
    ui.radio_value(&mut params.run_mode, RunMode::Single, "Single run");
    ui.radio_value(&mut params.run_mode, RunMode::Ablation, format!("Ablation ({ABLATION_STEPS} steps)"));
    ui.radio_value(&mut params.run_mode, RunMode::Ensemble, format!("Ensemble ({ENSEMBLE_SEEDS} seeds)"));
}

fn render_experiment_params(ui: &mut egui::Ui, params: &LabParams, cancer: &mut CancerParams) {
    match params.experiment {
        Experiment::CancerTherapy => {
            ui.heading("Cancer Therapy");
            ui.add(egui::Slider::new(&mut cancer.drug_potency, POTENCY_RANGE).text("Drug potency"));
            ui.add(egui::Slider::new(&mut cancer.drug_bandwidth, BANDWIDTH_RANGE).text("Drug bandwidth (Hz)"));
            ui.add(egui::Slider::new(&mut cancer.treatment_start, TREATMENT_START_RANGE).text("Treatment start (gen)"));
        }
        _ => {
            ui.label("No additional parameters for this experiment.");
        }
    }
}

fn render_run_button(ui: &mut egui::Ui, params: &LabParams, cancer: &CancerParams, state: &mut LabState) {
    let mode_label = match params.run_mode {
        RunMode::Single   => "Run Experiment",
        RunMode::Ablation => "Run Ablation",
        RunMode::Ensemble => "Run Ensemble",
    };
    if ui.button(mode_label).clicked() {
        run_experiment(params, cancer, state);
    }
    if state.wall_ms > 0 {
        ui.label(format!("Last run: {}ms", state.wall_ms));
    }

    // CSV export
    if !state.last_csv.is_empty() {
        ui.separator();
        if ui.button("Export CSV").clicked() {
            match std::fs::write(DEFAULT_EXPORT_PATH, &state.last_csv) {
                Ok(()) => { ui.label(format!("Saved to {}", DEFAULT_EXPORT_PATH)); }
                Err(e) => { ui.colored_label(egui::Color32::RED, format!("Error: {}", e)); }
            }
        }
    }
}

// ─── Experiment execution (stateless dispatch) ──────────────────────────────

fn run_experiment(params: &LabParams, cancer: &CancerParams, state: &mut LabState) {
    let start = Instant::now();
    let preset = preset_by_index(params.preset_index);

    // Dispatch by run mode first
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
        Experiment::Lab => {
            let r = lab_exp::run(&preset, params.seed, params.worlds, params.generations, params.ticks);
            LabResult::Lab(Box::new(r))
        }
        Experiment::Fermi => {
            let r = fermi::run(params.worlds, params.generations, params.ticks);
            LabResult::Fermi(Box::new(r))
        }
        Experiment::Speciation => {
            let r = speciation::run(&preset, params.seed, params.seed.wrapping_add(SPECIATION_SEED_OFFSET), params.generations, params.ticks, SPECIATION_THRESHOLD);
            LabResult::Speciation(Box::new(r))
        }
        Experiment::Cambrian => {
            let r = cambrian::run(&preset, params.seed, params.worlds, params.generations, params.ticks, CAMBRIAN_THRESHOLD);
            LabResult::Cambrian(Box::new(r))
        }
        Experiment::Debate => {
            let r = debate::run(&preset, params.worlds.min(DEBATE_MAX_SEEDS), params.generations, params.ticks);
            LabResult::Debate(Box::new(r))
        }
        Experiment::Convergence => {
            let r = convergence::run(&preset, params.worlds.min(CONVERGENCE_MAX_SEEDS), params.generations, params.ticks, CONVERGENCE_THRESHOLD);
            LabResult::Convergence(Box::new(r))
        }
        Experiment::CancerTherapy => {
            let cfg = cancer_therapy::TherapyConfig {
                drug_potency:        cancer.drug_potency,
                drug_bandwidth:      cancer.drug_bandwidth,
                treatment_start_gen: cancer.treatment_start,
                worlds:              params.worlds.min(CANCER_MAX_WORLDS),
                generations:         params.generations,
                ticks_per_gen:       params.ticks.min(CANCER_MAX_TICKS),
                seed:                params.seed,
                ..Default::default()
            };
            let r = cancer_therapy::run(&cfg);
            LabResult::Cancer(Box::new(r))
        }
        Experiment::Live2D => return, // Live2D is handled by render_live_2d, not batch
    };

    // Generate CSV for export
    state.last_csv = result_to_csv(&state.result);
    state.wall_ms = start.elapsed().as_millis() as u64;
}

/// Convierte cualquier resultado a CSV. Stateless, pure.
fn result_to_csv(result: &LabResult) -> String {
    match result {
        LabResult::None => String::new(),
        LabResult::Lab(r) => export::export_history_csv(&r.history),
        LabResult::Fermi(r) => {
            let mut csv = String::from("universe,species,fitness,diversity\n");
            for (i, rep) in r.reports.iter().enumerate() {
                let last = rep.history.last();
                use std::fmt::Write;
                let _ = write!(csv, "{},{:.2},{:.4},{:.4}\n",
                    i,
                    last.map(|s| s.species_mean).unwrap_or(0.0),
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0),
                );
            }
            csv
        }
        LabResult::Cancer(r) => {
            let mut csv = String::from("gen,cancer,normal,freq_mean,resistance,diversity,drug_active\n");
            for s in &r.timeline {
                use std::fmt::Write;
                let _ = write!(csv, "{},{:.2},{:.2},{:.2},{:.4},{:.2},{}\n",
                    s.generation, s.cancer_alive_mean, s.normal_alive_mean,
                    s.cancer_freq_mean, s.resistance_index, s.clonal_diversity,
                    s.drug_active as u8,
                );
            }
            csv
        }
        LabResult::Ablation(reports) => {
            let mut csv = String::from("step,best_fitness,mean_fitness,diversity,species\n");
            for (i, r) in reports.iter().enumerate() {
                let last = r.history.last();
                use std::fmt::Write;
                let _ = write!(csv, "{},{:.4},{:.4},{:.4},{:.2}\n",
                    i,
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.mean_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0),
                    last.map(|s| s.species_mean).unwrap_or(0.0),
                );
            }
            csv
        }
        LabResult::Ensemble(e) => {
            let mut csv = String::from("seed,best_fitness,diversity,species\n");
            for (i, r) in e.reports.iter().enumerate() {
                let last = r.history.last();
                use std::fmt::Write;
                let _ = write!(csv, "{},{:.4},{:.4},{:.2}\n",
                    i,
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0),
                    last.map(|s| s.species_mean).unwrap_or(0.0),
                );
            }
            csv
        }
        _ => String::new(), // Speciation, Cambrian, Debate, Convergence — add as needed
    }
}

// ─── Ablation & Ensemble execution (stateless) ─────────────────────────────

/// Ablation: sweep seed sobre N valores equidistantes. Stateless.
fn run_ablation(params: &LabParams, preset: &presets::UniversePreset, state: &mut LabState) {
    use resonance::batch::batch::BatchConfig;

    let base = BatchConfig {
        world_count:     params.worlds,
        max_generations: params.generations,
        ticks_per_eval:  params.ticks,
        seed:            params.seed,
        initial_entities: 12,
        ..Default::default()
    };

    let reports = orchestrators::ablate(&base, preset,
        &(0..ABLATION_STEPS).map(|i| i as f32).collect::<Vec<_>>(),
        |cfg, step| { cfg.seed = params.seed.wrapping_add(step as u64 * 1000); },
    );

    state.result = LabResult::Ablation(reports);
    state.last_csv = result_to_csv(&state.result);
}

/// Ensemble: misma config con N seeds distintas. Stateless.
fn run_ensemble(params: &LabParams, preset: &presets::UniversePreset, state: &mut LabState) {
    use resonance::batch::batch::BatchConfig;

    let base = BatchConfig {
        world_count:     params.worlds,
        max_generations: params.generations,
        ticks_per_eval:  params.ticks,
        seed:            params.seed,
        initial_entities: 12,
        ..Default::default()
    };

    let report = orchestrators::ensemble(&base, preset, ENSEMBLE_SEEDS);
    state.result = LabResult::Ensemble(Box::new(report));
    state.last_csv = result_to_csv(&state.result);
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
        LabResult::Ablation(runs)  => render_ablation_result(ui, runs),
        LabResult::Ensemble(e)     => render_ensemble_result(ui, e),
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
    Plot::new("cambrian_diversity").height(CHART_HEIGHT_CAMBRIAN).show(ui, |plot_ui| {
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
    Plot::new("cancer_pop").height(CHART_HEIGHT_MAIN).show(ui, |plot_ui| {
        plot_ui.line(Line::new(cancer).name("cancer").color(COLOR_CANCER));
        plot_ui.line(Line::new(normal).name("normal").color(COLOR_NORMAL));
    });

    // Resistance index
    let resist: PlotPoints = r.timeline.iter()
        .map(|s| [s.generation as f64, s.resistance_index as f64]).collect();
    Plot::new("cancer_resist").height(CHART_HEIGHT_SMALL).show(ui, |plot_ui| {
        plot_ui.line(Line::new(resist).name("resistance").color(COLOR_RESISTANCE));
    });
}

// ─── Live 2D Simulation ─────────────────────────────────────────────────────

/// HSV to RGB tuple. Pure, stateless. h in [0,1], s in [0,1], v in [0,1].
fn hsv_to_rgb_tuple(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.clamp(0.0, 1.0) * 6.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}

// ─── Ablation / Ensemble results ────────────────────────────────────────────

fn render_ablation_result(ui: &mut egui::Ui, runs: &[resonance::use_cases::ExperimentReport]) {
    ui.heading(format!("Ablation — {} runs", runs.len()));

    // Overlay fitness curves from all runs
    Plot::new("ablation_fitness").height(CHART_HEIGHT_MAIN).show(ui, |plot_ui| {
        for (i, r) in runs.iter().enumerate() {
            let data: PlotPoints = r.history.iter()
                .map(|s| [s.generation as f64, s.best_fitness as f64])
                .collect();
            let color = COLOR_ABLATION[i % COLOR_ABLATION.len()];
            plot_ui.line(Line::new(data).name(format!("run {}", i)).color(color));
        }
    });

    // Summary table
    ui.separator();
    egui::Grid::new("ablation_grid").striped(true).show(ui, |ui| {
        ui.label("Run"); ui.label("Best"); ui.label("Mean"); ui.label("Diversity"); ui.label("Species"); ui.end_row();
        for (i, r) in runs.iter().enumerate() {
            let Some(last) = r.history.last() else { continue };
            ui.label(format!("{}", i));
            ui.label(format!("{:.3}", last.best_fitness));
            ui.label(format!("{:.3}", last.mean_fitness));
            ui.label(format!("{:.3}", last.diversity));
            ui.label(format!("{:.1}", last.species_mean));
            ui.end_row();
        }
    });
}

fn render_ensemble_result(ui: &mut egui::Ui, e: &orchestrators::EnsembleReport) {
    ui.heading(format!("Ensemble — {} seeds", e.reports.len()));

    // Statistics
    egui::Grid::new("ensemble_stats").show(ui, |ui| {
        ui.label("Mean fitness:");  ui.label(format!("{:.3}", e.mean_fitness));   ui.end_row();
        ui.label("Std fitness:");   ui.label(format!("{:.3}", e.std_fitness));    ui.end_row();
        ui.label("Mean diversity:"); ui.label(format!("{:.3}", e.mean_diversity)); ui.end_row();
        ui.label("Mean species:");  ui.label(format!("{:.1}", e.mean_species));   ui.end_row();
    });

    ui.separator();

    // Overlay fitness curves from all seeds
    Plot::new("ensemble_fitness").height(CHART_HEIGHT_MAIN).show(ui, |plot_ui| {
        for (i, r) in e.reports.iter().enumerate() {
            let data: PlotPoints = r.history.iter()
                .map(|s| [s.generation as f64, s.best_fitness as f64])
                .collect();
            let color = COLOR_ABLATION[i % COLOR_ABLATION.len()];
            plot_ui.line(Line::new(data).name(format!("seed {}", i)).color(color));
        }
    });

    // Per-seed summary
    ui.separator();
    egui::Grid::new("ensemble_seeds").striped(true).show(ui, |ui| {
        ui.label("Seed"); ui.label("Best"); ui.label("Diversity"); ui.label("Species"); ui.end_row();
        for (i, r) in e.reports.iter().enumerate() {
            let Some(last) = r.history.last() else { continue };
            ui.label(format!("{}", i));
            ui.label(format!("{:.3}", last.best_fitness));
            ui.label(format!("{:.3}", last.diversity));
            ui.label(format!("{:.1}", last.species_mean));
            ui.end_row();
        }
    });
}

// ─── Shared render helpers ──────────────────────────────────────────────────

fn render_fitness_chart(ui: &mut egui::Ui, history: &[resonance::batch::harness::GenerationStats]) {
    if history.is_empty() { return; }
    let best: PlotPoints = history.iter()
        .map(|s| [s.generation as f64, s.best_fitness as f64]).collect();
    let mean: PlotPoints = history.iter()
        .map(|s| [s.generation as f64, s.mean_fitness as f64]).collect();
    Plot::new("fitness_chart").height(CHART_HEIGHT_MAIN).show(ui, |plot_ui| {
        plot_ui.line(Line::new(best).name("best").color(COLOR_BEST));
        plot_ui.line(Line::new(mean).name("mean").color(COLOR_MEAN));
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
