//! Laboratorio universal — todos los use cases en una interfaz composable.
//! Universal laboratory — all use cases in one composable interface.
//!
//! Arquitectura: LabMode state machine → dispatch controls + central view.
//! Cada experiment define qué controles muestra y cómo renderiza resultados.
//! Zero lógica condicional dispersa. Zero `if is_live` flags.
//! 15 experiments en 4 categorías (ADR-018).
//!
//! Usage:
//!   cargo run --release --bin lab
//!   RESONANCE_MAP=earth cargo run --release --bin lab

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use egui_plot::{Line, Plot, PlotPoints};

use resonance::layers::{BaseEnergy, OscillatorySignature, SpatialVolume};
use resonance::plugins::{LayersPlugin, SimulationPlugin, SimulationTickPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::SimulationClock;
use resonance::simulation::{GameState, PlayState};
use resonance::use_cases::cli::archetype_label;
use resonance::use_cases::experiments::{
    cambrian, cancer_therapy, convergence, debate, fermi, lab as lab_exp,
    paper_foo_michor2009, paper_michor2005, paper_sharma2010, paper_unified_axioms,
    paper_zhang2022, particle_lab, pathway_inhibitor_exp, personal, speciation,
};
use resonance::use_cases::export;
use resonance::use_cases::orchestrators;
use resonance::use_cases::presets;
use resonance::worldgen::{EnergyFieldGrid, NutrientFieldGrid};

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
const FREQ_HUE_MAX: f32 = 800.0;
const ENTITY_QE_BRIGHTNESS_REF: f32 = 50.0;

const COLOR_BEST: egui::Color32 = egui::Color32::GREEN;
const COLOR_MEAN: egui::Color32 = egui::Color32::YELLOW;
const COLOR_CANCER: egui::Color32 = egui::Color32::RED;
const COLOR_NORMAL: egui::Color32 = egui::Color32::GREEN;
const COLOR_RESISTANCE: egui::Color32 = egui::Color32::from_rgb(255, 180, 50);
const COLOR_CONTINUOUS: egui::Color32 = egui::Color32::from_rgb(66, 133, 244);
const COLOR_ADAPTIVE: egui::Color32 = egui::Color32::from_rgb(234, 67, 53);
const COLOR_DIFF: egui::Color32 = egui::Color32::from_rgb(234, 67, 53);
const COLOR_PROG: egui::Color32 = egui::Color32::from_rgb(251, 188, 4);
const COLOR_STEM: egui::Color32 = egui::Color32::from_rgb(52, 168, 83);
const COLOR_WILDTYPE: egui::Color32 = egui::Color32::from_rgb(66, 133, 244);
const COLOR_BONDS: egui::Color32 = egui::Color32::from_rgb(154, 66, 244);
const COLOR_KINETIC: egui::Color32 = egui::Color32::from_rgb(244, 154, 66);
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

// ─── State Machine (ADR-018) ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LabMode {
    #[default]
    Batch,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BatchExperiment {
    // Core Simulation
    #[default]
    Lab,
    Fermi,
    Speciation,
    Cambrian,
    Debate,
    Convergence,
    Personal,
    // Drug & Therapy
    CancerTherapy,
    PathwayInhibitor,
    // Paper Validation
    PaperZhang2022,
    PaperSharma2010,
    PaperFooMichor2009,
    PaperMichor2005,
    PaperUnifiedAxioms,
    // Physics
    ParticleLab,
}

/// Categorías de experiments para la UI.
const CATEGORY_CORE: &[(BatchExperiment, &str)] = &[
    (BatchExperiment::Lab, "Universe Lab"),
    (BatchExperiment::Fermi, "Fermi Paradox"),
    (BatchExperiment::Speciation, "Speciation"),
    (BatchExperiment::Cambrian, "Cambrian Explosion"),
    (BatchExperiment::Debate, "Debate (Cooperation)"),
    (BatchExperiment::Convergence, "Convergence"),
    (BatchExperiment::Personal, "Personal Universe"),
];

const CATEGORY_DRUG: &[(BatchExperiment, &str)] = &[
    (BatchExperiment::CancerTherapy, "Cancer Therapy"),
    (BatchExperiment::PathwayInhibitor, "Pathway Inhibitor"),
];

const CATEGORY_PAPER: &[(BatchExperiment, &str)] = &[
    (BatchExperiment::PaperZhang2022, "Zhang 2022 — Adaptive"),
    (BatchExperiment::PaperSharma2010, "Sharma 2010 — Persisters"),
    (BatchExperiment::PaperFooMichor2009, "Foo & Michor 2009 — Pulsed"),
    (BatchExperiment::PaperMichor2005, "Michor 2005 — Biphasic CML"),
    (BatchExperiment::PaperUnifiedAxioms, "PV-6 Unified Axioms"),
];

const CATEGORY_PHYSICS: &[(BatchExperiment, &str)] = &[
    (BatchExperiment::ParticleLab, "Particle Lab"),
];

impl BatchExperiment {
    /// Whether this experiment supports Ablation/Ensemble run modes.
    /// Only experiments returning ExperimentReport are compatible.
    fn supports_multi_run(self) -> bool {
        matches!(
            self,
            Self::Lab
                | Self::Fermi
                | Self::Speciation
                | Self::Cambrian
                | Self::Debate
                | Self::Convergence
                | Self::Personal
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RunMode {
    #[default]
    Single,
    Ablation,
    Ensemble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ViewLayer {
    #[default]
    FrequencyEnergy,
    EnergyOnly,
}

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
    personal_input: String,
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
            personal_input: String::new(),
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
    // New experiments (ADR-018)
    Personal(Box<resonance::use_cases::ExperimentReport>),
    PathwayInhibitor(Box<pathway_inhibitor_exp::InhibitorReport>),
    PaperZhang(Box<paper_zhang2022::ZhangReport>),
    PaperSharma(Box<paper_sharma2010::SharmaReport>),
    PaperFooMichor(Box<paper_foo_michor2009::FooMichorReport>),
    PaperMichor(Box<paper_michor2005::MichorReport>),
    PaperUnified(Box<paper_unified_axioms::UnifiedReport>),
    ParticleLab(Box<particle_lab::ParticleLabReport>),
}

#[derive(Resource, Default)]
struct LabState {
    result: LabResult,
    wall_ms: u64,
    last_csv: String,
}

// ─── LR-2: Live simulation controls (ADR-019) ──────────────────────────────

const BASE_HZ: f64 = 60.0;
const SPEED_MIN: f32 = 0.25;
const SPEED_MAX: f32 = 4.0;

/// Available map slugs (read from assets/maps/ at startup).
#[derive(Resource)]
struct AvailableMaps(Vec<String>);

impl Default for AvailableMaps {
    fn default() -> Self {
        let mut maps = Vec::new();
        if let Ok(entries) = std::fs::read_dir("assets/maps") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "ron") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        maps.push(stem.to_string());
                    }
                }
            }
        }
        maps.sort();
        Self(maps)
    }
}

/// Speed multiplier for the Live 2D simulation. UI-only state.
#[derive(Resource)]
struct SpeedScale(f32);

impl Default for SpeedScale {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Pending world reset. None = no reset. Some(None) = reset same map. Some(Some(slug)) = load new map.
#[derive(Resource, Default)]
struct PendingReset(Option<Option<String>>);

impl PendingReset {
    fn request_same_map(&mut self) {
        self.0 = Some(None);
    }
    fn request_new_map(&mut self, slug: String) {
        self.0 = Some(Some(slug));
    }
    fn take(&mut self) -> Option<Option<String>> {
        self.0.take()
    }
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
        .init_resource::<AvailableMaps>()
        .init_resource::<SpeedScale>()
        .init_resource::<PendingReset>()
        .add_systems(Update, (controls_system, central_system).chain())
        .add_systems(Update, reset_world_system.run_if(|r: Res<PendingReset>| r.0.is_some()))
        .run();
}

// ─── Composed dispatch (controls + central by mode) ─────────────────────────

fn controls_system(
    mut contexts: EguiContexts,
    mut params: ResMut<LabParams>,
    mut cancer: ResMut<CancerParams>,
    mut state: ResMut<LabState>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut time_fixed: ResMut<Time<Fixed>>,
    mut speed: ResMut<SpeedScale>,
    mut pending_reset: ResMut<PendingReset>,
    maps: Res<AvailableMaps>,
    active_map: Option<Res<resonance::worldgen::ActiveMapName>>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::SidePanel::left("lab_controls")
        .default_width(CONTROL_PANEL_WIDTH)
        .show(ctx, |ui| {
            ui.heading("Mode");
            ui.radio_value(&mut params.mode, LabMode::Batch, "Batch Experiments");
            ui.radio_value(&mut params.mode, LabMode::Live, "Live 2D Simulation");
            ui.separator();

            match params.mode {
                LabMode::Batch => {
                    render_batch_controls(ui, &mut params, &mut cancer, &mut state);
                }
                LabMode::Live => {
                    render_live_controls(
                        ui,
                        &mut params,
                        &game_state,
                        &mut next_game_state,
                        &mut time_fixed,
                        &mut speed,
                        &mut pending_reset,
                        &maps,
                        active_map.as_deref(),
                    );
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

// ─── Batch controls (contextual per experiment, categorized) ────────────────

fn render_experiment_category(
    ui: &mut egui::Ui,
    label: &str,
    items: &[(BatchExperiment, &str)],
    current: &mut BatchExperiment,
) {
    ui.label(egui::RichText::new(label).strong().size(11.0));
    for &(exp, name) in items {
        ui.radio_value(current, exp, name);
    }
    ui.add_space(4.0);
}

fn render_batch_controls(
    ui: &mut egui::Ui,
    params: &mut LabParams,
    cancer: &mut CancerParams,
    state: &mut LabState,
) {
    ui.heading("Experiment");
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            render_experiment_category(ui, "Core Simulation", CATEGORY_CORE, &mut params.experiment);
            render_experiment_category(ui, "Drug & Therapy", CATEGORY_DRUG, &mut params.experiment);
            render_experiment_category(ui, "Paper Validation", CATEGORY_PAPER, &mut params.experiment);
            render_experiment_category(ui, "Physics", CATEGORY_PHYSICS, &mut params.experiment);
        });
    ui.separator();

    // Contextual params per experiment
    render_experiment_params(ui, params, cancer);

    // Run mode (only for experiments that support it)
    if params.experiment.supports_multi_run() {
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
    } else {
        params.run_mode = RunMode::Single;
    }

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
    if !state.last_csv.is_empty() && ui.button("Export CSV").clicked() {
        let _ = std::fs::write(DEFAULT_EXPORT_PATH, &state.last_csv);
        ui.label(format!("Saved to {DEFAULT_EXPORT_PATH}"));
    }
}

fn render_experiment_params(ui: &mut egui::Ui, params: &mut LabParams, cancer: &mut CancerParams) {
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
        BatchExperiment::Personal => {
            ui.heading("Personal Universe");
            ui.label("Your name, birthday, or any text:");
            ui.text_edit_singleline(&mut params.personal_input);
        }
        BatchExperiment::PathwayInhibitor => {
            ui.heading("Pathway Inhibitor");
            ui.label("Metabolic compensation resistance.");
            ui.label("Uses default InhibitorConfig.");
            ui.add(egui::Slider::new(&mut params.worlds, 10..=500).text("Worlds"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, 50..=500).text("Ticks/gen"));
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        BatchExperiment::PaperZhang2022 => {
            ui.heading("Zhang 2022 — Adaptive Therapy");
            ui.label("eLife 11:e76284. 3-pop Lotka-Volterra.");
            ui.label("Continuous vs adaptive dosing.");
            ui.add(egui::Slider::new(&mut params.generations, 50..=500).text("Gens"));
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        BatchExperiment::PaperSharma2010 => {
            ui.heading("Sharma 2010 — Persisters");
            ui.label("Cell 141:69-80. Drug-tolerant persisters.");
            ui.label("~0.3% survive, recover in ~9 doublings.");
            ui.add(egui::Slider::new(&mut params.worlds, 10..=500).text("Worlds"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, 50..=500).text("Ticks/gen"));
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        BatchExperiment::PaperFooMichor2009 => {
            ui.heading("Foo & Michor 2009 — Pulsed");
            ui.label("PLoS Comp Bio. Continuous vs pulsed.");
            ui.label("Non-monotonic dose-resistance curve.");
            ui.add(egui::Slider::new(&mut params.worlds, 10..=500).text("Worlds"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, 50..=500).text("Ticks/gen"));
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        BatchExperiment::PaperMichor2005 => {
            ui.heading("Michor 2005 — Biphasic CML");
            ui.label("Nature 435:1267. Imatinib TKI.");
            ui.label("Fast phase + slow stem cell persistence.");
            ui.add(egui::Slider::new(&mut params.worlds, 10..=500).text("Worlds"));
            ui.add(egui::Slider::new(&mut params.generations, GENS_RANGE).text("Gens"));
            ui.add(egui::Slider::new(&mut params.ticks, 50..=500).text("Ticks/gen"));
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        BatchExperiment::PaperUnifiedAxioms => {
            ui.heading("PV-6 — Unified Axioms");
            ui.label("All 6 predictions from 4 fundamentals.");
            ui.label("Zero manual calibration.");
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        BatchExperiment::ParticleLab => {
            ui.heading("Particle Lab");
            ui.label("Coulomb + Lennard-Jones.");
            ui.label("Emergent bonds & molecules.");
            ui.add(egui::DragValue::new(&mut params.seed).prefix("Seed: ").speed(1.0));
        }
        // Core experiments with standard preset/seed/worlds/gens/ticks
        BatchExperiment::Lab
        | BatchExperiment::Speciation
        | BatchExperiment::Cambrian
        | BatchExperiment::Debate
        | BatchExperiment::Convergence => {
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
}

// ─── Live controls ─────────────────────────────────────────────────────────

fn render_live_controls(
    ui: &mut egui::Ui,
    params: &mut LabParams,
    game_state: &State<GameState>,
    next_game_state: &mut NextState<GameState>,
    time_fixed: &mut Time<Fixed>,
    speed: &mut SpeedScale,
    pending_reset: &mut PendingReset,
    maps: &AvailableMaps,
    active_map: Option<&resonance::worldgen::ActiveMapName>,
) {
    ui.heading("Live Simulation");

    // ── Pause / Resume ──
    let paused = **game_state == GameState::Paused;
    let label = if paused { "Resume" } else { "Pause" };
    if ui.button(label).clicked() {
        next_game_state.set(if paused {
            GameState::Playing
        } else {
            GameState::Paused
        });
    }
    if paused {
        ui.label("PAUSED");
    }

    // ── Speed slider ──
    ui.separator();
    let prev_speed = speed.0;
    ui.add(egui::Slider::new(&mut speed.0, SPEED_MIN..=SPEED_MAX).text("Speed"));
    if (speed.0 - prev_speed).abs() > 0.01 {
        let hz = BASE_HZ * speed.0 as f64;
        time_fixed.set_timestep(Duration::from_secs_f64(1.0 / hz));
    }

    // ── Map selector ──
    ui.separator();
    ui.heading("Map");
    let current_slug = active_map.map(|m| m.0.as_str()).unwrap_or("default");
    egui::ComboBox::from_id_salt("map_selector")
        .selected_text(current_slug)
        .show_ui(ui, |ui| {
            for slug in &maps.0 {
                if ui.selectable_label(slug == current_slug, slug.as_str()).clicked()
                    && slug != current_slug
                {
                    pending_reset.request_new_map(slug.clone());
                }
            }
        });

    // ── Reset ──
    if ui.button("Reset World").clicked() {
        pending_reset.request_same_map();
    }

    // ── View Layer ──
    ui.separator();
    ui.heading("View Layer");
    ui.radio_value(
        &mut params.view_layer,
        ViewLayer::FrequencyEnergy,
        "Frequency + Energy",
    );
    ui.radio_value(&mut params.view_layer, ViewLayer::EnergyOnly, "Energy Only");
}

// ─── LR-2: Reset world (exclusive system, ADR-019) ─────────────────────────

fn reset_world_system(world: &mut World) {
    let request = world.resource_mut::<PendingReset>().take();
    let Some(maybe_slug) = request else { return };

    // 1. If new map requested, reload config + grids + nuclei
    if let Some(slug) = maybe_slug {
        let config = match resonance::worldgen::load_map_config_from_slug(&slug) {
            Ok(c) => c,
            Err(err) => {
                warn!("Failed to load map '{slug}': {err}");
                return;
            }
        };
        // Despawn all existing nuclei
        let nuclei: Vec<Entity> = world
            .query_filtered::<Entity, With<resonance::worldgen::EnergyNucleus>>()
            .iter(world)
            .collect();
        for e in nuclei {
            world.despawn(e);
        }
        // Replace grids with new dimensions
        let new_grid = EnergyFieldGrid::new(
            config.width_cells,
            config.height_cells,
            config.cell_size,
            config.origin_vec2(),
        );
        let new_nutrients = NutrientFieldGrid::new(
            config.width_cells,
            config.height_cells,
            config.cell_size,
            config.origin_vec2(),
        );
        world.insert_resource(new_grid);
        world.insert_resource(new_nutrients);
        world.insert_resource(resonance::worldgen::ActiveMapName(slug));

        // Seed field if Big Bang mode
        if let Some(qe) = config.initial_field_qe {
            let freq = config.initial_field_freq.unwrap_or(85.0);
            world.resource_mut::<EnergyFieldGrid>().seed_uniform(qe, freq);
        }

        // Warmup ticks from config
        let ticks = config.warmup_ticks.unwrap_or(resonance::worldgen::WARMUP_TICKS);
        world.insert_resource(resonance::worldgen::WorldgenWarmupConfig { ticks });

        // Spawn nuclei from new config
        let emission_scale = config.emission_scale.unwrap_or(1.0);
        let layout = world
            .get_resource::<SimWorldTransformParams>()
            .copied()
            .unwrap_or_default();
        for spawn in resonance::worldgen::resolve_nuclei_for_spawn(&config) {
            let transform = if layout.use_xz_ground {
                Transform::from_xyz(spawn.position.x, layout.standing_y, spawn.position.y)
            } else {
                Transform::from_xyz(spawn.position.x, spawn.position.y, 0.0)
            };
            let mut nucleus = spawn.nucleus;
            nucleus.set_emission_rate_qe_s(
                (nucleus.emission_rate_qe_s() * emission_scale).max(0.0),
            );
            let mut ec = world.spawn((
                resonance::worldgen::StartupNucleus,
                Name::new(format!("nucleus::{}", spawn.name)),
                nucleus,
                transform,
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
            ));
            if let Some(qe) = spawn.reservoir {
                ec.insert(resonance::worldgen::NucleusReservoir { qe });
            }
        }
        world.insert_resource(config);
    } else {
        // Same map: just reset grids
        world.resource_mut::<EnergyFieldGrid>().reset_cells();
        world.resource_mut::<NutrientFieldGrid>().reset_cells();
    }

    // 2. Despawn all materialized/living entities (anything with BaseEnergy)
    let alive: Vec<Entity> = world
        .query_filtered::<Entity, With<BaseEnergy>>()
        .iter(world)
        .collect();
    for e in alive {
        world.despawn(e);
    }

    // 3. Reset clock
    if let Some(mut clock) = world.get_resource_mut::<SimulationClock>() {
        clock.tick_id = 0;
    }
    if let Some(mut elapsed) =
        world.get_resource_mut::<resonance::runtime_platform::simulation_tick::SimulationElapsed>()
    {
        elapsed.secs = 0.0;
    }

    // 4. Re-run warmup (propagation + materialization)
    let ticks = world.resource::<resonance::worldgen::WorldgenWarmupConfig>().ticks;
    resonance::worldgen::run_warmup_loop(world, ticks);

    // 5. Ensure state is Playing + Active
    world
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    world
        .resource_mut::<NextState<PlayState>>()
        .set(PlayState::Active);
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
        // ─── New experiments (ADR-018) ──────────────────────────────────
        BatchExperiment::Personal => {
            let input = if params.personal_input.is_empty() {
                "Resonance"
            } else {
                &params.personal_input
            };
            LabResult::Personal(Box::new(personal::run(input)))
        }
        BatchExperiment::PathwayInhibitor => {
            let cfg = pathway_inhibitor_exp::InhibitorConfig {
                worlds: params.worlds,
                generations: params.generations,
                ticks_per_gen: params.ticks,
                seed: params.seed,
                ..Default::default()
            };
            LabResult::PathwayInhibitor(Box::new(pathway_inhibitor_exp::run(&cfg)))
        }
        BatchExperiment::PaperZhang2022 => {
            let cfg = paper_zhang2022::ZhangConfig {
                generations: params.generations,
                seed: params.seed,
                ..Default::default()
            };
            LabResult::PaperZhang(Box::new(paper_zhang2022::run(&cfg)))
        }
        BatchExperiment::PaperSharma2010 => {
            let cfg = paper_sharma2010::SharmaConfig {
                worlds: params.worlds,
                generations: params.generations,
                ticks_per_gen: params.ticks,
                seed: params.seed,
                ..Default::default()
            };
            LabResult::PaperSharma(Box::new(paper_sharma2010::run(&cfg)))
        }
        BatchExperiment::PaperFooMichor2009 => {
            let cfg = paper_foo_michor2009::FooMichorConfig {
                worlds: params.worlds,
                generations: params.generations,
                ticks_per_gen: params.ticks,
                seed: params.seed,
                ..Default::default()
            };
            LabResult::PaperFooMichor(Box::new(paper_foo_michor2009::run(&cfg)))
        }
        BatchExperiment::PaperMichor2005 => {
            let cfg = paper_michor2005::MichorConfig {
                worlds: params.worlds,
                generations: params.generations,
                ticks_per_gen: params.ticks,
                seed: params.seed,
                ..Default::default()
            };
            LabResult::PaperMichor(Box::new(paper_michor2005::run(&cfg)))
        }
        BatchExperiment::PaperUnifiedAxioms => {
            LabResult::PaperUnified(Box::new(paper_unified_axioms::run(params.seed)))
        }
        BatchExperiment::ParticleLab => {
            let cfg = particle_lab::ParticleLabConfig {
                seed: params.seed,
                ..Default::default()
            };
            LabResult::ParticleLab(Box::new(particle_lab::run(&cfg)))
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

// ─── CSV export (stateless, all experiments) ────────────────────────────────

fn result_to_csv(result: &LabResult) -> String {
    use std::fmt::Write;
    match result {
        LabResult::None => String::new(),
        LabResult::Lab(r) | LabResult::Personal(r) => export::export_history_csv(&r.history),
        LabResult::Fermi(r) => {
            let mut csv = String::from("universe,species,fitness,diversity\n");
            for (i, rep) in r.reports.iter().enumerate() {
                let last = rep.history.last();
                let _ = writeln!(
                    csv,
                    "{},{:.2},{:.4},{:.4}",
                    i,
                    last.map(|s| s.species_mean).unwrap_or(0.0),
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0)
                );
            }
            csv
        }
        LabResult::Speciation(r) => {
            let mut csv = String::from("metric,value\n");
            let _ = writeln!(csv, "mean_freq_a,{:.2}", r.mean_freq_a);
            let _ = writeln!(csv, "mean_freq_b,{:.2}", r.mean_freq_b);
            let _ = writeln!(csv, "cross_interference,{:.4}", r.cross_interference);
            let _ = writeln!(csv, "speciated,{}", r.speciated);
            csv
        }
        LabResult::Cambrian(r) => {
            let mut csv = String::from("gen,diversity\n");
            for (i, &d) in r.diversity_curve.iter().enumerate() {
                let _ = writeln!(csv, "{},{:.4}", i, d);
            }
            if let Some(g) = r.explosion_gen {
                let _ = writeln!(csv, "# explosion_gen,{}", g);
            }
            csv
        }
        LabResult::Debate(r) => {
            let mut csv = String::from("metric,value\n");
            let _ = writeln!(csv, "life_rate,{:.4}", r.life_rate);
            let _ = writeln!(csv, "complexity_rate,{:.4}", r.complexity_rate);
            let _ = writeln!(csv, "cooperation_signal,{:.4}", r.cooperation_signal);
            csv
        }
        LabResult::Convergence(r) => {
            let mut csv = String::from("metric,value\n");
            let _ = writeln!(csv, "n_seeds,{}", r.n_seeds);
            let _ = writeln!(csv, "mean_distance,{:.4}", r.mean_distance);
            let _ = writeln!(csv, "convergence_rate,{:.4}", r.convergence_rate);
            csv
        }
        LabResult::Cancer(r) => {
            let mut csv =
                String::from("gen,cancer,normal,freq_mean,resistance,diversity,drug_active\n");
            for s in &r.timeline {
                let _ = writeln!(
                    csv,
                    "{},{:.2},{:.2},{:.2},{:.4},{:.2},{}",
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
                let _ = writeln!(
                    csv,
                    "{},{:.4},{:.4},{:.4},{:.2}",
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
                let _ = writeln!(
                    csv,
                    "{},{:.4},{:.4},{:.2}",
                    i,
                    last.map(|s| s.best_fitness).unwrap_or(0.0),
                    last.map(|s| s.diversity).unwrap_or(0.0),
                    last.map(|s| s.species_mean).unwrap_or(0.0)
                );
            }
            csv
        }
        // ─── New experiments CSV (ADR-018) ──────────────────────────────
        LabResult::PathwayInhibitor(r) => {
            let mut csv =
                String::from("gen,alive,wildtype,resistant,efficiency,drug_active\n");
            for s in &r.timeline {
                let _ = writeln!(
                    csv,
                    "{},{:.2},{:.2},{:.2},{:.4},{}",
                    s.generation,
                    s.alive_mean,
                    s.wildtype_alive_mean,
                    s.resistant_alive_mean,
                    s.mean_efficiency,
                    s.drug_active as u8
                );
            }
            csv
        }
        LabResult::PaperZhang(r) => {
            let mut csv = String::from("gen,continuous_pop,adaptive_pop,continuous_drug,adaptive_drug\n");
            let max_len = r.timeline_continuous.len().max(r.timeline_adaptive.len());
            for i in 0..max_len {
                let c = r.timeline_continuous.get(i);
                let a = r.timeline_adaptive.get(i);
                let _ = writeln!(
                    csv,
                    "{},{:.4},{:.4},{},{}",
                    i,
                    c.map(|s| s.alive_mean).unwrap_or(0.0),
                    a.map(|s| s.alive_mean).unwrap_or(0.0),
                    c.map(|s| s.drug_active as u8).unwrap_or(0),
                    a.map(|s| s.drug_active as u8).unwrap_or(0),
                );
            }
            csv
        }
        LabResult::PaperSharma(r) => {
            let mut csv = String::from("gen,alive,qe,persister_frac\n");
            for s in &r.timeline {
                let _ = writeln!(
                    csv,
                    "{},{:.2},{:.2},{:.4}",
                    s.generation, s.alive_mean, s.qe_mean, s.persister_frac
                );
            }
            csv
        }
        LabResult::PaperFooMichor(r) => {
            let mut csv = String::from("dose,resistance_rate\n");
            for &(dose, res) in &r.dose_resistance_curve {
                let _ = writeln!(csv, "{:.2},{:.4}", dose, res);
            }
            csv
        }
        LabResult::PaperMichor(r) => {
            let mut csv = String::from("gen,total,diff,prog,stem,drug_active\n");
            for s in &r.timeline {
                let _ = writeln!(
                    csv,
                    "{},{:.2},{:.2},{:.2},{:.2},{}",
                    s.generation,
                    s.total_alive,
                    s.diff_alive,
                    s.prog_alive,
                    s.stem_alive,
                    s.drug_active as u8
                );
            }
            csv
        }
        LabResult::PaperUnified(r) => {
            let mut csv = String::from("test,paper,prediction,passed,detail\n");
            for t in &r.results {
                let _ = writeln!(
                    csv,
                    "{},{},{},{},\"{}\"",
                    t.name, t.paper, t.prediction, t.passed, t.detail
                );
            }
            csv
        }
        LabResult::ParticleLab(r) => {
            let mut csv = String::from("step,particles,bonds,molecules,kinetic,potential\n");
            for s in &r.timeline {
                let _ = writeln!(
                    csv,
                    "{},{},{},{},{:.4},{:.4}",
                    s.step,
                    s.particle_count,
                    s.bond_count,
                    s.molecule_types,
                    s.mean_kinetic_energy,
                    s.mean_potential_energy
                );
            }
            csv
        }
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
                    ui.label(format!("Explosion at gen {g}"));
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
        // ─── New experiments rendering (ADR-018) ────────────────────────
        LabResult::Personal(r) => {
            ui.heading(format!("Personal Universe — {}", r.preset_name));
            render_fitness_chart(ui, &r.history);
            render_top_genomes(ui, &r.top_genomes);
        }
        LabResult::PathwayInhibitor(r) => {
            ui.heading("Pathway Inhibitor");
            egui::Grid::new("inhibitor").show(ui, |ui| {
                ui.label("Resistance:");
                ui.label(if r.resistance_detected { "YES" } else { "NO" });
                ui.end_row();
                if let Some(g) = r.resistance_gen {
                    ui.label("Resistance gen:");
                    ui.label(format!("{g}"));
                    ui.end_row();
                }
                ui.label("Compensation:");
                ui.label(if r.compensation_detected { "YES" } else { "NO" });
                ui.end_row();
            });
            ui.separator();
            let wt_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.wildtype_alive_mean as f64])
                .collect();
            let res_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.resistant_alive_mean as f64])
                .collect();
            Plot::new("inhibitor_pop")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    p.line(Line::new(wt_pts).name("wildtype").color(COLOR_WILDTYPE));
                    p.line(Line::new(res_pts).name("resistant").color(COLOR_RESISTANCE));
                });
        }
        LabResult::PaperZhang(r) => {
            ui.heading("Zhang 2022 — Adaptive vs Continuous");
            egui::Grid::new("zhang").show(ui, |ui| {
                ui.label("Prediction met:");
                ui.label(if r.prediction_met { "YES" } else { "NO" });
                ui.end_row();
                ui.label("TTP ratio:");
                ui.label(format!("{:.2}×", r.ttp_ratio));
                ui.end_row();
                ui.label("Drug exposure:");
                ui.label(format!("{:.1}%", r.drug_exposure_ratio * 100.0));
                ui.end_row();
                ui.label("Adaptive cycles:");
                ui.label(format!("{}", r.adaptive_cycles));
                ui.end_row();
                if let Some(g) = r.continuous_ttp_gen {
                    ui.label("Continuous TTP:");
                    ui.label(format!("gen {g}"));
                    ui.end_row();
                }
                if let Some(g) = r.adaptive_ttp_gen {
                    ui.label("Adaptive TTP:");
                    ui.label(format!("gen {g}"));
                    ui.end_row();
                }
            });
            ui.separator();
            let cont_pts: PlotPoints = r
                .timeline_continuous
                .iter()
                .map(|s| [s.generation as f64, s.alive_mean as f64])
                .collect();
            let adapt_pts: PlotPoints = r
                .timeline_adaptive
                .iter()
                .map(|s| [s.generation as f64, s.alive_mean as f64])
                .collect();
            Plot::new("zhang_pop")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    p.line(Line::new(cont_pts).name("continuous").color(COLOR_CONTINUOUS));
                    p.line(Line::new(adapt_pts).name("adaptive").color(COLOR_ADAPTIVE));
                });
        }
        LabResult::PaperSharma(r) => {
            ui.heading("Sharma 2010 — Drug-Tolerant Persisters");
            egui::Grid::new("sharma").show(ui, |ui| {
                ui.label("Persister fraction:");
                ui.label(format!("{:.2}%", r.persister_fraction * 100.0));
                ui.end_row();
                ui.label("Recovery:");
                ui.label(if r.recovery_detected { "YES" } else { "NO" });
                ui.end_row();
                if let Some(g) = r.recovery_gen {
                    ui.label("Recovery gen:");
                    ui.label(format!("{g}"));
                    ui.end_row();
                }
            });
            ui.separator();
            let pop_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.alive_mean as f64])
                .collect();
            Plot::new("sharma_pop")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    p.line(Line::new(pop_pts).name("population").color(COLOR_NORMAL));
                });
            let pers_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.persister_frac as f64])
                .collect();
            Plot::new("sharma_persister")
                .height(CHART_HEIGHT_SMALL)
                .show(ui, |p| {
                    p.line(Line::new(pers_pts).name("persister frac").color(COLOR_RESISTANCE));
                });
        }
        LabResult::PaperFooMichor(r) => {
            ui.heading("Foo & Michor 2009 — Dose-Resistance");
            egui::Grid::new("foo").show(ui, |ui| {
                ui.label("Optimal dose:");
                ui.label(format!("{:.2}", r.optimal_dose));
                ui.end_row();
                ui.label("Non-monotonic:");
                ui.label(if r.optimal_exists { "YES" } else { "NO" });
                ui.end_row();
                ui.label("Pulsed beats continuous:");
                ui.label(if r.pulsed_beats_continuous { "YES" } else { "NO" });
                ui.end_row();
                ui.label("Continuous res @0.8:");
                ui.label(format!("{:.3}", r.continuous_resistance_at_08));
                ui.end_row();
                ui.label("Pulsed res @0.8:");
                ui.label(format!("{:.3}", r.pulsed_resistance_at_08));
                ui.end_row();
            });
            ui.separator();
            let dose_pts: PlotPoints = r
                .dose_resistance_curve
                .iter()
                .map(|&(dose, res)| [dose as f64, res as f64])
                .collect();
            Plot::new("foo_dose")
                .height(CHART_HEIGHT_MAIN)
                .x_axis_label("Dose")
                .y_axis_label("Resistance rate")
                .show(ui, |p| {
                    p.line(Line::new(dose_pts).name("dose-resistance").color(COLOR_CANCER));
                });
        }
        LabResult::PaperMichor(r) => {
            ui.heading("Michor 2005 — Biphasic CML Decline");
            egui::Grid::new("michor").show(ui, |ui| {
                ui.label("Biphasic:");
                ui.label(if r.biphasic_detected { "YES" } else { "NO" });
                ui.end_row();
                ui.label("Slope ratio:");
                ui.label(format!("{:.1}×", r.slope_ratio));
                ui.end_row();
                ui.label("Phase 1 slope:");
                ui.label(format!("{:.4}", r.phase1_slope));
                ui.end_row();
                ui.label("Phase 2 slope:");
                ui.label(format!("{:.4}", r.phase2_slope));
                ui.end_row();
                if let Some(g) = r.inflection_gen {
                    ui.label("Inflection:");
                    ui.label(format!("gen {g}"));
                    ui.end_row();
                }
                ui.label("Stem survive:");
                ui.label(if r.stem_survive { "YES" } else { "NO" });
                ui.end_row();
            });
            ui.separator();
            let diff_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.diff_alive as f64])
                .collect();
            let prog_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.prog_alive as f64])
                .collect();
            let stem_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.generation as f64, s.stem_alive as f64])
                .collect();
            Plot::new("michor_pop")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    p.line(Line::new(diff_pts).name("differentiated").color(COLOR_DIFF));
                    p.line(Line::new(prog_pts).name("progenitor").color(COLOR_PROG));
                    p.line(Line::new(stem_pts).name("stem").color(COLOR_STEM));
                });
        }
        LabResult::PaperUnified(r) => {
            ui.heading("PV-6 — Unified Axioms Validation");
            ui.label(format!(
                "{}/{} passed ({}ms)",
                r.passed_count, r.total_count, r.wall_time_ms
            ));
            let badge = if r.all_passed { "ALL PASSED" } else { "SOME FAILED" };
            let color = if r.all_passed { COLOR_NORMAL } else { COLOR_CANCER };
            ui.colored_label(color, egui::RichText::new(badge).size(16.0).strong());
            ui.separator();
            egui::Grid::new("unified").striped(true).show(ui, |ui| {
                ui.label("Test");
                ui.label("Paper");
                ui.label("Result");
                ui.label("Detail");
                ui.end_row();
                for t in &r.results {
                    ui.label(t.name);
                    ui.label(t.paper);
                    let (icon, col) = if t.passed {
                        ("PASS", COLOR_NORMAL)
                    } else {
                        ("FAIL", COLOR_CANCER)
                    };
                    ui.colored_label(col, icon);
                    ui.label(&t.detail);
                    ui.end_row();
                }
            });
        }
        LabResult::ParticleLab(r) => {
            ui.heading("Particle Lab — Emergent Chemistry");
            egui::Grid::new("particle_summary").show(ui, |ui| {
                ui.label("Final bonds:");
                let last = r.timeline.last();
                ui.label(format!("{}", last.map(|s| s.bond_count).unwrap_or(0)));
                ui.end_row();
                ui.label("Molecule types:");
                ui.label(format!("{}", last.map(|s| s.molecule_types).unwrap_or(0)));
                ui.end_row();
                ui.label("Molecules:");
                ui.label(format!("{}", r.final_molecules.len()));
                ui.end_row();
            });
            ui.separator();
            let bond_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.step as f64, s.bond_count as f64])
                .collect();
            let ke_pts: PlotPoints = r
                .timeline
                .iter()
                .map(|s| [s.step as f64, s.mean_kinetic_energy as f64])
                .collect();
            Plot::new("particle_bonds")
                .height(CHART_HEIGHT_MAIN)
                .show(ui, |p| {
                    p.line(Line::new(bond_pts).name("bonds").color(COLOR_BONDS));
                    p.line(Line::new(ke_pts).name("kinetic energy").color(COLOR_KINETIC));
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
                (v / 4, v, v / 3)
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
